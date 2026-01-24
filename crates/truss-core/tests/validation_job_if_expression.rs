//! Tests for JobIfExpressionRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates if condition expressions in jobs in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        if_errors.is_empty(),
        "Valid job if expression with job result should not produce errors"
    );
}

#[test]
fn test_job_if_expression_error_missing_wrapper() {
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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("${{") || d.message.contains("wrapper") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !if_errors.is_empty(),
        "Job if condition missing ${{ }} wrapper should produce error"
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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
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
    let if_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("if") || d.message.contains("job")) && 
                (d.message.contains("expression") || d.message.contains("nonexistent") || d.message.contains("not found")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !if_errors.is_empty(),
        "Job if expression referencing non-existent job should produce error"
    );
}

