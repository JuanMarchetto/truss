# Contributing to Truss

Hey, thanks for wanting to contribute to Truss! Whether you're fixing a bug, adding a new validation rule, or just tidying something up, we appreciate the help. This guide should get you oriented quickly.

## Before You Start

- Take a look at [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) so you have a feel for how the project is put together.
- Browse the [open issues](https://github.com/JuanMarchetto/truss/issues) to see if someone's already working on the same thing or if there's a known bug.
- If you're planning something big, it's worth opening an issue first so we can talk through the approach together.

## Development Setup

### Prerequisites

- Rust 1.70+ (grab it from [rustup](https://rustup.rs/) if you haven't already)
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

These all need to pass before you open a PR:

```bash
# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check

# Type check
cargo check --workspace
```

## Project Principles

These aren't arbitrary -- they keep the codebase manageable as we grow beyond 41 rules and counting.

### Core First

All validation logic lives in `truss-core`. The CLI, LSP, and WASM crates are thin adapters and shouldn't contain any business logic. If you're adding a validation rule, it belongs in `truss-core/validation/rules/`.

### Stateless Rules

Every rule should be stateless and independently testable. Each one implements the `ValidationRule` trait and receives the parsed tree along with the source text. Rules shouldn't depend on each other or hold onto state between calls.

### Performance Matters

We take performance seriously. When you're adding or changing rules:
- Avoid unnecessary allocations
- Don't re-parse the tree -- use the provided `Node` and walk the AST
- Run benchmarks before and after your changes: `cargo bench -p truss-core`

### Determinism

The same input must always produce the same output. No randomness, no system-dependent behavior, no ordering that depends on hash map iteration.

## Adding a Validation Rule

This is the most common type of contribution. Here's the process:

1. Create a new file in `crates/truss-core/validation/rules/`
2. Implement the `ValidationRule` trait
3. Register the rule in `crates/truss-core/lib.rs` (inside the `TrussEngine::new()` constructor)
4. Add tests in `crates/truss-core/tests/`
5. Update `docs/VALIDATION_RULES.md`

### Rule Template

Here's a skeleton to get you started:

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

And a matching test structure:

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

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Make sure everything passes:
   - `cargo test --workspace`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo fmt --all -- --check`
4. Write a clear PR description -- explain what you changed and, more importantly, why
5. Link any related issues

## Reporting Bugs

If you've found a bug, please include:
- The workflow YAML that triggers it (or a minimal reproduction)
- What you expected to happen vs. what actually happened
- Your Truss version (or the commit hash if you're building from source)

## AI-Generated Code Notice

A significant portion of this codebase was AI-generated. If you spot code that looks wrong, unnecessarily complicated, or un-idiomatic for Rust, please don't hesitate to open an issue or submit a fix. Cleaning things up is a genuinely valued contribution -- not a nitpick.

## License

By contributing to Truss, you agree that your contributions will be licensed under the MIT License.
