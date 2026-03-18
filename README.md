# Writing Unicorns — Language Modules

Standalone, testable language support crates for the Writing Unicorns IDE.

## Structure
- `rust-lang/` — Rust syntax tokenizer  
- `typescript-lang/` — TypeScript syntax tokenizer
- `javascript-lang/` — JavaScript syntax tokenizer

## Running tests
```bash
cargo test                        # all modules
cargo test -p rust-lang           # just Rust
cargo test -p typescript-lang     # just TypeScript
```

## Adding a new language module
1. Create a new directory: `my-lang/`
2. Add `Cargo.toml` with `crate-type = ["rlib", "cdylib"]`
3. Implement `tokenize_line(line: &str) -> Vec<Token>`
4. Export `language_id()` and `file_extensions()` FFI functions
5. Add to workspace `Cargo.toml` members
6. Add to Writing Unicorns IDE as an extension

## FFI Interface
Each module exports:
- `language_id() -> *const c_char` — language identifier
- `file_extensions() -> *const c_char` — comma-separated extensions
- `tokenize_line_ffi(line: *const c_char) -> *mut c_char` — JSON token array
- `free_string(ptr: *mut c_char)` — free returned string
