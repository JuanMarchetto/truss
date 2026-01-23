//! Tests for JobOutputsRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates that job outputs reference valid step IDs in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_job_outputs_valid_reference() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build_step.outputs.result }}
    steps:
      - id: build_step
        run: echo "result=success" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Valid job output referencing existing step ID should not produce errors"
    );
}

#[test]
fn test_job_outputs_valid_multiple_outputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      hash: ${{ steps.hash.outputs.hash }}
    steps:
      - id: version
        run: echo "version=1.0.0" >> $GITHUB_OUTPUT
      - id: hash
        run: echo "hash=abc123" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Valid job outputs referencing multiple step IDs should not produce errors"
    );
}

#[test]
fn test_job_outputs_nonexistent_step() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.nonexistent.outputs.result }}
    steps:
      - id: build_step
        run: echo "Hello"
"#;
    
    let result = engine.analyze(yaml);
    // Job outputs must reference step IDs that exist in the same job
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("output") || d.message.contains("step")) && 
                (d.message.contains("nonexistent") || d.message.contains("not found")) &&
                d.severity == Severity::Error)
        .collect();
    
    // Note: JobOutputsRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !output_errors.is_empty(),
        "Job output referencing non-existent step ID should produce error"
    );
    assert!(
        output_errors.iter().any(|d| d.message.contains("nonexistent")),
        "Error message should mention 'nonexistent' step"
    );
}

#[test]
fn test_job_outputs_reference_different_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build_step
        run: echo "Build"
  deploy:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build_step.outputs.result }}
    steps:
      - run: echo "Deploy"
"#;
    
    let result = engine.analyze(yaml);
    // Job outputs can only reference steps from the same job
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("output") || d.message.contains("step")) && 
                d.severity == Severity::Error)
        .collect();
    
    // Note: JobOutputsRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !output_errors.is_empty(),
        "Job output referencing step from different job should produce error"
    );
}

#[test]
fn test_job_outputs_valid_no_outputs() {
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
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Job without outputs should not produce errors"
    );
}

#[test]
fn test_job_outputs_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build.outputs.result || 'default' }}
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Job output with expression referencing valid step should not produce errors"
    );
}

#[test]
fn test_job_outputs_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build.outputs }}
    steps:
      - id: build
        run: echo "Hello"
"#;
    
    let result = engine.analyze(yaml);
    // Output reference should include the output name, not just steps.step_id.outputs
    // This might be caught by expression validation, but job outputs rule should also check
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("output") || d.message.contains("syntax")) && 
                d.severity == Severity::Error)
        .collect();
    
    // Note: JobOutputsRule is not yet implemented
    // ExpressionValidationRule might catch this, but JobOutputsRule should also validate
    assert!(
        !output_errors.is_empty() || result.diagnostics.iter().any(|d| d.message.contains("expression")),
        "Invalid output syntax should produce error or expression error"
    );
}

#[test]
fn test_job_outputs_multiple_outputs_same_step() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.build.outputs.version }}
      hash: ${{ steps.build.outputs.hash }}
      tag: ${{ steps.build.outputs.tag }}
    steps:
      - id: build
        run: |
          echo "version=1.0.0" >> $GITHUB_OUTPUT
          echo "hash=abc123" >> $GITHUB_OUTPUT
          echo "tag=v1.0.0" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Multiple outputs referencing same step ID should be valid"
    );
}

#[test]
fn test_job_outputs_reference_step_without_id() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build_step.outputs.result }}
    steps:
      - run: echo "result=success" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    // Step without 'id' field cannot be referenced in outputs
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("output") || d.message.contains("step")) && 
                (d.message.contains("build_step") || d.message.contains("not found") || d.message.contains("missing")) &&
                d.severity == Severity::Error)
        .collect();
    
    // Note: JobOutputsRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !output_errors.is_empty(),
        "Job output referencing step without 'id' field should produce error"
    );
}

#[test]
fn test_job_outputs_valid_conditional_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build.outputs.result || steps.fallback.outputs.result }}
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - id: fallback
        run: echo "result=fallback" >> $GITHUB_OUTPUT
"#;
    
    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("output") && d.severity == Severity::Error)
        .collect();
    
    assert!(
        output_errors.is_empty(),
        "Job output with conditional expression referencing valid step IDs should not produce errors"
    );
}

