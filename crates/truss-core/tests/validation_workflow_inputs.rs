//! Tests for WorkflowInputsRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates workflow_dispatch inputs in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_workflow_inputs_valid_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
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
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_dispatch with string input should not produce errors"
    );
}

#[test]
fn test_workflow_inputs_valid_choice() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: choice
        description: 'Environment to deploy to'
        options:
          - staging
          - production
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
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_dispatch with choice input should not produce errors"
    );
}

#[test]
fn test_workflow_inputs_valid_boolean() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      dry_run:
        type: boolean
        description: 'Run in dry-run mode'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ inputs.dry_run }}
        run: echo "Dry run mode"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_dispatch with boolean input should not produce errors"
    );
}

#[test]
fn test_workflow_inputs_valid_environment() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: environment
        description: 'Environment to deploy to'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_dispatch with environment input should not produce errors"
    );
}

#[test]
fn test_workflow_inputs_reference_undefined() {
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
      - run: echo "Deploying to ${{ inputs.undefined_input }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("undefined"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !input_errors.is_empty(),
        "Reference to undefined input should produce error"
    );
    assert!(
        input_errors
            .iter()
            .any(|d| d.message.contains("undefined_input") || d.message.contains("undefined")),
        "Error message should mention undefined input"
    );
}

#[test]
fn test_workflow_inputs_invalid_type() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: invalid_type
        description: 'Environment to deploy to'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("input") || d.message.contains("type"))
                && (d.message.contains("invalid") || d.message.contains("invalid_type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !input_errors.is_empty(),
        "Invalid input type should produce error"
    );
}

#[test]
fn test_workflow_inputs_valid_no_inputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "workflow_dispatch without inputs should be valid"
    );
}

#[test]
fn test_workflow_inputs_valid_multiple_inputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: string
        description: 'Environment'
      version:
        type: string
        description: 'Version to deploy'
      dry_run:
        type: boolean
        description: 'Dry run mode'
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying ${{ inputs.version }} to ${{ inputs.environment }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("input") && d.severity == Severity::Error)
        .collect();

    assert!(
        input_errors.is_empty(),
        "Valid workflow_dispatch with multiple inputs should not produce errors"
    );
}
