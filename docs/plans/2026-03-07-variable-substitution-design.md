# Variable Substitution Feature Design

**Date:** 2026-03-07
**Status:** Approved
**Scope:** File-level variables only (Phase 1)

## Overview

Enable variable substitution in HTTP requests using the `{{variable}}` syntax. Variables are defined in `.http` files using `@variable = value` declarations and automatically resolved during parsing.

## Background

- **Existing infrastructure:** `VariableResolver` class is fully implemented in `src/variables.rs` with resolution logic, nested variable support, and comprehensive tests
- **Current state:** Parser recognizes `@variable` declarations but doesn't perform substitution
- **Target:** Make variables work in both CLI tool and Zed extension

## Design Decisions

### Resolution Timing: Parse-Time vs Execute-Time

**Chosen: Parse-Time Resolution**

Variables are resolved during parsing, before requests reach the executors. This means:
- Executor code (CLI and extension) receives fully-resolved requests
- No duplication of resolution logic
- Single source of truth in parser
- Simpler testing and maintenance

**Trade-off:** Variables must be defined before use in the file, but this is standard behavior in HTTP client tools.

### Scope: File-Level Only (Phase 1)

Starting with file-level variables (`@variable = value`) only. Environment files (`.http-client/environments.json`) are deferred to a future phase.

**Rationale:** Get core functionality working first, add complexity later.

## Architecture

### High-Level Flow

```
1. Parse file content into blocks (existing)
2. Extract @variable declarations → VariableResolver
3. Parse each request block (existing)
4. [NEW] Resolve {{variables}} in each request:
   - URLs
   - Header values
   - Request bodies
5. Return fully-resolved HttpRequest objects
```

### Components

**Modified Files:**
- `src/parser.rs` - Add variable resolution step

**Unchanged Files:**
- `src/bin/http_execute.rs` - Works with resolved requests
- `src/lib.rs` - Works with resolved requests
- `src/variables.rs` - Already has everything needed

## Implementation

### Changes to `src/parser.rs`

**1. Modify `parse_http_file()` function:**

```rust
pub fn parse_http_file(content: &str) -> Result<Vec<HttpRequest>, String> {
    // Create and populate VariableResolver
    let mut resolver = VariableResolver::new();
    resolver.parse_file_variables(content);

    // Parse blocks (existing code)
    let blocks = split_by_separator_with_lines(content);
    let mut requests = Vec::new();

    for block in blocks {
        // ... existing parsing logic ...
        match parse_request_block(&block.content, block.start_line, block.end_line) {
            Ok(mut request) => {
                // NEW: Resolve variables in the request
                resolve_request_variables(&mut request, &resolver);
                requests.push(request);
            }
            Err(e) => {
                eprintln!("Error parsing request at line {}: {}", block.start_line, e);
            }
        }
    }

    Ok(requests)
}
```

**2. Add new helper function:**

```rust
/// Resolve all variables in a parsed HTTP request
fn resolve_request_variables(request: &mut HttpRequest, resolver: &VariableResolver) {
    // Resolve URL
    request.url = resolver.resolve(&request.url);

    // Resolve header values
    for (_, value) in &mut request.headers {
        *value = resolver.resolve(value);
    }

    // Resolve body if present
    if let Some(RequestBody::Inline(body)) = &mut request.body {
        *body = resolver.resolve(body);
    }
}
```

## Error Handling

### Undefined Variables

When a variable is undefined, `VariableResolver.resolve()` leaves it unchanged:

```
Input:  GET {{baseUrl}}/users
Result: GET {{baseUrl}}/users  (if baseUrl undefined)
```

**Rationale:**
- User can see which variable is missing
- Request shows intended structure
- No crashes or confusing errors
- More forgiving for draft requests

**Alternative considered:** Error on undefined variables - rejected as too strict.

## Testing

### Unit Tests (in `src/parser.rs`)

```rust
#[test]
fn test_variable_substitution_in_url() {
    let content = r#"
@baseUrl = https://api.example.com
@userId = 123

GET {{baseUrl}}/users/{{userId}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests[0].url, "https://api.example.com/users/123");
}

#[test]
fn test_variable_substitution_in_headers() {
    let content = r#"
@token = secret123

GET https://api.example.com
Authorization: Bearer {{token}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests[0].headers[0].1, "Bearer secret123");
}

#[test]
fn test_variable_substitution_in_body() {
    let content = r#"
@name = John

POST https://api.example.com
Content-Type: application/json

{"user": "{{name}}"}
"#;
    let requests = parse_http_file(content).unwrap();
    if let Some(RequestBody::Inline(body)) = &requests[0].body {
        assert!(body.contains(r#""John""#));
    }
}

#[test]
fn test_undefined_variable_left_as_is() {
    let content = "GET https://{{undefined}}/api";
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests[0].url, "https://{{undefined}}/api");
}

#[test]
fn test_nested_variable_resolution() {
    let content = r#"
@host = api.example.com
@baseUrl = https://{{host}}
@endpoint = users

GET {{baseUrl}}/{{endpoint}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests[0].url, "https://api.example.com/users");
}
```

### Integration Testing

- Update `test.http` with variable examples
- Test manually with CLI tool: `cargo run --features cli --bin http-execute test.http`
- Test in Zed extension with Ctrl+Enter

## Examples

### Basic Variable Usage

```http
@baseUrl = https://jsonplaceholder.typicode.com
@userId = 5

### Get user
GET {{baseUrl}}/users/{{userId}}

### Get user's posts
GET {{baseUrl}}/posts?userId={{userId}}
```

### Variables in Headers

```http
@apiKey = my-secret-key
@contentType = application/json

### Authenticated request
POST https://api.example.com/data
Authorization: Bearer {{apiKey}}
Content-Type: {{contentType}}

{"data": "value"}
```

### Variables in Body

```http
@username = john_doe
@email = john@example.com

### Create user
POST https://api.example.com/users
Content-Type: application/json

{
  "username": "{{username}}",
  "email": "{{email}}"
}
```

### Nested Variables

```http
@protocol = https
@host = api.example.com
@baseUrl = {{protocol}}://{{host}}
@version = v1

### API call with nested resolution
GET {{baseUrl}}/{{version}}/users
```

## Future Enhancements (Out of Scope)

- Environment file support (`.http-client/environments.json`)
- Environment switching UI
- Dynamic variables from response handlers
- System environment variable integration (already supported by VariableResolver, just need to document)
- Variable autocomplete in editor

## Success Criteria

1. Variables defined with `@variable = value` are recognized
2. `{{variable}}` syntax is substituted in URLs, headers, and bodies
3. Nested variables resolve correctly
4. Undefined variables remain as `{{undefined}}`
5. Works identically in CLI tool and Zed extension
6. All tests pass
7. No breaking changes to existing functionality

## Documentation Updates

Update `README.md`:
- Change "Variable Support - parsing ready, execution coming soon" to "✅ Variable Support"
- Update "Variables (Coming Soon)" section to show working examples
- Add examples to roadmap completion

## Dependencies

- No new external dependencies
- Uses existing `src/variables.rs` module
- Compatible with current parser architecture
