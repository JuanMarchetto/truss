//! Tests for StepContinueOnErrorRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates continue-on-error is a boolean in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_continue_on_error_valid_true() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - continue-on-error: true
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let continue_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("continue-on-error") || d.message.contains("continue"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        continue_errors.is_empty(),
        "Valid continue-on-error: true should not produce errors"
    );
}

#[test]
fn test_step_continue_on_error_valid_false() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - continue-on-error: false
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let continue_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("continue-on-error") || d.message.contains("continue"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        continue_errors.is_empty(),
        "Valid continue-on-error: false should not produce errors"
    );
}

#[test]
fn test_step_continue_on_error_error_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - continue-on-error: "true"
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let continue_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("continue-on-error") || d.message.contains("continue"))
                && (d.message.contains("string")
                    || d.message.contains("boolean")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !continue_errors.is_empty(),
        "String continue-on-error value should produce error"
    );
}

#[test]
fn test_step_continue_on_error_error_number() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - continue-on-error: 1
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let continue_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("continue-on-error") || d.message.contains("continue"))
                && (d.message.contains("number")
                    || d.message.contains("boolean")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !continue_errors.is_empty(),
        "Number continue-on-error value should produce error"
    );
}

#[test]
fn test_step_continue_on_error_with_comment_before_step() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - # This comment appears between dash and step content
        continue-on-error: true
        run: echo "may fail"
"#;

    let result = engine.analyze(yaml);
    let continue_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("continue-on-error")
                && (d.message.contains("invalid") || d.message.contains("boolean"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        continue_errors.is_empty(),
        "Valid continue-on-error with comment between dash and step should not produce errors. Got: {:?}",
        continue_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
