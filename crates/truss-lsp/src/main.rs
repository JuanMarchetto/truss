//! Truss LSP Server
//!
//! Entry point for the Language Server Protocol server.

fn main() {
    if let Err(e) = truss_lsp::run() {
        eprintln!("LSP server error: {}", e);
        std::process::exit(1);
    }
}

