//! Truss LSP
//!
//! Language Server Protocol adapter for Truss.
//! This crate adapts `truss-core` to the LSP protocol,
//! handling documents, versions, and diagnostics.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use truss_core::{TrussEngine, Diagnostic as CoreDiagnostic, Severity as CoreSeverity};

/// LSP message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum LspMessage {
    Request(LspRequest),
    Response(LspResponse),
    Notification(LspNotification),
}

#[derive(Debug, Serialize, Deserialize)]
struct LspRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LspResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<LspError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LspError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LspNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// LSP server state
struct LspServer {
    engine: TrussEngine,
    documents: HashMap<String, DocumentState>,
    initialized: bool,
    shutdown: bool,
}

struct DocumentState {
    text: String,
    version: i32,
    tree: Option<tree_sitter::Tree>,
}

impl LspServer {
    fn new() -> Self {
        Self {
            engine: TrussEngine::new(),
            documents: HashMap::new(),
            initialized: false,
            shutdown: false,
        }
    }

    fn handle_message(&mut self, message: LspMessage) -> Vec<LspMessage> {
        let mut responses = Vec::new();

        match message {
            LspMessage::Request(req) => {
                if let Some(response) = self.handle_request(req) {
                    responses.push(LspMessage::Response(response));
                }
            }
            LspMessage::Notification(notif) => {
                let notifications = self.handle_notification(notif);
                responses.extend(notifications.into_iter().map(LspMessage::Notification));
            }
            LspMessage::Response(_) => {}
        }

        responses
    }

    fn handle_request(&mut self, req: LspRequest) -> Option<LspResponse> {
        match req.method.as_str() {
            "initialize" => {
                self.initialized = true;
                let result = serde_json::json!({
                    "capabilities": {
                        "textDocumentSync": {
                            "openClose": true,
                            "change": 1, // TextDocumentSyncKind.Full
                            "save": false
                        }
                    },
                    "serverInfo": {
                        "name": "truss",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                Some(LspResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(result),
                    error: None,
                })
            }
            "shutdown" => {
                self.shutdown = true;
                Some(LspResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(serde_json::Value::Null),
                    error: None,
                })
            }
            _ => {
                if !self.initialized {
                    return Some(LspResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(LspError {
                            code: -32002, // ServerNotInitialized
                            message: "Server not initialized".to_string(),
                            data: None,
                        }),
                    });
                }
                Some(LspResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(LspError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                        data: None,
                    }),
                })
            }
        }
    }

    fn handle_notification(&mut self, notif: LspNotification) -> Vec<LspNotification> {
        let mut notifications = Vec::new();

        match notif.method.as_str() {
            "initialized" => {
                // Client confirms initialization complete
            }
            "textDocument/didOpen" => {
                if !self.initialized {
                    return notifications;
                }
                if let Some(params) = notif.params {
                    if let Ok(did_open) = serde_json::from_value::<DidOpenTextDocumentParams>(params) {
                        self.handle_did_open(did_open, &mut notifications);
                    }
                }
            }
            "textDocument/didChange" => {
                if !self.initialized {
                    return notifications;
                }
                if let Some(params) = notif.params {
                    if let Ok(did_change) = serde_json::from_value::<DidChangeTextDocumentParams>(params) {
                        self.handle_did_change(did_change, &mut notifications);
                    }
                }
            }
            "textDocument/didClose" => {
                if !self.initialized {
                    return notifications;
                }
                if let Some(params) = notif.params {
                    if let Ok(did_close) = serde_json::from_value::<DidCloseTextDocumentParams>(params) {
                        self.handle_did_close(did_close, &mut notifications);
                    }
                }
            }
            "exit" => {
                self.shutdown = true;
            }
            _ => {}
        }

        notifications
    }

    fn handle_did_open(&mut self, params: DidOpenTextDocumentParams, notifications: &mut Vec<LspNotification>) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        let text_for_diagnostics = text.clone();
        let (result, tree) = self.engine.analyze_with_tree(&text);

        self.documents.insert(uri.clone(), DocumentState {
            text,
            version,
            tree: Some(tree),
        });

        let diagnostics = self.convert_diagnostics(&result.diagnostics, &text_for_diagnostics);
        notifications.push(LspNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            params: Some(serde_json::json!({
                "uri": uri,
                "diagnostics": diagnostics
            })),
        });
    }

    fn handle_did_change(&mut self, params: DidChangeTextDocumentParams, notifications: &mut Vec<LspNotification>) {
        let uri = params.text_document.uri;

        // We advertise TextDocumentSyncKind.Full (1), so the client always
        // sends the full document content in each change event.
        let new_text = match params.content_changes.into_iter().last() {
            Some(change) => change.text,
            None => return,
        };

        let version = params.text_document.version;

        let old_tree = self.documents.get(&uri)
            .and_then(|doc| doc.tree.as_ref());

        let (result, tree) = if let Some(old) = old_tree {
            self.engine.analyze_incremental_with_tree(&new_text, Some(old))
        } else {
            self.engine.analyze_with_tree(&new_text)
        };

        let text_for_diagnostics = new_text.clone();
        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.text = new_text;
            doc.version = version;
            doc.tree = Some(tree);
        } else {
            self.documents.insert(uri.clone(), DocumentState {
                text: new_text,
                version,
                tree: Some(tree),
            });
        }

        let diagnostics = self.convert_diagnostics(&result.diagnostics, &text_for_diagnostics);
        notifications.push(LspNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            params: Some(serde_json::json!({
                "uri": uri,
                "diagnostics": diagnostics
            })),
        });
    }

    fn handle_did_close(&mut self, params: DidCloseTextDocumentParams, notifications: &mut Vec<LspNotification>) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);

        // Clear diagnostics for the closed document
        notifications.push(LspNotification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            params: Some(serde_json::json!({
                "uri": uri,
                "diagnostics": []
            })),
        });
    }

    fn convert_diagnostics(&self, diagnostics: &[CoreDiagnostic], text: &str) -> Vec<Value> {
        diagnostics.iter().map(|d| {
            let (start_line, start_char) = byte_to_lsp_position(d.span.start, text);
            let (end_line, end_char) = byte_to_lsp_position(d.span.end, text);

            serde_json::json!({
                "range": {
                    "start": {
                        "line": start_line,
                        "character": start_char
                    },
                    "end": {
                        "line": end_line,
                        "character": end_char
                    }
                },
                "severity": match d.severity {
                    CoreSeverity::Error => 1,
                    CoreSeverity::Warning => 2,
                    CoreSeverity::Info => 3,
                },
                "message": d.message,
                "source": "truss"
            })
        }).collect()
    }
}

/// Convert a byte offset in `text` to an LSP position (line, character).
///
/// LSP positions use zero-based line numbers and character offsets measured
/// in UTF-16 code units. For ASCII text, UTF-16 code units equal byte offsets
/// within the line. For non-ASCII text, we must count UTF-16 code units properly.
fn byte_to_lsp_position(byte_offset: usize, text: &str) -> (u32, u32) {
    let clamped = byte_offset.min(text.len());
    let bytes_before = &text[..clamped];
    let line = bytes_before.matches('\n').count() as u32;
    let last_newline = bytes_before.rfind('\n').map(|i| i + 1).unwrap_or(0);

    // Count UTF-16 code units from last_newline to byte_offset
    let line_bytes = &text[last_newline..clamped];
    let character = line_bytes.chars()
        .map(|c| c.len_utf16() as u32)
        .sum::<u32>();

    (line, character)
}

#[derive(Debug, Deserialize)]
struct DidOpenTextDocumentParams {
    text_document: TextDocumentItem,
}

#[derive(Debug, Deserialize)]
struct DidChangeTextDocumentParams {
    text_document: VersionedTextDocumentIdentifier,
    content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Deserialize)]
struct DidCloseTextDocumentParams {
    text_document: TextDocumentIdentifier,
}

#[derive(Debug, Deserialize)]
struct TextDocumentItem {
    uri: String,
    #[allow(dead_code)]
    language_id: String,
    version: i32,
    text: String,
}

#[derive(Debug, Deserialize)]
struct VersionedTextDocumentIdentifier {
    uri: String,
    version: i32,
}

#[derive(Debug, Deserialize)]
struct TextDocumentIdentifier {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct TextDocumentContentChangeEvent {
    text: String,
}

/// Run the LSP server on stdin/stdout.
pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();
    let mut server = LspServer::new();

    loop {
        // Read headers until empty line
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;

            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }

            if let Some(value) = trimmed.strip_prefix("Content-Length:") {
                if let Ok(len) = value.trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }

        let content_length = match content_length {
            Some(len) if len > 0 => len,
            _ => continue,
        };

        // Read exactly content_length bytes
        let mut content = vec![0u8; content_length];
        reader.read_exact(&mut content)?;

        if let Ok(message) = serde_json::from_slice::<LspMessage>(&content) {
            let responses = server.handle_message(message);

            for response in responses {
                let json = serde_json::to_string(&response)?;
                let header = format!("Content-Length: {}\r\n\r\n", json.len());
                write!(stdout, "{}{}", header, json)?;
                stdout.flush()?;
            }
        } else {
            // Send parse error response for malformed JSON
            let error_response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32700,
                    "message": "Parse error: invalid JSON"
                }
            });
            let json = serde_json::to_string(&error_response)?;
            let header = format!("Content-Length: {}\r\n\r\n", json.len());
            write!(stdout, "{}{}", header, json)?;
            stdout.flush()?;
        }

        if server.shutdown {
            break;
        }
    }

    Ok(())
}
