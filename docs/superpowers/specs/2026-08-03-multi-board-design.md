# Multi-Board Design

**Date:** 2026-08-03
**Status:** Approved

## Overview

Replace the single hardcoded board with a full multi-board experience. Users land on a board management page, can create/rename/delete boards, and navigate into any board's kanban view.

## Routes

| Method | Path | Handler | Notes |
|--------|------|---------|-------|
| GET | `/` | `get_landing` | Landing page — lists all boards |
| POST | `/boards` | `post_board_form` | Create board → turbo-stream appends to list |
| GET | `/boards/{board_id}` | `get_board` | Board view (replaces `get_index`) |
| GET | `/boards/{board_id}/header` | `get_board_header` | Turbo frame — view state |
| GET | `/boards/{board_id}/edit` | `get_board_edit` | Turbo frame — rename form |
| POST | `/boards/{board_id}/rename` | `post_board_rename` | Rename → redirect to `/boards/{board_id}/header` |
| POST | `/boards/{board_id}/delete` | `post_board_delete` | Delete board → redirect to `/` |
| POST | `/boards/{board_id}/lists` | `post_list_form` | Replaces `POST /lists` |

All existing `/lists/{list_id}/...` and `/cards/{card_id}/...` routes are unchanged.

## Models

Add `Board` to `models.rs`:

```rust
#[derive(Debug, Clone, FromRow)]
pub struct Board {
    #[sqlx(rename = "board_id")]
    pub id: u64,
    pub name: String,
}
```

`List` is unchanged — `board_id` is always known from the URL, no need to carry it on the struct.

## Database

No new migration. The `boards` table and `lists.board_id` FK already exist in `0001_initial_schema.sql`.

Remove the hardcoded board seed from `create_application_state` in `state.rs`. An empty board list is a valid starting state once the landing page exists.

## Templates

### `landing.html`
Extends `base.html`. Displays all boards as cards in a grid/list. Each board card contains:
- A title link to `/boards/{id}`
- An edit (pencil) button — navigates the board's turbo frame to the edit form
- A delete (trash) button — opens an inline `<dialog>` confirmation modal

Includes a "New Board" form (name input + submit button) at the top.

### `turbo_new_board.html`
Turbo Stream that appends a newly created board card to the board list. Same pattern as `turbo_list_item.html`.

### `turbo_board_header.html`
Turbo Frame — view state. Renders board name with edit and delete buttons. Same pattern as `turbo_list_title.html`.

### `turbo_board_header_edit.html`
Turbo Frame — edit state. Renders rename form posting to `/boards/{id}/rename`, cancel link to `/boards/{id}/header`. Same pattern as `turbo_list_title_edit.html`.

### `board.html` (updated)
Add a breadcrumb link back to `/` in the board header area.

## Delete Confirmation Modal

Each board card on the landing page includes a native `<dialog>` element rendered inline. The delete button opens it via `dialog.showModal()`. The dialog contains:

- A warning message identifying the board by name
- A text input requiring the user to type the board name exactly
- A "Delete" submit button disabled until the input matches the board name (enforced client-side via an `input` event listener)
- A "Cancel" button that closes the dialog

The form inside the dialog POSTs to `/boards/{board_id}/delete`. Server-side simply deletes the board; the name-match check is UX-only (SQLite CASCADE handles list and card cleanup).

## Handlers

All handlers live in `src/handlers/web.rs`.

- **`get_landing`** — `SELECT board_id, name FROM boards ORDER BY board_id`, renders `landing.html`.
- **`post_board_form`** — inserts new board, retrieves generated ID via `last_insert_rowid()`, renders `turbo_new_board.html`.
- **`get_board`** — current `get_index` logic with `Path(board_id): Path<u64>` replacing the `LIMIT 1` query.
- **`get_board_header`** — fetches board by ID, renders `turbo_board_header.html`.
- **`get_board_edit`** — fetches board by ID, renders `turbo_board_header_edit.html`.
- **`post_board_rename`** — updates board name, redirects to `/boards/{board_id}/header`.
- **`post_board_delete`** — deletes board (CASCADE cleans up lists and cards), redirects to `/`.
- **`post_list_form`** — gains `Path(board_id): Path<u64>`, passes `board_id` to `create_list_common`. `create_list_common` in `handlers.rs` also changes: its signature gains a `board_id: u64` parameter, and its SQL changes from `INSERT INTO lists ... SELECT board_id FROM boards LIMIT 1` to `INSERT INTO lists (board_id, name) VALUES (?, ?)`.

## State

Remove from `create_application_state`:
```rust
sqlx::query("INSERT OR IGNORE INTO boards (board_id, name) VALUES (1, 'My Board')")
    .execute(&db)
    .await
    .unwrap();
```

## Out of Scope

- Per-board WebSocket scoping (card move events from other boards are silently ignored by Turbo since the DOM targets won't exist)
- Board ordering / drag-to-reorder
- Board membership / sharing
