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

// ---------------------------------------------------------------------------
// C stdlib shims for tree-sitter when targeting wasm32-unknown-unknown.
//
// tree-sitter compiles C code that references malloc/free/fprintf/etc.
// On native targets the system libc provides these; on WASM there is no
// libc, so the symbols end up as unresolved "env" imports that browsers
// cannot satisfy.
//
// By providing #[no_mangle] implementations here the linker resolves them
// at build time, and the generated JS no longer contains
// `import * from "env"`.
// ---------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wasm_compat {
    use core::ptr;
    use std::alloc::{alloc, alloc_zeroed, dealloc, realloc as std_realloc, Layout};

    // We prefix each allocation with an 8-byte header that stores the
    // requested size so that free() and realloc() can reconstruct the
    // Layout.
    const HEADER: usize = 8;
    const ALIGN: usize = 8;

    #[no_mangle]
    pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
        if size == 0 {
            return ptr::null_mut();
        }
        let total = size + HEADER;
        let layout = Layout::from_size_align_unchecked(total, ALIGN);
        let base = alloc(layout);
        if base.is_null() {
            return base;
        }
        *(base as *mut usize) = size;
        base.add(HEADER)
    }

    #[no_mangle]
    pub unsafe extern "C" fn calloc(count: usize, size: usize) -> *mut u8 {
        let total = count.saturating_mul(size);
        if total == 0 {
            return ptr::null_mut();
        }
        let alloc_total = total + HEADER;
        let layout = Layout::from_size_align_unchecked(alloc_total, ALIGN);
        let base = alloc_zeroed(layout);
        if base.is_null() {
            return base;
        }
        *(base as *mut usize) = total;
        base.add(HEADER)
    }

    #[no_mangle]
    pub unsafe extern "C" fn realloc(p: *mut u8, new_size: usize) -> *mut u8 {
        if p.is_null() {
            return malloc(new_size);
        }
        if new_size == 0 {
            free(p);
            return ptr::null_mut();
        }
        let base = p.sub(HEADER);
        let old_size = *(base as *const usize) + HEADER;
        let new_total = new_size + HEADER;
        let layout = Layout::from_size_align_unchecked(old_size, ALIGN);
        let new_base = std_realloc(base, layout, new_total);
        if new_base.is_null() {
            return new_base;
        }
        *(new_base as *mut usize) = new_size;
        new_base.add(HEADER)
    }

    #[no_mangle]
    pub unsafe extern "C" fn free(p: *mut u8) {
        if p.is_null() {
            return;
        }
        let base = p.sub(HEADER);
        let size = *(base as *const usize) + HEADER;
        let layout = Layout::from_size_align_unchecked(size, ALIGN);
        dealloc(base, layout);
    }

    // I/O stubs — tree-sitter declares these but never calls them in the
    // validation code path.  They must exist so the linker doesn't leave
    // unresolved "env" imports.

    #[no_mangle]
    pub extern "C" fn abort() {
        core::arch::wasm32::unreachable()
    }

    #[no_mangle]
    pub extern "C" fn fprintf(_stream: i32, _fmt: i32, _arg: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn snprintf(_buf: i32, _size: i32, _fmt: i32, _arg: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn vsnprintf(_buf: i32, _size: i32, _fmt: i32, _ap: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn fclose(_stream: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn clock() -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn fwrite(_ptr: i32, _size: i32, _nmemb: i32, _stream: i32) -> i32 {
        0
    }

    #[no_mangle]
    pub extern "C" fn fputc(_c: i32, _stream: i32) -> i32 {
        0
    }
}

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
