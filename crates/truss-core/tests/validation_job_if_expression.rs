//! Tests for JobIfExpressionRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates if condition expressions in jobs in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_job_if_expression_valid_github_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    if: ${{ github.ref == 'refs/heads/main' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid job if expression with github.ref should not produce errors"
    );
}

#[test]
fn test_job_if_expression_valid_event_name() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: [push, pull_request]
jobs:
  build:
    if: ${{ github.event_name == 'push' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Build"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid job if expression with event_name should not produce errors"
    );
}

#[test]
fn test_job_if_expression_valid_job_result() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Build"
  deploy:
    needs: build
    if: ${{ jobs.build.result == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid job if expression with job result should not produce errors"
    );
}

#[test]
fn test_job_if_expression_valid_bare_expression() {
    // GitHub Actions auto-wraps if: conditions in ${{ }}, so bare expressions are valid
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression")
                    || d.message.contains("${{")
                    || d.message.contains("wrapper")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare job if condition should be valid (GitHub Actions auto-wraps). Got errors: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_job_if_expression_error_invalid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    if: ${{ invalid.expression }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !if_errors.is_empty(),
        "Invalid job if expression should produce error"
    );
}

#[test]
fn test_job_if_expression_error_nonexistent_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    if: ${{ jobs.nonexistent.result == 'success' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("job"))
                && (d.message.contains("expression")
                    || d.message.contains("nonexistent")
                    || d.message.contains("not found"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !if_errors.is_empty(),
        "Job if expression referencing non-existent job should produce error"
    );
}

#[test]
fn test_job_if_expression_unknown_property_no_false_positive() {
    let mut engine = TrussEngine::new();
    // github.nonexistent.property starts with a known context (github.)
    // so it should not produce errors. Property-level validation is not implemented.
    let yaml = r#"
on: push
jobs:
  build:
    if: ${{ github.nonexistent.property }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("undefined")
                || d.message.contains("nonexistent")
                || d.message.contains("Invalid expression")
        })
        .collect();

    assert!(
        warnings.is_empty(),
        "Known context prefix (github.) should not produce false positives"
    );
}

#[test]
fn test_job_if_expression_valid_secrets_no_false_warning() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    if: ${{ secrets.DEPLOY_TOKEN != '' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let context_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("undefined") || d.message.contains("nonexistent"))
        .collect();

    assert!(
        context_warnings.is_empty(),
        "Valid secrets context reference should not produce undefined context warnings. Got: {:?}",
        context_warnings
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_job_if_expression_valid_matrix_no_false_warning() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    if: ${{ matrix.os == 'ubuntu-latest' }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    steps:
      - run: echo "Testing on ${{ matrix.os }}"
"#;

    let result = engine.analyze(yaml);
    let context_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("undefined") || d.message.contains("nonexistent"))
        .collect();

    assert!(
        context_warnings.is_empty(),
        "Valid matrix context reference should not produce undefined context warnings. Got: {:?}",
        context_warnings
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
