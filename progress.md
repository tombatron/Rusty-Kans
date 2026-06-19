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

## Next Task
**Milestone 14: Closures and function pointers**

Add a `card sort` command that sorts cards within a list by a chosen field (title, id). This means:
- Closure syntax: `|args| body`
- Passing closures to methods like `.sort_by()`
- `Fn`, `FnMut`, `FnOnce` trait bounds
- When to use closures vs function pointers (`fn`)
