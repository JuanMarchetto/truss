//! Truss LSP Server
//!
//! Entry point for the Language Server Protocol server.

fn main() {
    match truss_lsp::run() {
        Ok(true) => {
            // Clean shutdown: shutdown request was received before exit
            std::process::exit(0);
        }
        Ok(false) => {
            // Unclean exit: exit without prior shutdown request
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("LSP server error: {e}");
            std::process::exit(1);
        }
    }
}
