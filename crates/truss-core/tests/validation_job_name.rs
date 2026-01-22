//! Tests for JobNameRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates job names in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_job_name_valid_simple() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let job_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("job") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        job_errors.is_empty(),
        "Valid job name 'build' should not produce errors"
    );
}

#[test]
fn test_job_name_valid_with_hyphen() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build-and-test:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let job_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("job") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        job_errors.is_empty(),
        "Valid job name with hyphen should not produce errors"
    );
}

#[test]
fn test_job_name_duplicate() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
  build:
    runs-on: windows-latest
"#;
    
    let result = engine.analyze(yaml);
    let duplicate_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("duplicate") || 
                (d.message.contains("job") && d.message.contains("build")))
        .collect();
    
    assert!(
        !duplicate_errors.is_empty(),
        "Duplicate job names should produce error"
    );
    assert!(
        duplicate_errors.iter().any(|d| d.message.contains("duplicate")),
        "Error message should mention 'duplicate'"
    );
}

#[test]
fn test_job_name_invalid_characters() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  "build test":
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let invalid_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("job") && 
                (d.message.contains("invalid") || d.message.contains("character")))
        .collect();
    
    assert!(
        !invalid_errors.is_empty(),
        "Invalid job name characters should produce error"
    );
}

#[test]
fn test_job_name_reserved_names() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  if:
    runs-on: ubuntu-latest
"#;
    
    let result = engine.analyze(yaml);
    let reserved_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("job") && 
                (d.message.contains("reserved") || d.message.contains("if")))
        .collect();
    
    assert!(
        !reserved_errors.is_empty(),
        "Reserved job names should produce error"
    );
}

