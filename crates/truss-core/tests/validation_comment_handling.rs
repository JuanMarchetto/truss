//! Regression tests for YAML comment handling in tree-sitter-yaml.
//!
//! tree-sitter-yaml inserts `comment` nodes as children of any AST node.
//! When YAML has comments between a key and its value, naive `node.child(N)`
//! access can return a comment node instead of the actual content node.
//! These tests ensure our validation rules handle this correctly.

use truss_core::{Severity, TrussEngine};

// ---------------------------------------------------------------------------
// Concurrency: comments before group key
// ---------------------------------------------------------------------------

#[test]
fn comment_before_concurrency_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
concurrency:
  # This comment is between concurrency: and group:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("oncurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Concurrency with comment before group should not produce errors. Got: {:?}",
        concurrency_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn multiple_comments_before_concurrency_group() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
concurrency:
  # First comment
  # Second comment
  # Third comment
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
"#;

    let result = engine.analyze(yaml);
    let concurrency_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("oncurrency") && d.severity == Severity::Error)
        .collect();

    assert!(
        concurrency_errors.is_empty(),
        "Multiple comments before concurrency group should not cause errors. Got: {:?}",
        concurrency_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Matrix: comments between key and value
// ---------------------------------------------------------------------------

#[test]
fn comment_before_matrix_value() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        config:
          # Main builds
          - os: ubuntu-latest
            node-version: '20'
          - os: windows-latest
            node-version: '20'
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let matrix_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("atrix") && d.severity == Severity::Error)
        .collect();

    assert!(
        matrix_errors.is_empty(),
        "Matrix with comment before value array should not produce errors. Got: {:?}",
        matrix_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Schedule: comments between cron entries
// ---------------------------------------------------------------------------

#[test]
fn comment_between_schedule_cron_entries() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: Scheduled
on:
  schedule:
    # Run daily at midnight
    - cron: '0 0 * * *'
    # Run weekly on Sundays
    - cron: '0 0 * * 0'
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "scheduled"
"#;

    let result = engine.analyze(yaml);
    let schedule_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("chedule") && d.severity == Severity::Error)
        .collect();

    assert!(
        schedule_errors.is_empty(),
        "Schedule with comments between cron entries should not produce errors. Got: {:?}",
        schedule_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn comment_heavy_schedule() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: Scheduled
on:
  schedule:
    # These are generated/confirmed using https://crontab.cronhub.io/
    # Close stale issues, runs every day at 1am
    - cron: '0 1 * * *'
    # Identify stale pull requests, every Friday at 5am
    - cron: '0 5 * * FRI'
    # Close stale pull requests, every 6 hours
    - cron: '30 2,8,14,20 * * *'
    # Identify stale issues
    - cron: '30 3,9,15,21 * * *'
# Top-level permissions
permissions: {}
jobs:
  handler:
    runs-on: ubuntu-latest
    steps:
      - run: echo "handle"
"#;

    let result = engine.analyze(yaml);
    let schedule_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("chedule") && d.severity == Severity::Error)
        .collect();

    assert!(
        schedule_errors.is_empty(),
        "Comment-heavy schedule should not produce errors. Got: {:?}",
        schedule_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Expressions in YAML comments should not be validated
// ---------------------------------------------------------------------------

#[test]
fn expression_in_yaml_comment_not_validated() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    # This comment has an expression: ${{ invalid.context }}
    steps:
      - run: echo "hello"
"#;

    let result = engine.analyze(yaml);
    let expression_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("invalid") && d.message.contains("expression"))
        .collect();

    assert!(
        expression_errors.is_empty(),
        "Expressions in YAML comments should not be validated. Got: {:?}",
        expression_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn commented_out_code_not_validated() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "hello"
      # - if: ${{ nonexistent.context }}
      #   run: echo "disabled"
"#;

    let result = engine.analyze(yaml);
    let expression_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("nonexistent") || d.message.contains("invalid"))
        .collect();

    assert!(
        expression_errors.is_empty(),
        "Commented-out YAML code should not be validated. Got: {:?}",
        expression_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// General: comments within job definitions
// ---------------------------------------------------------------------------

#[test]
fn comment_between_job_key_and_body() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    # This job builds the project
    runs-on: ubuntu-latest
    steps:
      - run: echo "build"
  test:
    # This job runs tests
    # It depends on the build job
    needs: build
    runs-on: ubuntu-latest
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    assert!(
        result.is_ok(),
        "Jobs with comments between key and body should validate cleanly. Errors: {:?}",
        result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
