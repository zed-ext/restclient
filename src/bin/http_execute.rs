use std::env;
use std::fs;
use std::io::{self, Read};
use zed_restclient::parser::{parse_http_file, HttpMethod};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut file_path: Option<String> = None;
    let mut line_number: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--line" => {
                if i + 1 < args.len() {
                    line_number = args[i + 1].parse().ok();
                    i += 2;
                } else {
                    eprintln!("Error: --line requires a line number");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with("--") => {
                file_path = Some(arg.to_string());
                i += 1;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    // Read content from stdin or file
    let content = if let Some(path) = file_path {
        fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading file {}: {}", path, e);
            std::process::exit(1);
        })
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        });
        buffer
    };

    // Parse HTTP requests
    let requests = parse_http_file(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing HTTP request: {}", e);
        std::process::exit(1);
    });

    if requests.is_empty() {
        eprintln!("No HTTP requests found in input");
        std::process::exit(1);
    }

    // Select which request to execute
    let request = if let Some(line) = line_number {
        // Find request that contains this line
        find_request_at_line(&requests, line).unwrap_or_else(|| {
            eprintln!("No HTTP request found at line {}", line);
            eprintln!("Available requests:");
            for (i, req) in requests.iter().enumerate() {
                eprintln!(
                    "  {}. {} {} (lines {}-{})",
                    i + 1,
                    format!("{:?}", req.method),
                    req.url,
                    req.metadata.start_line,
                    req.metadata.end_line
                );
            }
            std::process::exit(1);
        })
    } else {
        // Default to first request
        &requests[0]
    };

    // Print which request we're executing
    eprintln!(
        ">>> Executing: {} {}",
        format!("{:?}", request.method),
        request.url
    );
    eprintln!();

    // Convert to reqwest
    let client = reqwest::blocking::Client::new();

    let method = match request.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
        _ => {
            eprintln!("Unsupported HTTP method: {:?}", request.method);
            std::process::exit(1);
        }
    };

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for (name, value) in &request.headers {
        req_builder = req_builder.header(name, value);
    }

    // Add body if present
    if let Some(ref body) = request.body {
        match body {
            zed_restclient::parser::RequestBody::Inline(s) => {
                req_builder = req_builder.body(s.clone());
            }
            _ => {
                eprintln!("File references not yet supported");
                std::process::exit(1);
            }
        }
    }

    // Execute request
    match req_builder.send() {
        Ok(response) => {
            println!(
                "HTTP/{:?} {} {}",
                response.version(),
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("")
            );
            println!();

            // Print headers
            for (name, value) in response.headers() {
                println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
            }
            println!();

            // Print body
            match response.text() {
                Ok(body) => println!("{}", body),
                Err(e) => eprintln!("Error reading response body: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error executing request: {}", e);
            std::process::exit(1);
        }
    }
}

fn find_request_at_line<'a>(
    requests: &'a [zed_restclient::parser::HttpRequest],
    line: usize,
) -> Option<&'a zed_restclient::parser::HttpRequest> {
    requests
        .iter()
        .find(|req| line >= req.metadata.start_line && line <= req.metadata.end_line)
}
