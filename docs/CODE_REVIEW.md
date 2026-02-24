# Truss Code Review Report

**Date:** February 2026
**Scope:** Full codebase review — core engine, validation rules, CLI, LSP, VS Code extension, tests, benchmarks, CI/CD, and project configuration.

---

## Executive Summary

Truss is a well-architected Rust project with a clean separation between core logic and adapter layers (CLI, LSP, WASM). The validation engine design — a trait-based rule system with parallel execution via rayon — is sound and scalable. The test suite is wide (294 tests across 42 files), benchmark infrastructure exists from day one, and the project structure is production-grade.

However, the implementation quality is uneven. The **LSP server has critical protocol bugs** that make it non-functional. The **validation rules contain massive code duplication** (~60-70% boilerplate per file) and several fake/stub validation functions. The **test suite is wide but shallow**, relying on fragile string-matching assertions with no shared test utilities. Dependencies are outdated (tree-sitter 0.21 vs current 0.24), and CI lacks security auditing and cross-platform coverage.

**An experienced tech lead would say:** *"The architecture is right, the scope is ambitious, and the bones are solid. But this reads like AI-generated code that was broadly correct but never had a human review pass. The LSP is broken, there's a lot of copy-paste, and the tests check vibes rather than contracts. Before going public, fix the LSP, deduplicate the rule traversal code, and add a rule-ID field to diagnostics so tests can be precise."*

### Overall Ratings

| Area | Rating | Notes |
|------|--------|-------|
| Architecture | 4.5/5 | Clean separation, sound design principles |
| Core Engine (lib.rs, parser.rs) | 3.5/5 | Works, but has duplication and unsafe code concerns |
| Validation Rules | 3/5 | Wide coverage, massive boilerplate, some fake validators |
| CLI | 4/5 | Well-implemented, minor gaps in error reporting |
| LSP Server | 2/5 | **Critically broken** — two protocol bugs prevent it from working |
| VS Code Extension | 4.5/5 | Clean, minimal, correct |
| Test Suite | 3/5 | Wide coverage, fragile assertions, no test helpers |
| Benchmarks | 3.5/5 | Good framework, engine-in-loop, no incremental benchmark |
| CI/CD | 3/5 | Basic coverage, missing security audit and cross-platform |
| Project Config | 3.5/5 | Mostly clean, outdated deps, no MSRV |

---

## Critical Issues (Must Fix)

### 1. LSP Server: `serde(untagged)` Misroutes All Notifications

**File:** `crates/truss-lsp/lib.rs:14-20`

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum LspMessage {
    Request(LspRequest),
    Notification(LspNotification),
}
```

Because `LspRequest` has `id: Option<Value>` and `LspNotification` has no `id` field, serde's untagged deserialization tries `Request` first and always succeeds (with `id: None`). Every `didOpen`, `didChange`, `didClose`, and `initialized` notification is misclassified as a request and receives a "Method not found" error response.

**Impact:** The LSP server cannot process any document changes. No diagnostics are ever published. The VS Code extension connects but does nothing.

**Fix:** Replace `serde(untagged)` with manual JSON-RPC message discrimination:
```rust
fn parse_message(value: &Value) -> LspMessage {
    if value.get("id").is_some() && value.get("method").is_some() {
        // Request
    } else if value.get("method").is_some() {
        // Notification
    } else {
        // Response
    }
}
```

### 2. LSP Server: Missing `camelCase` Rename on Parameter Structs

**File:** `crates/truss-lsp/lib.rs:367-406`

All LSP parameter structs use Rust `snake_case` field names (`text_document`, `language_id`, `content_changes`) but LSP JSON sends `camelCase` (`textDocument`, `languageId`, `contentChanges`). No struct has `#[serde(rename_all = "camelCase")]`.

**Impact:** Even if issue #1 is fixed, `serde_json::from_value` silently fails to deserialize every notification parameter. The `if let Ok(...)` guards swallow the errors.

**Fix:** Add `#[serde(rename_all = "camelCase")]` to all parameter structs.

### 3. LSP Server: Infinite Busy-Loop on Client Disconnect

**File:** `crates/truss-lsp/lib.rs:419-437`

When the client disconnects, `read_line` returns `Ok(0)`. This breaks out of the header loop, `content_length` remains `None`, `continue` restarts the outer loop, and `read_line` returns `Ok(0)` again — forever. The server spins at 100% CPU.

**Fix:** Check for EOF: `if reader.read_line(&mut line)? == 0 { return Ok(()); }`

### 4. Massive Code Duplication: `is_valid_expression_syntax`

**Files:**
- `validation/rules/expression.rs:16-97`
- `validation/rules/step_if_expression.rs:170-283`
- `validation/rules/job_if_expression.rs:244-357`

The ~80-line function is copy-pasted verbatim across three files. Additionally, 5 helper functions (`is_valid_github_context`, `is_valid_matrix_context`, `is_valid_secrets_context`, `is_potentially_always_true`, `is_potentially_always_false`) are duplicated between `step_if_expression.rs` and `job_if_expression.rs`.

**Fix:** Move all shared functions to `validation/utils.rs`.

### 5. Fake Validation Functions

**Files:** `validation/rules/step_if_expression.rs:286-301`, `validation/rules/job_if_expression.rs:360-375`

```rust
fn is_valid_github_context(expr: &str) -> bool {
    !expr.contains("github.nonexistent") && !expr.contains("github.invalid")
}
```

These functions only catch the literal strings "nonexistent" and "invalid" — they will never trigger on real user workflows. They give the appearance of validation without providing any. A user writing `${{ github.typo_event_name }}` passes undetected.

**Fix:** Either implement real context validation (list known `github.*` fields) or remove these stubs and their associated warnings.

---

## High-Priority Issues

### 6. Overlapping Rules: StepValidationRule vs ActionReferenceRule

`step.rs:132-171` performs action reference validation (checking for missing `@ref`, `invalid/` prefix) while `ActionReferenceRule` does the same thing more thoroughly. The same `uses:` reference can generate both an `Error` from `ActionReferenceRule` and a `Warning` from `StepValidationRule`. The `starts_with("invalid/")` check in `step.rs:155` is a test artifact that should not be in production code.

### 7. Duplicate Event Lists in workflow_trigger.rs

**File:** `validation/rules/workflow_trigger.rs`

Two separate `valid_events` arrays exist:
- Lines 88-126: Comprehensive list (~30 events) but contains duplicates (`pull_request_target`, `workflow_call`, `workflow_dispatch` each appear twice)
- Lines 157-165: Drastically incomplete list (~7 events) used for scalar `on:` values

A workflow with `on: fork` as a scalar would be incorrectly flagged as invalid.

### 8. Potential Panics from Byte-Level Slicing

Throughout `utils.rs` and all rules:
```rust
let text = &source[node.start_byte()..node.end_byte()];
```

If tree-sitter returns byte offsets that land mid-UTF-8 character, this panics. Defensive code should use `source.get(start..end).unwrap_or("")`.

### 9. Remaining `child(1)` Hardcoding

Despite a prior systemic child-index bug fix, bare `child(1)` for `block_sequence_item` remains in:
- `step_env.rs:71`
- `step_name.rs:71`
- `step_continue_on_error.rs:126`
- `event_payload.rs:283`

If a YAML comment appears between the `-` marker and the content, the index shifts. `step_if_expression.rs:68-82` shows the correct pattern (iterate from child(1), skip comments).

### 10. Non-Deterministic Diagnostic Messages

`step_output_reference.rs:144` and `job_needs.rs:329` iterate over `HashSet` which has non-deterministic order. The "Available outputs:" message and cycle-detection order vary across runs, breaking the engine's documented guarantee of deterministic output.

### 11. tree-sitter 0.21 Is Outdated

tree-sitter is at 0.24.x. Staying on 0.21 means missing bug fixes, performance improvements, and potentially safer initialization APIs that would eliminate the unsafe block in `parser.rs:13-19`.

### 12. No Security Audit in CI

No `cargo audit` or `cargo deny` step exists. For a tool that processes arbitrary YAML, auditing dependencies for known vulnerabilities is essential.

---

## Medium-Priority Issues

### 13. Core Engine: Four Duplicated `analyze*` Methods

**File:** `crates/truss-core/lib.rs:91-210`

The parse-error handling block (constructing a `TrussResult` with "Failed to parse YAML") is copy-pasted four times across `analyze`, `analyze_incremental`, `analyze_with_tree`, and `analyze_incremental_with_tree`. Extract a helper method.

### 14. Core Engine: Dummy Tree on Parse Failure

`analyze_with_tree` and `analyze_incremental_with_tree` return a dummy tree (from parsing `""`) when the actual parse fails. Callers receive a `(TrussResult, Tree)` where the tree has no relationship to the actual source. Consider returning `Option<Tree>`.

### 15. Core Engine: Unsafe Code in parser.rs

**File:** `crates/truss-core/parser.rs:13-19`

```rust
fn language_from_fn() -> Language {
    let lang_fn = ts_yaml::LANGUAGE.into_raw();
    unsafe {
        let raw_ptr = lang_fn();
        Language::from_raw(raw_ptr as *const _)
    }
}
```

The `as *const _` cast suppresses all type checking. The safety comment ("The unsafe code is safe in practice") does not meet Rust's `// SAFETY:` documentation standard. Upgrading tree-sitter may eliminate the need for this unsafe block entirely.

### 16. Three Independent Expression Parsers

`${{ }}` extraction is implemented three times with different semantics:
- `expression.rs`: Uses `find("${{")` and `find("}}")` — mishandles nested braces in format strings
- `secrets.rs:26-83`: Custom byte-level scanner with brace counting
- `step_output_reference.rs:414-629`: Third copy of expression scanning

Consolidate into a single shared utility.

### 17. `runs_on.rs` False Positive for Reusable Workflow Jobs

The rule checks that every job has `runs-on`, but reusable workflow call jobs (jobs with `uses:` at the job level) do not have `runs-on`. The rule does not check for `uses:` to skip validation, so all workflow_call consumer jobs are incorrectly flagged.

### 18. Permissions Rule Accepts Invalid Scope Keys

**File:** `validation/rules/permissions.rs:23-42`

`valid_scopes` includes `"write-all"`, `"read-all"`, `"none"`. These are valid as top-level values (`permissions: read-all`) but not as mapping keys (`permissions:\n  write-all: read` is nonsensical).

### 19. Concurrency Rule False Positive

**File:** `validation/rules/concurrency.rs:125-141`

The check `!group_cleaned.contains('.')` is meant to avoid flagging `github.ref`-style strings, but `"1.0"` (a number with a dot) passes while `"123"` is flagged. Should check for context prefixes instead.

### 20. Suspicious Code in step_output_reference.rs

**File:** `validation/rules/step_output_reference.rs:498-509`

A `.strip_prefix("s.")` workaround suggests an off-by-one bug in the string slicing above it. Rather than fixing the root cause, a band-aid was applied. This workaround also means any output name starting with "s." would be incorrectly parsed.

### 21. No `[workspace.dependencies]` Table

Shared dependencies (`serde`, `serde_json`, `tree-sitter`, `rayon`) are duplicated across crate Cargo.toml files. Use `[workspace.dependencies]` for consistency and atomic upgrades.

### 22. No MSRV Declared

No `rust-version` field in `[workspace.package]`, no `rust-toolchain.toml`. Contributors don't know the minimum supported Rust version, and CI doesn't verify it.

### 23. No Cross-Platform CI

All CI jobs run on `ubuntu-latest` only. tree-sitter has C FFI components that may behave differently on macOS and Windows.

---

## Test Suite Assessment

### Overall Rating: 3/5

**Strengths:**
- Excellent breadth — every rule has a dedicated test file (42 files, 294 tests)
- Real-world regression fixtures from major OSS projects (rust-lang/rust, actions/checkout, Azure SDK, TypeScript)
- Comment-handling regression tests that target known parser subtleties
- Determinism tests that run analysis twice and assert identical results
- No `#[ignore]` annotations — the full suite runs on every CI pass

**Weaknesses:**

| Issue | Severity | Details |
|-------|----------|---------|
| No test helpers | High | The same 8-line diagnostic filtering pattern is repeated ~290 times with zero shared utilities |
| Fragile string matching | High | Tests filter by `d.message.contains("keyword")` — if wording changes, tests silently pass when they shouldn't |
| No rule-ID on Diagnostic | High | Tests cannot distinguish which rule produced a diagnostic |
| No incremental parsing tests | High | The LSP's primary code path (`analyze_incremental`) is completely untested |
| Stale TDD comments | Medium | Multiple test files say "Rule not yet implemented" for rules that are implemented |
| Overly permissive assertions | Medium | Some tests use disjunctive assertions that pass for the wrong reasons |
| No utils.rs unit tests | Medium | Core helpers (`unwrap_node`, `find_value_for_key`, `node_text`) have zero tests |
| No stress tests | Low | No tests for large files, deeply nested YAML, binary content, or Unicode |

### Test Coverage Gaps

1. Incremental parsing — zero tests
2. `analyze_with_tree` / `analyze_incremental_with_tree` — zero tests
3. Parser error recovery — 1 test total
4. `validation/utils.rs` — zero unit tests
5. Custom rules via `add_rule()` — untested
6. Performance regression assertions — none
7. Unicode in keys/values — untested

---

## Benchmark Assessment

### Overall Rating: 3.5/5

**File:** `crates/truss-core/benches/parse.rs`

**Strengths:**
- Uses Criterion (industry standard)
- 4 complexity levels with real-world fixtures
- Compile-time fixture loading via `include_str!`

**Issues:**
- Engine instantiation is inside `b.iter()` — measures creation + parsing + validation, not just analysis
- No `criterion::black_box()` usage
- No incremental parsing benchmark (critical for LSP use case)
- No throughput metrics (`Throughput::Bytes`)
- No benchmark groups for regression detection

---

## CLI Assessment

### Overall Rating: 4/5

**File:** `crates/truss-cli/src/main.rs` (~382 lines)

**Strengths:**
- Distinct exit codes (0/1/2/3) following Unix conventions
- Idiomatic error type with `Display`, `Error`, `From<io::Error>`, and `exit_code()`
- Good clap derive usage
- Parallel file processing with rayon
- Clean severity filtering

**Issues:**
- `TrussEngine::new()` constructed per-file in the rayon parallel path (wasteful)
- I/O errors silently excluded from `--json` output
- Invalid file diagnostics printed without filename in multi-file mode
- `glob::glob` errors silently swallowed via `.flatten()`
- No guard against multiple `-` (stdin) arguments

---

## VS Code Extension Assessment

### Overall Rating: 4.5/5

**Files:** `editors/vscode/src/extension.ts`, `editors/vscode/package.json`

**Strengths:**
- Clean activation/deactivation lifecycle
- Configurable enable/disable and LSP path
- Correct stdio transport
- Proper document selector for workflow files

**Issues:**
- Configuration changes at runtime not detected (no listener)
- Module-level `client` variable could leak if `activate` is called twice

---

## Project Configuration Assessment

### CI Pipeline

**Strengths:** Uses modern actions (`checkout@v4`, `dtolnay/rust-toolchain@stable`, `rust-cache@v2`), clippy with `-D warnings`, format checking.

**Missing:**
- `cargo audit` / `cargo deny` (security)
- MSRV verification
- Cross-platform matrix (macOS, Windows)
- Job dependency graph (`test` should depend on `fmt` + `check`)
- `cargo doc` build check
- CI concurrency control

### Build System

Both a `justfile` and `makefile` exist with overlapping targets. The justfile is more comprehensive but lacks `lint`/`fmt`/`ci` targets. The makefile's `ci` target doesn't include clippy or fmt. Pick one and make it authoritative.

### .gitignore

Generally comprehensive. Missing patterns for extra safety: `*.key`, `*.pem`, `*.p12`, `*.pfx`, `.env.*`.

---

## Architectural Observations

### What an Experienced Tech Lead Would Note

**Positive impressions:**
1. The "Core First" architecture is genuinely well-executed — `truss-core` has zero knowledge of LSP or editors
2. The `ValidationRule` trait design is clean and extensible
3. Parallel rule execution with deterministic sorting is the right approach
4. Having benchmarks from day one, with real-world fixtures, shows engineering maturity
5. The 39-rule count for a v0.1.0 is ambitious and demonstrates thorough domain knowledge

**Concerns:**
1. **The LSP is the product's delivery mechanism, and it's broken.** Two critical protocol bugs mean no user has ever seen this tool produce diagnostics in an editor. This suggests the LSP was written but never end-to-end tested.
2. **The validation rules read like generated code.** ~60-70% of each rule file is boilerplate traversal code. A visitor pattern or shared traversal utility would reduce each rule to just its validation logic and make the codebase dramatically more maintainable.
3. **The test assertions are structural liabilities.** String-matching against diagnostic messages without rule IDs means tests are tightly coupled to prose and cannot precisely verify which rule fired. This will cause pain as the codebase grows.
4. **Three independent expression parsers** is a red flag for maintenance. When a bug is found in expression parsing, it must be fixed in three places.
5. **The "fake" context validation functions** (`is_valid_github_context` et al.) are the kind of thing that erodes trust in a tool. If a validator claims to check something but doesn't actually check it, users learn to ignore its output.

### Recommended Priority Actions

1. **Fix the LSP** — the three critical bugs (untagged enum, camelCase, EOF loop)
2. **Add `rule_id` field to `Diagnostic`** — enables precise test assertions and user-facing filtering
3. **Extract shared AST traversal utilities** — `visit_jobs()`, `visit_steps()`, `unwrap_sequence_item()`
4. **Consolidate expression parsing** into a single utility
5. **Remove or implement the fake context validators**
6. **Add `#[non_exhaustive]` to public enums** (`Severity`, `ParseError`)
7. **Upgrade tree-sitter** to 0.24.x
8. **Add `cargo audit` to CI**
9. **Add incremental parsing tests and benchmarks**
10. **Create shared test helpers** to replace the 290 copies of diagnostic filtering

---

## File-Level Summary

| File | Lines | Rating | Critical Issues |
|------|-------|--------|-----------------|
| `truss-core/lib.rs` | ~320 | 3.5/5 | 4x duplicated error handling, dummy tree return |
| `truss-core/parser.rs` | ~84 | 3/5 | Unsafe FFI concerns, impoverished error type |
| `truss-core/validation/mod.rs` | ~70 | 4/5 | Clean framework design |
| `truss-core/validation/utils.rs` | ~180 | 3.5/5 | UTF-8 panic risk, no unit tests |
| `truss-core/validation/rules/*` (41 files) | ~6000 | 3/5 | Massive duplication, fake validators, child(1) bugs |
| `truss-cli/src/main.rs` | ~382 | 4/5 | Engine-per-file in parallel path |
| `truss-lsp/lib.rs` | ~475 | 2/5 | **3 critical protocol bugs** |
| `editors/vscode/src/extension.ts` | ~57 | 4.5/5 | Minor runtime config issue |
| `truss-core/tests/*` (42 files) | ~5000 | 3/5 | Wide but shallow, fragile assertions |
| `truss-core/benches/parse.rs` | ~30 | 3.5/5 | Engine-in-loop, no incremental bench |
| `.github/workflows/ci.yml` | ~50 | 3/5 | No audit, no MSRV, no cross-platform |
