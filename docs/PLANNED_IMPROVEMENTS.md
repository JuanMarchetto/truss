# Planned Improvements

Items extracted from the full code review (February 2026) and comprehensive gap analysis against GitHub Actions documentation and actionlint. All critical and high-priority bugs have been fixed. New validation rules for security and deprecated commands were added. What remains are architectural improvements, quality-of-life enhancements, and hardening work.

---

## Recently Addressed (This PR)

The following gaps were identified through comprehensive research of GitHub Actions documentation, actionlint, and community best practices, and have been addressed:

- **Script injection detection** — new `script_injection` rule warns when untrusted inputs (`github.event.pull_request.title`, `.body`, `.head.ref`, `github.head_ref`, etc.) are used directly in `run:` blocks via `${{ }}` expressions
- **Deprecated workflow command detection** — new `deprecated_commands` rule warns about `::set-output`, `::save-state`, `::set-env`, `::add-path` usage in run scripts
- **Step `uses`/`run` mutual exclusion** — enhanced `step` rule now detects when a step has both `uses` and `run` (was only checking for "at least one")
- **Filter conflict detection** — enhanced `event_payload` rule detects `branches` + `branches-ignore`, `paths` + `paths-ignore`, `tags` + `tags-ignore` on the same event
- **GITHUB_ env var prefix** — enhanced `step_env` rule warns when user-defined env vars use the reserved `GITHUB_` prefix
- **Enhanced cron validation** — cron field ranges are now validated (minute 0-59, hour 0-23, day 1-31, month 1-12, weekday 0-6), not just field count
- **Complete PR activity types** — added missing valid `pull_request` activity types: `edited`, `ready_for_review`, `converted_to_draft`, `auto_merge_enabled`, `auto_merge_disabled`, `enqueued`, `dequeued`, `milestoned`, `demilestoned`, `locked`, `unlocked`

---

## High Impact

### 1. Add `rule_id` field to `Diagnostic`

Each diagnostic should carry the name of the rule that produced it (e.g., `"action_reference"`, `"concurrency"`). This enables:
- Precise test assertions (filter by rule ID instead of fragile string matching)
- User-facing `--ignore-rule` / `--only-rule` CLI filtering
- Per-rule severity overrides in configuration

**Files:** `crates/truss-core/lib.rs` (`Diagnostic` struct), `crates/truss-core/validation/mod.rs` (rule execution), all 41 rule files (return rule name).

### 2. Extract shared AST traversal utilities

~60-70% of each rule file is boilerplate for walking the YAML AST to find jobs, steps, and key-value pairs. Extract shared visitors:
- `visit_jobs(tree, source, callback)` — iterate over all job definitions
- `visit_steps(tree, source, callback)` — iterate over all steps across all jobs
- `unwrap_sequence_item(node)` — get content from a `block_sequence_item`, skipping comments

This would reduce most rules from ~150 lines to ~30 lines of pure validation logic.

**Files:** `crates/truss-core/validation/utils.rs`, all rule files.

### 3. Create shared test helpers

The same 8-line diagnostic filtering pattern is repeated ~300+ times across 44 test files. Extract:
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

## Future Validation Rules

The following validation gaps were identified through research but are lower priority or more complex to implement:

### 10. Context availability checking

GitHub Actions restricts which expression contexts are available at different workflow locations. For example, `env` context is not available in job-level `if:` conditions, and `jobs` context is only available in reusable workflow outputs. Implementing this requires mapping each workflow location to its allowed contexts.

### 11. Event activity type validation for all events

Currently only `pull_request` and `issues` have `types:` validation. Should extend to: `pull_request_review`, `pull_request_review_comment`, `release`, `check_run`, `check_suite`, `discussion`, `discussion_comment`, `label`, `milestone`, `project`, `project_card`, `project_column`, `registry_package`, `watch`, `workflow_run`, `branch_protection_rule`, `merge_group`.

### 12. YAML anchor/alias validation

Tree-sitter-yaml may not fully resolve YAML anchors (`&anchor`) and aliases (`*anchor`). Validate that aliases reference defined anchors and detect circular references.

### 13. Reusable workflow constraints

- Reusable workflows cannot call other reusable workflows (max depth = 1 currently, was 2, now 4 in some plans)
- `workflow_call` event cannot be combined with other events
- Reusable workflow inputs have type constraints (`boolean`, `number`, `string`, `choice`, `environment`)

### 14. Matrix strategy constraints

- Maximum 256 jobs per matrix expansion
- `include`/`exclude` interaction validation
- Type consistency within matrix dimensions

### 15. Action input validation

When using `with:` on a `uses:` step, validate that the input names match the action's expected inputs (requires fetching action.yml — advanced feature).

### 16. SHA pinning recommendations

Warn when third-party actions use tag references (`@v3`) instead of SHA pinning (`@abc123...`) for security best practices.

---

## Low Impact / Nice-to-Have

### 17. CLI error reporting improvements

- I/O errors are silently excluded from `--json` output
- Invalid file diagnostics are printed without filename in multi-file mode
- `glob::glob` errors are silently swallowed via `.flatten()`
- No guard against multiple `-` (stdin) arguments

**Files:** `crates/truss-cli/src/main.rs`.

### 18. VS Code extension: runtime config detection

Configuration changes (e.g., toggling `truss.enable` or changing `truss.lspPath`) are not detected at runtime — requires extension reload. Add a `workspace.onDidChangeConfiguration` listener.

**Files:** `editors/vscode/src/extension.ts`.

### 19. Consolidate build system

Both a `justfile` and `makefile` exist with overlapping targets. The justfile is more comprehensive but lacks `lint`/`fmt`/`ci` targets. The makefile's `ci` target doesn't include clippy or fmt. Pick one and make it authoritative.

**Files:** `justfile`, `makefile`.

### 20. Add stress tests

No tests exist for edge cases:
- Large files (10k+ lines)
- Deeply nested YAML
- Binary/non-UTF-8 content
- Unicode in keys and values
- Empty or malformed YAML

**Files:** `crates/truss-core/tests/` (new test file).

### 21. CI: Add `cargo doc` build check and MSRV verification

- Add `cargo doc --workspace --no-deps` to CI to catch broken doc links
- Add an MSRV verification job that tests with the declared `rust-version`

**Files:** `.github/workflows/ci.yml`.

### 22. Harden `.gitignore`

Add patterns for extra safety: `*.key`, `*.pem`, `*.p12`, `*.pfx`, `.env.*`.

**Files:** `.gitignore`.
