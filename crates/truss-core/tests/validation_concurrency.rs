//! Tests for ConcurrencyRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates concurrency syntax in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_concurrency_valid_workflow_level() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Valid workflow-level concurrency should not produce errors"
    );
}

#[test]
fn test_concurrency_valid_job_level() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    concurrency:
      group: build-${{ github.ref }}
      cancel-in-progress: false
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Valid job-level concurrency should not produce errors"
    );
}

#[test]
fn test_concurrency_valid_string_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: my-group
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Valid concurrency with string group should not produce errors"
    );
}

#[test]
fn test_concurrency_valid_expression_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Valid concurrency with expression group should not produce errors"
    );
}

#[test]
fn test_concurrency_invalid_cancel_in_progress() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci
  cancel-in-progress: "true"
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    // cancel-in-progress should be boolean, not string
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("cancel-in-progress"))
                && (d.message.contains("invalid") || d.message.contains("boolean"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: ConcurrencyRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !concurrency_errors.is_empty(),
        "Invalid cancel-in-progress value (string instead of boolean) should produce error"
    );
}

#[test]
fn test_concurrency_missing_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    // 'group' is required for concurrency
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("group"))
                && (d.message.contains("required") || d.message.contains("missing"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: ConcurrencyRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !concurrency_errors.is_empty(),
        "Concurrency missing required 'group' field should produce error"
    );
}

#[test]
fn test_concurrency_valid_no_cancel_in_progress() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Concurrency with only 'group' (cancel-in-progress is optional) should be valid"
    );
}

#[test]
fn test_concurrency_invalid_group_type() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: 123
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    // Group should be string or expression, not number
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("group"))
                && (d.message.contains("invalid") || d.message.contains("string"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: ConcurrencyRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !concurrency_errors.is_empty()
            || result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("expression")),
        "Invalid group type (number instead of string/expression) should produce error"
    );
}

#[test]
fn test_concurrency_job_level_invalid_cancel_in_progress() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    concurrency:
      group: build-ci
      cancel-in-progress: "false"
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    // cancel-in-progress should be boolean, not string
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("cancel-in-progress"))
                && (d.message.contains("invalid") || d.message.contains("boolean"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: ConcurrencyRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !concurrency_errors.is_empty(),
        "Job-level concurrency with invalid cancel-in-progress (string instead of boolean) should produce error"
    );
}

#[test]
fn test_concurrency_workflow_level_cancel_in_progress_false() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: false
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("concurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Workflow-level concurrency with cancel-in-progress: false should be valid"
    );
}

#[test]
fn test_concurrency_job_level_missing_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    concurrency:
      cancel-in-progress: true
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    // 'group' is required for concurrency at job level too
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("group"))
                && (d.message.contains("required") || d.message.contains("missing"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: ConcurrencyRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !concurrency_errors.is_empty(),
        "Job-level concurrency missing required 'group' field should produce error"
    );
}

#[test]
fn test_concurrency_invalid_float_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: 1.0
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("concurrency") || d.message.contains("group"))
                && (d.message.contains("number") || d.message.contains("string"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !concurrency_errors.is_empty(),
        "Float group value '1.0' should be flagged as invalid (not a string/expression)"
    );
}

#[test]
fn test_concurrency_valid_context_with_dot() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: github.ref
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("number")
                && d.message.contains("concurrency")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Context reference 'github.ref' should not be flagged as a number. Got: {:?}",
        concurrency_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
