//! Tests for RunsOnRequiredRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates that `runs-on` is required for all jobs in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_runs_on_valid_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        runs_on_errors.is_empty(),
        "Valid job with 'runs-on' should not produce errors"
    );
}

#[test]
fn test_runs_on_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        runs_on_errors.is_empty(),
        "Valid job with 'runs-on' expression should not produce errors"
    );
}

#[test]
fn test_runs_on_valid_multiple_jobs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
  test:
    runs-on: windows-latest
  deploy:
    runs-on: macos-latest
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        runs_on_errors.is_empty(),
        "All jobs with 'runs-on' should not produce errors"
    );
}

#[test]
fn test_runs_on_missing() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    steps:
      - run: echo "Hello"
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !runs_on_errors.is_empty(),
        "Job missing 'runs-on' should produce error"
    );
    assert!(
        runs_on_errors.iter().any(|d| d.message.contains("runs-on") || d.message.contains("required")),
        "Error message should mention 'runs-on' or 'required'"
    );
}

#[test]
fn test_runs_on_empty_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ""
    steps:
      - run: echo "Hello"
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                (d.message.contains("empty") || d.message.contains("required")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !runs_on_errors.is_empty(),
        "Job with empty 'runs-on' should produce error"
    );
}

#[test]
fn test_runs_on_missing_multiple_jobs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
  test:
    steps:
      - run: echo "Test"
  deploy:
    runs-on: macos-latest
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !runs_on_errors.is_empty(),
        "Job 'test' missing 'runs-on' should produce error"
    );
    assert!(
        runs_on_errors.iter().any(|d| d.message.contains("test") || d.message.contains("runs-on")),
        "Error should identify the job missing 'runs-on'"
    );
}

#[test]
fn test_runs_on_valid_with_other_fields() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    name: Build job
    needs: []
    steps:
      - run: echo "Build"
"#;
    
    let result = engine.analyze(yaml);
    let runs_on_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("runs-on") || d.message.contains("runs_on")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        runs_on_errors.is_empty(),
        "Job with 'runs-on' and other fields should be valid"
    );
}


