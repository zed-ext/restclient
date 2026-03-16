# Contributing to REST Client for Zed

Thank you for your interest in contributing!

## Prerequisites

- Rust (stable) via [rustup](https://rustup.rs/)
- Zed IDE
- `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

## Development Setup

```bash
git clone https://github.com/anthropics/zed-restclient.git
cd zed-restclient

cargo build --release --target wasm32-wasip1

cd lsp && cargo build --release && cd ..

./install_to_zed.sh
```

Restart Zed completely after installing.

## Project Architecture

```
restclient/
├── src/
│   └── lib.rs                  # WASM extension entry point (Zed integration)
├── lsp/
│   ├── src/
│   │   ├── main.rs             # LSP server (code lens, completions, symbols, runnables)
│   │   ├── parser.rs           # HTTP file parser
│   │   ├── variables.rs        # Variable resolution engine
│   │   ├── lib.rs              # Shared library exports
│   │   └── bin/
│   │       └── http-client.rs  # CLI binary (colored output, spinner, request execution)
│   └── Cargo.toml
├── languages/http/
│   ├── config.toml             # Language configuration
│   ├── highlights.scm          # Syntax highlighting queries
│   ├── runnables.scm           # Tree-sitter runnable captures (▶ buttons)
│   └── tasks.json              # Task definitions (Send Request / Send in New Tab)
├── grammars/                   # Tree-sitter HTTP grammar
├── extension.toml              # Extension manifest
├── Cargo.toml                  # WASM crate config
└── install_to_zed.sh           # Build + install script
```

### Key Components

**WASM Extension** (`src/lib.rs`): Entry point registered with Zed. Locates and launches the LSP binary. Cannot use tokio/reqwest (WASM constraint).

**LSP Server** (`lsp/src/main.rs`): Provides code lens (Generate cURL), completions, document symbols, and runnables. Independent workspace crate.

**HTTP Parser** (`lsp/src/parser.rs`): Parses `.http` files into structured request blocks. Handles methods, headers, bodies, variables, and `###` separators.

**Variable Resolver** (`lsp/src/variables.rs`): Resolves `{{variable}}` references with support for file-level `@var = value`, environment files, and nested resolution.

**CLI Binary** (`lsp/src/bin/http-client.rs`): Executes HTTP requests from the terminal. Features colored JSON output, braille spinner with elapsed time, and precise timing display.

## Development Workflow

1. Make changes
2. Build and test:
   ```bash
   cd lsp && cargo test && cargo clippy -- -D warnings && cd ..
   cargo build --release --target wasm32-wasip1
   ```
3. Install and test in Zed:
   ```bash
   ./install_to_zed.sh
   # Restart Zed, open a .http file, verify changes
   ```

## Code Style

```bash
cargo fmt
cargo clippy -- -D warnings
```

Both the root WASM crate and the `lsp/` crate should pass formatting and clippy checks.

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all checks pass:
   ```bash
   cargo fmt -- --check
   cargo clippy --target wasm32-wasip1 -- -D warnings
   cd lsp && cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
   ```
5. Open a PR with a clear description of what and why

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
