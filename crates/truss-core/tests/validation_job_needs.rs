//! Tests for JobNeedsRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates job dependencies (`needs:`) in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_job_needs_valid_single() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
  test:
    needs: build
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let needs_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("needs") && d.severity == Severity::Error)
        .collect();

    assert!(
        needs_errors.is_empty(),
        "Valid 'needs: build' should not produce errors"
    );
}

#[test]
fn test_job_needs_valid_multiple() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
  test:
    runs-on: ubuntu-latest
  deploy:
    needs: [build, test]
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let needs_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("needs") && d.severity == Severity::Error)
        .collect();

    assert!(
        needs_errors.is_empty(),
        "Valid 'needs: [build, test]' should not produce errors"
    );
}

#[test]
fn test_job_needs_nonexistent_job() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    needs: nonexistent
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let needs_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("needs")
                || (d.message.contains("job") && d.message.contains("nonexistent"))
        })
        .collect();

    assert!(
        !needs_errors.is_empty(),
        "Reference to non-existent job should produce error"
    );
    assert!(
        needs_errors
            .iter()
            .any(|d| d.message.contains("nonexistent")),
        "Error message should mention 'nonexistent' job"
    );
}

#[test]
fn test_job_needs_circular_dependency() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  job1:
    needs: job2
    runs-on: ubuntu-latest
  job2:
    needs: job1
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let circular_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("circular") || d.message.contains("dependency"))
        .collect();

    assert!(
        !circular_errors.is_empty(),
        "Circular dependency should produce error"
    );
    assert!(
        circular_errors
            .iter()
            .any(|d| d.message.contains("circular")),
        "Error message should mention 'circular'"
    );
}

#[test]
fn test_job_needs_self_reference() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    needs: build
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let self_ref_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("self")
                || (d.message.contains("needs") && d.message.contains("build"))
        })
        .collect();

    assert!(
        !self_ref_errors.is_empty(),
        "Self-reference should produce error"
    );
    assert!(
        self_ref_errors.iter().any(|d| d.message.contains("self")),
        "Error message should mention 'self'"
    );
}
