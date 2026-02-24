//! Tests for WorkflowCallInputsRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates workflow_call inputs and their usage in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_workflow_call_inputs_valid_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: string
        description: 'Environment to deploy to'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying to ${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_call with string input should not produce errors"
    );
}

#[test]
fn test_workflow_call_inputs_valid_required_and_optional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: string
        required: true
        description: 'Environment'
      version:
        type: string
        required: false
        description: 'Version'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }} ${{ inputs.version }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_call with required and optional inputs should not produce errors"
    );
}

#[test]
fn test_workflow_call_inputs_valid_choice() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: choice
        description: 'Environment'
        options:
          - staging
          - production
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_call with choice input should not produce errors"
    );
}

#[test]
fn test_workflow_call_inputs_valid_boolean() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      dry_run:
        type: boolean
        description: 'Dry run mode'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ inputs.dry_run }}
        run: echo "Dry run"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_call with boolean input should not produce errors"
    );
}

#[test]
fn test_workflow_call_inputs_error_undefined() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: string
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.undefined }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input")
                || d.message.contains("undefined")
                || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !input_errors.is_empty(),
        "Reference to undefined workflow_call input should produce error"
    );
}

#[test]
fn test_workflow_call_inputs_error_invalid_type() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: invalid_type
        description: 'Environment'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input")
                || d.message.contains("type")
                || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !input_errors.is_empty(),
        "Invalid workflow_call input type should produce error"
    );
}

#[test]
fn test_workflow_call_inputs_error_no_workflow_call() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !input_errors.is_empty(),
        "Input reference without workflow_call trigger should produce error"
    );
}

#[test]
fn test_workflow_call_inputs_valid_default_value() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: string
        default: staging
        description: 'Environment'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_call input with default value should not produce errors"
    );
}
