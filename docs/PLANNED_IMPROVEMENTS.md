# Planned Improvements

Items extracted from the full code review (February 2026). All critical and high-priority bugs have been fixed. What remains are architectural improvements, quality-of-life enhancements, and hardening work.

---

## High Impact

### 1. Add `rule_id` field to `Diagnostic`

Each diagnostic should carry the name of the rule that produced it (e.g., `"action_reference"`, `"concurrency"`). This enables:
- Precise test assertions (filter by rule ID instead of fragile string matching)
- User-facing `--ignore-rule` / `--only-rule` CLI filtering
- Per-rule severity overrides in configuration

**Files:** `crates/truss-core/lib.rs` (`Diagnostic` struct), `crates/truss-core/validation/mod.rs` (rule execution), all 39 rule files (return rule name).

### 2. Extract shared AST traversal utilities

~60-70% of each rule file is boilerplate for walking the YAML AST to find jobs, steps, and key-value pairs. Extract shared visitors:
- `visit_jobs(tree, source, callback)` — iterate over all job definitions
- `visit_steps(tree, source, callback)` — iterate over all steps across all jobs
- `unwrap_sequence_item(node)` — get content from a `block_sequence_item`, skipping comments

This would reduce most rules from ~150 lines to ~30 lines of pure validation logic.

**Files:** `crates/truss-core/validation/utils.rs`, all rule files.

### 3. Create shared test helpers

The same 8-line diagnostic filtering pattern is repeated ~290 times across 42 test files. Extract:
- `filter_diagnostics(result, rule_id)` — filter by rule (requires #1)
- `filter_diagnostics_by_message(result, keyword)` — current pattern, extracted
- `assert_no_errors(result)` / `assert_has_error(result, keyword)`

**Files:** new `crates/truss-core/tests/helpers.rs` or `test_utils` module.

### 4. Upgrade tree-sitter to 0.24.x

Currently on 0.21. Upgrading to 0.24 brings:
- Bug fixes and performance improvements
- Potentially safer initialization APIs that could eliminate the `unsafe` block in `parser.rs`
- Better error recovery

**Files:** `Cargo.toml` (workspace dependency), `crates/truss-core/parser.rs` (may need API changes).

---

## Medium Impact

### 5. Add `#[non_exhaustive]` to public enums

`Severity` and any future public enums should be `#[non_exhaustive]` so adding variants doesn't break downstream crates.

**Files:** `crates/truss-core/lib.rs`.

### 6. Add incremental parsing tests

The LSP's primary code path (`analyze_incremental` / `analyze_incremental_with_tree`) has zero test coverage. Add tests for:
- Basic incremental re-parse after edit
- Incremental parse producing same results as full parse
- `analyze_with_tree` returning `None` tree on parse failure
- `analyze_incremental_with_tree` with `None` old tree

**Files:** `crates/truss-core/tests/` (new test file).

### 7. Add `validation/utils.rs` unit tests

Core helpers (`unwrap_node`, `find_value_for_key`, `get_pair_value`, `node_text`, `find_expressions`, `is_valid_expression_syntax`) have zero direct unit tests. They are only tested indirectly through the rule tests.

**Files:** `crates/truss-core/validation/utils.rs` (add `#[cfg(test)]` module).

### 8. Improve benchmarks

Current issues:
- `TrussEngine::new()` is inside `b.iter()` — measures engine creation + parsing + validation instead of just analysis
- No `criterion::black_box()` usage
- No incremental parsing benchmark (critical for LSP performance)
- No `Throughput::Bytes` metrics
- No benchmark groups for regression detection

**Files:** `crates/truss-core/benches/parse.rs`.

### 9. CLI: Reuse `TrussEngine` across files in parallel path

`TrussEngine::new()` is constructed per-file in the rayon parallel processing path. Since the parser maintains state for incremental parsing, creating one per file is wasteful. Consider using a thread-local or shared engine.

**Files:** `crates/truss-cli/src/main.rs`.

---

## Low Impact / Nice-to-Have

### 10. CLI error reporting improvements

- I/O errors are silently excluded from `--json` output
- Invalid file diagnostics are printed without filename in multi-file mode
- `glob::glob` errors are silently swallowed via `.flatten()`
- No guard against multiple `-` (stdin) arguments

**Files:** `crates/truss-cli/src/main.rs`.

### 11. VS Code extension: runtime config detection

Configuration changes (e.g., toggling `truss.enable` or changing `truss.lspPath`) are not detected at runtime — requires extension reload. Add a `workspace.onDidChangeConfiguration` listener.

**Files:** `editors/vscode/src/extension.ts`.

### 12. Consolidate build system

Both a `justfile` and `makefile` exist with overlapping targets. The justfile is more comprehensive but lacks `lint`/`fmt`/`ci` targets. The makefile's `ci` target doesn't include clippy or fmt. Pick one and make it authoritative.

**Files:** `justfile`, `makefile`.

### 13. Add stress tests

No tests exist for edge cases:
- Large files (10k+ lines)
- Deeply nested YAML
- Binary/non-UTF-8 content
- Unicode in keys and values
- Empty or malformed YAML

**Files:** `crates/truss-core/tests/` (new test file).

### 14. CI: Add `cargo doc` build check and MSRV verification

- Add `cargo doc --workspace --no-deps` to CI to catch broken doc links
- Add an MSRV verification job that tests with the declared `rust-version`

**Files:** `.github/workflows/ci.yml`.

### 15. Harden `.gitignore`

Add patterns for extra safety: `*.key`, `*.pem`, `*.p12`, `*.pfx`, `.env.*`.

**Files:** `.gitignore`.
