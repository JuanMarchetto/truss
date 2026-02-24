//! Tests for StepTimeoutRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates timeout-minutes at step level in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_timeout_valid_positive() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - timeout-minutes: 30
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        timeout_errors.is_empty(),
        "Valid positive timeout-minutes at step level should not produce errors"
    );
}

#[test]
fn test_step_timeout_valid_decimal() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - timeout-minutes: 30.5
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        timeout_errors.is_empty(),
        "Valid decimal timeout-minutes should not produce errors"
    );
}

#[test]
fn test_step_timeout_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        timeout: [30, 60]
    steps:
      - timeout-minutes: ${{ matrix.timeout }}
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        timeout_errors.is_empty(),
        "Valid timeout-minutes expression should not produce errors"
    );
}

#[test]
fn test_step_timeout_error_negative() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - timeout-minutes: -5
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && (d.message.contains("negative") || d.message.contains("positive"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !timeout_errors.is_empty(),
        "Negative timeout-minutes should produce error"
    );
}

#[test]
fn test_step_timeout_error_zero() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - timeout-minutes: 0
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && (d.message.contains("zero") || d.message.contains("positive"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !timeout_errors.is_empty(),
        "Zero timeout-minutes should produce error"
    );
}

#[test]
fn test_step_timeout_error_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - timeout-minutes: "30"
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("timeout") || d.message.contains("timeout-minutes"))
                && (d.message.contains("string")
                    || d.message.contains("number")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !timeout_errors.is_empty(),
        "String timeout-minutes should produce error"
    );
}
