// WASM "env" shim — provides C stdlib stubs required by tree-sitter.
//
// tree-sitter compiles C code that references these symbols.  The Rust
// allocator handles actual memory management via wasm-bindgen exports,
// so malloc/free/calloc/realloc here are never called at runtime.  The
// I/O and clock functions are likewise unreachable in the browser
// context, but must be present for WebAssembly.instantiate to succeed.

export function malloc()    { return 0; }
export function free()      {}
export function calloc()    { return 0; }
export function realloc()   { return 0; }
export function abort()     { throw new Error("abort"); }
export function fprintf()   { return 0; }
export function snprintf()  { return 0; }
export function vsnprintf() { return 0; }
export function fclose()    { return 0; }
export function clock()     { return 0; }
export function fwrite()    { return 0; }
export function fputc()     { return 0; }
