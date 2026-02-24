//! Tests for WorkflowCallOutputsRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates workflow_call outputs are properly defined in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_workflow_call_outputs_valid_reference() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      result:
        description: "Build result"
        value: ${{ jobs.build.outputs.result }}
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
            (d.message.contains("output") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid workflow_call output referencing valid job output should not produce errors"
    );
}

#[test]
fn test_workflow_call_outputs_valid_multiple_outputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      version:
        description: "Version"
        value: ${{ jobs.build.outputs.version }}
      hash:
        description: "Hash"
        value: ${{ jobs.build.outputs.hash }}
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
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid workflow_call with multiple outputs should not produce errors"
    );
}

#[test]
fn test_workflow_call_outputs_valid_with_description() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      result:
        description: "The build result"
        value: ${{ jobs.build.outputs.result }}
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
            (d.message.contains("output") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        output_errors.is_empty(),
        "Valid workflow_call output with description should not produce errors"
    );
}

#[test]
fn test_workflow_call_outputs_error_nonexistent_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      result:
        description: "Build result"
        value: ${{ jobs.nonexistent.outputs.result }}
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Build"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output")
                || d.message.contains("workflow_call")
                || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "workflow_call output referencing non-existent job should produce error"
    );
}

#[test]
fn test_workflow_call_outputs_error_nonexistent_job_output() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      result:
        description: "Build result"
        value: ${{ jobs.build.outputs.nonexistent }}
jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
      - id: version
        run: echo "version=1.0.0" >> $GITHUB_OUTPUT
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output")
                || d.message.contains("workflow_call")
                || d.message.contains("nonexistent"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "workflow_call output referencing non-existent job output should produce error"
    );
}

#[test]
fn test_workflow_call_outputs_error_invalid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    outputs:
      result:
        description: "Build result"
        value: ${{ invalid.expression }}
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Build"
"#;

    let result = engine.analyze(yaml);
    let output_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("output")
                || d.message.contains("workflow_call")
                || d.message.contains("expression"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !output_errors.is_empty(),
        "workflow_call output with invalid expression should produce error"
    );
}
