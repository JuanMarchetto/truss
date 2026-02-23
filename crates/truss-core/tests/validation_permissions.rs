//! Tests for PermissionsRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates permissions configuration in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_permissions_valid_read_all() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions: read-all
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Valid 'read-all' permission should not produce errors"
    );
}

#[test]
fn test_permissions_valid_write_all() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions: write-all
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Valid 'write-all' permission should not produce errors"
    );
}

#[test]
fn test_permissions_valid_object() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions:
  contents: read
  issues: write
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Valid permissions object should not produce errors"
    );
}

#[test]
fn test_permissions_valid_empty() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions: {}
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Empty permissions object should be valid (no permissions granted)"
    );
}

#[test]
fn test_permissions_valid_job_level() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    permissions:
      contents: read
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Valid job-level permissions should not produce errors"
    );
}

#[test]
fn test_permissions_invalid_scope() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions:
  invalid-scope: read
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("permission")
                && (d.message.contains("invalid") || d.message.contains("scope"))
        })
        .collect();

    assert!(
        !perm_errors.is_empty(),
        "Invalid permission scope should produce error"
    );
    assert!(
        perm_errors
            .iter()
            .any(|d| d.message.contains("invalid") || d.message.contains("scope")),
        "Error message should mention 'invalid' or 'scope'"
    );
}

#[test]
fn test_permissions_invalid_value() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions:
  contents: invalid-value
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("permission")
                && (d.message.contains("invalid") || d.message.contains("value"))
        })
        .collect();

    assert!(
        !perm_errors.is_empty(),
        "Invalid permission value should produce error"
    );
    assert!(
        perm_errors.iter().any(|d| d.message.contains("read")
            || d.message.contains("write")
            || d.message.contains("none")),
        "Error message should mention valid values (read, write, none)"
    );
}

#[test]
fn test_permissions_valid_none() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
permissions:
  contents: none
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    let perm_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("permission") && d.severity == Severity::Error)
        .collect();

    assert!(
        perm_errors.is_empty(),
        "Valid 'none' permission value should not produce errors"
    );
}
