use crate::response::{ExecutionResult, HttpResponse};
use chrono::{DateTime, Local};
use std::time::SystemTime;

pub struct ResponseFormatter;

impl ResponseFormatter {
    /// Format an execution result into a human-readable string
    pub fn format(result: &ExecutionResult) -> String {
        let mut output = String::new();

        // Request line
        output.push_str(&format!(
            "### {} {}\n",
            result.request.method.as_str(),
            result.request.url
        ));

        // Status line
        output.push_str(&format!(
            "### Response: {} {}\n",
            result.response.status,
            result.response.status_text
        ));

        // Timing and metadata
        output.push_str(&format!(
            "### Duration: {}ms\n",
            result.duration.as_millis()
        ));

        output.push_str(&format!(
            "### Time: {}\n",
            Self::format_timestamp(result.timestamp)
        ));

        output.push('\n');

        // Headers
        if !result.response.headers.is_empty() {
            output.push_str("### Response Headers\n");
            for (name, value) in &result.response.headers {
                output.push_str(&format!("{}: {}\n", name, value));
            }
            output.push('\n');
        }

        // Body
        output.push_str("### Response Body\n");
        output.push_str(&Self::format_body(&result.response));

        output
    }

    /// Format response body based on content type
    fn format_body(response: &HttpResponse) -> String {
        if response.body.is_empty() {
            return "(empty body)\n".to_string();
        }

        // Try to parse as UTF-8 text
        match response.body_as_string() {
            Ok(body_str) => {
                // Format based on content type
                if response.is_json() {
                    Self::format_json(&body_str)
                } else if response.is_xml() || response.is_html() {
                    Self::format_xml(&body_str)
                } else {
                    // Plain text
                    body_str
                }
            }
            Err(_) => {
                // Binary content
                format!(
                    "(binary content, {} bytes)\n{}\n",
                    response.body.len(),
                    Self::format_hex_preview(&response.body)
                )
            }
        }
    }

    /// Pretty-print JSON with indentation
    fn format_json(json_str: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(value) => match serde_json::to_string_pretty(&value) {
                Ok(pretty) => format!("{}\n", pretty),
                Err(_) => format!("{}\n", json_str),
            },
            Err(_) => {
                // Not valid JSON, return as-is
                format!("{}\n", json_str)
            }
        }
    }

    /// Format XML/HTML with basic indentation
    fn format_xml(xml_str: &str) -> String {
        // For MVP, just return as-is with newline
        // Full XML formatting would require an XML parser
        // TODO: Add proper XML formatting library
        format!("{}\n", xml_str)
    }

    /// Format binary content as hex preview (first 256 bytes)
    fn format_hex_preview(bytes: &[u8]) -> String {
        let preview_len = bytes.len().min(256);
        let preview = &bytes[..preview_len];

        let mut output = String::new();
        output.push_str("Hex preview (first 256 bytes):\n");

        for (i, chunk) in preview.chunks(16).enumerate() {
            // Offset
            output.push_str(&format!("{:08x}  ", i * 16));

            // Hex values
            for byte in chunk {
                output.push_str(&format!("{:02x} ", byte));
            }

            // Padding for incomplete lines
            for _ in chunk.len()..16 {
                output.push_str("   ");
            }

            // ASCII representation
            output.push_str(" |");
            for byte in chunk {
                if byte.is_ascii_graphic() || *byte == b' ' {
                    output.push(*byte as char);
                } else {
                    output.push('.');
                }
            }
            output.push_str("|\n");
        }

        if bytes.len() > preview_len {
            output.push_str(&format!("... ({} more bytes)\n", bytes.len() - preview_len));
        }

        output
    }

    /// Format timestamp as human-readable string
    fn format_timestamp(time: SystemTime) -> String {
        let datetime: DateTime<Local> = time.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Format response in compact mode (one-liner)
    pub fn format_compact(result: &ExecutionResult) -> String {
        format!(
            "{} {} → {} {} ({}ms)",
            result.request.method.as_str(),
            result.request.url,
            result.response.status,
            result.response.status_text,
            result.duration.as_millis()
        )
    }

    /// Format only the status line
    pub fn format_status(response: &HttpResponse) -> String {
        format!("{} {}", response.status, response.status_text)
    }

    /// Format body size information
    pub fn format_body_info(response: &HttpResponse) -> String {
        let size = response.body.len();
        let content_type = response
            .content_type
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        if size == 0 {
            "Empty body".to_string()
        } else if size < 1024 {
            format!("{} bytes ({})", size, content_type)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB ({})", size as f64 / 1024.0, content_type)
        } else {
            format!("{:.1} MB ({})", size as f64 / (1024.0 * 1024.0), content_type)
        }
    }

    /// Format headers as a table-like display
    pub fn format_headers_table(headers: &std::collections::HashMap<String, String>) -> String {
        if headers.is_empty() {
            return "(no headers)".to_string();
        }

        let mut output = String::new();
        let max_name_len = headers.keys().map(|k| k.len()).max().unwrap_or(0);

        for (name, value) in headers {
            output.push_str(&format!(
                "{:width$}  {}\n",
                name,
                value,
                width = max_name_len
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{HttpMethod, HttpRequest, RequestMetadata};
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_result(status: u16, body: Vec<u8>, content_type: Option<&str>) -> ExecutionResult {
        let mut headers = HashMap::new();
        if let Some(ct) = content_type {
            headers.insert("Content-Type".to_string(), ct.to_string());
        }

        let response = HttpResponse::new(
            status,
            "OK".to_string(),
            headers,
            body,
        );

        let request = HttpRequest {
            method: HttpMethod::GET,
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

        ExecutionResult::new(request, response, Duration::from_millis(123))
    }

    #[test]
    fn test_format_json_response() {
        let json_body = r#"{"name":"John","age":30,"city":"New York"}"#;
        let result = create_test_result(200, json_body.as_bytes().to_vec(), Some("application/json"));

        let formatted = ResponseFormatter::format(&result);

        assert!(formatted.contains("GET https://api.example.com/test"));
        assert!(formatted.contains("Response: 200 OK"));
        assert!(formatted.contains("Duration: 123ms"));
        assert!(formatted.contains("Response Body"));
        // Should be pretty-printed
        assert!(formatted.contains("\"name\": \"John\""));
    }

    #[test]
    fn test_format_plain_text() {
        let text_body = "Hello, World!";
        let result = create_test_result(200, text_body.as_bytes().to_vec(), Some("text/plain"));

        let formatted = ResponseFormatter::format(&result);

        assert!(formatted.contains("Hello, World!"));
        assert!(formatted.contains("Response Body"));
    }

    #[test]
    fn test_format_binary_content() {
        let binary_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        let result = create_test_result(200, binary_data, Some("image/png"));

        let formatted = ResponseFormatter::format(&result);

        assert!(formatted.contains("(binary content"));
        assert!(formatted.contains("Hex preview"));
    }

    #[test]
    fn test_format_compact() {
        let result = create_test_result(200, b"test".to_vec(), None);
        let compact = ResponseFormatter::format_compact(&result);

        assert_eq!(compact, "GET https://api.example.com/test → 200 OK (123ms)");
    }

    #[test]
    fn test_format_empty_body() {
        let result = create_test_result(204, vec![], None);
        let formatted = ResponseFormatter::format(&result);

        assert!(formatted.contains("(empty body)"));
    }

    #[test]
    fn test_format_body_info() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResponse::new(200, "OK".to_string(), headers, vec![0u8; 2048]);
        let info = ResponseFormatter::format_body_info(&response);

        assert!(info.contains("2.0 KB"));
        assert!(info.contains("application/json"));
    }

    #[test]
    fn test_format_headers_table() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Cache-Control".to_string(), "no-cache".to_string());

        let formatted = ResponseFormatter::format_headers_table(&headers);

        assert!(formatted.contains("Content-Type"));
        assert!(formatted.contains("Cache-Control"));
        assert!(formatted.contains("application/json"));
    }

    #[test]
    fn test_format_invalid_json() {
        let invalid_json = "{not valid json}";
        let result = create_test_result(200, invalid_json.as_bytes().to_vec(), Some("application/json"));

        let formatted = ResponseFormatter::format(&result);

        // Should still display even if JSON is invalid
        assert!(formatted.contains("{not valid json}"));
    }
}
