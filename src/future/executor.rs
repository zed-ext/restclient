use crate::file_loader::FileLoader;
use crate::handler::{HandlerContext, ResponseHandlerRuntime};
use crate::parser::{HttpMethod, HttpRequest, RequestBody, ResponseHandler};
use crate::response::{ExecutionResult, HttpResponse};
use crate::variables::VariableResolver;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct RequestExecutor {
    variable_resolver: Arc<Mutex<VariableResolver>>,
    file_loader: Option<FileLoader>,
}

impl RequestExecutor {
    pub fn new(variable_resolver: Arc<Mutex<VariableResolver>>) -> Self {
        Self {
            variable_resolver,
            file_loader: None,
        }
    }

    /// Set the file loader (needed for resolving file references)
    pub fn set_file_loader(&mut self, file_loader: FileLoader) {
        self.file_loader = Some(file_loader);
    }

    /// Execute an HTTP request
    pub async fn execute(&self, request: HttpRequest) -> Result<ExecutionResult, String> {
        let start = Instant::now();

        // 1. Resolve variables in URL
        let url = {
            let resolver = self.variable_resolver.lock().unwrap();
            resolver.resolve(&request.url)
        };

        // 2. Resolve variables in headers
        let headers = self.resolve_headers(&request.headers)?;

        // 3. Resolve and load request body
        let body = self.resolve_body(&request.body).await?;

        // 4. Build and execute HTTP request
        let response = self.execute_http_request(
            &request.method,
            &url,
            &headers,
            body.as_deref(),
        ).await?;

        let duration = start.elapsed();

        // 5. Create execution result
        let result = ExecutionResult::new(request.clone(), response, duration);

        // 6. Execute response handler if present
        if let Some(handler) = &request.response_handler {
            self.execute_handler(handler, &request, &result.response).await?;
        }

        Ok(result)
    }

    /// Execute a response handler
    async fn execute_handler(
        &self,
        handler: &ResponseHandler,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> Result<(), String> {
        // Load handler script
        let script = match handler {
            ResponseHandler::Inline(code) => code.clone(),
            ResponseHandler::File(path) => {
                let file_loader = self.file_loader.as_ref()
                    .ok_or_else(|| "File loader not configured for handler file".to_string())?;
                file_loader.load_handler(path)?
            }
        };

        // Create handler context
        let context = HandlerContext {
            request: request.clone(),
            response: response.clone(),
        };

        // Execute handler
        let mut runtime = ResponseHandlerRuntime::new(self.variable_resolver.clone());
        runtime.execute(&script, context)?;

        // Log test results
        for test_result in runtime.get_test_results() {
            let status = if test_result.passed { "✓" } else { "✗" };
            println!("{} Test: {}", status, test_result.name);
        }

        if !runtime.all_tests_passed() {
            eprintln!("Warning: Some tests failed");
        }

        Ok(())
    }

    /// Resolve variables in headers
    fn resolve_headers(
        &self,
        headers: &[(String, String)],
    ) -> Result<Vec<(String, String)>, String> {
        let resolver = self.variable_resolver.lock().unwrap();
        let resolved: Vec<(String, String)> = headers
            .iter()
            .map(|(name, value)| {
                let resolved_name = resolver.resolve(name);
                let resolved_value = resolver.resolve(value);
                (resolved_name, resolved_value)
            })
            .collect();

        Ok(resolved)
    }

    /// Resolve and load request body
    async fn resolve_body(&self, body: &Option<RequestBody>) -> Result<Option<Vec<u8>>, String> {
        match body {
            None => Ok(None),
            Some(RequestBody::Inline(content)) => {
                // Resolve variables in inline body
                let resolver = self.variable_resolver.lock().unwrap();
                let resolved = resolver.resolve(content);
                Ok(Some(resolved.into_bytes()))
            }
            Some(RequestBody::FileReference(path)) => {
                // Load file content
                let file_loader = self.file_loader.as_ref()
                    .ok_or_else(|| "File loader not configured".to_string())?;

                let (bytes, _content_type) = file_loader.load_request_body(path)?;
                Ok(Some(bytes))
            }
            Some(RequestBody::Multipart(_parts)) => {
                // TODO: Implement multipart form data encoding
                // For now, return error
                Err("Multipart form data not yet implemented".to_string())
            }
        }
    }

    /// Execute HTTP request
    /// Note: This is a simplified implementation that will need to be
    /// updated based on the actual Zed HTTP API once we have access to it
    async fn execute_http_request(
        &self,
        method: &HttpMethod,
        url: &str,
        _headers: &[(String, String)],
        _body: Option<&[u8]>,
    ) -> Result<HttpResponse, String> {
        // Validate URL
        let _parsed_url = url::Url::parse(url)
            .map_err(|e| format!("Invalid URL '{}': {}", url, e))?;

        // For now, return a mock response
        // This will be replaced with actual Zed HTTP API calls
        // when the exact API is determined

        // Mock response for testing
        let mut response_headers = HashMap::new();
        response_headers.insert("Content-Type".to_string(), "application/json".to_string());

        let mock_body = format!(
            r#"{{"message":"Mock response for {} {}","timestamp":"{}","note":"Replace with Zed HTTP API"}}"#,
            method.as_str(),
            url,
            chrono::Utc::now().to_rfc3339()
        );

        Ok(HttpResponse::new(
            200,
            "OK".to_string(),
            response_headers,
            mock_body.into_bytes(),
        ))
    }

    /// Execute multiple requests in sequence
    pub async fn execute_all(
        &self,
        requests: Vec<HttpRequest>,
    ) -> Vec<Result<ExecutionResult, String>> {
        let mut results = Vec::new();

        for request in requests {
            let result = self.execute(request).await;
            results.push(result);
        }

        results
    }
}

/// Builder for constructing RequestExecutor with configuration
pub struct RequestExecutorBuilder {
    variable_resolver: Option<Arc<Mutex<VariableResolver>>>,
    file_loader: Option<FileLoader>,
}

impl RequestExecutorBuilder {
    pub fn new() -> Self {
        Self {
            variable_resolver: None,
            file_loader: None,
        }
    }

    pub fn with_variable_resolver(mut self, resolver: Arc<Mutex<VariableResolver>>) -> Self {
        self.variable_resolver = Some(resolver);
        self
    }

    pub fn with_file_loader(mut self, loader: FileLoader) -> Self {
        self.file_loader = Some(loader);
        self
    }

    pub fn build(self) -> Result<RequestExecutor, String> {
        let variable_resolver = self.variable_resolver
            .ok_or_else(|| "Variable resolver is required".to_string())?;

        let mut executor = RequestExecutor::new(variable_resolver);

        if let Some(loader) = self.file_loader {
            executor.set_file_loader(loader);
        }

        Ok(executor)
    }
}

impl Default for RequestExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{HttpMethod, RequestMetadata};

    #[tokio::test]
    async fn test_execute_simple_request() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let executor = RequestExecutor::new(resolver);

        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://api.example.com/users".to_string(),
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

        let result = executor.execute(request).await;
        assert!(result.is_ok());

        let execution_result = result.unwrap();
        assert_eq!(execution_result.response.status, 200);
        assert!(execution_result.duration.as_millis() >= 0);
    }

    #[tokio::test]
    async fn test_execute_with_variable_substitution() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        {
            let mut r = resolver.lock().unwrap();
            r.set_file_variable("base_url".to_string(), "https://api.example.com".to_string());
            r.set_file_variable("endpoint".to_string(), "users".to_string());
        }

        let executor = RequestExecutor::new(resolver);

        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "{{base_url}}/{{endpoint}}".to_string(),
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

        let result = executor.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_with_response_handler() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let executor = RequestExecutor::new(resolver.clone());

        let handler_script = r#"
            client.global.set("test_var", "test_value");
            client.test("Status check", function() {
                client.assert(response.status === 200);
            });
        "#;

        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://api.example.com/test".to_string(),
            http_version: None,
            headers: vec![],
            body: None,
            response_handler: Some(ResponseHandler::Inline(handler_script.to_string())),
            metadata: RequestMetadata {
                start_line: 0,
                end_line: 1,
                name: None,
            },
        };

        let result = executor.execute(request).await;
        assert!(result.is_ok());

        // Check that handler set the variable
        let r = resolver.lock().unwrap();
        assert_eq!(r.get_global("test_var"), Some(&"test_value".to_string()));
    }

    #[tokio::test]
    async fn test_execute_with_headers() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        {
            let mut r = resolver.lock().unwrap();
            r.set_file_variable("auth_token".to_string(), "Bearer abc123".to_string());
        }

        let executor = RequestExecutor::new(resolver);

        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://api.example.com/protected".to_string(),
            http_version: None,
            headers: vec![
                ("Authorization".to_string(), "{{auth_token}}".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body: None,
            response_handler: None,
            metadata: RequestMetadata {
                start_line: 0,
                end_line: 3,
                name: None,
            },
        };

        let result = executor.execute(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_with_inline_body() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        {
            let mut r = resolver.lock().unwrap();
            r.set_file_variable("user_id".to_string(), "42".to_string());
        }

        let executor = RequestExecutor::new(resolver);

        let request = HttpRequest {
            method: HttpMethod::POST,
            url: "https://api.example.com/users".to_string(),
            http_version: None,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(RequestBody::Inline(
                r#"{"name":"John","userId":{{user_id}}}"#.to_string()
            )),
            response_handler: None,
            metadata: RequestMetadata {
                start_line: 0,
                end_line: 5,
                name: None,
            },
        };

        let result = executor.execute(request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let resolver = Arc::new(Mutex::new(VariableResolver::new()));
        let file_loader = FileLoader::new(".");

        let executor = RequestExecutorBuilder::new()
            .with_variable_resolver(resolver.clone())
            .with_file_loader(file_loader)
            .build();

        assert!(executor.is_ok());
    }

    #[test]
    fn test_builder_missing_resolver() {
        let result = RequestExecutorBuilder::new().build();
        assert!(result.is_err());
    }
}
