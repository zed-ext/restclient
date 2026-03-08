use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zed_extension_api::{self as zed, Result};

pub mod parser;
pub mod variables;

// Future features - currently unused
// pub mod file_loader;
// pub mod response;
// pub mod executor;
// pub mod formatter;
// pub mod history;
// pub mod handler;
// pub mod simple_executor;

use parser::parse_http_file;
use variables::VariableResolver;

struct RestClientExtension {
    _variable_resolver: Arc<Mutex<VariableResolver>>,
}

impl zed::Extension for RestClientExtension {
    fn new() -> Self {
        let variable_resolver = Arc::new(Mutex::new(VariableResolver::new()));

        // Load environments.json if it exists in workspace
        let environments_path = ".http-client/environments.json";
        if std::path::Path::new(environments_path).exists() {
            if let Ok(mut resolver) = variable_resolver.lock() {
                let _ = resolver.load_environments(environments_path);
            }
        }

        Self {
            _variable_resolver: variable_resolver,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // No language server needed for this extension
        Err("No language server available".to_string())
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "send" => self.run_send_command(args, worktree),
            "http" => self.run_http_command(args, worktree),
            cmd => Err(format!("Unknown slash command: '{}'", cmd)),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "http" => {
                if args.is_empty() {
                    // Suggest .http files in the workspace
                    Ok(vec![zed::SlashCommandArgumentCompletion {
                        label: "test.http".to_string(),
                        new_text: "test.http".to_string(),
                        run_command: false,
                    }])
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]),
        }
    }
}

impl RestClientExtension {
    fn run_send_command(
        &self,
        _args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        // Get the current .http file from worktree
        let worktree = worktree.ok_or("No worktree available. Please open a workspace.")?;

        // Find .http files in the workspace
        let http_files = self.find_http_files(worktree)?;

        if http_files.is_empty() {
            return Ok(zed::SlashCommandOutput {
                text:
                    "No .http files found in workspace. Create a file with .http extension first."
                        .to_string(),
                sections: vec![],
            });
        }

        // Use the first .http file found
        let file_path = &http_files[0];
        self.execute_http_file(file_path, None)
    }

    fn run_http_command(
        &self,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("Usage: /http <file.http> [request-number]".to_string());
        }

        let file_path = &args[0];
        let request_index = if args.len() > 1 {
            args[1].parse::<usize>().ok().map(|n| n.saturating_sub(1))
        } else {
            None
        };

        self.execute_http_file(file_path, request_index)
    }

    fn execute_http_file(
        &self,
        file_path: &str,
        request_index: Option<usize>,
    ) -> Result<zed::SlashCommandOutput, String> {
        // Read the file
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;

        // Parse requests
        let requests =
            parse_http_file(&content).map_err(|e| format!("Failed to parse HTTP file: {}", e))?;

        if requests.is_empty() {
            return Err("No HTTP requests found in file".to_string());
        }

        // Select which request to execute
        let idx = request_index.unwrap_or(0);
        if idx >= requests.len() {
            return Err(format!(
                "Request #{} not found. File contains {} request(s).",
                idx + 1,
                requests.len()
            ));
        }

        let request = &requests[idx];

        // Log progress to stderr (visible in Zed's log panel)
        let method_str = format!("{:?}", request.method);
        eprintln!("⏳ Sending {} request to {}...", method_str, request.url);
        let _ = std::io::stderr().flush();

        let start_time = std::time::Instant::now();

        // Execute using Zed's HTTP client
        let response = self.execute_request_with_zed(request)?;

        let elapsed = start_time.elapsed();
        eprintln!("✓ Request completed in {:.2}s", elapsed.as_secs_f64());

        // Format output
        let output_text = format!(
            "# HTTP Request Executed\n\n\
             **{} {}**\n\n\
             ## Response\n\n\
             Headers:\n{}\n\n\
             Body:\n```\n{}\n```",
            method_str, request.url, response.headers_formatted, response.body_text
        );

        Ok(zed::SlashCommandOutput {
            text: output_text,
            sections: vec![],
        })
    }

    fn execute_request_with_zed(
        &self,
        request: &parser::HttpRequest,
    ) -> Result<HttpResponse, String> {
        use zed::http_client::{HttpMethod, HttpRequest};

        // Convert method
        let method = match request.method {
            parser::HttpMethod::GET => HttpMethod::Get,
            parser::HttpMethod::POST => HttpMethod::Post,
            parser::HttpMethod::PUT => HttpMethod::Put,
            parser::HttpMethod::DELETE => HttpMethod::Delete,
            parser::HttpMethod::PATCH => HttpMethod::Patch,
            parser::HttpMethod::HEAD => HttpMethod::Head,
            parser::HttpMethod::OPTIONS => HttpMethod::Options,
            _ => return Err(format!("Unsupported HTTP method: {:?}", request.method)),
        };

        // Build request
        let mut builder = HttpRequest::builder().method(method).url(&request.url);

        // Add headers
        for (name, value) in &request.headers {
            builder = builder.header(name.clone(), value.clone());
        }

        // Add body if present
        if let Some(ref body) = request.body {
            let body_bytes = match body {
                parser::RequestBody::Inline(s) => s.as_bytes().to_vec(),
                _ => return Err("File references not yet supported in slash commands".to_string()),
            };
            builder = builder.body(body_bytes);
        }

        // Execute request
        let http_request = builder.build()?;
        let response = http_request.fetch()?;

        // Parse response (HttpResponse only has headers and body, no status)
        let headers_formatted = response
            .headers
            .iter()
            .map(|(k, v)| format!("  {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        let body_text = String::from_utf8_lossy(&response.body).to_string();

        Ok(HttpResponse {
            headers_formatted,
            body_text,
        })
    }

    fn find_http_files(&self, worktree: &zed::Worktree) -> Result<Vec<String>, String> {
        // For now, return common .http file names
        // In a real implementation, we'd scan the worktree
        let candidates = vec!["test.http", "requests.http", "api.http"];

        let root = worktree.root_path();
        let mut found = Vec::new();

        for candidate in candidates {
            let path = PathBuf::from(&root).join(candidate);
            if path.exists() {
                found.push(candidate.to_string());
            }
        }

        Ok(found)
    }
}

struct HttpResponse {
    headers_formatted: String,
    body_text: String,
}

zed::register_extension!(RestClientExtension);
