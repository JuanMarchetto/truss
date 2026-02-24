//! Tests for WorkflowTriggerRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates `on:` trigger configuration in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_workflow_trigger_valid_push() {
    let mut engine = TrussEngine::new();
    let yaml = "on: push";

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Valid 'on: push' should not produce errors"
    );
}

#[test]
fn test_workflow_trigger_valid_multiple_events() {
    let mut engine = TrussEngine::new();
    let yaml = "on: [push, pull_request]";

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Valid multiple events should not produce errors"
    );
}

#[test]
fn test_workflow_trigger_valid_with_branches() {
    let mut engine = TrussEngine::new();
    let yaml = "on:\n  push:\n    branches: [main]";

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Valid 'on' with branches should not produce errors"
    );
}

#[test]
fn test_workflow_trigger_missing_on_field() {
    let mut engine = TrussEngine::new();
    let yaml = "name: CI\njobs:\n  build:\n    runs-on: ubuntu-latest";

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
        .collect();

    assert!(
        !trigger_errors.is_empty(),
        "Missing 'on' field should produce error"
    );
    assert!(
        trigger_errors.iter().any(|d| d.message.contains("on")),
        "Error message should mention 'on' field"
    );
}

#[test]
fn test_workflow_trigger_invalid_event_type() {
    let mut engine = TrussEngine::new();
    let yaml = "on: invalid_event_type";

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("trigger")
                || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !trigger_errors.is_empty(),
        "Invalid event type should produce error"
    );
}

#[test]
fn test_workflow_trigger_valid_scalar_fork() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: fork
jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Forked"
"#;

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("event") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Scalar 'on: fork' should be a valid event type. Got errors: {:?}",
        trigger_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_workflow_trigger_valid_scalar_create() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: create
jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Branch created"
"#;

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("event") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Scalar 'on: create' should be a valid event type. Got errors: {:?}",
        trigger_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_workflow_trigger_valid_scalar_release() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: release
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Released"
"#;

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("event") && d.severity == Severity::Error)
        .collect();

    assert!(
        trigger_errors.is_empty(),
        "Scalar 'on: release' should be a valid event type. Got errors: {:?}",
        trigger_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_workflow_trigger_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = "on: [push, ]"; // Invalid syntax

    let result = engine.analyze(yaml);
    let trigger_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("on")
                || d.message.contains("trigger")
                || d.message.contains("syntax"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !trigger_errors.is_empty(),
        "Invalid trigger syntax should produce error"
    );
}
