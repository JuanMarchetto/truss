# CI Workflow Fixtures

These are GitHub Actions workflow files we use for parsing, analysis, and benchmarking in Truss.

They're not meant to actually run CI -- they exist so we can throw realistic workflows at Truss and see how it handles YAML parsing, expression evaluation, job graph construction, matrix logic, and conditionals under real-world conditions.

All of them are adapted from real, widely-used open-source projects. We trimmed and tweaked them for our purposes, but they reflect the kind of complexity developers actually deal with.

---

## Fixture Levels

### `simple.yml`

**Complexity:** Simple

A single job, straight-line execution, no conditionals or matrices. Derived from the `licensed.yml` workflow in `actions/checkout`.

This is our baseline -- if something breaks here, we have bigger problems.

---

### `medium.yml`

**Complexity:** Medium

A cron-triggered workflow with conditional step execution, scoped permissions, and multiple execution paths within a single job. Derived from a scheduled event processor in `Azure/azure-sdk-for-js`.

This covers the kind of branching logic you see in most real automation workflows, without the added complexity of job graphs or matrices.

---

### `complex-static.yml`

**Complexity:** Complex (static)

Multiple jobs with matrix strategies, conditional exclusions, cross-job dependencies via `needs`, `if` conditions, and artifact handling. Derived from the main CI workflow in `microsoft/TypeScript`.

Everything here is statically defined in YAML -- no runtime generation involved.

---

### `complex-dynamic.yml`

**Complexity:** Complex (dynamic)

Runtime-generated job matrices, cross-file CI configuration, dynamic graph construction through job outputs, advanced concurrency, and environment gating. Derived from the primary CI workflow in `rust-lang/rust`.

This is about as complex as GitHub Actions gets in the wild. Truss doesn't try to fully resolve the runtime-generated matrices here. Instead, we use this fixture to validate that parsing, expression handling, and partial graph construction hold up under extreme conditions.

---

## Notes on Provenance

- These are **not** exact copies of the original workflows.
- External actions, scripts, and referenced files are intentionally left unresolved.
- The purpose is static analysis, not CI execution.
- Cross-file resolution and runtime evaluation are out of scope for Truss v0.

---

## Why Real Workflows?

Synthetic benchmarks are tidy but they miss the weird stuff. Using workflows from actual projects means we're testing against:

- Patterns that developers actually use in production
- Edge cases that show up organically, not ones we thought to invent
- The workflows people interact with every day

This keeps Truss grounded in real developer pain points rather than artificial scenarios.
