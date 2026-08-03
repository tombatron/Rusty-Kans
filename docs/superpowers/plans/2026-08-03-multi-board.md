# Multi-Board Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single hardcoded board with a full multi-board experience — landing page, board view, and board CRUD (create, rename, delete with confirmation modal).

**Architecture:** The landing page (`/`) lists all boards; each board opens at `/boards/{board_id}`. Board CRUD uses the same Turbo Frame POST/Redirect/GET pattern already established for list and card rename. Delete uses a native `<dialog>` element with a retype-to-confirm input and a delegated JS event listener so dynamically appended boards work automatically.

**Tech Stack:** Rust/Axum, SQLx (SQLite), Askama templates, Hotwire Turbo Frames/Streams, native HTML `<dialog>`

## Global Constraints

- All handlers return `Result<_, KanbanError>` — never panic
- Template structs derive `Debug` and `Template`; form structs derive `Debug` and `Deserialize`
- SQLite integers bind as `i64`; model IDs are `u64` — cast at the bind site
- `fetch_optional(...).await?.ok_or(KanbanError::BoardNotFound(id))` for lookups that may miss
- Build with `cargo build` must be clean (zero errors) at the end of every task
- Run `cargo test` after every task — all tests must pass

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `src/models.rs` | Add `Board` struct |
| Modify | `src/errors.rs` | Add `BoardNotFound` variant |
| Modify | `src/handlers.rs` | Update `create_list_common` signature + SQL |
| Modify | `src/handlers/web.rs` | All new handlers + route registration |
| Modify | `src/state.rs` | Remove hardcoded board seed |
| Modify | `templates/board.html` | Add breadcrumb + fix list creation form action |
| Modify | `templates/base.html` | CSS additions (landing, board header, delete dialog) |
| Create | `templates/landing.html` | Landing page — board list + create form |
| Create | `templates/turbo_new_board.html` | Stream: append new board card to `#boards` |
| Create | `templates/turbo_board_header.html` | Turbo Frame — board title view state |
| Create | `templates/turbo_board_header_edit.html` | Turbo Frame — board rename form |

---

### Task 1: Board model and BoardNotFound error

**Files:**
- Modify: `src/models.rs`
- Modify: `src/errors.rs`

**Interfaces:**
- Produces: `Board { id: u64, name: String }` (FromRow, used by all board handlers)
- Produces: `KanbanError::BoardNotFound(u64)` (used by get_board, get_board_header, get_board_edit)

- [ ] **Step 1: Add `Board` struct to `src/models.rs`**

Add after the `List` impl block:

```rust
#[derive(Debug, Clone, FromRow)]
pub struct Board {
    #[sqlx(rename = "board_id")]
    pub id: u64,
    pub name: String,
}
```

- [ ] **Step 2: Add `BoardNotFound` to `src/errors.rs`**

Add the variant to the enum:
```rust
pub enum KanbanError {
    BoardNotFound(u64),   // ← add this line
    ListNotFound(u64),
    CardNotFound(u64),
    // ...existing variants
}
```

Add the Display arm inside `impl Display for KanbanError`:
```rust
KanbanError::BoardNotFound(board_id) => {
    write!(f, "Board with ID ({}) was not found.", board_id)
}
```

Add the IntoResponse arm inside `impl IntoResponse for KanbanError`:
```rust
KanbanError::BoardNotFound(_) => {
    (StatusCode::NOT_FOUND, self.to_string()).into_response()
}
```

- [ ] **Step 3: Build and run tests**

```bash
cargo build
cargo test
```

Expected: zero errors, all existing tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/models.rs src/errors.rs
git commit -m "$(cat <<'EOF'
Add Board model and BoardNotFound error variant

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Landing page — list and create boards

**Files:**
- Modify: `src/handlers/web.rs`
- Modify: `templates/base.html`
- Create: `templates/landing.html`
- Create: `templates/turbo_new_board.html`

**Interfaces:**
- Consumes: `Board` from Task 1
- Produces: `get_landing` at `GET /`, `post_board_form` at `POST /boards`

- [ ] **Step 1: Add handlers to `src/handlers/web.rs`**

Add at the top of the `use` imports (add `Board` to the models import):
```rust
use crate::models::{Board, Card, CardMoveEvent, List};
```

Add after the existing imports:
```rust
#[derive(Debug, Template)]
#[template(path = "landing.html")]
struct LandingTemplate {
    boards: Vec<Board>,
}

pub async fn get_landing(
    State(state): State<ApplicationState>,
) -> Result<Html<String>, KanbanError> {
    let boards =
        sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards ORDER BY board_id")
            .fetch_all(&state.db)
            .await?;

    Ok(Html(LandingTemplate { boards }.render()?))
}

#[derive(Debug, Deserialize)]
pub struct CreateBoardRequest {
    name: String,
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_board.html")]
struct NewBoardTemplate {
    board: Board,
}

pub async fn post_board_form(
    State(state): State<ApplicationState>,
    Form(board_info): Form<CreateBoardRequest>,
) -> Result<TurboStream, KanbanError> {
    let result = sqlx::query("INSERT INTO boards (name) VALUES (?)")
        .bind(&board_info.name)
        .execute(&state.db)
        .await?;

    let board = Board {
        id: result.last_insert_rowid() as u64,
        name: board_info.name,
    };

    Ok(TurboStream(NewBoardTemplate { board }.render()?))
}
```

- [ ] **Step 2: Update route registration in `get_router_configuration`**

Change the existing `/` route and add `/boards`:
```rust
pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))           // was get(get_index)
        .route("/boards", post(post_board_form)) // new
        .route("/lists", post(post_list_form))   // unchanged for now
        // ... rest unchanged
}
```

- [ ] **Step 3: Create `templates/landing.html`**

```html
{% extends "base.html" %}

{% block title %}Boards{% endblock %}

{% block content %}
<div class="landing">
    <header class="landing-header">
        <h1>Boards</h1>
        <form class="create-board-form" action="/boards" method="post">
            <input type="text" name="name" placeholder="New board&hellip;" required>
            <button type="submit">+ New Board</button>
        </form>
    </header>
    <div id="boards" class="boards-grid">
        {% for board in boards %}
        <div class="board-card" id="board-{{ board.id }}">
            <turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
                <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
                <div class="board-actions">
                    <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
                        <i data-lucide="pencil-line"></i>
                    </a>
                </div>
            </turbo-frame>
            <a href="/boards/{{ board.id }}" class="board-open-link">Open board &rarr;</a>
        </div>
        {% endfor %}
    </div>
</div>
{% endblock %}
```

- [ ] **Step 4: Create `templates/turbo_new_board.html`**

```html
<turbo-stream action="append" target="boards">
    <template>
        <div class="board-card" id="board-{{ board.id }}">
            <turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
                <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
                <div class="board-actions">
                    <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
                        <i data-lucide="pencil-line"></i>
                    </a>
                </div>
            </turbo-frame>
            <a href="/boards/{{ board.id }}" class="board-open-link">Open board &rarr;</a>
        </div>
    </template>
</turbo-stream>
```

- [ ] **Step 5: Add landing page CSS to `templates/base.html`**

Add before the closing `</style>` tag:

```css
/* ── Landing page ────────────────────────────────── */
.landing {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

.landing-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2rem;
}

.landing-header h1 {
    font-family: 'Oswald', sans-serif;
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text);
}

.create-board-form {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

.create-board-form input[type="text"] {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    font-family: 'DM Sans', sans-serif;
    font-size: 13px;
    padding: 0.45rem 0.7rem;
    outline: none;
    transition: border-color 0.2s;
}

.create-board-form input[type="text"]:focus { border-color: var(--accent); }

.boards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 1rem;
}

.board-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

.board-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1rem 0.75rem;
}

.board-name {
    font-family: 'Oswald', sans-serif;
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text);
    text-decoration: none;
    transition: color 0.15s;
}

.board-name:hover { color: var(--accent); }

.board-card .board-actions {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    opacity: 0;
    transition: opacity 0.15s;
}

.board-card-header:hover .board-actions { opacity: 1; }

.board-edit-btn,
button.board-delete-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    color: var(--text-muted);
    text-decoration: none;
    background: transparent;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
}

.board-edit-btn:hover { background: rgba(255,255,255,0.06); color: var(--text); }
button.board-delete-btn:hover { background: rgba(220,50,50,0.12); color: #e05555; }
.board-edit-btn svg, button.board-delete-btn svg { width: 13px; height: 13px; }

.board-open-link {
    display: block;
    padding: 0.6rem 1rem;
    font-size: 12px;
    color: var(--text-muted);
    text-decoration: none;
    border-top: 1px solid var(--border);
    transition: color 0.15s, background 0.15s;
}

.board-open-link:hover { color: var(--accent); background: var(--card-hover); }

/* ── Board card rename form ──────────────────────── */
.board-card-header form {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
}

.board-card-header input[type="text"] {
    flex: 1;
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    color: var(--text);
    font-family: 'Oswald', sans-serif;
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 0.2rem 0.4rem;
    outline: none;
}

.board-edit-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
}
```

- [ ] **Step 6: Write the landing page test**

Add to the `tests` module in `src/router.rs`:

```rust
#[tokio::test]
async fn landing_page_returns_200() {
    let state = create_application_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

- [ ] **Step 7: Run test — expect FAIL (route not wired yet if running before Step 2, or PASS if already done)**

```bash
cargo test landing_page_returns_200 -- --nocapture
```

Expected after all steps: PASS with status 200.

- [ ] **Step 8: Build and run all tests**

```bash
cargo build && cargo test
```

Expected: zero errors, all tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/handlers/web.rs templates/landing.html templates/turbo_new_board.html templates/base.html src/router.rs
git commit -m "$(cat <<'EOF'
Add landing page with board list and create board form

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Board view page and list creation route

**Files:**
- Modify: `src/handlers/web.rs`
- Modify: `src/handlers.rs`
- Modify: `src/state.rs`
- Modify: `templates/board.html`
- Modify: `templates/base.html`

**Interfaces:**
- Consumes: `Board` from Task 1, `KanbanError::BoardNotFound` from Task 1
- Produces: `get_board` at `GET /boards/{board_id}`, `post_list_form` at `POST /boards/{board_id}/lists`

- [ ] **Step 1: Write a failing test for the board view**

Add to `src/router.rs` tests module:

```rust
#[tokio::test]
async fn board_view_returns_200() {
    let state = create_application_state().await;

    let result = sqlx::query("INSERT INTO boards (name) VALUES (?)")
        .bind("Test Board")
        .execute(&state.db)
        .await
        .unwrap();
    let board_id = result.last_insert_rowid();

    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/boards/{}", board_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

You'll need to add `sqlx` to the test imports at the top of the test module:
```rust
use sqlx::SqlitePool;  // only if not already present
```

Actually the state already has `state.db: SqlitePool`, so just use it directly from the state object.

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test board_view_returns_200 -- --nocapture
```

Expected: FAIL — route `/boards/{board_id}` does not exist yet.

- [ ] **Step 3: Convert `get_index` to `get_board` in `src/handlers/web.rs`**

Replace the `IndexTemplate` struct and `get_index` function entirely:

```rust
#[derive(Debug, Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    id: u64,
    name: String,
    lists: Vec<List>,
}

pub async fn get_board(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
) -> Result<impl IntoResponse, KanbanError> {
    let board = sqlx::query("SELECT board_id, name FROM boards WHERE board_id = ?")
        .bind(board_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?;

    let mut lists: Vec<List> =
        sqlx::query_as::<_, List>("SELECT list_id, name FROM lists WHERE board_id = ?")
            .bind(board.get::<i64, _>("board_id"))
            .fetch_all(&state.db)
            .await?;

    let cards: Vec<Card> = if lists.is_empty() {
        vec![]
    } else {
        let list_ids_placeholders = lists.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query_text = format!(
            "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id in ({})",
            list_ids_placeholders
        );
        let mut query = sqlx::query_as::<_, Card>(AssertSqlSafe(query_text));
        for id in &lists {
            query = query.bind(id.id as i64);
        }
        query.fetch_all(&state.db).await?
    };

    for l in lists.iter_mut() {
        l.cards = cards.iter().filter(|c| c.list_id == l.id).cloned().collect();
    }

    Ok(Html(BoardTemplate {
        id: board.get::<u64, _>("board_id"),
        name: board.get::<String, _>("name"),
        lists,
    }
    .render()?))
}
```

- [ ] **Step 4: Update `post_list_form` in `src/handlers/web.rs`**

Replace the existing `post_list_form`:

```rust
pub async fn post_list_form(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Form(list_info): Form<CreateListRequest>,
) -> Result<TurboStream, KanbanError> {
    let created_list = create_list_common(State(state), board_id, list_info).await?;
    Ok(TurboStream(created_list.render()?))
}
```

- [ ] **Step 5: Update `create_list_common` in `src/handlers.rs`**

Replace the existing `create_list_common`:

```rust
async fn create_list_common(
    State(state): State<ApplicationState>,
    board_id: u64,
    list_info: CreateListRequest,
) -> Result<ListItemTemplate, KanbanError> {
    let insert_list_result =
        sqlx::query("INSERT INTO lists (board_id, name) VALUES (?, ?)")
            .bind(board_id as i64)
            .bind(&list_info.name)
            .execute(&state.db)
            .await?;

    let list_id = insert_list_result.last_insert_rowid();

    Ok(ListItemTemplate {
        id: list_id as u64,
        name: list_info.name,
    })
}
```

- [ ] **Step 6: Update routes in `get_router_configuration` in `src/handlers/web.rs`**

Replace the `/lists` POST route and add the board route:

```rust
pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))
        .route("/boards/{board_id}/lists", post(post_list_form))
        // remove the old: .route("/lists", post(post_list_form))
        .route("/lists/{list_id}/cards", post(post_card_form))
        .route(
            "/lists/{list_id}/cards/{card_id}/move",
            post(post_move_card_action),
        )
        .route("/lists/{list_id}/cards/{card_id}/delete", post(delete_card_form))
        .route("/lists/{list_id}/edit", get(get_list_edit))
        .route("/lists/{list_id}/rename", post(post_list_rename))
        .route("/lists/{list_id}/header", get(get_list_header))
        .route("/lists/{list_id}/delete", post(post_list_delete))
        .route("/cards/{card_id}/edit", get(get_card_edit).post(patch_card_form))
        .route("/cards/{card_id}/view", get(get_card_by_id))
}
```

- [ ] **Step 7: Remove the hardcoded board seed from `src/state.rs`**

Delete these lines from `create_application_state`:

```rust
sqlx::query("INSERT OR IGNORE INTO boards (board_id, name) VALUES (1, 'My Board')")
    .execute(&db)
    .await
    .unwrap();
```

The function should now just connect and set up the broadcast channel.

- [ ] **Step 8: Update `templates/board.html`**

Replace the `<header>` block and fix the list creation form action:

```html
{% block content %}
    <header class="board-page-header">
        <div class="board-page-header-left">
            <a href="/" class="board-breadcrumb">&#8592; Boards</a>
            <h1 class="board-title">{{ name }}</h1>
        </div>
        <form action="/boards/{{ id }}/lists" method="post">
            <input type="text" name="name" placeholder="New list&hellip;">
            <button type="submit">+ Add List</button>
        </form>
    </header>
    <!-- rest of block unchanged -->
```

- [ ] **Step 9: Add board page header CSS to `templates/base.html`**

Add after the landing page CSS block:

```css
/* ── Board page header ───────────────────────────── */
.board-page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.875rem 1.25rem;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}

.board-page-header-left {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
}

.board-breadcrumb {
    font-size: 11px;
    color: var(--text-muted);
    text-decoration: none;
    letter-spacing: 0.04em;
    transition: color 0.15s;
}

.board-breadcrumb:hover { color: var(--accent); }
```

- [ ] **Step 10: Run the test — expect PASS**

```bash
cargo test board_view_returns_200 -- --nocapture
```

Expected: PASS with status 200.

- [ ] **Step 11: Build and run all tests**

```bash
cargo build && cargo test
```

Expected: zero errors, all tests pass.

- [ ] **Step 12: Commit**

```bash
git add src/handlers/web.rs src/handlers.rs src/state.rs templates/board.html templates/base.html src/router.rs
git commit -m "$(cat <<'EOF'
Replace single-board index with board-scoped view and list creation

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Board rename

**Files:**
- Modify: `src/handlers/web.rs`
- Create: `templates/turbo_board_header.html`
- Create: `templates/turbo_board_header_edit.html`

**Interfaces:**
- Consumes: `Board` from Task 1, `KanbanError::BoardNotFound` from Task 1
- Produces: `get_board_header` at `GET /boards/{board_id}/header`, `get_board_edit` at `GET /boards/{board_id}/edit`, `post_board_rename` at `POST /boards/{board_id}/rename`

- [ ] **Step 1: Write a failing test**

Add to `src/router.rs` tests module:

```rust
#[tokio::test]
async fn board_header_returns_200() {
    let state = create_application_state().await;

    let result = sqlx::query("INSERT INTO boards (name) VALUES (?)")
        .bind("Rename Test Board")
        .execute(&state.db)
        .await
        .unwrap();
    let board_id = result.last_insert_rowid();

    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/boards/{}/header", board_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test board_header_returns_200 -- --nocapture
```

Expected: FAIL — route not registered yet.

- [ ] **Step 3: Add rename handlers to `src/handlers/web.rs`**

Add after the existing list-header handlers:

```rust
#[derive(Debug, Template)]
#[template(path = "turbo_board_header.html")]
struct BoardHeader {
    board: Board,
}

pub async fn get_board_header(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let board = sqlx::query_as::<_, Board>(
        "SELECT board_id, name FROM boards WHERE board_id = ?",
    )
    .bind(board_id as i64)
    .fetch_optional(&state.db)
    .await?
    .ok_or(KanbanError::BoardNotFound(board_id))?;

    Ok(Html(BoardHeader { board }.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_header_edit.html")]
struct BoardHeaderEdit {
    board: Board,
}

pub async fn get_board_edit(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let board = sqlx::query_as::<_, Board>(
        "SELECT board_id, name FROM boards WHERE board_id = ?",
    )
    .bind(board_id as i64)
    .fetch_optional(&state.db)
    .await?
    .ok_or(KanbanError::BoardNotFound(board_id))?;

    Ok(Html(BoardHeaderEdit { board }.render()?))
}

#[derive(Debug, Deserialize)]
pub struct BoardRename {
    name: String,
}

pub async fn post_board_rename(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Form(rename): Form<BoardRename>,
) -> Result<Redirect, KanbanError> {
    let result = sqlx::query("UPDATE boards SET name = ? WHERE board_id = ?")
        .bind(rename.name)
        .bind(board_id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() != 1 {
        return Err(KanbanError::DatabaseError(
            "Update failed. Please try again.".to_string(),
        ));
    }

    Ok(Redirect::to(format!("/boards/{board_id}/header").as_str()))
}
```

- [ ] **Step 4: Register the new routes in `get_router_configuration`**

Add after the `/boards/{board_id}/lists` route:

```rust
.route("/boards/{board_id}/header", get(get_board_header))
.route("/boards/{board_id}/edit", get(get_board_edit))
.route("/boards/{board_id}/rename", post(post_board_rename))
```

- [ ] **Step 5: Create `templates/turbo_board_header.html`**

```html
<turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
    <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
    <div class="board-actions">
        <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
            <i data-lucide="pencil-line"></i>
        </a>
    </div>
</turbo-frame>
```

- [ ] **Step 6: Create `templates/turbo_board_header_edit.html`**

```html
<turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
    <form action="/boards/{{ board.id }}/rename" method="post">
        <input type="text" name="name" value="{{ board.name }}" required>
        <div class="board-edit-actions">
            <button type="submit">Save</button>
            <a href="/boards/{{ board.id }}/header" class="btn-cancel">Cancel</a>
        </div>
    </form>
</turbo-frame>
```

- [ ] **Step 7: Run the test — expect PASS**

```bash
cargo test board_header_returns_200 -- --nocapture
```

Expected: PASS with status 200.

- [ ] **Step 8: Build and run all tests**

```bash
cargo build && cargo test
```

Expected: zero errors, all tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/handlers/web.rs templates/turbo_board_header.html templates/turbo_board_header_edit.html src/router.rs
git commit -m "$(cat <<'EOF'
Add board rename with turbo frame inline edit

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Board delete with confirmation modal

**Files:**
- Modify: `src/handlers/web.rs`
- Modify: `templates/landing.html`
- Modify: `templates/turbo_new_board.html`
- Modify: `templates/turbo_board_header.html`
- Modify: `templates/base.html`

**Interfaces:**
- Consumes: nothing new
- Produces: `post_board_delete` at `POST /boards/{board_id}/delete`

- [ ] **Step 1: Write a failing test**

Add to `src/router.rs` tests module:

```rust
#[tokio::test]
async fn board_delete_redirects_to_landing() {
    let state = create_application_state().await;

    let result = sqlx::query("INSERT INTO boards (name) VALUES (?)")
        .bind("Delete Me")
        .execute(&state.db)
        .await
        .unwrap();
    let board_id = result.last_insert_rowid();

    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/boards/{}/delete", board_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test board_delete_redirects_to_landing -- --nocapture
```

Expected: FAIL — route not registered yet.

- [ ] **Step 3: Add `post_board_delete` to `src/handlers/web.rs`**

Add after `post_board_rename`:

```rust
pub async fn post_board_delete(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
) -> Result<Redirect, KanbanError> {
    sqlx::query("DELETE FROM boards WHERE board_id = ?")
        .bind(board_id as i64)
        .execute(&state.db)
        .await?;

    Ok(Redirect::to("/"))
}
```

- [ ] **Step 4: Register the delete route in `get_router_configuration`**

Add after the `/boards/{board_id}/rename` route:

```rust
.route("/boards/{board_id}/delete", post(post_board_delete))
```

- [ ] **Step 5: Run the test — expect PASS**

```bash
cargo test board_delete_redirects_to_landing -- --nocapture
```

Expected: PASS with status 303 (SEE_OTHER redirect).

- [ ] **Step 6: Add the delete button and dialog to `templates/landing.html`**

Update each board card inside `{% for board in boards %}` — add the delete button inside `.board-actions` and the `<dialog>` after the turbo-frame:

```html
{% for board in boards %}
<div class="board-card" id="board-{{ board.id }}">
    <turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
        <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
        <div class="board-actions">
            <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
                <i data-lucide="pencil-line"></i>
            </a>
            <button type="button" class="board-delete-btn"
                    onclick="document.getElementById('delete-dialog-{{ board.id }}').showModal()">
                <i data-lucide="trash-2"></i>
            </button>
        </div>
    </turbo-frame>
    <a href="/boards/{{ board.id }}" class="board-open-link">Open board &rarr;</a>
    <dialog id="delete-dialog-{{ board.id }}" class="delete-dialog">
        <p>Type <strong>{{ board.name }}</strong> to confirm deletion.</p>
        <input type="text" id="delete-input-{{ board.id }}" autocomplete="off">
        <div class="dialog-actions">
            <button type="button" onclick="this.closest('dialog').close()">Cancel</button>
            <form action="/boards/{{ board.id }}/delete" method="post">
                <button type="submit" id="delete-confirm-{{ board.id }}" disabled class="btn-danger">
                    Delete
                </button>
            </form>
        </div>
    </dialog>
</div>
{% endfor %}
```

Also add the delegated event listener at the bottom of the `{% block content %}` block, just before `{% endblock %}`:

```html
<script>
document.addEventListener('input', e => {
    if (!e.target.matches('[id^="delete-input-"]')) return;
    const boardId = e.target.id.replace('delete-input-', '');
    const btn = document.getElementById(`delete-confirm-${boardId}`);
    const expectedName = document.querySelector(`#delete-dialog-${boardId} strong`).textContent;
    btn.disabled = e.target.value !== expectedName;
});
</script>
```

- [ ] **Step 7: Update `templates/turbo_new_board.html`** with delete button and dialog

Replace the file entirely so newly created boards also get the delete button:

```html
<turbo-stream action="append" target="boards">
    <template>
        <div class="board-card" id="board-{{ board.id }}">
            <turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
                <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
                <div class="board-actions">
                    <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
                        <i data-lucide="pencil-line"></i>
                    </a>
                    <button type="button" class="board-delete-btn"
                            onclick="document.getElementById('delete-dialog-{{ board.id }}').showModal()">
                        <i data-lucide="trash-2"></i>
                    </button>
                </div>
            </turbo-frame>
            <a href="/boards/{{ board.id }}" class="board-open-link">Open board &rarr;</a>
            <dialog id="delete-dialog-{{ board.id }}" class="delete-dialog">
                <p>Type <strong>{{ board.name }}</strong> to confirm deletion.</p>
                <input type="text" id="delete-input-{{ board.id }}" autocomplete="off">
                <div class="dialog-actions">
                    <button type="button" onclick="this.closest('dialog').close()">Cancel</button>
                    <form action="/boards/{{ board.id }}/delete" method="post">
                        <button type="submit" id="delete-confirm-{{ board.id }}" disabled class="btn-danger">
                            Delete
                        </button>
                    </form>
                </div>
            </dialog>
        </div>
    </template>
</turbo-stream>
```

- [ ] **Step 8: Update `templates/turbo_board_header.html`** with delete button

Replace the file so that after a rename the view state also shows the delete button:

```html
<turbo-frame id="board-title-{{ board.id }}" class="board-card-header">
    <a href="/boards/{{ board.id }}" class="board-name">{{ board.name }}</a>
    <div class="board-actions">
        <a href="/boards/{{ board.id }}/edit" class="board-edit-btn">
            <i data-lucide="pencil-line"></i>
        </a>
        <button type="button" class="board-delete-btn"
                onclick="document.getElementById('delete-dialog-{{ board.id }}').showModal()">
            <i data-lucide="trash-2"></i>
        </button>
    </div>
</turbo-frame>
```

Note: The rename flow updates only the turbo-frame — the `<dialog>` element remains in the DOM because it lives outside the frame in `.board-card`. The delete button in the updated frame correctly targets the existing dialog by ID.

- [ ] **Step 9: Add delete dialog CSS to `templates/base.html`**

Add after the board page header CSS:

```css
/* ── Delete confirmation dialog ──────────────────── */
.delete-dialog {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text);
    font-family: 'DM Sans', sans-serif;
    padding: 1.5rem;
    max-width: 360px;
    width: 100%;
}

.delete-dialog::backdrop {
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(2px);
}

.delete-dialog p {
    font-size: 14px;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
    line-height: 1.5;
}

.delete-dialog strong { color: var(--text); }

.delete-dialog input[type="text"] {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    font-family: 'DM Sans', sans-serif;
    font-size: 13px;
    padding: 0.4rem 0.6rem;
    outline: none;
    transition: border-color 0.2s;
    margin-bottom: 1rem;
    box-sizing: border-box;
}

.delete-dialog input[type="text"]:focus { border-color: var(--accent); }

.dialog-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    align-items: center;
}

.btn-danger {
    background: rgba(220, 50, 50, 0.15);
    border: 1px solid rgba(220, 50, 50, 0.3);
    border-radius: var(--radius);
    color: #e05555;
    font-family: 'DM Sans', sans-serif;
    font-size: 13px;
    font-weight: 500;
    padding: 0.4rem 0.9rem;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
}

.btn-danger:hover:not(:disabled) {
    background: rgba(220, 50, 50, 0.25);
    border-color: rgba(220, 50, 50, 0.5);
}

.btn-danger:disabled { opacity: 0.4; cursor: not-allowed; }
```

- [ ] **Step 10: Build and run all tests**

```bash
cargo build && cargo test
```

Expected: zero errors, all tests pass.

- [ ] **Step 11: Manual verification**

Run the app and verify:
1. `/` shows board list (empty if fresh DB)
2. Creating a board appends it via turbo-stream
3. Clicking the pencil opens the rename form inline; saving updates the name and returns to view state; cancel returns to view state
4. Clicking the trash opens the dialog; the Delete button is disabled until you type the exact board name; typing the name enables it; submitting redirects to `/` with the board gone
5. Navigating to `/boards/{id}` shows the board with a "← Boards" breadcrumb; the "+ Add List" form correctly posts to `/boards/{id}/lists`

- [ ] **Step 12: Commit**

```bash
git add src/handlers/web.rs templates/landing.html templates/turbo_new_board.html templates/turbo_board_header.html templates/base.html src/router.rs
git commit -m "$(cat <<'EOF'
Add board delete with retype-to-confirm dialog

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```
