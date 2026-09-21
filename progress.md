# kanban-rs Progress

## Current Position
**Phase 1, Milestone 4** — CLI interface with `clap`

## Concepts Covered

### Phase 1, Milestone 1 — Project setup
- Cargo as unified build/package/test tool
- `env!` macro for compile-time constants
- Captured identifiers in format strings (`{version}` vs `{}`, version)
- Why `println!` requires a string literal (macros run at compile time)

### Phase 1, Milestone 2 — Core data model
- Structs, enums, `Vec<T>`, `Option<T>`
- `#[derive(Debug)]` and `{:?}` formatting
- `String::from()` vs `.to_string()`
- `mod` vs `use` — module declaration vs import
- Rust module system: explicit declaration, no directory scanning
- `mod.rs` style vs file-per-module style
- Visibility: `pub`, `pub(crate)`, `pub(super)`, default-private
- Path syntax: `crate::`, `super::`, `self::`
- `pub use` re-exports
- Inline modules (`mod tests { ... }`)

### Phase 1, Milestone 3 — In-memory board operations
- `&self` vs `&mut self` — take least permissive reference
- Mutable borrowing rules: one `&mut` at a time
- Borrow ordering: resolve IDs/reads before taking mutable borrows
- `iter()` vs `iter_mut()` — immutable vs mutable iteration
- Two-phase mutation: find+remove first (borrow ends), then push (fresh borrow)
- `Vec::remove(pos)` returns owned `T` — borrow ends, value is yours
- Index-based approach as alternative to reference-based (avoids lifetime issues)
- `find_map` with `?` inside closures
- `Option::is_none()` / `is_some()` over `None ==` comparison
- `+=` operator
- `let _ =` to discard values

## Struggles / Watch For
- Borrow checker: student needed guidance on ordering borrows and the two-phase mutation pattern. Will surface again in Phase 2 with `Arc<Mutex<T>>` — good moment to revisit.
- `.unwrap()` habit: student used double-unwrap in `move_card`. Revisit naturally in Milestone 6 (error handling audit).

## Concepts Covered (continued)

### Phase 1, Milestone 4 — CLI with clap
- clap derive API: `Parser`, `Subcommand`, `Args`
- Nested subcommands: `Parser → Args → Subcommand` chain
- Why tuple variants in `Subcommand` enums require `Args`, not `Subcommand`
- `&str` vs `String` in function signatures: take `&str` to read, `String` to own
- `{}` (Display) vs `{:?}` (Debug) — user-facing output vs developer output
- Dereferencing with `*` to write through a mutable reference
- `Option::as_mut()` to get `&mut T` from `&mut Option<T>`
- Guard clauses with early `return`

### Phase 1, Milestone 5 — Persistence with serde + JSON
- `#[derive(Serialize, Deserialize)]` on structs and enums
- `serde_json::from_reader` with `BufReader` for deserialization
- `serde_json::to_string` + `fs::write` for serialization
- `Result::ok()` to convert `Result` to `Option` (discard error)
- `Option::ok_or` / `ok_or_else` to convert `Option` to `Result`
- `.ok()?` pattern for early-return on failure in `Option`-returning functions
- `map_err` to convert library error types to `String`
- `as_ref()` to get `Option<&T>` from `&Option<T>`
- `&` in for loops to iterate by reference without consuming the collection
- `{}` (Display) vs `{:?}` (Debug) — reinforced repeatedly

### Phase 1, Milestone 6 — Error handling audit
- Custom error enums as the idiomatic alternative to `String` errors
- `From` trait: implementing it so `?` can convert across error type boundaries
- `?` operator: unwraps `Ok`, converts and returns `Err` via `From`
- `ok_or(...)`: converts `Option` → `Result` at the error site
- `let-else`: check + bind in one expression, else branch must diverge
- When `let-else` doesn't apply: `is_some()` guard is still correct when you want to proceed on `None`

## Struggles / Watch For
- One-liner chaining instinct: student noticed the temptation themselves. Reinforce: one transformation per line is readable; beyond that, use named variables.

### Phase 1, Milestone 7 — Traits and display
- `std::fmt::Display` trait: implementing `fmt` to power `{}` formatting
- `write!` / `writeln!` macros: write into a `Formatter` instead of stdout
- `Ok(())` required as the final expression in `fmt` when the body uses a `for` loop
- Composing `Display` implementations: `Board::fmt` delegates to `{}` on `List`, which delegates to `{}` on `Card`
- `write!` vs `writeln!`: choosing carefully to avoid double newlines when the inner type already ends with `\n`
- Blanket impl: implementing `Display` gives `.to_string()` for free
- Trait impl vs derive: `Display` must be written by hand; `Debug` can be derived

## Struggles / Watch For
- Missing `Ok(())`: student forgot it after the `for` loop in `fmt`. Will internalize with repetition.
- `Display` vs `ToString` analogy from C#: student correctly identified it resembles both overriding `ToString()` and implementing an interface simultaneously.

### Phase 1, Milestone 8 — Iterators and functional patterns
- `.flat_map()`: flattens nested iterators (lists of cards) into a single stream
- `.filter()`: keep items matching a predicate
- `.collect()`: materializes a lazy iterator into a `Vec`
- `Vec<_>`: type hole — tell the compiler the container, let it infer the element type
- `&str` vs `&String` in parameters: prefer `&str` — strictly more flexible, accepts both
- Refactoring logic into model methods: handlers parse/print, models own behavior

## Struggles / Watch For
- `FromIterator` confusion: student tried implementing it on `Card` when collect() errored. Root cause was missing type annotation + `.cards` instead of `.cards.iter()`. Clarify: `FromIterator` is for *defining* what collect() produces, not something you implement on the item type.
- String types: student flagged `&str` vs `&String` vs `String` as something to build intuition on. Reinforce naturally as it comes up.

### Phase 1, Milestone 9 — Lifetimes (introduction)
- Every reference has a lifetime — how long it's valid
- Lifetime elision rule 3: if `&self` is a parameter, output lifetime matches `self`
- Explicit annotation syntax: `fn search<'a>(&'a self, ...) -> Vec<&'a Card>`
- `'a` links the output lifetime to the input — "returned refs live as long as self"
- When elision works vs when explicit annotations are needed (multiple input refs, structs holding refs)
- Student intuition: "referencing members and returning from them obviously ties lifetime to the instance" — correct

### Phase 1, Milestone 10 — Generics
- `<T>` type parameters: write once, works for any type
- Trait bounds: `T: HasId` constrains what T must be capable of
- Defining custom traits: `pub trait HasId { fn id(&self) -> u64; }`
- Lifetime + type parameters together: `<'a, T: HasId>`
- `&&T` smell: when `T` is already `&Card`, `find_by_id` returns `Option<&&Card>` — double ref
- Deref coercion: Rust auto-dereferences through multiple levels for `Display` and method calls
- Generic methods are best suited for flat slices; nested data (lists of cards) fights the abstraction
- Future improvement: `HashMap<u64, List>` would make `find_by_id` genuinely useful (O(1) lookup)

### Phase 1, Milestone 11 — Testing
- `#[cfg(test)]` module: only compiled during `cargo test`, not in release builds
- `#[test]` attribute: marks a function as a test case
- `use super::*`: imports parent module's items into the test module
- `assert_eq!`, `assert_ne!`: compare values, panic with a diff on failure
- `.unwrap()` and `.expect("msg")` are idiomatic in tests — a panic is a test failure
- `PartialEq` derive: required for `assert_eq!` on custom types; compiler will tell you when it's needed
- `PartialEq` vs `Eq`: `PartialEq` for most types; `Eq` is the stronger total-equality variant
- Student workflow: compiler-driven derive — not coincidence, just good iteration

### Phase 1, Milestone 12 — HashMap
- `HashMap<K, V>`: key-value store with O(1) insert and lookup
- `HashMap::new()`, `.insert(k, v)`, `.get(&k)`, `.get_mut(&k)`, `.values()`, `.values_mut()`
- Iterating HashMap: `.values()` yields `&V` directly — no `&` needed on the iterator
- `&` on `Vec` iteration vs `.values()` on HashMap: different mechanisms, same effect
- Breaking schema changes: `Vec` → `HashMap` changed the JSON format, old `board.json` unreadable
- Tests as a safety net: all 3 tests passed after the refactor, confirming behavior unchanged

### Phase 1, Milestone 13 — `impl Trait` vs `dyn Trait`
- `impl Trait`: static dispatch — compiler stamps out a concrete version per type, zero cost
- `dyn Trait`: dynamic dispatch — type resolved at runtime via vtable, like C# interface references
- `dyn Trait` is unsized — must always live behind a pointer (`&dyn`, `Box<dyn>`, `Arc<dyn>`)
- `&dyn Trait`: borrow when you just need to read; `Box<dyn Trait>`: own when you need to store
- Prefer `impl Trait` for function parameters; reach for `dyn Trait` for mixed-type collections or runtime-determined types
- C# analogy: `impl Trait` ≅ generics, `dyn Trait` ≅ interface references
- `&str`/`&[T]`/`&dyn Trait` pattern: borrowed form when reading, owned form when storing

### Phase 1, Milestone 14 — Closures and function pointers
- Closure syntax: `|a, b| a.title.cmp(&b.title)` — anonymous functions that capture their environment
- `.sort_by()`: takes a closure returning `Ordering` (Less, Equal, Greater)
- `Ordering`: the return type of `.cmp()`, same concept as C#/Java Comparator
- Closures vs `fn`: closures can capture surrounding scope, plain `fn` cannot
- HashMap O(1) lookup paying off: `get_mut(&id)` with `let-else` — no iteration needed
- Student wrote a test unprompted with correct before/after assertion pattern

### Phase 1, Milestone 15 — Phase 1 cleanup
- Bug fix: `move_card` was removing the card before checking the target list existed — data loss on invalid move
- Removed `HasId`/`find_by_id` dead code — learning exercise, no real production usage
- Test-only `impl` blocks: methods only needed in tests belong inside `#[cfg(test)]` module
- Multiple `impl` blocks for the same type are valid and idiomatic
- Unused imports generate warnings just like unused code — keep them clean
- Lesson: always commit working state before refactoring

## Phase 2

### Phase 2, Milestone 1 — Introduce Tokio, make `main` async
- `#[tokio::main]` is a proc macro — rewrites `async fn main` into a sync `fn main` that sets up the Tokio runtime and calls `block_on`
- Rust has no built-in async runtime — you choose one (Tokio is standard for servers)
- `async fn` returns a `Future` — lazy, does nothing until polled
- Semver ranges in `Cargo.toml`: prefer `"1"` over pinned `"1.52.3"` — Cargo.lock handles reproducibility

### Phase 2, Milestone 2 — Basic HTTP server, health endpoint
- `axum::Router::new().route(path, method(handler))` — declarative routing
- Handler functions: `async fn` returning `impl IntoResponse` (or a concrete type like `Json<Value>`)
- `Json(json!({...}))` — serialize a value and set correct content-type header
- `tokio::net::TcpListener` — async version of std's TcpListener, must `.await` the bind
- `axum::serve(listener, app).await` — drives the server; blocks until shutdown
- CLI layer removed entirely; models/errors temporarily orphaned until routes are wired up

### Phase 2, Milestone 3 — Board read operations via HTTP
- `Arc<Mutex<T>>`: shared ownership (`Arc`) + safe mutable access (`Mutex`) — the pattern for shared state across async handlers
- `MutexGuard` drops at end of scope, releasing the lock — don't hold it across `.await`
- Axum `State` extractor: destructure with `State(inner)` to get the inner value
- `Result<Json<Value>, StatusCode>`: idiomatic handler return type when a route can 404
- `.map().ok_or()` chain on `Option` to produce `Result` without `if let`
- `json!(struct)` works when the struct derives `Serialize` — full nested serialization for free
- Axum 0.8 path syntax: `{id}` not `:id`
- `.ok()?` in `Option`-returning functions: convert `Result` to `Option` and early-return on `None`
- `fs::read_to_string` as a simpler alternative to `File::open` + `BufReader` for small files

### Phase 2, Milestone 4 — Write operations
- `Json<T>` as an extractor (input): deserializes request body, returns 422 on failure before handler runs
- `#[derive(Deserialize)]` on request structs — required for `Json<T>` extraction
- `Path<(u64, u64)>` tuple destructuring for multiple path params — can't use two separate `Path` extractors
- `.map(|_| value).map_err(|_| value)` — transform `Result` arms functionally without `match`
- `axum::routing::post` — register POST handlers alongside `get`
- `Result<StatusCode, StatusCode>` — return type for handlers that succeed/fail with no body
- curl defaults to GET — always use `-X POST` for write operations

### Phase 2, Milestone 5 — Shared mutable state and persistence
- `Arc<Mutex<T>>`: `Arc` = shared ownership, `Mutex` = safe mutation — neither alone is sufficient
- Never hold `MutexGuard` across `.await` — guard isn't `Send`, task may resume on different thread
- Block scope pattern: lock → mutate → serialize → drop guard → `.await` write
- `&*guard` to get `&T` from `MutexGuard<T>` — explicit deref coercion through the smart pointer
- `MutexGuard<T>` implements `Deref<Target = T>` — same mechanism as `Box<T>`, `Arc<T>`
- Only persist on success — check `result.is_ok()` before writing to disk

### Phase 3, Milestone 1 — App structure
- Split monolithic `main.rs` into `handlers.rs`, `router.rs`, `state.rs`
- `Router<S>` type parameter: S = "state still needed"; `.with_state()` converts to `Router<()>`
- `axum::serve` requires `Router<()>` — the state must be fully provided before serving
- `pub` on handler fns required for router module to see them; request structs need matching visibility
- Type alias `ApplicationState = Arc<Mutex<Board>>` — readable handler signatures throughout

### Phase 3, Milestone 2 — Query parameters
- `Query<T>` extractor: deserializes URL query string into a struct using serde field names as keys
- `#[derive(Deserialize)]` on query structs — same as JSON body structs
- `axum::extract::Query` alongside `Path`, `State`, `Json`
- `#[serde(rename = "...")]` available to decouple field name from query param name

### Phase 3, Milestone 3 — Proper error handling
- `impl IntoResponse for KanbanError`: map domain errors to HTTP status + body automatically
- Handlers return `Result<T, KanbanError>` — Axum calls `into_response()` on the error variant
- `?` on fallible model methods: propagates `KanbanError` and removes `.map_err()` noise
- `ok_or(KanbanError::...)`: convert `Option` → `Result` when model returns `None`
- Block scope + `?`: early-return on error before releasing lock, serialize inside block
- `to_string()` on `Display` types reuses existing impl — avoid duplicating message strings

### Phase 3, Milestone 4 — Application state and dependency injection
- Dependency injection: router accepts state as a parameter rather than constructing it internally
- `main` owns state construction — the right place for wiring concerns together
- `match` on `TcpListener::bind` for explicit error handling at startup
- `axum::serve(...).await` blocks until shutdown — startup messages must precede it, not follow
- `if let Err(e)` on serve to handle shutdown errors without panicking
- Named constants for config values (`LOCAL_ADDRESS`) — avoids magic strings

### Phase 3, Milestone 5 — SQLite via sqlx
- `SqlitePool` replaces `Arc<Mutex<Board>>` — connection pool handles concurrency, no manual locking
- `sqlx::query().bind().fetch_one/fetch_optional/fetch_all/execute` — the core query API
- `.bind()` for safe parameter passing — never interpolate values into SQL strings
- `.get::<Type, _>("column")` to read typed column values from a row — requires `use sqlx::Row`
- `u64` doesn't map to SQLite — cast to `i64` with `as i64` before binding
- `Option<String>` for nullable columns — `.get::<Option<String>, _>()` handles NULL correctly
- `fetch_optional` returns `Option<Row>` — use `.ok_or(KanbanError::...)` to convert to 404
- `.execute()` returns a result with `.last_insert_rowid()` for getting new IDs
- `FROM<sqlx::Error> for KanbanError` — `?` converts database errors automatically
- `INSERT OR IGNORE` for idempotent seeding at startup
- `LIKE ?` with `format!("%{}%", keyword)` for wildcard search — wildcards go in the bound value
- `models.rs` entirely replaced by inline SQL queries — no intermediate model layer needed

### Phase 4 — Turbo, CSRF, and card status removal
*(This phase happened across several sessions before this log was brought current — summarized from commit history rather than turn-by-turn notes.)*

- Hotwire Turbo integration: `turbo-stream` responses for board/list/card partials, `turbo-frame` for inline edit forms, a `turbo-stream-source` websocket for live updates
- CSRF protection built from scratch: `csrf.rs` (secret generation, HMAC-style mask/verify), `require_csrf_token` middleware parsing `application/x-www-form-urlencoded` bodies for a `csrf_token` field, a per-session secret stored via `tower-sessions`
- Every form template threading a hidden `csrf_token` input through `macros.html`
- Per-user CSRF secrets extended to the websocket endpoint (`ws.rs`)
- Base test helpers consolidated (`base_auth_get_assertion`, `base_csrf_rejection_assertion`, etc.) to cut duplicate test setup across `boards.rs`/`cards.rs`/`lists.rs`
- Card `status` removed entirely (column, model field, queries, tests) in favor of `sort_order` — ordering, not a status enum, is now how a card's position is tracked
- Minor UI/layout pass: `:focus-visible` outline for buttons/links, Turbo stream inserts placing new lists/boards before the add-form instead of appending after it

### Phase 5 — Drag-and-drop card reordering
The board already had client-side drag-and-drop (Sortable.js) but reorders didn't survive a page reload — nothing persisted them. This phase closed that gap end-to-end: client, CSRF, and a new Rust endpoint with a DB transaction.

- Split the inline Stimulus/Sortable script out of `base.html` into `static/js/board.js` as a real ES module (`<script type="module" src="...">`), including that module scripts are deferred automatically and run in their own scope (no accidental globals)
- Diagnosed a live CSRF bug: `fetch`'s `body: someFormDataInstance` sends `multipart/form-data`, but `require_csrf_token` only parses `application/x-www-form-urlencoded` — switched to `URLSearchParams` so the client's encoding actually matches what the middleware expects
- Extended `require_csrf_token` to also accept the token via an `X-CSRF-Token` header (for JSON POSTs, where a form-encoded body doesn't make sense) — the header check reads `parts.headers.get(...)` directly rather than binding the whole `HeaderMap` to a variable, so `parts` stays whole and movable into `Request::from_parts` afterward
- `Cow<'a, str>` from `form_urlencoded::parse`: `.to_string()` detaches an owned value that can escape a closure; `.as_ref()`/`Deref` ties the output lifetime to the local borrow instead, which can't escape — this is *why* the existing form-token code already used `.to_string()`
- Verified `verify("")` fails safely (its `token.split(":")` length check rejects it before any indexing) — meant collapsing "no token found" to an empty-string sentinel was safe to do without an `Option` check first
- Designed the reorder endpoint's shape through several iterations: rejected embedding `list_id` inside each card's position data once it became clear the existing `/lists/{id}/cards/{id}/move` endpoint already owns `list_id`, leaving the new endpoint to do one thing — `UPDATE cards SET sort_order = ? WHERE card_id = ?` per card, no list scoping needed at all
- `sqlx::Pool::begin()` → `Transaction`, looping `.execute(&mut *tx)`, `tx.commit().await?` — and *why* `&mut *tx` is required: `Transaction` only implements `Deref`/`DerefMut` to the underlying connection, not `Executor` itself, and Rust's automatic deref coercion doesn't apply when the target is a generic trait-bound parameter being inferred, only when there's a known concrete expected type
- `axum::Json<T>` (a request/response extractor, `FromRequest`/`IntoResponse`) vs `serde_json::Value` (just data) — conflated once, corrected: a test helper building a request to *send* has no axum extraction happening, so it wants a plain serializable value, not `Json<T>`
- Test helper design: a shared helper taking `Option<T: Serialize>` needs `T` concrete at every call site (including `None::<T>` ones) if it's generic — `serde_json::Value`-style concreteness avoids forcing turbofish annotations everywhere the helper is already called with `None`
- `pub` makes an item reachable from other modules, but doesn't add it to another module's namespace — still need an explicit `use` at each call site (`E0425: cannot find function` despite the function existing and being `pub`)
- A `TestResponse` that's `.await`ed but never bound to a variable, with no `assert_*` call on it, compiles clean with no warning and the test passes regardless of the actual response — passing, but testing nothing

## Struggles / Watch For
- CSRF body-encoding mismatches bit twice in a row: once via `FormData` (multipart) against a form-urlencoded-only middleware, once in reasoning about a hypothetical JSON body against the same middleware before it was extended. Watch for this pattern recurring — "the client's Content-Type doesn't match what the server parses" is an easy one to reintroduce when adding a new endpoint.
- Confused `elements.length` (the rest-parameter argument count — 1 or 2 depending on same-list vs. cross-list) with "how many cards actually changed." The fix was checking the *flattened* array's length instead. Worth remembering when reasoning about rest parameters generally: the outer arity and the inner data size are unrelated.
- The `&mut *tx` deref-for-a-generic-bound explanation landed as genuinely new/hard ("I would need some time to really learn that") — good candidate to revisit once more generics/trait material comes up again, per the existing Milestone 10 note on this same territory (`Deref` coercion, trait bounds).

### Phase 5 (continued) — Same-list reorder broadcast + per-user websocket scoping
Closed the "next candidate" gap noted below: `post_list_sort_order` now broadcasts a `SocketEvents::ListsSorted(CardSortUpdate { list_ids })` event, rendered via `turbo_card_sort_refresh.html` into one `<turbo-stream action="reload" target="list-{id}">` per affected list. `reload` isn't a native Turbo Stream action — added a custom `Turbo.StreamActions.reload` in `static/js/turbo_extensions.js` that calls `frame.reload()`.

While wiring this up, caught a real bug on their own: the websocket broadcast channel (`ApplicationState.tx`) was global, but the DB layer is already per-user (`db_pools: DashMap<String, SqlitePool>` keyed by a hash of user id + source). A global `tx` meant every connected user would see every other user's card moves. Refactored to `tx_pool: Arc<DashMap<String, SocketEventsSender>>` with a `UserBroadcast` extractor mirroring `UserDb` exactly (same `get_database_id` keying, lazy-create-on-first-request pattern).

- Recognized an existing pattern (`db_pools` + `UserDb`) needed to be applied to a second resource (`tx`) for the same reason (multi-tenant isolation) — not prompted, self-diagnosed from reading their own code
- Copy-a-pattern trap: `get_or_create_ws_sender` initially returned `Result<SocketEventsSender, KanbanError>` purely by mirroring `get_or_create_pool`'s shape, even though neither branch could fail (`DashMap` ops and `Sender::new` aren't fallible, unlike the DB connect that justifies `Result` in the original). Caught after a nudge, fixed cleanly — dropped `Result` from the fn and the `tx?` at the call site
- The refactor's commit (`b195f27`) broke `cargo test` compilation (`cargo build` stayed green, masking it) — `git status` clean is not proof tests compile. Root cause: one shared test fixture (`get_fake_application_state` in `handlers.rs`) still constructed `ApplicationState { tx, ... }`
- Fixed independently across two follow-up commits: updated the fixture, then hit a second-order problem — `base_test_server`'s synthesized `user_id` (hashed from the route path) wasn't exposed, so a test needing the *specific* per-user channel out of `tx_pool` had no way to derive the same key `UserBroadcast` would
- First instinct for that gap was "just grab the only entry in the DashMap" (works today, single-user-per-test) — talked through why that doesn't actually test per-user isolation and would rot the moment a test exercises two users; landed on adding `user_id` to `base_test_server`'s return tuple instead, so the test recomputes the same `get_database_id` key the extractor uses
- Result: `websocket_will_broadcast_card_move_events` now genuinely proves the right channel is used for the right user, not just that *a* channel receives the message

### Phase 5 (continued) — Websocket error handling fix
`handle_socket` was rendering the outgoing message with `.render().map_err(|e| format!("<div>{e}</div>")).unwrap()` — the `.unwrap()` on a `Result<String, String>` discarded exactly the error text `map_err` had just formatted, panicking the task instead of ever sending it. Fixed by sending whichever `String` came out of either arm.

- `unwrap_or_else(|e| e)` on a `Result<String, String>`: both arms are the same type, so the "error" can be extracted and used directly instead of panicking — a `Result` doesn't have to mean success/failure semantically, just "which branch produced this value"

### Phase 5 (continued) — Cleanup pass (Claude-flagged fixes)
A batch of fixes from feedback Claude gave on the existing code, spanning error handling and a dead shared-helper split.

- `create_redis_pool` extracted out of `create_application_state`: the inline block used three `.unwrap()`s (`Config::from_url`, `Pool::new`, `wait_for_connect`) that would panic at startup if Redis was misconfigured or unreachable; now returns `Result<Pool, KanbanError>` and the call site does `.ok()` to fall back to `None` (sessions/redis stay optional) instead of crashing
- `impl From<tower_sessions_redis_store::fred::error::Error> for KanbanError` added so `?` can convert redis errors the same way other error types already do
- Resolved a long-standing TODO by recognizing `create_list_common` was serving two genuinely different response shapes wearing one signature: the web-form path renders a CSRF-templated `turbo-stream` HTML fragment, the JSON API path needs neither the template nor a CSRF token. The shared helper's `"_".to_string()` CSRF-token placeholder for the API path was a symptom of forcing both through one function
- Deleted `create_list_common`, its `ListItemTemplate`, and the now-pointless test from `handlers.rs`; `ListItemTemplate` moved into `handlers/web/lists.rs` where it's actually used; `handlers/api/lists.rs` now builds its JSON response directly with `serde_json::json!`, no template, no CSRF token in an API response at all

## Current Position
**Post-curriculum, feature work.** CSRF protection, drag-and-drop reordering, live same-list-reorder broadcast, and the follow-up cleanup pass are all done and committed. `cargo build` is green. Card `status` has been fully replaced by list membership + `sort_order`.

Working tree is clean — nothing pending. Open for new feature work.

No specific next feature queued yet — last session was entirely a catch-up/review + fixing test-suite drift after independent feature work. Good moment to ask the student what they want to tackle next, or propose: broadcasting board-level events (new list/card added) the same way card moves and sorts already are, since that's the one remaining gap between "some things are live" and "the whole board is live."
