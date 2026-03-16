# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.1] - 2026-03-14

### Added
- ▶ Send Request button (reuses terminal) and ⊕ Send in New Tab button (new terminal per request)
- Color-cycling braille spinner with elapsed time during requests
- Syntax-highlighted JSON response output (keys, strings, numbers, booleans, null)
- Precise elapsed time display (e.g. `3.315s`, `42ms`)
- Hidden task runner output lines for clean terminal display

### Changed
- Cleaned up codebase for open-source release
- Removed unused dependencies and dead code
- Updated documentation to reflect current architecture

## [0.2.0] - 2026-03-10

### Added
- LSP code lens for "Generate cURL" command
- HTTP autocompletion (methods, headers, header values, variable references)
- Document symbols for request navigation
- Variable support with `@var = value` and `{{var}}` substitution
- Environment file support (`.http-client/environments.json`)
- Nested variable resolution

## [0.1.0] - 2026-03-07

### Added
- Initial release
- HTTP file parser supporting JetBrains HTTP Client format
- Syntax highlighting for `.http` and `.rest` files
- Tree-sitter grammar integration
