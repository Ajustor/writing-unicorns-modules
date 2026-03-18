# rust-lang

Rust language support for Writing Unicorns IDE.

## LSP Server

This module uses `rust-analyzer` for full language intelligence.

### Installation
```bash
rustup component add rust-analyzer
# or
cargo install rust-analyzer
```

## Features
- Syntax highlighting
- Hover signatures (via rust-analyzer)
- Go-to-definition (via rust-analyzer)
- Auto-completion (via rust-analyzer)
- Diagnostics / error highlighting (via rust-analyzer)
