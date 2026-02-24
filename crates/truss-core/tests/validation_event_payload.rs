//! Tests for EventPayloadValidationRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates event-specific fields in on: triggers in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_event_payload_valid_push_branches() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  push:
    branches:
      - main
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("push")
                || d.message.contains("trigger"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        event_errors.is_empty(),
        "Valid push event with branches filter should not produce errors"
    );
}

#[test]
fn test_event_payload_valid_pull_request_types() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    types: [opened, synchronize, closed]
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("pull_request")
                || d.message.contains("trigger"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        event_errors.is_empty(),
        "Valid pull_request event with types filter should not produce errors"
    );
}

#[test]
fn test_event_payload_valid_workflow_dispatch_inputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_dispatch:
    inputs:
      environment:
        type: string
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("workflow_dispatch")
                || d.message.contains("trigger"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        event_errors.is_empty(),
        "Valid workflow_dispatch event with inputs should not produce errors"
    );
}

#[test]
fn test_event_payload_error_invalid_field_for_push() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  push:
    branches:
      - main
    tags:
      - v*
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            let msg_lower = d.message.to_lowercase();
            (d.message.contains("event")
                || d.message.contains("push")
                || d.message.contains("tags")
                || d.message.contains("trigger"))
                && (msg_lower.contains("invalid") || msg_lower.contains("not valid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !event_errors.is_empty(),
        "Invalid field for push event (tags) should produce error"
    );
}

#[test]
fn test_event_payload_error_invalid_pr_type() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    types: [opened, invalid_type]
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("pull_request")
                || d.message.contains("type")
                || d.message.contains("trigger"))
                && (d.message.contains("invalid") || d.message.contains("invalid_type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !event_errors.is_empty(),
        "Invalid pull_request event type should produce error"
    );
}

#[test]
fn test_event_payload_error_invalid_activity_type() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  issues:
    types: [opened, invalid_activity]
"#;

    let result = engine.analyze(yaml);
    let event_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("event")
                || d.message.contains("issues")
                || d.message.contains("type")
                || d.message.contains("trigger"))
                && (d.message.contains("invalid") || d.message.contains("invalid_activity"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !event_errors.is_empty(),
        "Invalid activity type for event should produce error"
    );
}
