# Truss Playground

Browser-based GitHub Actions workflow validator powered by WebAssembly.

## Building

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):

   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Build the WASM package:

   ```bash
   wasm-pack build crates/truss-wasm --target web --out-dir ../../playground/pkg
   ```

3. Serve the playground:

   ```bash
   cd playground
   python3 -m http.server 8080
   # or: npx serve .
   ```

4. Open http://localhost:8080

## How It Works

The playground loads `truss-wasm` as a WebAssembly module and provides a
CodeMirror YAML editor with live validation. As you type, Truss validates
the workflow in real-time directly in the browser -- no server required.

All 41 validation rules run client-side with the same engine used by the CLI.
