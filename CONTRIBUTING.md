# Contributing to Truss

Thank you for your interest in contributing to Truss! This document provides guidelines and information to help you get started.

## Before You Start

- Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to understand the project's design principles
- Check the [open issues](https://github.com/JuanMarchetto/truss/issues) for existing discussions or known bugs
- For large changes, open an issue first to discuss the approach

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- `just` (recommended) or `make` for build automation

### Building

```bash
git clone https://github.com/JuanMarchetto/truss.git
cd truss

# Debug build
cargo build --workspace

# Release build
cargo build --workspace --release
```

### Running Tests

```bash
# Run all tests
just test
# or: cargo test --workspace

# Run core tests only
cargo test -p truss-core
```

### Code Quality Checks

All of these must pass before submitting a PR:

```bash
# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check

# Type check
cargo check --workspace
```

## Project Principles

### Core First

All validation logic belongs in `truss-core`. The CLI, LSP, and WASM crates are thin adapters that should contain no business logic. If you're adding a validation rule, it goes in `truss-core/validation/rules/`.

### Stateless Rules

Validation rules must be stateless and independently testable. Each rule implements the `ValidationRule` trait and receives the parsed tree and source text. Rules should not depend on each other or maintain state between calls.

### Performance Matters

Performance is a first-class requirement. When adding or modifying rules:
- Avoid unnecessary allocations
- Don't re-parse the tree — use the provided `Node` and walk the AST
- Run benchmarks before and after your changes: `cargo bench -p truss-core`

### Determinism

The same input must always produce the same output. No randomness, no system-dependent behavior, no ordering that depends on hash maps.

## Adding a Validation Rule

1. Create a new file in `crates/truss-core/validation/rules/`
2. Implement the `ValidationRule` trait
3. Register the rule in `crates/truss-core/lib.rs` (in the `TrussEngine::new()` constructor)
4. Add tests in `crates/truss-core/tests/`
5. Update `docs/VALIDATION_RULES.md`

### Rule Template

```rust
use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

pub struct MyNewRule;

impl ValidationRule for MyNewRule {
    fn name(&self) -> &str {
        "my_new_rule"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        // Your validation logic here

        diagnostics
    }
}
```

### Test Template

```rust
use truss_core::{Severity, TrussEngine};

#[test]
fn test_my_rule_valid_case() {
    let input = r#"
name: Test
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
"#;
    let mut engine = TrussEngine::new();
    let result = engine.analyze(input);
    assert!(result.is_ok());
}

#[test]
fn test_my_rule_error_case() {
    let input = r#"
name: Test
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
"#;
    let mut engine = TrussEngine::new();
    let result = engine.analyze(input);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(!errors.is_empty());
}
```

## Submitting a Pull Request

1. Fork the repository and create a branch from `main`
2. Make your changes
3. Ensure all checks pass:
   - `cargo test --workspace`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo fmt --all -- --check`
4. Write a clear PR description explaining what changed and why
5. Link any related issues

## Reporting Bugs

When filing an issue, please include:
- The workflow YAML that triggers the bug (or a minimal reproduction)
- Expected behavior vs actual behavior
- Truss version (commit hash if building from source)

## AI-Generated Code Notice

A significant portion of this codebase was AI-generated. If you find code that looks incorrect, overly complex, or inconsistent with Rust idioms, please don't hesitate to open an issue or submit a fix. Improving code quality is a valued contribution.

## License

By contributing to Truss, you agree that your contributions will be licensed under the MIT License.
