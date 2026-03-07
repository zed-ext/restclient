# Contributing to REST Client for Zed

Thank you for your interest in contributing! This guide will help you get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Architecture](#project-architecture)
- [How to Contribute](#how-to-contribute)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing](#testing)
- [Common Tasks](#common-tasks)
- [Pull Request Process](#pull-request-process)

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo
- Git
- Zed IDE (for testing)
- Basic understanding of HTTP and REST APIs

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/zed-restclient.git
cd zed-restclient
```

## Development Setup

### 1. Install Dependencies

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy (linter)
cargo clippy -- -D warnings
```

### 2. Install in Zed (Development Mode)

```bash
# Create symlink to Zed extensions directory
ln -s $(pwd) ~/.config/zed/extensions/restclient

# Or copy the directory
cp -r . ~/.config/zed/extensions/restclient
```

### 3. Development Workflow

```bash
# Make changes to code
vim src/parser.rs

# Run tests
cargo test

# Test in Zed
# 1. Reload Zed (Cmd+R or restart)
# 2. Open a .http file
# 3. Test your changes
```

## Project Architecture

### Directory Structure

```
restclient/
├── src/
│   ├── lib.rs           # Extension entry point, Zed integration
│   ├── parser.rs        # HTTP request parsing logic
│   ├── variables.rs     # Variable resolution system
│   ├── file_loader.rs   # File loading utilities
│   ├── executor.rs      # HTTP request execution
│   ├── response.rs      # Response data structures
│   ├── formatter.rs     # Response formatting/display
│   ├── history.rs       # Request history management
│   └── handler.rs       # JavaScript response handlers
├── grammars/http/       # Tree-sitter grammar
├── languages/http/      # Syntax highlighting config
├── tests/
│   ├── fixtures/        # Test .http files
│   └── integration/     # Integration tests
├── examples/            # Documentation examples
└── .http-client/        # Sample environments
```

### Module Overview

#### lib.rs (Extension Entry Point)
- Registers extension with Zed
- Manages shared VariableResolver
- Creates RequestExecutor instances
- TODO: Command registration

#### parser.rs (Request Parsing)
- Parses `.http` files into `HttpRequest` structures
- Handles all JetBrains HTTP spec features
- Splits requests by `###` separator
- Extracts request line, headers, body, handlers

**Key Functions:**
- `parse_http_file(content)` - Main entry point
- `parse_request_block(block)` - Parse single request
- `parse_request_line(line)` - Extract method/URL/version
- `parse_headers(lines)` - Parse header lines
- `parse_body(lines)` - Parse request body

#### variables.rs (Variable Management)
- 4-tier variable resolution: file → global → environment → system
- Environment loading from JSON
- Variable substitution with `{{variable}}` syntax
- Nested variable resolution

**Key Functions:**
- `load_environments(path)` - Load environment JSON
- `set_active_environment(name)` - Switch environment
- `resolve(template)` - Substitute all variables
- `parse_file_variables(content)` - Extract `@var = value`

#### executor.rs (Request Execution)
- Executes HTTP requests asynchronously
- Resolves variables before execution
- Loads file bodies
- Executes response handlers
- Captures timing information

**Key Functions:**
- `execute(request)` - Execute single request
- `execute_all(requests)` - Execute multiple requests
- `execute_handler(handler, request, response)` - Run JS handler

#### handler.rs (JavaScript Handlers)
- Pattern-based JavaScript interpreter
- Supports `client.global.set()`, `client.test()`
- JSON.parse() with path navigation
- Test execution and assertion

**Key Functions:**
- `execute(script, context)` - Run handler script
- `execute_statement(stmt)` - Execute single statement
- `evaluate_expression(expr)` - Evaluate JS expression

#### formatter.rs (Response Formatting)
- Pretty-prints responses
- JSON formatting with indentation
- Hex dump for binary content
- Multiple output formats

**Key Functions:**
- `format(result)` - Detailed response display
- `format_compact(result)` - One-line summary
- `format_json(json)` - Pretty-print JSON

#### history.rs (History Management)
- JSON file-based storage
- Search and filtering
- Statistics aggregation
- Automatic cleanup

**Key Functions:**
- `save(result)` - Store execution result
- `get_all()` - Retrieve all history
- `search_by_url(pattern)` - Search history

## How to Contribute

### Types of Contributions

1. **Bug Fixes**: Fix issues in existing functionality
2. **New Features**: Add new capabilities
3. **Documentation**: Improve guides and examples
4. **Tests**: Add test coverage
5. **Performance**: Optimize code
6. **Examples**: Add .http file examples

### Finding Issues

- Check GitHub Issues for "good first issue" label
- Look for "help wanted" issues
- Check TODO comments in code

### Contribution Ideas

#### Easy
- Add more test fixtures
- Improve error messages
- Add examples to documentation
- Fix typos and grammar
- Add JSDoc comments

#### Medium
- Add new JavaScript patterns to handler
- Implement XML/HTML pretty-printing
- Add more HTTP methods support
- Improve response formatting
- Add environment variable validation

#### Hard
- Implement multipart form data
- Add full JavaScript interpreter
- Implement LSP for code completion
- Add streaming response support
- Implement WebSocket support

## Code Style Guidelines

### Rust Style

Follow standard Rust conventions:

```rust
// Use descriptive names
fn parse_http_request(content: &str) -> Result<HttpRequest, String> {
    // Implementation
}

// Document public APIs
/// Parse an HTTP request file into structured requests
///
/// # Arguments
/// * `content` - The .http file content as a string
///
/// # Returns
/// A vector of parsed HttpRequest structures
pub fn parse_http_file(content: &str) -> Result<Vec<HttpRequest>, String> {
    // Implementation
}

// Use ? for error propagation
let requests = parse_http_file(content)?;

// Prefer match over if-let when handling multiple cases
match request.body {
    Some(RequestBody::Inline(content)) => { /* ... */ }
    Some(RequestBody::FileReference(path)) => { /* ... */ }
    None => { /* ... */ }
}
```

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix
```

### Error Handling

- Use `Result<T, String>` for operations that can fail
- Provide descriptive error messages
- Use `?` operator for error propagation
- Log errors with context

```rust
// Good
fn load_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {:?}: {}", path, e))
}

// Bad
fn load_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test parser

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration
```

### Writing Tests

#### Unit Tests

Add tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let content = "GET https://api.example.com/users";
        let requests = parse_http_file(content).unwrap();

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, HttpMethod::GET);
        assert_eq!(requests[0].url, "https://api.example.com/users");
    }
}
```

#### Integration Tests

Add tests in `tests/integration/`:

```rust
#[tokio::test]
async fn test_request_execution_flow() {
    // Test complete flow from parsing to execution
}
```

#### Test Coverage

Aim for:
- 80%+ line coverage
- All public APIs tested
- Edge cases covered
- Error paths tested

### Test Fixtures

Add `.http` files to `tests/fixtures/`:

```http
### Test case name
GET https://api.example.com/test

### Expected: 200 OK
### Tests: variable resolution, JSON response
```

## Common Tasks

### Adding a New JavaScript Pattern

1. **Add pattern to handler.rs**:

```rust
// In ResponseHandlerRuntime
fn match_new_pattern(&self, stmt: &str) -> Option<(String, String)> {
    let re = Regex::new(r#"pattern_regex"#).unwrap();
    // Parse pattern
}
```

2. **Add to execute_statement**:

```rust
fn execute_statement(&mut self, stmt: &str, context: &HandlerContext) -> Result<(), String> {
    // Add new pattern check
    if let Some(captures) = self.match_new_pattern(stmt) {
        // Handle pattern
        return Ok(());
    }
    // ... existing patterns
}
```

3. **Add test**:

```rust
#[test]
fn test_new_pattern() {
    let runtime = ResponseHandlerRuntime::new(resolver);
    let script = "new.pattern.syntax()";
    runtime.execute(script, context).unwrap();
    // Assert behavior
}
```

4. **Document in examples/handler_examples.md**

### Adding a New Variable Source

1. **Extend VariableResolver**:

```rust
pub struct VariableResolver {
    // ... existing fields
    custom_variables: HashMap<String, String>,
}

impl VariableResolver {
    pub fn set_custom(&mut self, name: String, value: String) {
        self.custom_variables.insert(name, value);
    }
}
```

2. **Update get_variable() resolution order**

3. **Add tests**

4. **Document in README.md**

### Adding a New Response Format

1. **Add formatter in formatter.rs**:

```rust
impl ResponseFormatter {
    fn format_yaml(yaml_str: &str) -> String {
        // Implement YAML formatting
    }
}
```

2. **Update format_body()**:

```rust
fn format_body(response: &HttpResponse) -> String {
    if response.is_yaml() {
        return Self::format_yaml(&body_str);
    }
    // ... existing formats
}
```

3. **Add content-type detection in response.rs**

### Extending the Parser

1. **Update grammar.js** if syntax changes
2. **Modify parse_* functions in parser.rs**
3. **Add tests for new syntax**
4. **Update test fixtures**

## Pull Request Process

### Before Submitting

1. **Run all checks**:
```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

2. **Update documentation** if needed

3. **Add tests** for new functionality

4. **Test manually** in Zed

### PR Guidelines

1. **Title**: Use descriptive title
   - Good: "Add support for PATCH method in parser"
   - Bad: "Fix bug"

2. **Description**: Include:
   - What the PR does
   - Why it's needed
   - How to test it
   - Related issues

3. **Commits**:
   - Use meaningful commit messages
   - Follow conventional commits format
   - Squash WIP commits

4. **Size**:
   - Keep PRs focused and small
   - Split large changes into multiple PRs
   - One feature/fix per PR

### PR Template

```markdown
## Description
[Describe what this PR does]

## Motivation
[Why is this change needed?]

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manually tested in Zed
- [ ] All tests passing

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. Maintainer reviews code
2. Address feedback
3. Update PR as needed
4. Once approved, PR is merged

## Questions?

- Open an issue for bugs
- Start a discussion for questions
- Join Discord/Slack (if available)
- Check existing documentation

## Code of Conduct

- Be respectful and inclusive
- Constructive feedback only
- Focus on the code, not the person
- Help others learn

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing! 🎉
