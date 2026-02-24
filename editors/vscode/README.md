# Truss - GitHub Actions Validator (VS Code Extension)

Real-time GitHub Actions workflow validation powered by the Truss LSP server.

## Features

- Real-time diagnostics as you type in `.github/workflows/*.yml` files
- 39 semantic validation rules (not just syntax checking)
- Sub-millisecond validation via incremental parsing
- Error, warning, and info severity levels

## Prerequisites

You need the `truss-lsp` binary installed and available on your PATH.

### Build from source

```bash
git clone https://github.com/JuanMarchetto/truss.git
cd truss
cargo build --release -p truss-lsp

# Add to PATH (or copy to a directory on your PATH)
cp target/release/truss-lsp ~/.local/bin/
```

## Installation

### From VSIX (local build)

```bash
cd editors/vscode
npm install
npm run compile
npx vsce package
code --install-extension truss-validator-0.1.0.vsix
```

### From source (development)

```bash
cd editors/vscode
npm install
npm run compile
```

Then press F5 in VS Code to launch a development Extension Host window.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `truss.lspPath` | `truss-lsp` | Path to the `truss-lsp` binary |
| `truss.enable` | `true` | Enable or disable the language server |

## How It Works

The extension launches the `truss-lsp` binary as a child process using the Language Server Protocol over stdio. When you open or edit a `.github/workflows/*.yml` file, the LSP server validates the workflow and returns diagnostics that appear inline in your editor.

The LSP server uses incremental parsing internally for performance, validating complex workflows in ~11ms.
