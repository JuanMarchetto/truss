//! Truss WASM
//!
//! WebAssembly bindings for Truss, enabling browser-based validation
//! of GitHub Actions workflow files.
//!
//! # Usage from JavaScript
//!
//! ```js
//! import init, { validate } from './truss_wasm.js';
//!
//! await init();
//! const result = validate(yamlSource);
//! const diagnostics = JSON.parse(result);
//! ```

use truss_core::TrussEngine;
use wasm_bindgen::prelude::*;

/// Validate a GitHub Actions workflow YAML string.
///
/// Returns a JSON string containing an array of diagnostics.
/// Each diagnostic has: `message`, `severity`, `span` (with `start` and `end`).
///
/// # Example
///
/// ```js
/// const result = validate("name: test\non: push");
/// // Returns: '{"diagnostics":[]}'
/// ```
#[wasm_bindgen]
pub fn validate(source: &str) -> String {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(source);
    serde_json::to_string(&result).unwrap_or_else(|_| r#"{"diagnostics":[]}"#.to_string())
}

/// Validate and return a pretty-printed JSON result.
///
/// Same as `validate()` but with indented JSON output,
/// useful for debugging in the browser console.
#[wasm_bindgen]
pub fn validate_pretty(source: &str) -> String {
    let mut engine = TrussEngine::new();
    let result = engine.analyze(source);
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| r#"{"diagnostics":[]}"#.to_string())
}

/// Get the version of the Truss engine.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
