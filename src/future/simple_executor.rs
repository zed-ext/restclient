use crate::parser::{HttpMethod, HttpRequest};
use zed_extension_api as zed;

pub fn execute_request(request: &HttpRequest) -> Result<String, String> {
    // Convert our HttpMethod to Zed's HttpMethod
    let method = match request.method {
        HttpMethod::GET => zed::http_client::HttpMethod::Get,
        HttpMethod::POST => zed::http_client::HttpMethod::Post,
        HttpMethod::PUT => zed::http_client::HttpMethod::Put,
        HttpMethod::DELETE => zed::http_client::HttpMethod::Delete,
        HttpMethod::PATCH => zed::http_client::HttpMethod::Patch,
        HttpMethod::HEAD => zed::http_client::HttpMethod::Head,
        HttpMethod::OPTIONS => zed::http_client::HttpMethod::Options,
        HttpMethod::CONNECT | HttpMethod::TRACE => {
            return Err(format!("{:?} method not supported yet", request.method));
        }
    };

    // Build the HTTP request
    let mut builder = zed::http_client::HttpRequest::builder()
        .method(method)
        .url(&request.url);

    // Add headers
    for (name, value) in &request.headers {
        builder = builder.header(name.clone(), value.clone());
    }

    // Add body if present
    if let Some(ref body) = request.body {
        let body_string = match body {
            crate::parser::RequestBody::Inline(s) => s.clone(),
            _ => return Err("File references not yet supported".to_string()),
        };
        builder = builder.body(body_string.into_bytes());
    }

    // Execute the request
    let http_request = builder.build()?;
    let response = http_request.fetch()?;

    // Format the response
    let response_text = format!(
        "# Response\n\n## Headers\n{}\n\n## Body\n{}",
        response.headers.iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n"),
        String::from_utf8_lossy(&response.body)
    );

    Ok(response_text)
}
