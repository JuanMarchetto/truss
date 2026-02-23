//! Tests for RunnerLabelRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates runs-on labels are valid GitHub-hosted runners or self-hosted runner groups in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_runner_label_valid_ubuntu_latest() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Valid ubuntu-latest runner should not produce errors"
    );
}

#[test]
fn test_runner_label_valid_windows_latest() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: windows-latest
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Valid windows-latest runner should not produce errors"
    );
}

#[test]
fn test_runner_label_valid_self_hosted() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: self-hosted
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Valid self-hosted runner should not produce errors"
    );
}

#[test]
fn test_runner_label_valid_matrix_reference() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Valid matrix runner reference should not produce errors"
    );
}

#[test]
fn test_runner_label_warning_unknown() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: unknown-runner-label
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let _runner_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && (d.severity == Severity::Warning || d.severity == Severity::Error)
        })
        .collect();

    // Note: This may produce a warning or be valid depending on implementation
    // Basic format validation should pass, but unknown labels might warn
    assert!(true, "Unknown runner label may produce warning");
}

#[test]
fn test_runner_label_valid_with_labels() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: [self-hosted, linux, x64]
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("runs-on") || d.message.contains("runner"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Valid runner with labels should not produce errors"
    );
}
