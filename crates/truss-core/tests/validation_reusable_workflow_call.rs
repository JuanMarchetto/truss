//! Tests for ReusableWorkflowCallRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates uses: workflow calls reference valid reusable workflows in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_reusable_workflow_call_valid_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo/.github/workflows/reusable.yml@main
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        workflow_errors.is_empty(),
        "Valid reusable workflow call format should not produce errors"
    );
}

#[test]
fn test_reusable_workflow_call_valid_with_inputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo/.github/workflows/reusable.yml@main
    with:
      environment: production
      version: 1.0.0
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        workflow_errors.is_empty(),
        "Valid reusable workflow call with inputs should not produce errors"
    );
}

#[test]
fn test_reusable_workflow_call_valid_with_secrets() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo/.github/workflows/reusable.yml@main
    secrets:
      DEPLOY_KEY: ${{ secrets.DEPLOY_KEY }}
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        workflow_errors.is_empty(),
        "Valid reusable workflow call with secrets should not produce errors"
    );
}

#[test]
fn test_reusable_workflow_call_valid_with_strategy() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo/.github/workflows/reusable.yml@main
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        workflow_errors.is_empty(),
        "Valid reusable workflow call with strategy should not produce errors"
    );
}

#[test]
fn test_reusable_workflow_call_error_invalid_format_missing_path() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo@main
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid")
                    || d.message.contains("format")
                    || d.message.contains("path"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !workflow_errors.is_empty(),
        "Reusable workflow call with invalid format (missing path) should produce error"
    );
}

#[test]
fn test_reusable_workflow_call_error_invalid_format_missing_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: owner/repo/.github/workflows/reusable.yml
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid")
                    || d.message.contains("format")
                    || d.message.contains("@"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !workflow_errors.is_empty(),
        "Reusable workflow call with invalid format (missing @ref) should produce error"
    );
}

#[test]
fn test_reusable_workflow_call_valid_local_path() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-workflow:
    uses: ./.github/workflows/reusable.yml@main
"#;

    let result = engine.analyze(yaml);
    let workflow_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("workflow") || d.message.contains("uses"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        workflow_errors.is_empty(),
        "Valid local reusable workflow call should not produce errors"
    );
}
