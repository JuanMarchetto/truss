# CI Workflow Fixtures

This directory contains GitHub Actions workflow fixtures used for
**parsing, analysis, and benchmarking** in Truss.

The goal of these fixtures is **not** to execute CI jobs, but to represent
real-world GitHub Actions workflows across a spectrum of complexity, in order
to evaluate:

- YAML parsing performance
- Expression handling
- Job graph construction
- Matrix and conditional semantics
- Behavior under realistic CI complexity

All fixtures are **derived from real, widely used open-source repositories**.
They are included here in adapted or reduced form for benchmarking and testing
purposes only.

---

## Fixture Levels

### `simple.yml`

**Complexity:** Simple  
**Characteristics:**
- Single job
- Linear execution
- No conditionals, matrices, or job dependencies

**Source:**
- Derived from the `licensed.yml` workflow in `actions/checkout`

This fixture serves as a **baseline** for correctness and performance.

---

### `medium-scheduled.yml`

**Complexity:** Medium  
**Characteristics:**
- Scheduled (`cron`) triggers
- Conditional step execution based on schedule identity
- Single job with multiple execution paths
- Scoped permissions

**Source:**
- Derived from the scheduled event processor workflow in `Azure/azure-sdk-for-js`

This fixture represents common real-world automation workflows that include
branching logic without introducing job graphs or matrices.

---

### `complex-static.yml`

**Complexity:** Complex (static)  
**Characteristics:**
- Multiple jobs
- Matrix strategies with conditional exclusions
- Cross-job dependencies (`needs`)
- Conditional execution (`if`)
- Artifact handling and aggregation jobs

**Source:**
- Derived from the primary CI workflow in `microsoft/TypeScript`

All job definitions and matrices are statically defined in YAML.

---

### `complex-dynamic.yml`

**Complexity:** Complex (dynamic)  
**Characteristics:**
- Runtime-generated job matrices
- Cross-file CI configuration
- Dynamic graph construction via job outputs
- Advanced concurrency and environment gating
- Partial static observability by design

**Source:**
- Derived from the primary CI workflow in `rust-lang/rust`

This fixture represents the upper bound of GitHub Actions complexity commonly
seen in large, long-lived repositories.

Truss does **not** attempt to fully resolve runtime-generated matrices in this
fixture. Instead, it is used to validate parsing, expression handling, and
partial graph construction under extreme real-world conditions.

---

## Notes on Provenance and Scope

- These fixtures are **not exact copies** of the original workflows.
- External actions, scripts, and referenced files are intentionally **not**
  resolved or executed.
- The purpose of these fixtures is **static analysis**, not CI execution.
- Cross-file resolution and runtime evaluation are explicitly **out of scope**
  for Truss v0.

---

## Why Real-World Fixtures

Using real workflows ensures that Truss is evaluated against:

- Actual patterns used in production CI systems
- Edge cases that arise organically, not synthetically
- The kinds of workflows developers interact with daily

This approach avoids artificial benchmarks and grounds Truss in practical,
observable developer pain points.
