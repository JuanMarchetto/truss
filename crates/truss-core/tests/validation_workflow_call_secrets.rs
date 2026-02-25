//! Tests for WorkflowCallSecretsRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates workflow_call secrets and their usage in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_workflow_call_secrets_valid_with_usage() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      DEPLOY_KEY:
        required: true
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.DEPLOY_KEY }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid workflow_call with secret and usage should not produce errors"
    );
}

#[test]
fn test_workflow_call_secrets_valid_required_and_optional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      DEPLOY_KEY:
        required: true
      API_TOKEN:
        required: false
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.DEPLOY_KEY }}"
      - run: echo "${{ secrets.API_TOKEN }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid workflow_call with required and optional secrets should not produce errors"
    );
}

#[test]
fn test_workflow_call_secrets_valid_in_step_env() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      DEPLOY_KEY:
        required: true
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - env:
          KEY: ${{ secrets.DEPLOY_KEY }}
        run: echo "$KEY"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid workflow_call secret reference in step env should not produce errors"
    );
}

#[test]
fn test_workflow_call_secrets_error_undefined() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      DEPLOY_KEY:
        required: true
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.UNDEFINED }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret")
                || d.message.contains("undefined")
                || d.message.contains("UNDEFINED"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !secret_errors.is_empty(),
        "Reference to undefined workflow_call secret should produce error"
    );
}

#[test]
fn test_workflow_call_secrets_error_no_definition() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.DEPLOY_KEY }}"
"#;

    let result = engine.analyze(yaml);
    // When workflow_call has no secrets section, the caller may use `secrets: inherit`
    // to pass all secrets through — so we don't flag secret references as errors.
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("no secrets defined") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Secret references without secrets section should be valid (caller may use secrets: inherit), got: {:?}",
        secret_errors
    );
}

#[test]
fn test_workflow_call_secrets_valid_no_secrets() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploy"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("workflow_call"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        secret_errors.is_empty(),
        "workflow_call without secrets should be valid"
    );
}
