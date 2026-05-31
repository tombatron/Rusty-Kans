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

## Next Task
**Milestone 6: Error handling audit**

Replace all `.unwrap()` calls with proper `Result` propagation. This means:
- Defining a custom error type (an enum) for the application
- Implementing `From` for library error types so `?` works across error boundaries
- Propagating errors up to `main` and printing them cleanly
- No panics in production code paths
