use std::env;
use std::fs;
use std::path::Path;
use std::process;

use http_lsp::parser::{
    parse_http_file_with_resolver, HttpMethod, HttpRequest, RequestBody, ResponseHandler,
};
use http_lsp::response_handler::{execute_handler, load_globals, save_globals, ResponseContext};
use http_lsp::variables::VariableResolver;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

fn status_color(code: u16) -> &'static str {
    match code {
        200..=299 => GREEN,
        300..=399 => YELLOW,
        400..=499 => RED,
        500..=599 => RED,
        _ => RESET,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: http-client <file.http> <line-number> [--curl]");
        process::exit(1);
    }

    let file_path = &args[1];
    let line_number: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Invalid line number: {}", args[2]);
        process::exit(1);
    });
    let curl_mode = args.iter().any(|a| a == "--curl");

    let content = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", file_path, e);
        process::exit(1);
    });

    let file_parent = Path::new(file_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let mut globals = load_globals(file_parent);

    let mut resolver = VariableResolver::new();
    for (name, value) in &globals {
        resolver.set_global(name.clone(), value.clone());
    }

    resolver.load_environments_from_dir(file_parent);

    let requests = parse_http_file_with_resolver(&content, &mut resolver).unwrap_or_else(|e| {
        eprintln!("Failed to parse HTTP file: {}", e);
        process::exit(1);
    });

    let request = requests
        .iter()
        .find(|r| line_number >= r.metadata.block_start_line && line_number <= r.metadata.end_line)
        // Fallback: if the line falls on a ### separator or gap between blocks,
        // find the nearest request block that starts at or right after this line
        .or_else(|| {
            requests
                .iter()
                .filter(|r| r.metadata.block_start_line >= line_number)
                .min_by_key(|r| r.metadata.block_start_line)
        })
        .unwrap_or_else(|| {
            eprintln!("No HTTP request found at line {}", line_number);
            process::exit(1);
        });

    if curl_mode {
        generate_curl(request);
    } else {
        let method_str = format!("{:?}", request.method);

        println!("{BOLD}{CYAN}▶ {method_str} {}{RESET}", request.url);
        println!("{DIM}─────────────────────────────────────{RESET}");
        println!();

        match execute_request(request, &mut globals, file_parent) {
            Ok(()) => {}
            Err(e) => {
                println!("{RED}{BOLD}✗ Request failed:{RESET} {RED}{e}{RESET}");
                process::exit(1);
            }
        }
    }
}

fn generate_curl(request: &HttpRequest) {
    let mut parts = vec![
        "curl".to_string(),
        "-X".to_string(),
        format!("{:?}", request.method),
        shell_quote(&request.url),
    ];

    for (name, value) in &request.headers {
        parts.push("-H".to_string());
        parts.push(shell_quote(&format!("{}: {}", name, value)));
    }

    if let Some(RequestBody::Inline(body)) = &request.body {
        parts.push("--data-raw".to_string());
        parts.push(shell_quote(body));
    }

    let curl_command = parts.join(" ");

    match copy_to_clipboard(&curl_command) {
        Ok(()) => {
            println!("{GREEN}{BOLD}✓ Copied to clipboard{RESET}");
            println!();
            println!("{DIM}{curl_command}{RESET}");
        }
        Err(e) => {
            // Fallback: print so user can copy manually
            eprintln!("{RED}{BOLD}✗ Clipboard failed:{RESET} {RED}{e}{RESET}");
            println!();
            println!("{curl_command}");
        }
    }
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("pbcopy", &[])
    } else {
        ("xclip", &["-selection", "clipboard"])
    };

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;

    child
        .stdin
        .as_mut()
        .ok_or("Failed to open stdin")?
        .write_all(text.as_bytes())
        .map_err(|e| format!("Write failed: {e}"))?;

    let output = child.wait_with_output().map_err(|e| format!("{e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{cmd} failed: {stderr}"))
    }
}

fn execute_request(
    request: &HttpRequest,
    globals: &mut std::collections::HashMap<String, String>,
    http_file_dir: &Path,
) -> Result<(), String> {
    use reqwest::blocking::Client;
    use std::io::Write;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let client = Client::new();

    let method = match request.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
        _ => return Err(format!("Unsupported method: {:?}", request.method)),
    };

    let mut req_builder = client.request(method, &request.url);

    for (name, value) in &request.headers {
        req_builder = req_builder.header(name, value);
    }

    if let Some(ref body) = request.body {
        match body {
            RequestBody::Inline(s) => {
                req_builder = req_builder.body(s.clone());
            }
            _ => return Err("File references not supported".to_string()),
        }
    }

    let start = Instant::now();
    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    let spinner_handle = std::thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let colors = [
            "\x1b[36m", // cyan
            "\x1b[96m", // bright cyan
            "\x1b[34m", // blue
            "\x1b[94m", // bright blue
            "\x1b[35m", // magenta
            "\x1b[95m", // bright magenta
            "\x1b[33m", // yellow
            "\x1b[93m", // bright yellow
            "\x1b[32m", // green
            "\x1b[92m", // bright green
        ];
        let mut i = 0;
        loop {
            if done_clone.load(Ordering::Relaxed) {
                break;
            }
            let ms = start.elapsed().as_millis();
            let display = if ms >= 1000 {
                format!("{:.1}s", ms as f64 / 1000.0)
            } else {
                format!("{}ms", ms)
            };
            let color = colors[i % colors.len()];
            print!(
                "\r{color}{} Waiting... {display}{RESET}    ",
                frames[i % frames.len()],
            );
            let _ = std::io::stdout().flush();
            i += 1;
            std::thread::sleep(Duration::from_millis(80));
        }
        print!("\r\x1b[2K");
        let _ = std::io::stdout().flush();
    });

    let result = req_builder.send();
    let elapsed = start.elapsed();
    done.store(true, Ordering::Relaxed);
    let _ = spinner_handle.join();

    let response = result.map_err(|e| {
        // Walk the error chain to show the root cause (DNS failure, connection refused, etc.)
        let mut msg = e.to_string();
        let mut source = std::error::Error::source(&e);
        while let Some(cause) = source {
            msg = format!("{msg}\n  → {cause}");
            source = std::error::Error::source(cause);
        }
        msg
    })?;

    let status = response.status();
    let status_code = status.as_u16();
    let status_text = status.canonical_reason().unwrap_or("");
    let headers = response.headers().clone();
    let body = response
        .text()
        .map_err(|e| format!("Failed to read body: {}", e))?;

    let color = status_color(status_code);

    let secs = elapsed.as_secs_f64();
    let elapsed_display = if secs >= 1.0 {
        format!("{secs:.3}s")
    } else {
        format!("{}ms", elapsed.as_millis())
    };
    println!(
        "{BOLD}{color}HTTP {status_code} {status_text}{RESET}  {DIM}({elapsed_display}){RESET}"
    );
    println!();

    for (name, value) in headers.iter() {
        let val = value.to_str().unwrap_or("<binary>");
        println!("{MAGENTA}{name}{RESET}{DIM}:{RESET} {val}");
    }

    println!();

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
            print_colored_json(&parsed, 0);
            println!();
        }
    } else {
        println!("{body}");
    }

    // Execute response handler after displaying the response
    if let Some(ResponseHandler::Inline(script)) = &request.response_handler {
        let mut headers_map = std::collections::HashMap::new();
        for (name, value) in &headers {
            headers_map.insert(
                name.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            );
        }

        let response_context = ResponseContext {
            status: status_code,
            headers: headers_map,
            body: serde_json::from_str::<serde_json::Value>(&body).ok(),
            body_raw: body.clone(),
        };

        match execute_handler(script, &response_context) {
            Ok(captured) => {
                if !captured.is_empty() {
                    for (k, v) in &captured {
                        globals.insert(k.clone(), v.clone());
                    }

                    save_globals(http_file_dir, globals)?;

                    let mut names: Vec<_> = captured.keys().cloned().collect();
                    names.sort();
                    println!();
                    println!("{DIM}─────────────────────────────────────{RESET}");
                    println!("{GREEN}{BOLD}✓ Saved: {}{RESET}", names.join(", "));
                }
            }
            Err(e) => {
                eprintln!("{YELLOW}Response handler skipped: {e}{RESET}");
            }
        }
    }

    Ok(())
}

fn print_colored_json(value: &serde_json::Value, indent: usize) {
    const BLUE: &str = "\x1b[34m";
    const STR_GREEN: &str = "\x1b[32m";
    const NUM_CYAN: &str = "\x1b[36m";
    const BOOL_YELLOW: &str = "\x1b[33m";
    const NULL_RED: &str = "\x1b[31m";
    const KEY_MAGENTA: &str = "\x1b[35m";
    const BRACE: &str = "\x1b[1m";

    let pad = "  ".repeat(indent);
    let inner_pad = "  ".repeat(indent + 1);

    match value {
        serde_json::Value::Null => print!("{NULL_RED}null{RESET}"),
        serde_json::Value::Bool(b) => print!("{BOOL_YELLOW}{b}{RESET}"),
        serde_json::Value::Number(n) => print!("{NUM_CYAN}{n}{RESET}"),
        serde_json::Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            print!("{STR_GREEN}\"{escaped}\"{RESET}");
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                print!("{BRACE}[]{RESET}");
                return;
            }
            println!("{BRACE}[{RESET}");
            for (i, item) in arr.iter().enumerate() {
                print!("{inner_pad}");
                print_colored_json(item, indent + 1);
                if i < arr.len() - 1 {
                    println!(",");
                } else {
                    println!();
                }
            }
            print!("{pad}{BRACE}]{RESET}");
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                print!("{BRACE}{{}}{RESET}");
                return;
            }
            println!("{BRACE}{{{RESET}");
            let len = obj.len();
            for (i, (key, val)) in obj.iter().enumerate() {
                print!("{inner_pad}{KEY_MAGENTA}\"{key}\"{RESET}{BLUE}:{RESET} ");
                print_colored_json(val, indent + 1);
                if i < len - 1 {
                    println!(",");
                } else {
                    println!();
                }
            }
            print!("{pad}{BRACE}}}{RESET}");
        }
    }
}
