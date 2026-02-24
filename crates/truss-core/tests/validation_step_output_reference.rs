//! Tests for StepOutputReferenceRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates that step output references (steps.<step_id>.outputs.<output_name>) reference valid outputs
//! in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_output_reference_valid_existing_output() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.build.outputs.result }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found")
                    || d.message.contains("nonexistent")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid step output reference should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_multiple_outputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: |
          echo "version=1.0.0" >> $GITHUB_OUTPUT
          echo "hash=abc123" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.build.outputs.version }}"
      - run: echo "${{ steps.build.outputs.hash }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid references to multiple outputs from same step should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_in_job_outputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      result: ${{ steps.build.outputs.result }}
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid step output reference in job outputs should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_in_step_if() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - if: ${{ steps.build.outputs.result == 'success' }}
        run: echo "Build succeeded"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid step output reference in step if condition should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_in_step_env() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "version=1.0.0" >> $GITHUB_OUTPUT
      - env:
          VERSION: ${{ steps.build.outputs.version }}
        run: echo "$VERSION"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid step output reference in step env should not produce errors"
    );
}

#[test]
fn test_step_output_reference_error_nonexistent_output() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.build.outputs.nonexistent }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found")
                    || d.message.contains("nonexistent")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "Reference to non-existent step output should produce error"
    );
    assert!(
        output_errors
            .iter()
            .any(|d| d.message.contains("nonexistent") || d.message.contains("not found")),
        "Error message should mention non-existent output"
    );
}

#[test]
fn test_step_output_reference_error_step_without_id() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "result=success" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.build.outputs.result }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found")
                    || d.message.contains("missing")
                    || d.message.contains("id"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "Reference to step without id field should produce error"
    );
}

#[test]
fn test_step_output_reference_error_different_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ steps.build.outputs.result }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found")
                    || d.message.contains("different")
                    || d.message.contains("job"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "Reference to step output from different job should produce error"
    );
}

#[test]
fn test_step_output_reference_valid_expression_fallback() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "result=success" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.build.outputs.result || 'default' }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid step output reference with expression fallback should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_dot_notation_multiple_steps() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: setup
        run: echo "version=1.0" >> $GITHUB_OUTPUT
      - id: build
        run: echo "status=ok" >> $GITHUB_OUTPUT
      - run: echo "${{ steps.setup.outputs.version }} ${{ steps.build.outputs.status }}"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Multiple valid dot-notation output references should not produce errors"
    );
}

#[test]
fn test_step_output_reference_valid_in_comparison() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: check
        run: echo "result=success" >> $GITHUB_OUTPUT
      - if: ${{ steps.check.outputs.result == 'success' }}
        run: echo "Passed"
      - if: ${{ steps.check.outputs.result != 'failure' }}
        run: echo "Not failed"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("step"))
                && (d.message.contains("not found") || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Output references in comparisons should not produce errors"
    );
}
