//! Tests for ExpressionValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates GitHub Actions expressions (${{ }}).

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_expression_valid_context_variable() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.number }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("${{"))
        .collect();
    
    assert!(
        expr_errors.is_empty(),
        "Valid expression with context variable should not produce errors"
    );
}

#[test]
fn test_expression_valid_matrix_variable() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest]
    steps:
      - run: echo "${{ matrix.os }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("matrix"))
        .collect();
    
    assert!(
        expr_errors.is_empty(),
        "Valid expression with matrix variable should not produce errors"
    );
}

#[test]
fn test_expression_valid_workflow_dispatch_input() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: string
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("inputs"))
        .collect();
    
    assert!(
        expr_errors.is_empty(),
        "Valid expression with workflow_dispatch input should not produce errors"
    );
}

#[test]
fn test_expression_valid_conditional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ github.ref == 'refs/heads/main' }}
        run: echo "On main branch"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || d.message.contains("if"))
        .collect();
    
    assert!(
        expr_errors.is_empty(),
        "Valid conditional expression should not produce errors"
    );
}

#[test]
#[ignore = "ExpressionValidationRule not yet implemented"]
fn test_expression_invalid_unclosed() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || 
                d.message.contains("unclosed") ||
                d.message.contains("${{"))
        .collect();
    
    assert!(
        !expr_errors.is_empty(),
        "Unclosed expression should produce error"
    );
    assert!(
        expr_errors.iter().any(|d| d.message.contains("unclosed")),
        "Error message should mention 'unclosed'"
    );
}

#[test]
#[ignore = "ExpressionValidationRule not yet implemented"]
fn test_expression_invalid_undefined_context() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.nonexistent.property }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || 
                d.message.contains("undefined") ||
                d.message.contains("nonexistent"))
        .collect();
    
    assert!(
        !expr_errors.is_empty(),
        "Undefined context should produce error/warning"
    );
}

#[test]
#[ignore = "ExpressionValidationRule not yet implemented"]
fn test_expression_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ invalid syntax here }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") || 
                d.message.contains("syntax") ||
                d.message.contains("invalid"))
        .collect();
    
    assert!(
        !expr_errors.is_empty(),
        "Invalid expression syntax should produce error"
    );
}

#[test]
fn test_expression_valid_nested() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.head.ref }}"
"#;
    
    let result = engine.analyze(yaml);
    let expr_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        expr_errors.is_empty(),
        "Valid nested expression should not produce errors"
    );
}

