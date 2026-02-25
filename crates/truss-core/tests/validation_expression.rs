//! Tests for ExpressionValidationRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates GitHub Actions expressions (${{ }}).

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_expression_valid_context_variable() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.number }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("${{"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Valid expression with context variable should not produce errors"
    );
}

#[test]
fn test_expression_valid_matrix_variable() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest]
    steps:
      - run: echo "${{ matrix.os }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("matrix"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Valid expression with matrix variable should not produce errors"
    );
}

#[test]
fn test_expression_valid_workflow_dispatch_input() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: string
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("inputs"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Valid expression with workflow_dispatch input should not produce errors"
    );
}

#[test]
fn test_expression_valid_conditional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ github.ref == 'refs/heads/main' }}
        run: echo "On main branch"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("if"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Valid conditional expression should not produce errors"
    );
}

#[test]
fn test_expression_invalid_unclosed() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("expression")
                || d.message.contains("unclosed")
                || d.message.contains("${{")
        })
        .collect();

    assert!(
        !expr_errors.is_empty(),
        "Unclosed expression should produce error"
    );
    assert!(
        expr_errors.iter().any(|d| d.message.contains("unclosed")),
        "Error message should mention 'unclosed'"
    );
}

#[test]
fn test_expression_unknown_github_property_no_false_positive() {
    let mut engine = TrussEngine::new();
    // github.nonexistent.property starts with a known context (github.)
    // so it should not produce expression syntax errors.
    // Property-level validation is not implemented — only context-level.
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.nonexistent.property }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("undefined")
                || d.message.contains("nonexistent")
                || d.message.contains("Invalid expression")
        })
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Known context prefix (github.) should not produce false positives for unknown properties"
    );
}

#[test]
fn test_expression_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ invalid syntax here }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("expression")
                || d.message.contains("syntax")
                || d.message.contains("invalid")
        })
        .collect();

    assert!(
        !expr_errors.is_empty(),
        "Invalid expression syntax should produce error"
    );
}

#[test]
fn test_expression_valid_nested() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.head.ref }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") && d.severity == Severity::Error)
        .collect();

    assert!(
        expr_errors.is_empty(),
        "Valid nested expression should not produce errors"
    );
}

// === Regression tests for case-insensitive context names (Bug #3) ===

#[test]
fn test_expression_uppercase_github_context() {
    let mut engine = TrussEngine::new();
    // GITHUB.repository_owner is valid — contexts are case-insensitive
    let yaml = r#"
on: [gollum]
jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: '${{ GITHUB.repository_owner }}/my-repo'
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("Invalid expression") || d.message.contains("GITHUB"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "GITHUB.repository_owner (uppercase) should be a valid context reference. Got: {:?}",
        expr_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_expression_mixed_case_context() {
    let mut engine = TrussEngine::new();
    // Mixed case like GitHub.ref should also work
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ GitHub.ref }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("Invalid expression") && d.message.contains("GitHub"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "GitHub.ref (mixed case) should be valid. Got: {:?}",
        expr_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_expression_uppercase_env_context() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ ENV.MY_VAR }}"
"#;

    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("Invalid expression") && d.message.contains("ENV"))
        .collect();

    assert!(
        expr_errors.is_empty(),
        "ENV.MY_VAR (uppercase) should be valid. Got: {:?}",
        expr_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
