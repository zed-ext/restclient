use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct ResponseContext {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub body_raw: String,
}

pub fn execute_handler(
    script: &str,
    context: &ResponseContext,
) -> Result<HashMap<String, String>, String> {
    let mut captured = HashMap::new();

    for statement in script.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (key, expr) = parse_set_statement(trimmed)?;
        let value = evaluate_expression(expr, context)
            .ok_or_else(|| format!("Unable to resolve expression: {}", expr))?;
        captured.insert(key.to_string(), value);
    }

    Ok(captured)
}

pub fn load_globals(http_file_dir: &Path) -> HashMap<String, String> {
    let globals_path = globals_file_path(http_file_dir);
    let content = match fs::read_to_string(&globals_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    serde_json::from_str::<HashMap<String, String>>(&content).unwrap_or_default()
}

pub fn save_globals(http_file_dir: &Path, globals: &HashMap<String, String>) -> Result<(), String> {
    let globals_dir = http_file_dir.join(".http-client");
    fs::create_dir_all(&globals_dir)
        .map_err(|e| format!("Failed to create globals directory: {}", e))?;

    let path = globals_dir.join(".global-variables.json");
    let mut merged = load_globals(http_file_dir);
    for (k, v) in globals {
        merged.insert(k.clone(), v.clone());
    }

    let serialized = serde_json::to_string_pretty(&merged)
        .map_err(|e| format!("Failed to serialize globals: {}", e))?;
    fs::write(path, serialized).map_err(|e| format!("Failed to write globals: {}", e))
}

fn globals_file_path(http_file_dir: &Path) -> std::path::PathBuf {
    http_file_dir
        .join(".http-client")
        .join(".global-variables.json")
}

fn parse_set_statement(statement: &str) -> Result<(&str, &str), String> {
    let prefix = "client.global.set";
    if !statement.starts_with(prefix) {
        return Err(format!("Unsupported statement: {}", statement));
    }

    let open_paren = statement
        .find('(')
        .ok_or_else(|| format!("Invalid set() call: {}", statement))?;
    let close_paren = statement
        .rfind(')')
        .ok_or_else(|| format!("Invalid set() call: {}", statement))?;

    if close_paren <= open_paren {
        return Err(format!("Malformed set() call: {}", statement));
    }

    let args_str = &statement[open_paren + 1..close_paren];
    let args = split_top_level_args(args_str);
    if args.len() != 2 {
        return Err(format!("set() expects 2 arguments: {}", statement));
    }

    let key = parse_string_literal(args[0])
        .ok_or_else(|| format!("set() key must be string literal: {}", statement))?;
    Ok((key, args[1].trim()))
}

fn split_top_level_args(input: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0i32;
    let mut bracket_depth = 0i32;
    let mut in_string: Option<char> = None;
    let mut escaped = false;

    for (idx, ch) in input.char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                in_string = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => in_string = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                args.push(input[start..idx].trim());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    let tail = input[start..].trim();
    if !tail.is_empty() {
        args.push(tail);
    }

    args
}

fn evaluate_expression(expr: &str, context: &ResponseContext) -> Option<String> {
    let trimmed = expr.trim();

    if let Some(lit) = parse_string_literal(trimmed) {
        return Some(lit.to_string());
    }

    if trimmed == "response.status" {
        return Some(context.status.to_string());
    }

    if trimmed.starts_with("response.headers.valueOf(") && trimmed.ends_with(')') {
        let inner = &trimmed["response.headers.valueOf(".len()..trimmed.len() - 1];
        let header_name = parse_string_literal(inner)?;

        if let Some(found) = context.headers.get(header_name) {
            return Some(found.clone());
        }

        for (k, v) in &context.headers {
            if k.eq_ignore_ascii_case(header_name) {
                return Some(v.clone());
            }
        }

        return None;
    }

    if let Some(path) = trimmed.strip_prefix("response.body.") {
        let body = context.body.as_ref()?;
        return resolve_dot_path(body, path);
    }

    None
}

fn parse_string_literal(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    if trimmed.len() < 2 {
        return None;
    }

    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return Some(&trimmed[1..trimmed.len() - 1]);
    }

    None
}

fn json_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(s.clone()),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).ok(),
    }
}

pub(crate) fn resolve_dot_path(value: &Value, path: &str) -> Option<String> {
    let tokens = parse_dot_path_tokens(path)?;
    let mut current = value;

    for token in tokens {
        match token {
            PathToken::Key(key) => {
                current = current.get(&key)?;
            }
            PathToken::Index(i) => {
                current = current.get(i)?;
            }
        }
    }

    json_value_to_string(current)
}

#[derive(Debug, PartialEq, Eq)]
enum PathToken {
    Key(String),
    Index(usize),
}

fn parse_dot_path_tokens(path: &str) -> Option<Vec<PathToken>> {
    let mut tokens = Vec::new();

    for segment in path.split('.') {
        let seg = segment.trim();
        if seg.is_empty() {
            return None;
        }

        let chars: Vec<char> = seg.chars().collect();
        let mut i = 0usize;

        let mut key = String::new();
        while i < chars.len() && chars[i] != '[' {
            key.push(chars[i]);
            i += 1;
        }

        if !key.is_empty() {
            tokens.push(PathToken::Key(key));
        }

        while i < chars.len() {
            if chars[i] != '[' {
                return None;
            }
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }

            if i >= chars.len() {
                return None;
            }

            let idx_text: String = chars[start..i].iter().collect();
            let index = idx_text.parse::<usize>().ok()?;
            tokens.push(PathToken::Index(index));
            i += 1;
        }
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn ctx(body: Value) -> ResponseContext {
        let mut headers = HashMap::new();
        headers.insert("X-Request-Id".to_string(), "req-123".to_string());

        ResponseContext {
            status: 201,
            headers,
            body_raw: body.to_string(),
            body: Some(body),
        }
    }

    #[test]
    fn test_parse_simple_set() {
        let context = ctx(serde_json::json!({"token": "abc123"}));
        let out = execute_handler(
            "client.global.set(\"token\", response.body.token);",
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("token"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_parse_nested_path() {
        let context = ctx(serde_json::json!({"user": {"profile": {"name": "alice"}}}));
        let out = execute_handler(
            "client.global.set(\"name\", response.body.user.profile.name);",
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("name"), Some(&"alice".to_string()));
    }

    #[test]
    fn test_parse_array_access() {
        let context = ctx(serde_json::json!({"data": [{"id": 42}]}));
        let out = execute_handler(
            "client.global.set(\"id\", response.body.data[0].id);",
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_parse_header_access() {
        let context = ctx(serde_json::json!({"ok": true}));
        let out = execute_handler(
            "client.global.set(\"request_id\", response.headers.valueOf(\"X-Request-Id\"));",
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("request_id"), Some(&"req-123".to_string()));
    }

    #[test]
    fn test_parse_status_access() {
        let context = ctx(serde_json::json!({"ok": true}));
        let out = execute_handler(
            "client.global.set(\"status_code\", response.status);",
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("status_code"), Some(&"201".to_string()));
    }

    #[test]
    fn test_parse_string_literal() {
        let context = ctx(serde_json::json!({"ok": true}));
        let out = execute_handler("client.global.set(\"env\", \"production\");", &context)
            .expect("handler should execute");

        assert_eq!(out.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_parse_multiple_statements() {
        let context = ctx(serde_json::json!({"token": "abc", "user": {"id": 7}}));
        let out = execute_handler(
            r#"
            client.global.set("token", response.body.token);
            client.global.set("user_id", response.body.user.id);
            "#,
            &context,
        )
        .expect("handler should execute");

        assert_eq!(out.get("token"), Some(&"abc".to_string()));
        assert_eq!(out.get("user_id"), Some(&"7".to_string()));
    }

    #[test]
    fn test_dot_path_traversal() {
        let body = serde_json::json!({
            "token": "abc",
            "user": {"id": 123},
            "data": [{"name": "first"}],
            "items": [
                {"tags": ["a", "b"]},
                {"tags": ["c"]},
                {"tags": ["x", "y"]}
            ]
        });

        assert_eq!(resolve_dot_path(&body, "token"), Some("abc".to_string()));
        assert_eq!(resolve_dot_path(&body, "user.id"), Some("123".to_string()));
        assert_eq!(
            resolve_dot_path(&body, "data[0].name"),
            Some("first".to_string())
        );
        assert_eq!(
            resolve_dot_path(&body, "items[2].tags[0]"),
            Some("x".to_string())
        );
    }

    #[test]
    fn test_globals_persistence_round_trip() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let base_dir = std::env::temp_dir().join(format!("http-client-globals-{}", nanos));

        fs::create_dir_all(&base_dir).expect("temp dir should be created");

        let mut initial = HashMap::new();
        initial.insert("token".to_string(), "abc".to_string());
        save_globals(&base_dir, &initial).expect("save should succeed");

        let mut update = HashMap::new();
        update.insert("user_id".to_string(), "42".to_string());
        save_globals(&base_dir, &update).expect("merge save should succeed");

        let loaded = load_globals(&base_dir);
        assert_eq!(loaded.get("token"), Some(&"abc".to_string()));
        assert_eq!(loaded.get("user_id"), Some(&"42".to_string()));

        let _ = fs::remove_dir_all(base_dir);
    }
}
