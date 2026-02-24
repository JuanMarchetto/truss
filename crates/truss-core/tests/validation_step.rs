//! Tests for StepValidationRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates step structure in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_valid_with_uses() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("step") && d.severity == Severity::Error)
        .collect();

    assert!(
        step_errors.is_empty(),
        "Valid step with 'uses' should not produce errors"
    );
}

#[test]
fn test_step_valid_with_run() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Hello"
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("step") && d.severity == Severity::Error)
        .collect();

    assert!(
        step_errors.is_empty(),
        "Valid step with 'run' should not produce errors"
    );
}

#[test]
fn test_step_missing_uses_and_run() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Invalid step
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("uses") || d.message.contains("run"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !step_errors.is_empty(),
        "Step missing both 'uses' and 'run' should produce error"
    );
    assert!(
        step_errors
            .iter()
            .any(|d| d.message.contains("uses") || d.message.contains("run")),
        "Error message should mention 'uses' or 'run'"
    );
}

#[test]
fn test_step_invalid_action_reference() {
    let mut engine = TrussEngine::new();
    // Missing @ref — this is a genuinely invalid action reference format
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    assert!(
        !step_errors.is_empty(),
        "Invalid action reference should produce warning/error"
    );
}

#[test]
fn test_step_valid_with_both_uses_and_run() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: npm install
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("step") && d.severity == Severity::Error)
        .collect();

    assert!(
        step_errors.is_empty(),
        "Steps with both 'uses' and 'run' (in different steps) should be valid"
    );
}

#[test]
fn test_step_uses_and_run_same_step_error() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        run: echo "Hello"
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("cannot have both")
                && d.message.contains("uses")
                && d.message.contains("run")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !step_errors.is_empty(),
        "Step with both 'uses' and 'run' in same step should produce error. Got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_step_only_uses_no_mutual_exclusion_error() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("cannot have both") && d.severity == Severity::Error)
        .collect();

    assert!(
        step_errors.is_empty(),
        "Step with only 'uses' should not trigger mutual exclusion error"
    );
}

#[test]
fn test_step_only_run_no_mutual_exclusion_error() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build
        run: npm run build
"#;

    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("cannot have both") && d.severity == Severity::Error)
        .collect();

    assert!(
        step_errors.is_empty(),
        "Step with only 'run' should not trigger mutual exclusion error"
    );
}
