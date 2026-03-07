use crate::parser::HttpRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

impl HttpResponse {
    pub fn new(
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        let content_type = headers
            .get("content-type")
            .or_else(|| headers.get("Content-Type"))
            .cloned();

        Self {
            status,
            status_text,
            headers,
            body,
            content_type,
        }
    }

    /// Get body as UTF-8 string
    pub fn body_as_string(&self) -> Result<String, String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| format!("Failed to parse body as UTF-8: {}", e))
    }

    /// Check if response is successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if response is JSON
    pub fn is_json(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Check if response is XML
    pub fn is_xml(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.contains("application/xml") || ct.contains("text/xml"))
            .unwrap_or(false)
    }

    /// Check if response is HTML
    pub fn is_html(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false)
    }

    /// Get a header value (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        let lower_name = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower_name)
            .map(|(_, v)| v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub request: HttpRequest,
    pub response: HttpResponse,
    pub duration: Duration,
    pub timestamp: SystemTime,
}

impl ExecutionResult {
    pub fn new(
        request: HttpRequest,
        response: HttpResponse,
        duration: Duration,
    ) -> Self {
        Self {
            request,
            response,
            duration,
            timestamp: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_content_type() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResponse::new(200, "OK".to_string(), headers, vec![]);

        assert!(response.is_json());
        assert!(!response.is_xml());
        assert!(!response.is_html());
    }

    #[test]
    fn test_response_success() {
        let response = HttpResponse::new(
            200,
            "OK".to_string(),
            HashMap::new(),
            vec![],
        );
        assert!(response.is_success());

        let response = HttpResponse::new(
            404,
            "Not Found".to_string(),
            HashMap::new(),
            vec![],
        );
        assert!(!response.is_success());
    }

    #[test]
    fn test_body_as_string() {
        let body = b"Hello, World!".to_vec();
        let response = HttpResponse::new(
            200,
            "OK".to_string(),
            HashMap::new(),
            body,
        );

        assert_eq!(response.body_as_string().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_get_header_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

        let response = HttpResponse::new(200, "OK".to_string(), headers, vec![]);

        assert_eq!(
            response.get_header("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            response.get_header("x-custom-header"),
            Some(&"custom-value".to_string())
        );
        assert_eq!(response.get_header("not-exists"), None);
    }
}
