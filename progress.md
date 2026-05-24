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

## Next Task
**Milestone 4: CLI interface with `clap`**

Add `clap` as a dependency and build a real CLI. The app should support subcommands:
- `kanban board create <name>`
- `kanban list add <name>`
- `kanban card add <list-id> <title> [--description <desc>]`
- `kanban card move <card-id> <to-list-id>`
- `kanban show` — debug-print the board

For now, the board lives only in memory (loaded fresh each run). Persistence comes in Milestone 5.
