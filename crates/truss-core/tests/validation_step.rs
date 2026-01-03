//! Tests for StepValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates step structure in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

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
    let step_errors: Vec<_> = result.diagnostics
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
    let step_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("step") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        step_errors.is_empty(),
        "Valid step with 'run' should not produce errors"
    );
}

#[test]
#[ignore = "StepValidationRule not yet implemented"]
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
    let step_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("step") || d.message.contains("uses") || d.message.contains("run")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !step_errors.is_empty(),
        "Step missing both 'uses' and 'run' should produce error"
    );
    assert!(
        step_errors.iter().any(|d| d.message.contains("uses") || d.message.contains("run")),
        "Error message should mention 'uses' or 'run'"
    );
}

#[test]
#[ignore = "StepValidationRule not yet implemented"]
fn test_step_invalid_action_reference() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: invalid/action@v1
"#;
    
    let result = engine.analyze(yaml);
    let step_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("step") || d.message.contains("action") || d.message.contains("uses")) &&
                (d.severity == Severity::Error || d.severity == Severity::Warning))
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
    let step_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("step") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        step_errors.is_empty(),
        "Steps with both 'uses' and 'run' (in different steps) should be valid"
    );
}

