//! Tests for StepIdUniquenessRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates that step IDs are unique within a job in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_id_uniqueness_valid_unique_ids() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: checkout
        uses: actions/checkout@v3
      - id: build
        run: echo "Building"
      - id: test
        run: echo "Testing"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        step_id_errors.is_empty(),
        "Unique step IDs within a job should not produce errors"
    );
}

#[test]
fn test_step_id_uniqueness_valid_different_jobs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: step1
        run: echo "Build step"
  test:
    runs-on: ubuntu-latest
    steps:
      - id: step1
        run: echo "Test step"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        step_id_errors.is_empty(),
        "Same step IDs in different jobs should be valid"
    );
}

#[test]
fn test_step_id_uniqueness_error_duplicate_in_same_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        run: echo "Build step 1"
      - id: build
        run: echo "Build step 2"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate")
                    || d.message.contains("unique")
                    || d.message.contains("build"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !step_id_errors.is_empty(),
        "Duplicate step ID in same job should produce error"
    );
    assert!(
        step_id_errors
            .iter()
            .any(|d| d.message.contains("duplicate") || d.message.contains("build")),
        "Error message should mention duplicate step ID"
    );
}

#[test]
fn test_step_id_uniqueness_error_multiple_duplicates() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: step1
        run: echo "Step 1"
      - id: step2
        run: echo "Step 2"
      - id: step1
        run: echo "Step 1 duplicate"
      - id: step2
        run: echo "Step 2 duplicate"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !step_id_errors.is_empty(),
        "Multiple duplicate step IDs should produce errors"
    );
}

#[test]
fn test_step_id_uniqueness_error_different_step_types() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: build
        uses: actions/checkout@v3
      - id: build
        run: echo "Build"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !step_id_errors.is_empty(),
        "Duplicate step ID with different step types should produce error"
    );
}

#[test]
fn test_step_id_uniqueness_valid_no_ids() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Step 1"
      - run: echo "Step 2"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        step_id_errors.is_empty(),
        "Steps without IDs should not produce uniqueness errors"
    );
}

#[test]
fn test_step_id_uniqueness_valid_mixed_with_and_without_ids() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - id: checkout
        uses: actions/checkout@v3
      - run: echo "Step without ID"
      - id: build
        run: echo "Build"
"#;

    let result = engine.analyze(yaml);
    let step_id_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("step") || d.message.contains("id"))
                && (d.message.contains("duplicate") || d.message.contains("unique"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        step_id_errors.is_empty(),
        "Mixed steps with and without IDs should be valid if IDs are unique"
    );
}
