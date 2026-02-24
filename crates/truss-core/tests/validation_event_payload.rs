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

// ===== Filter conflict detection tests =====

#[test]
fn test_push_branches_and_branches_ignore_conflict() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  push:
    branches:
      - main
    branches-ignore:
      - develop
"#;

    let result = engine.analyze(yaml);
    let conflict_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("mutually exclusive")
                && d.message.contains("Cannot use both")
                && d.message.contains("branches")
                && d.message.contains("branches-ignore")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !conflict_errors.is_empty(),
        "Using both 'branches' and 'branches-ignore' on push should produce a mutually exclusive error"
    );
}

#[test]
fn test_push_paths_and_paths_ignore_conflict() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  push:
    paths:
      - src/**
    paths-ignore:
      - docs/**
"#;

    let result = engine.analyze(yaml);
    let conflict_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("mutually exclusive")
                && d.message.contains("Cannot use both")
                && d.message.contains("paths")
                && d.message.contains("paths-ignore")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !conflict_errors.is_empty(),
        "Using both 'paths' and 'paths-ignore' on push should produce a mutually exclusive error"
    );
}

#[test]
fn test_push_tags_and_tags_ignore_conflict() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  push:
    tags:
      - v*
    tags-ignore:
      - v0.*
"#;

    let result = engine.analyze(yaml);
    let conflict_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("mutually exclusive")
                && d.message.contains("Cannot use both")
                && d.message.contains("tags")
                && d.message.contains("tags-ignore")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !conflict_errors.is_empty(),
        "Using both 'tags' and 'tags-ignore' on push should produce a mutually exclusive error"
    );
}

#[test]
fn test_pr_branches_and_branches_ignore_conflict() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    branches:
      - main
    branches-ignore:
      - feature/*
"#;

    let result = engine.analyze(yaml);
    let conflict_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("mutually exclusive")
                && d.message.contains("Cannot use both")
                && d.message.contains("branches")
                && d.message.contains("branches-ignore")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !conflict_errors.is_empty(),
        "Using both 'branches' and 'branches-ignore' on pull_request should produce a mutually exclusive error"
    );
}

#[test]
fn test_pr_paths_and_paths_ignore_conflict() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    paths:
      - src/**
    paths-ignore:
      - test/**
"#;

    let result = engine.analyze(yaml);
    let conflict_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("mutually exclusive")
                && d.message.contains("Cannot use both")
                && d.message.contains("paths")
                && d.message.contains("paths-ignore")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !conflict_errors.is_empty(),
        "Using both 'paths' and 'paths-ignore' on pull_request should produce a mutually exclusive error"
    );
}

// ===== Enhanced cron field range validation tests =====

#[test]
fn test_cron_minute_out_of_range() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  schedule:
    - cron: '60 0 * * *'
"#;

    let result = engine.analyze(yaml);
    let range_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("out of range")
                && d.message.contains("minute")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !range_errors.is_empty(),
        "Cron minute value 60 should produce an out of range error"
    );
}

#[test]
fn test_cron_hour_out_of_range() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  schedule:
    - cron: '0 25 * * *'
"#;

    let result = engine.analyze(yaml);
    let range_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("out of range")
                && d.message.contains("hour")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !range_errors.is_empty(),
        "Cron hour value 25 should produce an out of range error"
    );
}

#[test]
fn test_cron_valid_ranges() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  schedule:
    - cron: '30 2 15 6 3'
"#;

    let result = engine.analyze(yaml);
    let cron_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("cron") && d.severity == Severity::Error)
        .collect();

    assert!(
        cron_errors.is_empty(),
        "Valid cron expression '30 2 15 6 3' should not produce any cron errors, but got: {:?}",
        cron_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_cron_valid_wildcards() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  schedule:
    - cron: '*/15 * * * *'
"#;

    let result = engine.analyze(yaml);
    let cron_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("cron") && d.severity == Severity::Error)
        .collect();

    assert!(
        cron_errors.is_empty(),
        "Valid cron expression '*/15 * * * *' should not produce any cron errors, but got: {:?}",
        cron_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ===== Missing PR activity types now valid tests =====

#[test]
fn test_pr_type_edited_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    types: [edited]
"#;

    let result = engine.analyze(yaml);
    let type_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Invalid pull_request type")
                && d.message.contains("edited")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        type_errors.is_empty(),
        "'edited' should be a valid pull_request activity type, but got errors: {:?}",
        type_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_pr_type_ready_for_review_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  pull_request:
    types: [ready_for_review]
"#;

    let result = engine.analyze(yaml);
    let type_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Invalid pull_request type")
                && d.message.contains("ready_for_review")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        type_errors.is_empty(),
        "'ready_for_review' should be a valid pull_request activity type, but got errors: {:?}",
        type_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
