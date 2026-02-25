# Planned Improvements

This list came out of a full code review and gap analysis we did in February 2026, comparing truss against the GitHub Actions docs and actionlint. The good news: all the critical and high-priority bugs have been fixed, and we added new rules for security and deprecated commands. What's left is mostly architectural cleanup, quality-of-life stuff, and hardening work that will make the codebase easier to maintain going forward.

---

## Recently Addressed

### Performance Optimization (PRs #22, #23, #24)

Three tiers of performance optimization reduced end-to-end CLI latency by **87%** (from 6.8ms to 0.89ms for batch validation):

- **Tier 1: Engine-level** -- Cache `is_github_actions_workflow()` check once per validation (was called 39 times), skip rayon thread pool overhead for single files
- **Tier 2: Zero-copy strings & shared utilities** -- Changed `node_text()` from `String` to `&str` (zero allocation), added `get_jobs_node()` and `clean_key()` utilities (migrated 35 rules, eliminated ~257 lines), removed `format!()` allocation in expression validation hot path
- **Tier 3: Borrowed collections & byte-level comparisons** -- Converted `HashSet<String>`/`HashMap<String, Vec<String>>` to `&str` variants in job_needs cycle detection, added `contains_ignore_ascii_case()` byte-level sliding window (no String allocation), cached `find_value_for_key()` results to eliminate 12 duplicate tree traversals in event_payload, replaced 11 `.to_lowercase()` allocations with `eq_ignore_ascii_case()`

### Validation Rules & Security (PR #3)

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

## Remaining Performance Optimizations

Current state: all scenarios complete in **under 1ms** (0.85-0.89ms). The process startup floor is ~700-800µs, meaning actual validation logic takes only ~100-150µs. Diminishing returns ahead -- the gains below are measured in microseconds.

### Already Done (Tiers 1-3)
- [x] Cache `is_github_actions_workflow()` once per run (was 39 redundant calls)
- [x] Skip rayon for single-file validation
- [x] Zero-copy `node_text()` returning `&str` instead of `String`
- [x] Shared `get_jobs_node()` utility (25 rules migrated)
- [x] Shared `clean_key()` utility (52 call sites migrated)
- [x] Eliminate `format!()` in expression syntax hot path
- [x] Eliminate `Vec<String>` allocation in `is_github_actions_workflow()`
- [x] Borrowed `HashSet<&str>`/`HashMap<&str, Vec<&str>>` in job_needs
- [x] `contains_ignore_ascii_case()` byte-level comparison (no String allocation)
- [x] Cached `find_value_for_key()` results in event_payload (12 fewer tree walks)
- [x] `eq_ignore_ascii_case()` for 11 list lookups across 8 rules

### Tier 4: Remaining Allocation Reduction

**4a. Convert remaining `HashSet<String>` / `Vec<String>` to borrowed variants**
- 72 remaining `.to_string()` calls across rule files
- Largest offenders: `step_output_reference.rs` (14), `workflow_call_outputs.rs` (7), `job_outputs.rs` (5)
- **Estimated gain:** 5-15µs per complex workflow
- **Complexity:** Medium -- requires lifetime annotations threading through nested functions
- **Trade-off:** More complex function signatures; some `.to_string()` calls are in error-path `format!()` messages which are unavoidable and acceptable

**4b. Replace remaining `.to_lowercase()` with `eq_ignore_ascii_case()`**
- 8 remaining calls: `secrets.rs`, `workflow_call_outputs.rs` (x2), `workflow_call_inputs.rs`, `workflow_call_secrets.rs`, `workflow_inputs.rs`, `job_outputs.rs`, `step_output_reference.rs`
- **Estimated gain:** 2-5µs (8 fewer String allocations)
- **Complexity:** Low
- **Trade-off:** None

### Tier 5: Shared Tree Traversal Visitor

**5a. Extract `visit_jobs()` / `visit_steps()` utilities**
- 25+ rules implement nearly identical recursive job-finding traversal
- 15+ rules additionally traverse into steps with the same pattern
- A shared visitor would eliminate ~60 duplicated match blocks
- **Estimated gain:** 10-30µs from better instruction cache locality; ~500 lines of code removed
- **Complexity:** High -- must handle the variety of callback shapes (some rules need key+value, some need the pair node, some need nested contexts)
- **Trade-off:** Rules become shorter but lose self-contained readability; debugging traversal issues becomes harder since logic is split between visitor and callback

### Tier 6: Engine-Level Structural Changes

**6a. Reuse `TrussEngine` across files in CLI parallel path**
- Currently creates a new engine (and tree-sitter parser) per file in the rayon path
- Thread-local engine pool would eliminate repeated parser initialization
- **Estimated gain:** 20-50µs per file in batch mode
- **Complexity:** Medium -- needs `thread_local!` or `rayon::ThreadPool` scoping
- **Trade-off:** More stateful CLI; parser state from previous file could theoretically leak (though tree-sitter handles this correctly)

**6b. Pre-compute shared data across rules**
- Several rules independently look up the same keys (e.g., multiple rules call `find_value_for_key(root, "on")`)
- A pre-computed context struct with commonly-needed nodes could be passed to all rules
- **Estimated gain:** 10-20µs (eliminates ~30 redundant `find_value_for_key` calls)
- **Complexity:** High -- changes the `ValidationRule` trait signature, affects all 41 rules
- **Trade-off:** Rules lose independence; the context struct becomes a coupling point

**6c. Lazy diagnostic message formatting**
- 146 `format!()` calls for diagnostic messages allocate even when diagnostics are filtered by severity
- Deferring formatting to display time would eliminate allocations for filtered diagnostics
- **Estimated gain:** 5-15µs when using `--severity error` (skips warning message formatting)
- **Complexity:** High -- requires changing `Diagnostic.message` from `String` to a lazy type
- **Trade-off:** More complex `Diagnostic` type; marginal gain since most users see all diagnostics

### Summary Table

| Tier | Optimization | Est. Gain | Complexity | Worth It? |
|------|-------------|-----------|------------|-----------|
| 4a | Remaining `String` → `&str` | 5-15µs | Medium | Yes for clean code; marginal perf |
| 4b | Remaining `.to_lowercase()` | 2-5µs | Low | Yes -- easy win |
| 5a | Shared visitor pattern | 10-30µs | High | Yes for maintainability; moderate perf |
| 6a | Reuse engine in CLI | 20-50µs | Medium | Yes for batch workloads |
| 6b | Pre-computed context | 10-20µs | High | Maybe -- high coupling cost |
| 6c | Lazy diagnostic formatting | 5-15µs | High | No -- rarely filtered |

**Bottom line:** We've captured ~95% of the available performance gains. The remaining opportunities total ~50-130µs combined, against a ~700-800µs process startup floor. Further work should prioritize **code maintainability** (Tier 5a visitor pattern) and **batch throughput** (Tier 6a engine reuse) over raw single-file latency.

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
