# Project Structure

```
restclient/
├── README.md                   # Main documentation
├── LICENSE                     # MIT License
├── Cargo.toml                  # Rust dependencies and build config
├── extension.toml              # Zed extension manifest
├── .gitignore                  # Git ignore rules
│
├── docs/                       # Documentation
│   ├── SETUP.md               # Quick setup guide
│   ├── CURSOR_BASED.md        # Cursor-based execution guide
│   └── CONTRIBUTING.md        # Contributing guidelines
│
├── src/                        # Rust source code
│   ├── lib.rs                 # Extension entry point (WASM)
│   ├── parser.rs              # HTTP request parser
│   ├── variables.rs           # Variable resolution system
│   │
│   ├── bin/
│   │   └── http_execute.rs   # CLI tool for executing requests
│   │
│   └── [future modules]       # Planned features
│       ├── executor.rs        # Request executor
│       ├── handler.rs         # Response handlers
│       ├── response.rs        # Response formatting
│       ├── file_loader.rs     # File loading utilities
│       ├── formatter.rs       # Response formatters
│       └── history.rs         # Request history
│
├── languages/http/             # Language configuration
│   ├── config.toml            # Language settings
│   ├── highlights.scm         # Syntax highlighting rules
│   └── tasks.json             # Task definitions (cursor-based execution)
│
├── grammars/
│   └── http.wasm              # Compiled Tree-sitter grammar
│
├── test.http                   # Example HTTP requests
└── install_to_zed.sh          # Installation script
```

## Key Files

### Core Extension

- **src/lib.rs** - WASM extension entry point, handles Zed integration and slash commands
- **src/parser.rs** - Parses .http files into structured HttpRequest objects
- **src/bin/http_execute.rs** - CLI binary that executes HTTP requests with reqwest

### Configuration

- **extension.toml** - Extension metadata and configuration
- **languages/http/config.toml** - Language-specific settings
- **languages/http/highlights.scm** - Syntax highlighting rules
- **languages/http/tasks.json** - Task definition for "Send Request at Cursor"

### Grammar

- **grammars/http.wasm** - Compiled Tree-sitter grammar (from external repo)

## Build Artifacts (gitignored)

- `target/` - Rust build output
- `extension.wasm` - Built extension
- `*.log` - Log files
- Old markdown docs (superseded by docs/ folder)

## Development Workflow

1. Edit source code in `src/`
2. Build: `cargo build --release --target wasm32-wasip1`
3. Install: `./install_to_zed.sh`
4. Test in Zed with `.http` files
