//! Tests for StepIfExpressionRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates if condition expressions in steps in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_if_expression_valid_github_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ github.ref == 'refs/heads/main' }}
        run: echo "On main branch"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid step if expression with github.ref should not produce errors"
    );
}

#[test]
fn test_step_if_expression_valid_step_outputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - if: ${{ steps.build.outputs.result == 'success' }}
        run: echo "Build succeeded"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid step if expression with step outputs should not produce errors"
    );
}

#[test]
fn test_step_if_expression_valid_matrix() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    steps:
      - if: ${{ matrix.os == 'ubuntu-latest' }}
        run: echo "Ubuntu"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid step if expression with matrix should not produce errors"
    );
}

#[test]
fn test_step_if_expression_valid_bare_expression() {
    // GitHub Actions auto-wraps if: conditions in ${{ }}, so bare expressions are valid
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: github.ref == 'refs/heads/main'
        run: echo "On main branch"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression")
                    || d.message.contains("${{")
                    || d.message.contains("wrapper")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare step if condition should be valid (GitHub Actions auto-wraps). Got errors: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_step_if_expression_error_invalid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ invalid.expression }}
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !if_errors.is_empty(),
        "Invalid step if expression should produce error"
    );
}

#[test]
fn test_step_if_expression_error_undefined_context() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ github.nonexistent.property }}
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression")
                    || d.message.contains("undefined")
                    || d.message.contains("nonexistent"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    assert!(
        !if_errors.is_empty(),
        "Step if expression with undefined context should produce error/warning"
    );
}

#[test]
fn test_step_if_expression_valid_nested_conditional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ github.ref == 'refs/heads/main' && github.event_name == 'push' }}
        run: echo "Main branch push"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("step"))
                && (d.message.contains("expression") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Valid nested step if conditional should not produce errors"
    );
}
