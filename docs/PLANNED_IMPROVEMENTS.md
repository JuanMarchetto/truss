# Planned Improvements

This list came out of a full code review and gap analysis we did in February 2026, comparing truss against the GitHub Actions docs and actionlint. The good news: all the critical and high-priority bugs have been fixed, and we added new rules for security and deprecated commands. What's left is mostly architectural cleanup, quality-of-life stuff, and hardening work that will make the codebase easier to maintain going forward.

---

## Recently Addressed (This PR)

These gaps came up during research into the GitHub Actions docs, actionlint, and community best practices. All of them have been addressed:

- **Script injection detection** -- new `script_injection` rule warns when untrusted inputs (`github.event.pull_request.title`, `.body`, `.head.ref`, `github.head_ref`, etc.) flow directly into `run:` blocks via `${{ }}` expressions
- **Deprecated workflow command detection** -- new `deprecated_commands` rule flags `::set-output`, `::save-state`, `::set-env`, `::add-path` usage in run scripts
- **Step `uses`/`run` mutual exclusion** -- the `step` rule now catches when a step has both `uses` and `run` (previously it only checked that at least one was present)
- **Filter conflict detection** -- the `event_payload` rule now detects `branches` + `branches-ignore`, `paths` + `paths-ignore`, `tags` + `tags-ignore` on the same event
- **GITHUB_ env var prefix** -- the `step_env` rule warns when user-defined env vars use the reserved `GITHUB_` prefix
- **Enhanced cron validation** -- cron field ranges are now properly validated (minute 0-59, hour 0-23, day 1-31, month 1-12, weekday 0-6), not just the field count
- **Complete PR activity types** -- added the missing valid `pull_request` activity types: `edited`, `ready_for_review`, `converted_to_draft`, `auto_merge_enabled`, `auto_merge_disabled`, `enqueued`, `dequeued`, `milestoned`, `demilestoned`, `locked`, `unlocked`

---

## High Impact

### 1. Add `rule_id` field to `Diagnostic`

Every diagnostic should carry the name of the rule that produced it (e.g., `"action_reference"`, `"concurrency"`). This unlocks a bunch of things at once:
- Test assertions that filter by rule ID instead of relying on fragile string matching
- `--ignore-rule` / `--only-rule` CLI filtering for users
- Per-rule severity overrides in configuration

**Files:** `crates/truss-core/lib.rs` (`Diagnostic` struct), `crates/truss-core/validation/mod.rs` (rule execution), all 41 rule files (return rule name).

### 2. Extract shared AST traversal utilities

Somewhere around 60-70% of each rule file is boilerplate for walking the YAML AST to find jobs, steps, and key-value pairs. If we pull that into shared visitors, most rules shrink from around 150 lines down to about 30 lines of actual validation logic:
- `visit_jobs(tree, source, callback)` -- iterate over all job definitions
- `visit_steps(tree, source, callback)` -- iterate over all steps across all jobs
- `unwrap_sequence_item(node)` -- get content from a `block_sequence_item`, skipping comments

**Files:** `crates/truss-core/validation/utils.rs`, all rule files.

### 3. Create shared test helpers

The same 8-line diagnostic filtering pattern shows up over 300 times across 44 test files. That's a lot of copy-paste. Worth extracting into helpers:
- `filter_diagnostics(result, rule_id)` -- filter by rule (depends on #1 being done first)
- `filter_diagnostics_by_message(result, keyword)` -- what we're doing now, just pulled out
- `assert_no_errors(result)` / `assert_has_error(result, keyword)`

**Files:** new `crates/truss-core/tests/helpers.rs` or `test_utils` module.

### 4. Upgrade tree-sitter to 0.24.x

We're currently on 0.21. The 0.24 release brings:
- Bug fixes and performance improvements
- Potentially safer initialization APIs that could let us drop the `unsafe` block in `parser.rs`
- Better error recovery

**Files:** `Cargo.toml` (workspace dependency), `crates/truss-core/parser.rs` (may need API changes).

---

## Medium Impact

### 5. Add `#[non_exhaustive]` to public enums

`Severity` and any future public enums should be `#[non_exhaustive]` so that adding variants down the road doesn't break downstream crates.

**Files:** `crates/truss-core/lib.rs`.

### 6. Add incremental parsing tests

The LSP's main code path (`analyze_incremental` / `analyze_incremental_with_tree`) has zero test coverage right now. Should add tests for:
- Basic incremental re-parse after an edit
- Incremental parse producing the same results as a full parse
- `analyze_with_tree` returning `None` tree on parse failure
- `analyze_incremental_with_tree` with a `None` old tree

**Files:** `crates/truss-core/tests/` (new test file).

### 7. Add `validation/utils.rs` unit tests

The core helpers (`unwrap_node`, `find_value_for_key`, `get_pair_value`, `node_text`, `find_expressions`, `is_valid_expression_syntax`) don't have any direct unit tests. They're only tested indirectly through the rule tests, which isn't great if something subtle breaks.

**Files:** `crates/truss-core/validation/utils.rs` (add `#[cfg(test)]` module).

### 8. Improve benchmarks

The current benchmark setup has a few issues:
- `TrussEngine::new()` is inside `b.iter()`, so it's measuring engine creation + parsing + validation instead of just the analysis step
- No `criterion::black_box()` usage
- No incremental parsing benchmark (this matters a lot for LSP performance)
- No `Throughput::Bytes` metrics
- No benchmark groups for regression detection

**Files:** `crates/truss-core/benches/parse.rs`.

### 9. CLI: Reuse `TrussEngine` across files in the parallel path

Right now `TrussEngine::new()` gets constructed per-file in the rayon parallel processing path. Since the parser maintains state for incremental parsing, spinning up a fresh one each time is wasteful. A thread-local or shared engine would help here.

**Files:** `crates/truss-cli/src/main.rs`.

---

## Future Validation Rules

These validation gaps turned up during research but are either lower priority or more involved to implement.

### 10. Context availability checking

GitHub Actions restricts which expression contexts are available depending on where you are in the workflow. For instance, the `env` context isn't available in job-level `if:` conditions, and the `jobs` context is only available in reusable workflow outputs. Getting this right means mapping each workflow location to its allowed contexts.

### 11. Event activity type validation for all events

Right now only `pull_request` and `issues` have `types:` validation. There's a long tail of events that should also get this treatment: `pull_request_review`, `pull_request_review_comment`, `release`, `check_run`, `check_suite`, `discussion`, `discussion_comment`, `label`, `milestone`, `project`, `project_card`, `project_column`, `registry_package`, `watch`, `workflow_run`, `branch_protection_rule`, `merge_group`.

### 12. YAML anchor/alias validation

Tree-sitter-yaml may not fully resolve YAML anchors (`&anchor`) and aliases (`*anchor`). It would be useful to validate that aliases reference defined anchors and catch circular references.

### 13. Reusable workflow constraints

A few things to watch for:
- Reusable workflows can't call other reusable workflows (max depth = 1 currently, though this has shifted over time)
- `workflow_call` can't be combined with other events
- Reusable workflow inputs have type constraints (`boolean`, `number`, `string`, `choice`, `environment`)

### 14. Matrix strategy constraints

- Maximum 256 jobs per matrix expansion
- `include`/`exclude` interaction validation
- Type consistency within matrix dimensions

### 15. Action input validation

When someone uses `with:` on a `uses:` step, it'd be nice to validate that the input names actually match what the action expects. This is an advanced feature though -- it requires fetching the action's `action.yml`.

### 16. SHA pinning recommendations

Warn when third-party actions use tag references (`@v3`) instead of SHA pinning (`@abc123...`). This is a security best practice that a lot of teams care about.

---

## Low Impact / Nice-to-Have

### 17. CLI error reporting improvements

A few rough edges in the CLI:
- I/O errors silently disappear from `--json` output
- Invalid file diagnostics print without a filename in multi-file mode
- `glob::glob` errors get swallowed via `.flatten()`
- Nothing stops you from passing `-` (stdin) multiple times

**Files:** `crates/truss-cli/src/main.rs`.

### 18. VS Code extension: runtime config detection

If you change configuration (e.g., toggle `truss.enable` or update `truss.lspPath`), nothing happens until you reload the extension. Adding a `workspace.onDidChangeConfiguration` listener would fix that.

**Files:** `editors/vscode/src/extension.ts`.

### 19. Consolidate build system

There's both a `justfile` and a `makefile` with overlapping targets. The justfile is more complete but is missing `lint`/`fmt`/`ci` targets. The makefile's `ci` target doesn't include clippy or fmt. Should pick one and make it the single source of truth.

**Files:** `justfile`, `makefile`.

### 20. Add stress tests

There are no tests for edge cases like:
- Large files (10k+ lines)
- Deeply nested YAML
- Binary/non-UTF-8 content
- Unicode in keys and values
- Empty or malformed YAML

**Files:** `crates/truss-core/tests/` (new test file).

### 21. CI: Add `cargo doc` build check and MSRV verification

Two useful additions to CI:
- `cargo doc --workspace --no-deps` to catch broken doc links
- An MSRV verification job that tests with the declared `rust-version`

**Files:** `.github/workflows/ci.yml`.

### 22. Harden `.gitignore`

Add patterns for extra safety: `*.key`, `*.pem`, `*.p12`, `*.pfx`, `.env.*`.

**Files:** `.gitignore`.
