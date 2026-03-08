# Variable Substitution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable `{{variable}}` substitution in HTTP requests using `@variable = value` declarations in `.http` files.

**Architecture:** Parse-time resolution - the parser extracts variable declarations, creates a VariableResolver, and resolves all `{{variable}}` references in URLs, headers, and bodies before returning HttpRequest objects.

**Tech Stack:** Rust, existing VariableResolver in `src/variables.rs`, modify `src/parser.rs`

---

## Task 1: Add variable resolution helper function

**Files:**
- Modify: `src/parser.rs` (add new function after line 340)
- Test: Unit tests in `src/parser.rs`

**Step 1: Write the failing test**

Add to the bottom of `src/parser.rs` (after line 340, in the tests module):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ... existing tests ...

    #[test]
    fn test_variable_substitution_in_url() {
        let content = r#"
@baseUrl = https://api.example.com
@userId = 123

GET {{baseUrl}}/users/{{userId}}
"#;
        let requests = parse_http_file(content).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://api.example.com/users/123");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_variable_substitution_in_url`

Expected: FAIL - the URL will still contain `{{baseUrl}}` and `{{userId}}`

**Step 3: Import VariableResolver at top of parser.rs**

At the top of `src/parser.rs` (around line 3), add:

```rust
use crate::variables::VariableResolver;
```

**Step 4: Write helper function**

Add this function after the `parse_request_line` function (around line 340):

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

**Step 5: Modify parse_http_file to use variable resolution**

In `src/parser.rs`, modify the `parse_http_file` function (starts at line 97). Replace it with:

```rust
/// Parse an .http file content into a list of HTTP requests
pub fn parse_http_file(content: &str) -> Result<Vec<HttpRequest>, String> {
    // Create and populate VariableResolver
    let mut resolver = VariableResolver::new();
    resolver.parse_file_variables(content);

    let mut requests = Vec::new();

    // Split by request separator (###) while tracking line numbers
    let blocks = split_by_separator_with_lines(content);

    for block in blocks {
        let trimmed = block.content.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip blocks that only contain comments and variable declarations
        let has_non_comment_content = trimmed.lines().any(|line| {
            let line = line.trim();
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with("//")
                && !line.starts_with('@')
        });

        if !has_non_comment_content {
            continue;
        }

        match parse_request_block(&block.content, block.start_line, block.end_line) {
            Ok(mut request) => {
                // Resolve variables in the request
                resolve_request_variables(&mut request, &resolver);
                requests.push(request);
            }
            Err(e) => {
                // Log error but continue parsing other requests
                eprintln!("Error parsing request at line {}: {}", block.start_line, e);
            }
        }
    }

    Ok(requests)
}
```

**Step 6: Run test to verify it passes**

Run: `cargo test test_variable_substitution_in_url`

Expected: PASS

**Step 7: Commit**

```bash
git add src/parser.rs
git commit -m "feat: add variable resolution in parse_http_file

- Import VariableResolver
- Add resolve_request_variables helper function
- Integrate variable resolution in parse_http_file
- Add test for URL variable substitution"
```

---

## Task 2: Add tests for header variable substitution

**Files:**
- Modify: `src/parser.rs` (add test in tests module)

**Step 1: Write the failing test**

Add to the tests module in `src/parser.rs`:

```rust
#[test]
fn test_variable_substitution_in_headers() {
    let content = r#"
@token = secret123
@contentType = application/json

GET https://api.example.com
Authorization: Bearer {{token}}
Content-Type: {{contentType}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests.len(), 1);

    // Find headers by name
    let auth_header = requests[0]
        .headers
        .iter()
        .find(|(name, _)| name == "Authorization")
        .map(|(_, value)| value);
    assert_eq!(auth_header, Some(&"Bearer secret123".to_string()));

    let content_header = requests[0]
        .headers
        .iter()
        .find(|(name, _)| name == "Content-Type")
        .map(|(_, value)| value);
    assert_eq!(content_header, Some(&"application/json".to_string()));
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_variable_substitution_in_headers`

Expected: PASS (our implementation already handles headers)

**Step 3: Commit**

```bash
git add src/parser.rs
git commit -m "test: add header variable substitution test"
```

---

## Task 3: Add tests for body variable substitution

**Files:**
- Modify: `src/parser.rs` (add test in tests module)

**Step 1: Write the test**

Add to the tests module in `src/parser.rs`:

```rust
#[test]
fn test_variable_substitution_in_body() {
    let content = r#"
@username = john_doe
@email = john@example.com

POST https://api.example.com/users
Content-Type: application/json

{
  "username": "{{username}}",
  "email": "{{email}}"
}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests.len(), 1);

    if let Some(RequestBody::Inline(body)) = &requests[0].body {
        assert!(body.contains(r#""username": "john_doe""#));
        assert!(body.contains(r#""email": "john@example.com""#));
    } else {
        panic!("Expected inline body");
    }
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_variable_substitution_in_body`

Expected: PASS

**Step 3: Commit**

```bash
git add src/parser.rs
git commit -m "test: add body variable substitution test"
```

---

## Task 4: Add test for nested variable resolution

**Files:**
- Modify: `src/parser.rs` (add test in tests module)

**Step 1: Write the test**

Add to the tests module in `src/parser.rs`:

```rust
#[test]
fn test_nested_variable_resolution() {
    let content = r#"
@protocol = https
@host = api.example.com
@baseUrl = {{protocol}}://{{host}}
@version = v1
@endpoint = users

GET {{baseUrl}}/{{version}}/{{endpoint}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://api.example.com/v1/users");
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_nested_variable_resolution`

Expected: PASS (VariableResolver already supports nested resolution)

**Step 3: Commit**

```bash
git add src/parser.rs
git commit -m "test: add nested variable resolution test"
```

---

## Task 5: Add test for undefined variables

**Files:**
- Modify: `src/parser.rs` (add test in tests module)

**Step 1: Write the test**

Add to the tests module in `src/parser.rs`:

```rust
#[test]
fn test_undefined_variable_left_as_is() {
    let content = r#"
@defined = value

GET https://{{undefined}}/api/{{defined}}
Authorization: Bearer {{missingToken}}
"#;
    let requests = parse_http_file(content).unwrap();
    assert_eq!(requests.len(), 1);

    // Undefined variables should remain as-is
    assert_eq!(requests[0].url, "https://{{undefined}}/api/value");

    let auth_header = requests[0]
        .headers
        .iter()
        .find(|(name, _)| name == "Authorization")
        .map(|(_, value)| value);
    assert_eq!(auth_header, Some(&"Bearer {{missingToken}}".to_string()));
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_undefined_variable_left_as_is`

Expected: PASS

**Step 3: Commit**

```bash
git add src/parser.rs
git commit -m "test: verify undefined variables remain as-is"
```

---

## Task 6: Run all tests to ensure no regressions

**Files:**
- None (verification step)

**Step 1: Run full test suite**

Run: `cargo test`

Expected: All tests pass

**Step 2: Run clippy to check for issues**

Run: `cargo clippy --all-targets --all-features`

Expected: No warnings or errors

**Step 3: Format code**

Run: `cargo fmt`

Expected: Code is formatted

**Step 4: Commit any formatting changes**

```bash
git add -A
git commit -m "style: run cargo fmt" || echo "No formatting changes needed"
```

---

## Task 7: Update test.http with variable examples

**Files:**
- Modify: `test.http` (add examples at bottom)

**Step 1: Add variable examples to test.http**

Add to the end of `test.http`:

```http
###
### ============================================
### Variable Substitution Examples
### ============================================

@baseUrl = https://jsonplaceholder.typicode.com
@userId = 5

### Get user with variables
GET {{baseUrl}}/users/{{userId}}

### Get user's posts with variables
GET {{baseUrl}}/posts?userId={{userId}}

###
### Nested variables example
@protocol = https
@host = httpbin.org
@apiBase = {{protocol}}://{{host}}

### Request using nested variables
GET {{apiBase}}/get?test=nested

###
### Variables in headers
@authToken = my-test-token-12345
@userAgent = REST-Client-Zed/1.0

### Authenticated request with variable headers
GET https://httpbin.org/headers
Authorization: Bearer {{authToken}}
User-Agent: {{userAgent}}
Accept: application/json

###
### Variables in POST body
@newTitle = Created via Variable Substitution
@newBody = This post uses variables in the JSON body
@authorId = 1

### Create post with variables in body
POST https://jsonplaceholder.typicode.com/posts
Content-Type: application/json

{
  "title": "{{newTitle}}",
  "body": "{{newBody}}",
  "userId": {{authorId}}
}
```

**Step 2: Test manually with CLI**

Run: `cargo run --release --features cli --bin http-execute test.http --line 88`

(Line 88 should be in the "Get user with variables" request)

Expected: Request executes successfully with substituted values visible in output

**Step 3: Commit**

```bash
git add test.http
git commit -m "docs: add variable substitution examples to test.http"
```

---

## Task 8: Update README.md documentation

**Files:**
- Modify: `README.md`

**Step 1: Update Features section**

Find the "Features" section (around line 99-108) and change:

```markdown
✅ **Variable Support** - Use `{{variable}}` syntax (parsing ready, execution coming soon)
```

To:

```markdown
✅ **Variable Support** - Use `@variable = value` and `{{variable}}` syntax
```

**Step 2: Update Variables section**

Find the "Variables (Coming Soon)" section (around line 227-239) and replace with:

```markdown
### Variables

Declare variables with `@name = value`:

```http
@baseUrl = https://api.example.com
@userId = 123

### Use variables with {{name}}
GET {{baseUrl}}/users/{{userId}}
```

Variables are substituted at parse time and work in:
- URLs
- Header values
- Request bodies

Variables support nested resolution:

```http
@protocol = https
@host = api.example.com
@baseUrl = {{protocol}}://{{host}}

GET {{baseUrl}}/api/users
```
```

**Step 3: Update Roadmap**

Find the Roadmap section (around line 337-351) and change:

```markdown
- [ ] Variable substitution in requests
```

To:

```markdown
- [x] Variable substitution in requests
```

**Step 4: Test the CLI with variables**

Run: `cargo run --release --features cli --bin http-execute test.http --line 88`

Expected: Success with substituted variables

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: update README with variable substitution feature

- Mark variable support as complete in features list
- Update variables section with working examples
- Show nested variable support
- Update roadmap to mark variable substitution complete"
```

---

## Task 9: Final integration test in Zed

**Files:**
- None (manual verification)

**Step 1: Build the extension**

Run: `cargo build --release --target wasm32-wasip1`

Expected: Successful build

**Step 2: Install to Zed**

Run: `./install_to_zed.sh`

Expected: Extension installed

**Step 3: Restart Zed completely**

Action: Quit Zed (`Cmd+Q` on macOS) and reopen

**Step 4: Test variable substitution**

1. Open `test.http` in Zed
2. Navigate to line 88 (the "Get user with variables" request)
3. Execute with `Ctrl+Enter` or Command Palette → "task spawn" → "Send Request at Cursor"
4. Verify in terminal output that:
   - URL shows `https://jsonplaceholder.typicode.com/users/5` (substituted)
   - Response is successful

**Step 5: Test with undefined variables**

Create a test:

```http
@defined = works

GET https://httpbin.org/get?defined={{defined}}&undefined={{missing}}
```

Execute and verify URL shows: `https://httpbin.org/get?defined=works&undefined={{missing}}`

**Step 6: Document success**

If all tests pass, the feature is complete!

---

## Task 10: Final commit and summary

**Files:**
- None (documentation)

**Step 1: Verify all changes are committed**

Run: `git status`

Expected: Working tree clean

**Step 2: Review commit history**

Run: `git log --oneline -10`

Expected: See all commits from this implementation

**Step 3: Create implementation summary**

Create a summary of what was implemented:

```
Variable Substitution Feature - Implementation Complete

Changes:
- Modified src/parser.rs to integrate VariableResolver
- Added resolve_request_variables() helper function
- Variables now resolve in URLs, headers, and bodies
- Added 5 comprehensive tests covering all scenarios
- Updated test.http with working examples
- Updated README.md documentation

Test Results:
- All unit tests passing
- CLI tool works with variable substitution
- Zed extension works with variable substitution
- Undefined variables gracefully remain as {{var}}

Next Steps:
- Environment file support (.http-client/environments.json)
- Dynamic variables from response handlers
```

**Step 4: Optional - tag the release**

If this is a release-worthy feature:

```bash
git tag -a v0.2.0 -m "Add variable substitution support"
```

---

## Verification Checklist

Before considering this complete, verify:

- [ ] `cargo test` passes all tests
- [ ] `cargo clippy` shows no warnings
- [ ] `cargo fmt` has been run
- [ ] CLI tool works: `cargo run --features cli --bin http-execute test.http --line 88`
- [ ] Extension builds: `cargo build --release --target wasm32-wasip1`
- [ ] Extension works in Zed with variable substitution
- [ ] Documentation updated (README.md)
- [ ] Examples added (test.http)
- [ ] All changes committed with clear messages

---

## Rollback Plan

If something goes wrong:

```bash
# View commits from this feature
git log --oneline -10

# Rollback to before implementation
git reset --hard <commit-hash-before-changes>

# Or revert specific commits
git revert <commit-hash>
```

---

## Notes for Future Enhancements

**Phase 2 - Environment Files:**
- Load `.http-client/environments.json`
- Allow switching active environment
- UI for environment selection in Zed

**Phase 3 - Dynamic Variables:**
- Response handlers can set variables
- Variables persist across requests in a session
- Access to previous response data

**Phase 4 - System Variables:**
- Document that system env vars already work
- Add special variables like `{{$timestamp}}`, `{{$randomInt}}`
- Add `{{$guid}}` for unique IDs
