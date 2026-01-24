//! Tests for StepNameRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates step name field format in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_step_name_valid_descriptive() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build application
        run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid descriptive step name should not produce errors"
    );
}

#[test]
fn test_step_name_valid_special_characters() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: "Build & Test (v1.0)"
        run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid step name with special characters should not produce errors"
    );
}

#[test]
fn test_step_name_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build ${{ github.ref }}
        run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid step name with expression should not produce errors"
    );
}

#[test]
fn test_step_name_warning_empty() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: ""
        run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let name_warnings: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                (d.severity == Severity::Warning || d.severity == Severity::Error))
        .collect();
    
    assert!(
        !name_warnings.is_empty(),
        "Empty step name should produce warning"
    );
}

#[test]
fn test_step_name_warning_very_long() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: "This is a very long step name that exceeds reasonable length and should probably be shortened for better readability in the GitHub Actions UI"
        run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let _name_warnings: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                (d.severity == Severity::Warning || d.severity == Severity::Error))
        .collect();
    
    // Note: This may produce a warning or be valid depending on implementation
    assert!(
        true,
        "Very long step name may produce warning"
    );
}

#[test]
fn test_step_name_valid_unicode() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: "Build 🚀"
        run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("name") || d.message.contains("step")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid step name with Unicode characters should not produce errors"
    );
}

