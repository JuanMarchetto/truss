//! Tests for WorkflowNameRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates workflow name field in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_workflow_name_valid_simple() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid workflow name should not produce errors"
    );
}

#[test]
fn test_workflow_name_valid_with_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: ${{ github.event.pull_request.title }}
on: pull_request
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Valid workflow name with expression should not produce errors"
    );
}

#[test]
fn test_workflow_name_valid_optional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    // Workflow name is optional, so missing name should not be an error
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        name_errors.is_empty(),
        "Missing workflow name should not produce errors (name is optional)"
    );
}

#[test]
fn test_workflow_name_invalid_too_long() {
    let mut engine = TrussEngine::new();
    // 300 characters should exceed typical GitHub Actions name limit (255)
    let yaml = format!(r#"
name: {}
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#, "A".repeat(300));
    
    let result = engine.analyze(&yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && 
                (d.message.contains("long") || d.message.contains("length")))
        .collect();
    
    assert!(
        !name_errors.is_empty(),
        "Very long workflow name should produce error"
    );
}

#[test]
fn test_workflow_name_valid_special_characters() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: "CI/CD Pipeline"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && d.severity == Severity::Error)
        .collect();
    
    // Special characters in quotes should be valid
    assert!(
        name_errors.is_empty(),
        "Workflow name with special characters in quotes should be valid"
    );
}

#[test]
fn test_workflow_name_invalid_empty() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: ""
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && 
                (d.message.contains("empty") || d.message.contains("required")))
        .collect();
    
    assert!(
        !name_errors.is_empty(),
        "Empty workflow name should produce error"
    );
    assert!(
        name_errors.iter().any(|d| d.message.contains("empty")),
        "Error message should mention 'empty'"
    );
}

#[test]
fn test_workflow_name_valid_unicode() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: "CI/CD 🚀"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let name_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("name") && d.severity == Severity::Error)
        .collect();
    
    // Unicode characters should be valid in workflow names
    assert!(
        name_errors.is_empty(),
        "Workflow name with unicode should be valid"
    );
}

