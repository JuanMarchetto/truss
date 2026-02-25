//! Tests for WorkflowInputsRule
//!
//! **Status:** Rule implemented and tested
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

// === Regression tests for inputs without type field (Bug #2) ===

#[test]
fn test_workflow_inputs_without_type_field() {
    let mut engine = TrussEngine::new();
    // Inputs without `type` field default to string in GitHub Actions
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      clone_url:
        description: 'Git url of a fork'
        required: true
      branch_name:
        description: 'Name of the feature branch'
        required: true
      commit_hash:
        description: 'Optional commit hash'
        required: false
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.clone_url }} ${{ inputs.branch_name }} ${{ inputs.commit_hash }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("undefined") && d.message.contains("input"))
        .collect();

    assert!(
        input_errors.is_empty(),
        "Inputs without explicit type field should be recognized (default to string). Got: {:?}",
        input_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_workflow_inputs_mixed_typed_and_untyped() {
    let mut engine = TrussEngine::new();
    // Mix of inputs with and without type field
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      pr:
        description: 'PR number'
        required: true
        type: number
      target_branch:
        description: 'Target branch'
        required: true
        type: string
      distinct_id:
        description: 'A distinct ID'
        required: false
        default: ''
      source_issue:
        description: 'The issue that triggered this'
        required: false
        default: ''
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ inputs.pr }} ${{ inputs.target_branch }} ${{ inputs.distinct_id }} ${{ inputs.source_issue }}"
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("undefined") && d.message.contains("input"))
        .collect();

    assert!(
        input_errors.is_empty(),
        "Mix of typed and untyped inputs should all be recognized. Got: {:?}",
        input_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_workflow_inputs_github_event_inputs_reference() {
    let mut engine = TrussEngine::new();
    // github.event.inputs.X is an alternative way to reference inputs
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      clone_url:
        description: 'Git url'
        required: true
      branch_name:
        description: 'Branch name'
        required: true
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - run: bash check.sh ${{ github.event.inputs.clone_url }} ${{ github.event.inputs.branch_name }}
"#;

    let result = engine.analyze(yaml);
    let input_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("undefined") && d.message.contains("input"))
        .collect();

    assert!(
        input_errors.is_empty(),
        "github.event.inputs.X references should be recognized for untyped inputs. Got: {:?}",
        input_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
