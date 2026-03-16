mod completions;
mod parser;
mod variables;

use completions::{header_values, AUTH_SCHEMES, HEADER_NAMES, HTTP_METHODS};
use parser::{parse_http_file, HttpRequest};
use variables::VariableResolver;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// Must match Zed's lsp_ext_command.rs Runnable/ShellRunnableArgs structs exactly

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunnablesParams {
    text_document: TextDocumentIdentifier,
    _position: Option<Position>,
}

#[derive(Debug, serde::Serialize)]
struct Runnable {
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<LocationLink>,
    kind: &'static str,
    args: ShellRunnableArgs,
}

#[derive(Debug, serde::Serialize)]
struct ShellRunnableArgs {
    program: String,
    args: Vec<String>,
    cwd: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    environment: HashMap<String, String>,
}

#[derive(Debug)]
struct Document {
    content: String,
    version: i32,
    requests: Vec<HttpRequest>,
}

#[derive(Debug)]
struct HttpLsp {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, Document>>>,
}

impl HttpLsp {
    fn new(client: Client) -> Self {
        HttpLsp {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn load_env_vars_for_file(uri: &Url) -> HashMap<String, String> {
        let file_path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => return HashMap::new(),
        };
        let dir = file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let mut resolver = VariableResolver::new();
        if !resolver.load_environments_from_dir(dir) {
            return HashMap::new();
        }
        resolver.get_environment_variables()
    }

    async fn load_document_for_uri(&self, uri: &Url) -> Option<Document> {
        let docs = self.documents.read().await;
        if let Some(doc) = docs.get(uri) {
            return Some(Document {
                content: doc.content.clone(),
                version: doc.version,
                requests: doc.requests.clone(),
            });
        }
        drop(docs);

        let file_path = uri.to_file_path().ok()?;
        let content = std::fs::read_to_string(file_path).ok()?;
        let requests = parse_http_file(&content).ok()?;

        Some(Document {
            content,
            version: 0,
            requests,
        })
    }

    async fn handle_runnables(&self, params: RunnablesParams) -> Result<Vec<Runnable>> {
        let uri = params.text_document.uri;

        let doc = match self.load_document_for_uri(&uri).await {
            Some(d) => d,
            None => return Ok(Vec::new()),
        };

        if doc.requests.is_empty() {
            return Ok(Vec::new());
        }

        let file_path = uri
            .to_file_path()
            .unwrap_or_else(|_| PathBuf::from(uri.path()));

        let file_path_str = file_path.to_string_lossy().to_string();

        let cwd = file_path
            .parent()
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let mut runnables = Vec::new();

        for request in &doc.requests {
            let method_str = request.method.as_str();
            let start_line = request.metadata.start_line.saturating_sub(1) as u32;
            let end_line = request.metadata.end_line.saturating_sub(1) as u32;

            let label = format!("▶ Send {} {}", method_str, request.url);

            let location = LocationLink {
                origin_selection_range: None,
                target_uri: uri.clone(),
                target_range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: end_line,
                        character: 0,
                    },
                },
                target_selection_range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: start_line,
                        character: 100,
                    },
                },
            };

            runnables.push(Runnable {
                label,
                location: Some(location),
                kind: "shell",
                args: ShellRunnableArgs {
                    program: "http-client".to_string(),
                    args: vec![
                        file_path_str.clone(),
                        request.metadata.start_line.to_string(),
                    ],
                    cwd: cwd.clone(),
                    environment: HashMap::new(),
                },
            });
        }

        Ok(runnables)
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for HttpLsp {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(
                        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ :/{"
                            .chars()
                            .map(|c| c.to_string())
                            .collect(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "HTTP LSP initialized")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        let requests = parse_http_file(&content).unwrap_or_default();
        let doc = Document {
            content,
            version,
            requests,
        };
        let mut docs = self.documents.write().await;
        docs.insert(uri, doc);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().last() {
            let requests = parse_http_file(&change.text).unwrap_or_default();
            let doc = Document {
                content: change.text,
                version,
                requests,
            };
            let mut docs = self.documents.write().await;
            docs.insert(uri, doc);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().await;
        docs.remove(&params.text_document.uri);
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let docs = self.documents.read().await;

        let requests = if let Some(doc) = docs.get(&uri) {
            doc.requests.clone()
        } else {
            if let Ok(file_path) = uri.to_file_path() {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    parse_http_file(&content).unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        if requests.is_empty() {
            return Ok(None);
        }

        let mut symbols = Vec::new();

        for request in &requests {
            let start_line = request.metadata.start_line.saturating_sub(1) as u32;
            let end_line = request.metadata.end_line.saturating_sub(1) as u32;

            #[allow(deprecated)]
            let symbol = DocumentSymbol {
                name: format!("{:?} {}", request.method, request.url),
                detail: Some("HTTP Request".to_string()),
                kind: SymbolKind::METHOD,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: end_line,
                        character: 0,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: start_line,
                        character: 100,
                    },
                },
                children: None,
            };

            symbols.push(symbol);
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().await;
        let doc = if let Some(d) = docs.get(&uri) {
            d
        } else {
            return Ok(None);
        };

        let lines: Vec<&str> = doc.content.split('\n').collect();

        if position.line as usize >= lines.len() {
            return Ok(None);
        }

        let current_line = lines[position.line as usize];
        let char_pos = position.character.min(current_line.len() as u32) as usize;
        let before_cursor = &current_line[..char_pos];
        let trimmed = before_cursor.trim();

        let full_line_trimmed = current_line.trim();

        let mut items = Vec::new();

        let is_separator = full_line_trimmed.starts_with("###");
        let is_comment = full_line_trimmed.starts_with('#') || full_line_trimmed.starts_with("//");

        if is_separator || is_comment {
            return Ok(None);
        }

        let line_has_url =
            trimmed.contains("http://") || trimmed.contains("https://") || trimmed.contains("://");
        let line_has_colon_header = trimmed.contains(':') && !line_has_url;

        let line_num = position.line;

        let line_start_char = (current_line.len() - current_line.trim_start().len()) as u32;

        let block_start = (0..=position.line as usize)
            .rev()
            .find(|&i| lines[i].trim_start().starts_with("###"))
            .map(|i| i + 1)
            .unwrap_or(0);

        let has_request_line_before_cursor = (block_start..position.line as usize).any(|i| {
            let l = lines[i].trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with("//") {
                return false;
            }
            let mut parts = l.split_whitespace();
            let first = parts.next().unwrap_or("").to_ascii_uppercase();
            let second = parts.next().unwrap_or("");
            !second.is_empty()
                && matches!(
                    first.as_str(),
                    "GET"
                        | "POST"
                        | "PUT"
                        | "PATCH"
                        | "DELETE"
                        | "HEAD"
                        | "OPTIONS"
                        | "CONNECT"
                        | "TRACE"
                )
        });

        let first_meaningful_line_in_block = (block_start..=position.line as usize).find(|&i| {
            let l = lines[i].trim();
            !l.is_empty() && !l.starts_with('#') && !l.starts_with("//")
        });

        let is_first_meaningful_line =
            first_meaningful_line_in_block == Some(position.line as usize);

        // --- Context 1: HTTP methods ---
        let is_empty_line = trimmed.is_empty();

        let is_method_prefix = if trimmed.is_empty() {
            false
        } else {
            let trimmed_upper = trimmed.to_ascii_uppercase();
            trimmed.chars().all(|c| c.is_ascii_alphabetic())
                && HTTP_METHODS
                    .iter()
                    .any(|(m, _)| m.starts_with(&trimmed_upper))
        };
        let at_method_position = is_empty_line || is_method_prefix;

        if at_method_position
            && !line_has_url
            && !line_has_colon_header
            && is_first_meaningful_line
            && !has_request_line_before_cursor
        {
            let prefix_upper = trimmed.to_uppercase();
            let replace_start = line_start_char;
            for (method, desc) in HTTP_METHODS {
                if prefix_upper.is_empty() || method.starts_with(&prefix_upper) {
                    items.push(CompletionItem {
                        label: method.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(desc.to_string()),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                            range: Range {
                                start: Position {
                                    line: line_num,
                                    character: replace_start,
                                },
                                end: Position {
                                    line: line_num,
                                    character: position.character,
                                },
                            },
                            new_text: format!("{} ", method),
                        })),
                        filter_text: Some(method.to_string()),
                        sort_text: Some(format!("0{}", method)),
                        ..Default::default()
                    });
                }
            }
        }

        // --- Context 2: Header names ---
        let typing_header_name = !line_has_colon_header
            && !line_has_url
            && !is_empty_line
            && (!is_method_prefix || has_request_line_before_cursor);

        if typing_header_name {
            let prefix = trimmed.to_lowercase();
            let replace_start = line_start_char;
            for (header, default_value) in HEADER_NAMES {
                if prefix.is_empty() || header.to_lowercase().starts_with(&prefix) {
                    let new_text = format!("{}: ", header);
                    items.push(CompletionItem {
                        label: header.to_string(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(if default_value.is_empty() {
                            "Header".to_string()
                        } else {
                            default_value.to_string()
                        }),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                            range: Range {
                                start: Position {
                                    line: line_num,
                                    character: replace_start,
                                },
                                end: Position {
                                    line: line_num,
                                    character: position.character,
                                },
                            },
                            new_text,
                        })),
                        filter_text: Some(header.to_string()),
                        sort_text: Some(format!("1{}", header)),
                        ..Default::default()
                    });
                }
            }
        }

        // --- Context 3: Header values ---
        if line_has_colon_header {
            let colon_pos = trimmed.find(':').unwrap_or(0);
            let header_name = trimmed[..colon_pos].trim().to_lowercase();
            let after_colon = trimmed[colon_pos + 1..].trim();

            let value_start_char = if let Some(raw_colon) = before_cursor.find(':') {
                let after = &before_cursor[raw_colon + 1..];
                let spaces = after.len() - after.trim_start().len();
                (raw_colon + 1 + spaces) as u32
            } else {
                position.character
            };

            let value_items = header_values(&header_name);

            for val in value_items {
                if after_colon.is_empty()
                    || val.to_lowercase().starts_with(&after_colon.to_lowercase())
                {
                    items.push(CompletionItem {
                        label: format!("{}: {}", header_name, val),
                        kind: Some(CompletionItemKind::VALUE),
                        detail: Some(header_name.clone()),
                        filter_text: Some(format!("{}: {}", header_name, val)),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                            range: Range {
                                start: Position {
                                    line: line_num,
                                    character: value_start_char,
                                },
                                end: Position {
                                    line: line_num,
                                    character: position.character,
                                },
                            },
                            new_text: val.to_string(),
                        })),
                        sort_text: Some(format!("0{}", val)),
                        ..Default::default()
                    });
                }
            }

            if header_name == "authorization" {
                for val in AUTH_SCHEMES {
                    if after_colon.is_empty()
                        || val.to_lowercase().starts_with(&after_colon.to_lowercase())
                    {
                        items.push(CompletionItem {
                            label: format!("{}: {}", header_name, val.trim()),
                            kind: Some(CompletionItemKind::VALUE),
                            detail: Some("Auth scheme".to_string()),
                            filter_text: Some(format!("{}: {}", header_name, val.trim())),
                            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                range: Range {
                                    start: Position {
                                        line: line_num,
                                        character: value_start_char,
                                    },
                                    end: Position {
                                        line: line_num,
                                        character: position.character,
                                    },
                                },
                                new_text: val.to_string(),
                            })),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // --- Context 4: Variable references {{...}} ---
        if before_cursor.ends_with('{') || before_cursor.ends_with("{{") {
            let var_start_char = if before_cursor.ends_with("{{") {
                position.character - 2
            } else {
                position.character - 1
            };

            let mut seen_vars = std::collections::HashSet::new();

            for line in &lines {
                let l = line.trim();
                if l.starts_with('@') {
                    if let Some(eq_pos) = l.find('=') {
                        let var_name = l[1..eq_pos].trim();
                        if !var_name.is_empty() {
                            seen_vars.insert(var_name.to_string());
                            let var_value = l[eq_pos + 1..].trim();
                            items.push(CompletionItem {
                                label: format!("{{{{{}}}}}", var_name),
                                kind: Some(CompletionItemKind::VARIABLE),
                                detail: Some(var_value.to_string()),
                                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                                    range: Range {
                                        start: Position {
                                            line: line_num,
                                            character: var_start_char,
                                        },
                                        end: Position {
                                            line: line_num,
                                            character: position.character,
                                        },
                                    },
                                    new_text: format!("{{{{{}}}}}", var_name),
                                })),
                                filter_text: Some(var_name.to_string()),
                                sort_text: Some(format!("0{}", var_name)),
                                ..Default::default()
                            });
                        }
                    }
                }
            }

            let env_vars = Self::load_env_vars_for_file(&uri);
            for (var_name, var_value) in &env_vars {
                if seen_vars.contains(var_name) {
                    continue;
                }
                items.push(CompletionItem {
                    label: format!("{{{{{}}}}}", var_name),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(format!("env: {}", var_value)),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        range: Range {
                            start: Position {
                                line: line_num,
                                character: var_start_char,
                            },
                            end: Position {
                                line: line_num,
                                character: position.character,
                            },
                        },
                        new_text: format!("{{{{{}}}}}", var_name),
                    })),
                    filter_text: Some(var_name.to_string()),
                    sort_text: Some(format!("1{}", var_name)),
                    ..Default::default()
                });
            }
        }

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(HttpLsp::new)
        .custom_method("experimental/runnables", HttpLsp::handle_runnables)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
