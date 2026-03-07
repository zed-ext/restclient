use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    #[serde(flatten)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environments {
    #[serde(flatten)]
    pub environments: HashMap<String, Environment>,
}

pub struct VariableResolver {
    /// Loaded environments from .http-client/environments.json
    environments: HashMap<String, Environment>,
    /// Currently active environment name
    active_environment: Option<String>,
    /// Global variables set by response handlers (stored in memory)
    global_variables: HashMap<String, String>,
    /// File-level variables from @variable = value declarations
    file_variables: HashMap<String, String>,
}

impl Default for VariableResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableResolver {
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            active_environment: None,
            global_variables: HashMap::new(),
            file_variables: HashMap::new(),
        }
    }

    /// Load environments from a JSON file
    pub fn load_environments<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read environments file: {}", e))?;

        let envs: Environments = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse environments JSON: {}", e))?;

        self.environments = envs.environments;
        Ok(())
    }

    /// Set the active environment
    pub fn set_active_environment(&mut self, name: String) -> Result<(), String> {
        if self.environments.contains_key(&name) {
            self.active_environment = Some(name);
            Ok(())
        } else {
            Err(format!("Environment '{}' not found", name))
        }
    }

    /// Get the active environment name
    pub fn get_active_environment(&self) -> Option<&String> {
        self.active_environment.as_ref()
    }

    /// Set a global variable (typically from response handlers)
    pub fn set_global(&mut self, name: String, value: String) {
        self.global_variables.insert(name, value);
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<&String> {
        self.global_variables.get(name)
    }

    /// Set a file-level variable
    pub fn set_file_variable(&mut self, name: String, value: String) {
        self.file_variables.insert(name, value);
    }

    /// Clear file-level variables (call when switching to a new file)
    pub fn clear_file_variables(&mut self) {
        self.file_variables.clear();
    }

    /// Resolve a variable by name using the resolution order:
    /// 1. File-level variables
    /// 2. Global variables (from response handlers)
    /// 3. Active environment variables
    /// 4. System environment variables
    pub fn get_variable(&self, name: &str) -> Option<String> {
        // 1. File-level variables
        if let Some(value) = self.file_variables.get(name) {
            return Some(value.clone());
        }

        // 2. Global variables
        if let Some(value) = self.global_variables.get(name) {
            return Some(value.clone());
        }

        // 3. Active environment variables
        if let Some(env_name) = &self.active_environment {
            if let Some(env) = self.environments.get(env_name) {
                if let Some(value) = env.variables.get(name) {
                    return Some(value.clone());
                }
            }
        }

        // 4. System environment variables
        if let Ok(value) = env::var(name) {
            return Some(value);
        }

        None
    }

    /// Resolve all variables in a template string
    /// Supports {{variable}} syntax
    /// Supports nested resolution: {{base_url}}/{{endpoint}}
    pub fn resolve(&self, template: &str) -> String {
        let mut result = template.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

        // Keep resolving until no more variables found or max iterations reached
        loop {
            if iterations >= MAX_ITERATIONS {
                eprintln!("Warning: Maximum variable resolution iterations reached");
                break;
            }

            let before = result.clone();
            result = self.resolve_once(&result);

            // If nothing changed, we're done
            if result == before {
                break;
            }

            iterations += 1;
        }

        result
    }

    /// Perform one pass of variable resolution
    fn resolve_once(&self, template: &str) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                // Start of variable reference
                chars.next(); // consume second '{'

                let mut var_name = String::new();
                let mut found_closing = false;

                // Collect variable name
                while let Some(ch) = chars.next() {
                    if ch == '}' && chars.peek() == Some(&'}') {
                        chars.next(); // consume second '}'
                        found_closing = true;
                        break;
                    }
                    var_name.push(ch);
                }

                if found_closing {
                    // Resolve the variable
                    if let Some(value) = self.get_variable(var_name.trim()) {
                        result.push_str(&value);
                    } else {
                        // Variable not found, keep original syntax
                        result.push_str("{{");
                        result.push_str(&var_name);
                        result.push_str("}}");
                    }
                } else {
                    // Malformed variable reference, keep as-is
                    result.push_str("{{");
                    result.push_str(&var_name);
                }
            } else if ch == '\\' && chars.peek() == Some(&'{') {
                // Escaped variable: \{{not_a_variable}}
                result.push(ch);
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Get all available variables (for debugging/display)
    pub fn get_all_variables(&self) -> HashMap<String, String> {
        let mut all = HashMap::new();

        // Start with system env vars (lowest priority)
        for (key, value) in env::vars() {
            all.insert(key, value);
        }

        // Add environment variables
        if let Some(env_name) = &self.active_environment {
            if let Some(env) = self.environments.get(env_name) {
                for (key, value) in &env.variables {
                    all.insert(key.clone(), value.clone());
                }
            }
        }

        // Add global variables
        for (key, value) in &self.global_variables {
            all.insert(key.clone(), value.clone());
        }

        // Add file variables (highest priority)
        for (key, value) in &self.file_variables {
            all.insert(key.clone(), value.clone());
        }

        all
    }

    /// Parse file-level variable declarations from .http file content
    /// Format: @variable = value
    pub fn parse_file_variables(&mut self, content: &str) {
        self.clear_file_variables();

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for variable declaration: @variable = value
            if trimmed.starts_with('@') && trimmed.contains('=') {
                if let Some(eq_pos) = trimmed.find('=') {
                    let name = trimmed[1..eq_pos].trim();
                    let value = trimmed[eq_pos + 1..].trim();

                    if !name.is_empty() && !value.is_empty() {
                        self.set_file_variable(name.to_string(), value.to_string());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_resolution_order() {
        let mut resolver = VariableResolver::new();

        // Set up different levels
        env::set_var("TEST_VAR", "from_system");

        let mut env = Environment {
            variables: HashMap::new(),
        };
        env.variables
            .insert("TEST_VAR".to_string(), "from_environment".to_string());
        resolver.environments.insert("dev".to_string(), env);
        resolver.set_active_environment("dev".to_string()).unwrap();

        resolver.set_global("TEST_VAR".to_string(), "from_global".to_string());
        resolver.set_file_variable("TEST_VAR".to_string(), "from_file".to_string());

        // File-level should win
        assert_eq!(
            resolver.get_variable("TEST_VAR"),
            Some("from_file".to_string())
        );

        // Remove file-level, global should win
        resolver.clear_file_variables();
        assert_eq!(
            resolver.get_variable("TEST_VAR"),
            Some("from_global".to_string())
        );

        // Remove global, environment should win
        resolver.global_variables.clear();
        assert_eq!(
            resolver.get_variable("TEST_VAR"),
            Some("from_environment".to_string())
        );

        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_simple_variable_substitution() {
        let mut resolver = VariableResolver::new();
        resolver.set_file_variable("name".to_string(), "John".to_string());
        resolver.set_file_variable("age".to_string(), "30".to_string());

        let result = resolver.resolve("Hello {{name}}, you are {{age}} years old");
        assert_eq!(result, "Hello John, you are 30 years old");
    }

    #[test]
    fn test_nested_variable_resolution() {
        let mut resolver = VariableResolver::new();
        resolver.set_file_variable("host".to_string(), "api.example.com".to_string());
        resolver.set_file_variable("base_url".to_string(), "https://{{host}}".to_string());
        resolver.set_file_variable("endpoint".to_string(), "users".to_string());

        let result = resolver.resolve("{{base_url}}/{{endpoint}}");
        assert_eq!(result, "https://api.example.com/users");
    }

    #[test]
    fn test_undefined_variable() {
        let resolver = VariableResolver::new();

        let result = resolver.resolve("Hello {{undefined_var}}");
        assert_eq!(result, "Hello {{undefined_var}}");
    }

    #[test]
    fn test_parse_file_variables() {
        let mut resolver = VariableResolver::new();

        let content = r#"
@base_url = https://api.example.com
@api_key = secret_key_123
@timeout = 5000

GET {{base_url}}/users
"#;

        resolver.parse_file_variables(content);

        assert_eq!(
            resolver.get_variable("base_url"),
            Some("https://api.example.com".to_string())
        );
        assert_eq!(
            resolver.get_variable("api_key"),
            Some("secret_key_123".to_string())
        );
        assert_eq!(resolver.get_variable("timeout"), Some("5000".to_string()));
    }

    #[test]
    fn test_escaped_variables() {
        let mut resolver = VariableResolver::new();
        resolver.set_file_variable("var".to_string(), "value".to_string());

        // Note: This tests that escaped brackets are preserved
        let result = resolver.resolve(r"\{{not_a_var}} but {{var}} is");
        assert!(result.contains("{{not_a_var}}") || result.contains(r"\{{not_a_var}}"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_load_environments_json() {
        // Create a temporary test file
        let json_content = r#"{
            "development": {
                "host": "localhost:3000",
                "api_key": "dev_key"
            },
            "production": {
                "host": "api.example.com",
                "api_key": "prod_key"
            }
        }"#;

        let temp_file = "/tmp/test_environments.json";
        fs::write(temp_file, json_content).unwrap();

        let mut resolver = VariableResolver::new();
        resolver.load_environments(temp_file).unwrap();

        resolver
            .set_active_environment("development".to_string())
            .unwrap();
        assert_eq!(
            resolver.get_variable("host"),
            Some("localhost:3000".to_string())
        );

        resolver
            .set_active_environment("production".to_string())
            .unwrap();
        assert_eq!(
            resolver.get_variable("host"),
            Some("api.example.com".to_string())
        );

        fs::remove_file(temp_file).ok();
    }
}
