//! Tests for TimeoutRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates timeout-minutes is a positive number in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_timeout_valid_positive() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("timeout") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        timeout_errors.is_empty(),
        "Valid positive timeout-minutes should not produce errors"
    );
}

#[test]
fn test_timeout_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: ${{ matrix.timeout }}
    strategy:
      matrix:
        timeout: [30, 60, 120]
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("timeout") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        timeout_errors.is_empty(),
        "Valid timeout-minutes expression should not produce errors"
    );
}

#[test]
fn test_timeout_invalid_negative() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: -5
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("timeout") || d.message.contains("negative")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !timeout_errors.is_empty(),
        "Negative timeout-minutes should produce error"
    );
    assert!(
        timeout_errors.iter().any(|d| d.message.contains("negative") || d.message.contains("positive")),
        "Error message should mention that timeout must be positive"
    );
}

#[test]
fn test_timeout_invalid_zero() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 0
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("timeout") || d.message.contains("zero")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !timeout_errors.is_empty(),
        "Zero timeout-minutes should produce error"
    );
}

#[test]
fn test_timeout_invalid_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: "60"
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("timeout") || d.message.contains("type")) && 
                (d.message.contains("invalid") || d.message.contains("number")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !timeout_errors.is_empty(),
        "String timeout-minutes should produce error (must be number)"
    );
}

#[test]
fn test_timeout_valid_no_timeout() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("timeout") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        timeout_errors.is_empty(),
        "Job without timeout-minutes should be valid (timeout is optional)"
    );
}

#[test]
fn test_timeout_valid_large_number() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 360
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("timeout") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        timeout_errors.is_empty(),
        "Large positive timeout-minutes should be valid"
    );
}

#[test]
fn test_timeout_valid_decimal() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 30.5
    steps:
      - run: echo "Building"
"#;
    
    let result = engine.analyze(yaml);
    // GitHub Actions accepts decimal numbers for timeout-minutes
    let timeout_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("timeout") && d.severity == Severity::Error)
        .collect();
    
    // Decimal should be valid, but if we want to enforce integers, we can error
    // For now, we'll allow decimals as GitHub Actions accepts them
    assert!(
        timeout_errors.is_empty(),
        "Decimal timeout-minutes should be valid (GitHub Actions accepts decimals)"
    );
}

