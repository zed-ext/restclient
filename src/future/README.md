# Future Features

This directory contains modules for features that are planned but not yet implemented in the current release.

## Planned Modules

- **executor.rs** - Advanced request executor with Zed HTTP API integration
- **file_loader.rs** - File loading utilities for request bodies and handlers
- **formatter.rs** - Response formatters for different content types
- **handler.rs** - JavaScript response handler interpreter
- **history.rs** - Request history tracking and management
- **response.rs** - Response data structures and utilities
- **simple_executor.rs** - Simplified executor implementation
- **lsp.rs** - Language Server Protocol integration (if needed)

These will be integrated in future releases as the extension matures.

For the current implementation, see:
- `parser.rs` - HTTP request parsing (active)
- `variables.rs` - Variable resolution (active)
- `bin/http_execute.rs` - CLI execution with reqwest (active)
