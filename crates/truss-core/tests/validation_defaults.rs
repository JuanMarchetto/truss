//! Tests for DefaultsValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates defaults configuration at workflow and job levels in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_defaults_valid_workflow_level_shell() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
defaults:
  run:
    shell: bash
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("defaults") || d.message.contains("shell")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        defaults_errors.is_empty(),
        "Valid workflow-level defaults with shell should not produce errors"
    );
}

#[test]
fn test_defaults_valid_job_level_working_directory() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./src
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("defaults") || d.message.contains("working-directory")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        defaults_errors.is_empty(),
        "Valid job-level defaults with working-directory should not produce errors"
    );
}

#[test]
fn test_defaults_valid_with_expressions() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
defaults:
  run:
    shell: ${{ matrix.shell || 'bash' }}
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        shell: [bash, sh]
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    
    // Filter for defaults errors only (not matrix errors that mention "shell")
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("defaults") && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        defaults_errors.is_empty(),
        "Valid defaults with expressions should not produce errors"
    );
}

#[test]
fn test_defaults_error_invalid_shell() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
defaults:
  run:
    shell: invalid-shell
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("defaults") || d.message.contains("shell")) && 
                (d.message.contains("invalid") || d.message.contains("unknown")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !defaults_errors.is_empty(),
        "Invalid shell in defaults should produce error"
    );
}

#[test]
fn test_defaults_error_invalid_working_directory() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
defaults:
  run:
    working-directory: ""
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("defaults") || d.message.contains("working-directory")) && 
                (d.message.contains("invalid") || d.message.contains("empty")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !defaults_errors.is_empty(),
        "Invalid working-directory in defaults should produce error"
    );
}

#[test]
fn test_defaults_valid_inheritance() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
defaults:
  run:
    shell: bash
jobs:
  build:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./src
    steps:
      - run: echo "Test"
"#;
    
    let result = engine.analyze(yaml);
    let defaults_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("defaults") || d.message.contains("shell") || d.message.contains("working-directory")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        defaults_errors.is_empty(),
        "Valid defaults inheritance should not produce errors"
    );
}

