use crate::variables::VariableResolver;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "CONNECT" => Ok(HttpMethod::CONNECT),
            "TRACE" => Ok(HttpMethod::TRACE),
            _ => Err(format!("Unknown HTTP method: {}", s)),
        }
    }
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::CONNECT => "CONNECT",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestBody {
    Inline(String),
    FileReference(PathBuf),
    Multipart(Vec<MultipartPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartPart {
    pub name: String,
    pub headers: Vec<(String, String)>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseHandler {
    File(PathBuf),
    Inline(String), // JavaScript code
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    pub start_line: usize,
    pub end_line: usize,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub http_version: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Option<RequestBody>,
    pub response_handler: Option<ResponseHandler>,
    pub metadata: RequestMetadata,
}

struct RequestBlock {
    content: String,
    start_line: usize,
    end_line: usize,
}

/// Parse an .http file content into a list of HTTP requests
pub fn parse_http_file(content: &str) -> Result<Vec<HttpRequest>, String> {
    let mut requests = Vec::new();

    // Create a variable resolver and parse file-level variables
    let mut resolver = VariableResolver::new();
    resolver.parse_file_variables(content);

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

/// Split content by ### separator while tracking line numbers
fn split_by_separator_with_lines(content: &str) -> Vec<RequestBlock> {
    let mut blocks = Vec::new();
    let mut current_block = String::new();
    let mut block_start_line = 1usize;
    let mut current_line = 1usize;

    for line in content.lines() {
        if line.trim().starts_with("###") {
            // Save previous block if it has content
            if !current_block.trim().is_empty() {
                blocks.push(RequestBlock {
                    content: current_block.clone(),
                    start_line: block_start_line,
                    end_line: current_line - 1,
                });
                current_block.clear();
            }
            // Start new block at this separator line (not after)
            // This allows users to click on the ### line to execute the request
            block_start_line = current_line;
        } else {
            current_block.push_str(line);
            current_block.push('\n');
        }
        current_line += 1;
    }

    // Push final block
    if !current_block.trim().is_empty() {
        blocks.push(RequestBlock {
            content: current_block,
            start_line: block_start_line,
            end_line: current_line - 1,
        });
    }

    blocks
}

/// Parse a single request block
fn parse_request_block(
    block: &str,
    start_line: usize,
    end_line: usize,
) -> Result<HttpRequest, String> {
    let lines: Vec<&str> = block.lines().collect();
    let mut line_idx = 0;

    // Skip comments, empty lines, and variable declarations; extract request name
    let mut request_name: Option<String> = None;
    while line_idx < lines.len() {
        let line = lines[line_idx].trim();
        if line.is_empty() {
            line_idx += 1;
            continue;
        }

        // Skip variable declarations: @variable = value
        if line.starts_with("@") {
            line_idx += 1;
            continue;
        }

        // Check for request name: # @name RequestName or // @name RequestName
        if (line.starts_with("#") || line.starts_with("//")) && line.contains("@name") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(idx) = parts.iter().position(|&p| p == "@name") {
                if idx + 1 < parts.len() {
                    request_name = Some(parts[idx + 1].to_string());
                }
            }
            line_idx += 1;
            continue;
        }

        // Skip comment lines
        if line.starts_with("#") || line.starts_with("//") {
            line_idx += 1;
            continue;
        }

        // Found non-comment line, break
        break;
    }

    if line_idx >= lines.len() {
        return Err("No request line found".to_string());
    }

    // Parse request line
    let (method, url, http_version) = parse_request_line(lines[line_idx])?;
    line_idx += 1;

    // Parse headers
    let mut headers = Vec::new();
    while line_idx < lines.len() {
        let line = lines[line_idx].trim();

        // Empty line marks end of headers
        if line.is_empty() {
            line_idx += 1;
            break;
        }

        // Parse header
        if let Some(colon_idx) = line.find(':') {
            let name = line[..colon_idx].trim().to_string();
            let value = line[colon_idx + 1..].trim().to_string();
            headers.push((name, value));
        }

        line_idx += 1;
    }

    // Parse body (everything after headers until response handler)
    let mut body_lines = Vec::new();
    let mut response_handler = None;

    while line_idx < lines.len() {
        let line = lines[line_idx];

        // Check for response handler
        if line.trim().starts_with(">") {
            let handler_content = line.trim()[1..].trim();
            if handler_content.starts_with("{%") {
                // Inline handler
                let mut handler_code = String::new();
                // Collect until %}
                let first_line = handler_content.trim_start_matches("{%").trim();
                if first_line.ends_with("%}") {
                    handler_code = first_line.trim_end_matches("%}").to_string();
                } else {
                    handler_code.push_str(first_line);
                    line_idx += 1;
                    while line_idx < lines.len() {
                        let next_line = lines[line_idx];
                        if next_line.trim().ends_with("%}") {
                            handler_code.push('\n');
                            handler_code.push_str(next_line.trim().trim_end_matches("%}"));
                            break;
                        }
                        handler_code.push('\n');
                        handler_code.push_str(next_line);
                        line_idx += 1;
                    }
                }
                response_handler = Some(ResponseHandler::Inline(handler_code));
            } else {
                // File reference
                response_handler = Some(ResponseHandler::File(PathBuf::from(handler_content)));
            }
            break;
        }

        body_lines.push(line);
        line_idx += 1;
    }

    // Process body
    let body = if body_lines.is_empty() || body_lines.iter().all(|l| l.trim().is_empty()) {
        None
    } else {
        let body_text = body_lines.join("\n").trim().to_string();

        // Check for file reference
        if let Some(stripped) = body_text.strip_prefix("<") {
            let file_path = stripped.trim();
            Some(RequestBody::FileReference(PathBuf::from(file_path)))
        } else {
            Some(RequestBody::Inline(body_text))
        }
    };

    Ok(HttpRequest {
        method,
        url,
        http_version,
        headers,
        body,
        response_handler,
        metadata: RequestMetadata {
            start_line,
            end_line,
            name: request_name,
        },
    })
}

/// Parse the request line: METHOD URL [HTTP-version]
fn parse_request_line(line: &str) -> Result<(HttpMethod, String, Option<String>), String> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(format!("Invalid request line: {}", line));
    }

    let method = parts[0].parse::<HttpMethod>()?;
    let url = parts[1].to_string();
    let http_version = if parts.len() > 2 {
        Some(parts[2].to_string())
    } else {
        None
    };

    Ok((method, url, http_version))
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
