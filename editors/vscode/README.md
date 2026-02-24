# Truss for VS Code

Real-time GitHub Actions workflow validation, right in your editor.

## What You Get

- Diagnostics as you type in `.github/workflows/*.yml` files
- 41 semantic validation rules -- not just syntax checking, but actual workflow logic
- Sub-millisecond validation thanks to incremental parsing
- Error, warning, and info severity levels

## Prerequisites

You'll need the `truss-lsp` binary on your PATH.

### Building from source

```bash
git clone https://github.com/JuanMarchetto/truss.git
cd truss
cargo build --release -p truss-lsp

# Put it somewhere on your PATH
cp target/release/truss-lsp ~/.local/bin/
```

## Installation

### From a local VSIX build

```bash
cd editors/vscode
npm install
npm run compile
npx vsce package
code --install-extension truss-validator-0.1.0.vsix
```

### For development

```bash
cd editors/vscode
npm install
npm run compile
```

Then hit F5 in VS Code to launch a development Extension Host window.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `truss.lspPath` | `truss-lsp` | Path to the `truss-lsp` binary |
| `truss.enable` | `true` | Enable or disable the language server |

## How It Works

The extension starts `truss-lsp` as a child process and talks to it over stdio using the Language Server Protocol. When you open or edit a workflow file, the LSP server validates it and sends back diagnostics that show up inline in your editor.

Internally, the server uses incremental parsing so it stays fast -- even complex workflows validate in around 11ms.
