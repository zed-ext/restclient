use crate::parser::HttpRequest;
use crate::response::HttpResponse;
use crate::variables::VariableResolver;
use regex::Regex;
use std::sync::{Arc, Mutex};

/// Context provided to response handlers
pub struct HandlerContext {
    pub request: HttpRequest,
    pub response: HttpResponse,
}

/// Result of a test execution
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Runtime for executing JavaScript response handlers
/// Uses pattern-based interpretation for MVP (subset of JavaScript)
pub struct ResponseHandlerRuntime {
    variable_resolver: Arc<Mutex<VariableResolver>>,
    test_results: Vec<TestResult>,
}

impl ResponseHandlerRuntime {
    pub fn new(variable_resolver: Arc<Mutex<VariableResolver>>) -> Self {
        Self {
            variable_resolver,
            test_results: Vec::new(),
        }
    }

    /// Execute a JavaScript handler script
    pub fn execute(&mut self, script: &str, context: HandlerContext) -> Result<(), String> {
        // Clear previous test results
        self.test_results.clear();

        // Parse and execute each statement
        for line in script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Try to match and execute various patterns
            if let Err(e) = self.execute_statement(trimmed, &context) {
                eprintln!("Warning: Failed to execute statement '{}': {}", trimmed, e);
                // Continue execution even if one statement fails
            }
        }

        Ok(())
    }

    /// Execute a single JavaScript statement
    fn execute_statement(&mut self, stmt: &str, context: &HandlerContext) -> Result<(), String> {
        // Pattern: client.global.set("name", "value")
        if let Some(captures) = self.match_global_set(stmt) {
            let name = captures.0;
            let value = self.evaluate_expression(&captures.1, context)?;
            let mut resolver = self.variable_resolver.lock().unwrap();
            resolver.set_global(name, value);
            return Ok(());
        }

        // Pattern: var name = expression;
        if let Some(captures) = self.match_variable_declaration(stmt) {
            let _var_name = captures.0;
            let _value = self.evaluate_expression(&captures.1, context)?;
            // For MVP, we don't maintain local variables, just evaluate for side effects
            return Ok(());
        }

        // Pattern: client.test("name", function() { ... })
        if let Some(captures) = self.match_test(stmt, context) {
            self.test_results.push(captures);
            return Ok(());
        }

        // If no pattern matched, it's likely unsupported syntax
        // For MVP, silently ignore
        Ok(())
    }

    /// Match: client.global.set("name", value)
    fn match_global_set(&self, stmt: &str) -> Option<(String, String)> {
        let re = Regex::new(r#"client\.global\.set\s*\(\s*["']([^"']+)["']\s*,\s*(.+?)\s*\)"#).unwrap();
        if let Some(captures) = re.captures(stmt) {
            let name = captures.get(1)?.as_str().to_string();
            let value_expr = captures.get(2)?.as_str().trim_end_matches(';').to_string();
            return Some((name, value_expr));
        }
        None
    }

    /// Match: var name = value;
    fn match_variable_declaration(&self, stmt: &str) -> Option<(String, String)> {
        let re = Regex::new(r#"var\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?);"#).unwrap();
        if let Some(captures) = re.captures(stmt) {
            let name = captures.get(1)?.as_str().to_string();
            let value_expr = captures.get(2)?.as_str().to_string();
            return Some((name, value_expr));
        }
        None
    }

    /// Match: client.test("name", function() { assertions })
    fn match_test(&self, stmt: &str, context: &HandlerContext) -> Option<TestResult> {
        let re = Regex::new(r#"client\.test\s*\(\s*["']([^"']+)["']\s*,\s*function\s*\(\s*\)\s*\{([^}]+)\}"#).unwrap();
        if let Some(captures) = re.captures(stmt) {
            let test_name = captures.get(1)?.as_str().to_string();
            let test_body = captures.get(2)?.as_str();

            // Execute test assertions
            let passed = self.execute_test_body(test_body, context);

            return Some(TestResult {
                name: test_name,
                passed,
                message: None,
            });
        }
        None
    }

    /// Execute test body and check assertions
    fn execute_test_body(&self, body: &str, context: &HandlerContext) -> bool {
        // Look for client.assert() calls
        let re = Regex::new(r#"client\.assert\s*\(([^)]+)\)"#).unwrap();

        for captures in re.captures_iter(body) {
            let assertion = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            if !self.evaluate_assertion(assertion, context) {
                return false;
            }
        }

        true
    }

    /// Evaluate an assertion expression
    fn evaluate_assertion(&self, expr: &str, context: &HandlerContext) -> bool {
        let expr = expr.trim();

        // Pattern: response.status === 200
        if expr.contains("response.status") {
            if let Some(expected) = self.extract_number_comparison(expr) {
                return context.response.status == expected;
            }
        }

        // Pattern: response.body.contains("text")
        if expr.contains("response.body") && expr.contains("contains") {
            if let Some(text) = self.extract_string_literal(expr) {
                if let Ok(body) = context.response.body_as_string() {
                    return body.contains(&text);
                }
            }
        }

        // Default: assume true if we can't parse
        true
    }

    /// Evaluate a JavaScript expression to a string value
    fn evaluate_expression(&self, expr: &str, context: &HandlerContext) -> Result<String, String> {
        let expr = expr.trim();

        // String literal: "value" or 'value'
        if (expr.starts_with('"') && expr.ends_with('"')) || (expr.starts_with('\'') && expr.ends_with('\'')) {
            return Ok(expr[1..expr.len()-1].to_string());
        }

        // Number literal
        if expr.parse::<f64>().is_ok() {
            return Ok(expr.to_string());
        }

        // JSON.parse(response.body).field
        if expr.starts_with("JSON.parse(response.body)") {
            return self.evaluate_json_path(expr, context);
        }

        // response.body.field (assuming it's JSON)
        if expr.starts_with("response.body.") {
            return self.evaluate_json_path(&format!("JSON.parse(response.body).{}", &expr[14..]), context);
        }

        // response.body
        if expr == "response.body" {
            return context.response.body_as_string();
        }

        // response.status
        if expr == "response.status" {
            return Ok(context.response.status.to_string());
        }

        // client.global.get("name")
        if expr.starts_with("client.global.get(") {
            if let Some(name) = self.extract_string_literal(expr) {
                let resolver = self.variable_resolver.lock().unwrap();
                if let Some(value) = resolver.get_global(&name) {
                    return Ok(value.clone());
                }
            }
        }

        Err(format!("Cannot evaluate expression: {}", expr))
    }

    /// Evaluate JSON path expression: JSON.parse(response.body).field.subfield
    fn evaluate_json_path(&self, expr: &str, context: &HandlerContext) -> Result<String, String> {
        // Extract the path after JSON.parse(response.body)
        let path_start = expr.find(").").map(|i| i + 2);
        let path = if let Some(start) = path_start {
            &expr[start..]
        } else {
            return context.response.body_as_string();
        };

        // Parse response body as JSON
        let body_str = context.response.body_as_string()?;
        let json: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse response body as JSON: {}", e))?;

        // Navigate the path
        let mut current = &json;
        for part in path.split('.') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            current = current.get(part)
                .ok_or_else(|| format!("Field '{}' not found in JSON", part))?;
        }

        // Convert to string
        match current {
            serde_json::Value::String(s) => Ok(s.clone()),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            serde_json::Value::Bool(b) => Ok(b.to_string()),
            serde_json::Value::Null => Ok("null".to_string()),
            other => Ok(other.to_string()),
        }
    }

    /// Extract string literal from expression
    fn extract_string_literal(&self, expr: &str) -> Option<String> {
        let re = Regex::new(r#"["']([^"']+)["']"#).unwrap();
        re.captures(expr).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
    }

    /// Extract number from comparison expression
    fn extract_number_comparison(&self, expr: &str) -> Option<u16> {
        let re = Regex::new(r#"===\s*(\d+)"#).unwrap();
        re.captures(expr)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u16>().ok())
    }

    /// Get test results
    pub fn get_test_results(&self) -> &[TestResult] {
        &self.test_results
    }

    /// Check if all tests passed
    pub fn all_tests_passed(&self) -> bool {
        self.test_results.iter().all(|t| t.passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{HttpMethod, RequestMetadata};
    use std::collections::HashMap;

    fn create_test_context(status: u16, body: &str) -> HandlerContext {
        let request = HttpRequest {
            method: HttpMethod::POST,
            url: "https://api.example.com/test".to_string(),
            http_version: None,
            headers: vec![],
            body: None,
            response_handler: None,
            metadata: RequestMetadata {
                start_line: 0,
                end_line: 1,
                name: None,
            },
        };

        let response = HttpResponse::new(
            status,
            "OK".to_string(),
            HashMap::new(),
            body.as_bytes().to_vec(),
        );

        HandlerContext { request, response }
    }

    #[test]
    fn test_client_global_set() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver.clone());

        let script = r#"client.global.set("token", "abc123");"#;
        let context = create_test_context(200, "{}");

        runtime.execute(script, context).unwrap();

        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("token"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_json_parse_extraction() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver.clone());

        let script = r#"client.global.set("user_id", JSON.parse(response.body).id);"#;
        let context = create_test_context(200, r#"{"id": 42, "name": "John"}"#);

        runtime.execute(script, context).unwrap();

        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("user_id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_json_nested_path() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver.clone());

        let script = r#"client.global.set("email", JSON.parse(response.body).user.email);"#;
        let context = create_test_context(200, r#"{"user": {"email": "test@example.com", "id": 1}}"#);

        runtime.execute(script, context).unwrap();

        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_client_test_passing() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver);

        let script = r#"client.test("Status is 200", function() { client.assert(response.status === 200); });"#;
        let context = create_test_context(200, "{}");

        runtime.execute(script, context).unwrap();

        assert_eq!(runtime.get_test_results().len(), 1);
        assert!(runtime.get_test_results()[0].passed);
        assert_eq!(runtime.get_test_results()[0].name, "Status is 200");
    }

    #[test]
    fn test_client_test_failing() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver);

        let script = r#"client.test("Status is 404", function() { client.assert(response.status === 404); });"#;
        let context = create_test_context(200, "{}");

        runtime.execute(script, context).unwrap();

        assert_eq!(runtime.get_test_results().len(), 1);
        assert!(!runtime.get_test_results()[0].passed);
    }

    #[test]
    fn test_multiple_statements() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver.clone());

        let script = r#"
            var token = JSON.parse(response.body).token;
            client.global.set("auth_token", JSON.parse(response.body).token);
            client.global.set("user_id", JSON.parse(response.body).userId);
            client.test("Has token", function() { client.assert(response.status === 201); });
        "#;

        let context = create_test_context(201, r#"{"token": "xyz789", "userId": 42}"#);

        runtime.execute(script, context).unwrap();

        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("auth_token"), Some(&"xyz789".to_string()));
        assert_eq!(r.get_global("user_id"), Some(&"42".to_string()));
        assert_eq!(runtime.get_test_results().len(), 1);
        assert!(runtime.all_tests_passed());
    }

    #[test]
    fn test_response_body_access() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let mut runtime = ResponseHandlerRuntime::new(resolver.clone());

        let script = r#"client.global.set("raw_body", response.body);"#;
        let context = create_test_context(200, "plain text response");

        runtime.execute(script, context).unwrap();

        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("raw_body"), Some(&"plain text response".to_string()));
    }
}
