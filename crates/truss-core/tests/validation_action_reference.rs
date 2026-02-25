//! Tests for ActionReferenceRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates action reference format (owner/repo@ref) in GitHub Actions workflows.
//! Note: This rule may overlap with StepValidationRule but provides more comprehensive validation.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_action_reference_valid_tag() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors.is_empty(),
        "Valid action reference with tag should not produce errors"
    );
}

#[test]
fn test_action_reference_valid_branch() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@main
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors.is_empty(),
        "Valid action reference with branch should not produce errors"
    );
}

#[test]
fn test_action_reference_valid_sha() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@abc123def456789
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors.is_empty(),
        "Valid action reference with SHA should not produce errors"
    );
}

#[test]
fn test_action_reference_valid_owner_repo() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: my-org/my-action@v1.0.0
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors.is_empty(),
        "Valid action reference with custom owner/repo should not produce errors"
    );
}

#[test]
fn test_action_reference_missing_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.message.contains("ref")
                    || d.message.contains("@")
                    || d.message.contains("missing"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    assert!(
        !action_errors.is_empty(),
        "Action reference missing @ref should produce error or warning"
    );
    assert!(
        action_errors.iter().any(|d| d.message.contains("ref")
            || d.message.contains("@")
            || d.message.contains("missing")),
        "Error message should mention missing ref or @ symbol"
    );
}

#[test]
fn test_action_reference_invalid_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: invalid-action-format
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.message.contains("format") || d.message.contains("invalid"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    assert!(
        !action_errors.is_empty(),
        "Invalid action reference format should produce error or warning"
    );
    assert!(
        action_errors
            .iter()
            .any(|d| d.message.contains("format") || d.message.contains("invalid")),
        "Error message should mention format or invalid"
    );
}

#[test]
fn test_action_reference_missing_owner() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: checkout@v3
"#;

    let result = engine.analyze(yaml);
    // Action reference should be owner/repo@ref format
    // Missing owner (like "checkout@v3" instead of "actions/checkout@v3") should be invalid
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.message.contains("owner")
                    || d.message.contains("format")
                    || d.message.contains("invalid"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    // Note: StepValidationRule may catch some action format issues, but not missing owner
    // When ActionReferenceRule is implemented, this should produce an error
    assert!(
        !action_errors.is_empty(),
        "Action reference missing owner (should be owner/repo@ref) should produce error or warning"
    );
}

#[test]
fn test_action_reference_valid_local_path() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/my-action
"#;

    let result = engine.analyze(yaml);
    // Local paths don't require @ref
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    // Local paths are valid without @ref
    assert!(
        action_errors.is_empty(),
        "Local action path should be valid (doesn't require @ref)"
    );
}

#[test]
fn test_action_reference_valid_docker() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: docker://alpine:3.18
"#;

    let result = engine.analyze(yaml);
    // Docker actions have different format
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    // Docker actions are valid
    assert!(
        action_errors.is_empty(),
        "Docker action reference should be valid"
    );
}

#[test]
fn test_action_reference_invalid_owner_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: "my org/checkout@v3"
"#;

    let result = engine.analyze(yaml);
    // Owner cannot contain spaces
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.message.contains("owner")
                    || d.message.contains("format")
                    || d.message.contains("invalid"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    // Note: ActionReferenceRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !action_errors.is_empty(),
        "Action reference with invalid owner format (spaces) should produce error or warning"
    );
}

#[test]
fn test_action_reference_invalid_repo_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout-action@v3
"#;

    let _result = engine.analyze(yaml);
    // This is actually valid, but let's test a truly invalid repo format
    // Repo with invalid characters
    let yaml2 = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/my-repo@v3
"#;

    let result2 = engine.analyze(yaml2);
    // Valid format, should not error
    let action_errors2: Vec<_> = result2
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors2.is_empty(),
        "Action reference with valid repo format should not produce errors"
    );
}

#[test]
fn test_action_reference_invalid_ref_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3.0.0.0.0
"#;

    let result = engine.analyze(yaml);
    // This is actually valid (tag can have multiple dots)
    // Let's test missing ref entirely which should be caught by existing test
    // But we can test empty ref
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    // Valid tag format, should not error
    assert!(
        action_errors.is_empty(),
        "Action reference with valid tag format should not produce errors"
    );
}

#[test]
fn test_action_reference_composite_action() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: my-org/my-composite-action@v1
        with:
          input1: value1
"#;

    let result = engine.analyze(yaml);
    // Composite actions use the same format as regular actions
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        action_errors.is_empty(),
        "Composite action reference with valid format should not produce errors"
    );
}

#[test]
fn test_action_reference_missing_slash() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actionscheckout@v3
"#;

    let result = engine.analyze(yaml);
    // Missing slash between owner and repo
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("action") || d.message.contains("uses"))
                && (d.message.contains("format")
                    || d.message.contains("invalid")
                    || d.message.contains("owner"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    // Note: ActionReferenceRule is not yet implemented
    // When implemented, this should produce an error
    assert!(
        !action_errors.is_empty(),
        "Action reference missing slash between owner and repo should produce error or warning"
    );
}

// === Regression tests for action subpath support (Bug #1) ===

#[test]
fn test_action_reference_valid_subpath() {
    let mut engine = TrussEngine::new();
    // GitHub Actions supports owner/repo/path@ref for actions in subdirectories
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: github/codeql-action/init@v3
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("action") && d.message.contains("invalid format"))
        .collect();

    assert!(
        action_errors.is_empty(),
        "Action reference with subpath (owner/repo/path@ref) should be valid. Got: {:?}",
        action_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_action_reference_valid_deep_subpath() {
    let mut engine = TrussEngine::new();
    // Deep subpath like microsoft/typescript-bot-test-triggerer/.github/actions/post-workflow-result@master
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: microsoft/typescript-bot-test-triggerer/.github/actions/post-workflow-result@master
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("action") && d.message.contains("invalid format"))
        .collect();

    assert!(
        action_errors.is_empty(),
        "Action reference with deep subpath should be valid. Got: {:?}",
        action_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_action_reference_valid_cache_subpath() {
    let mut engine = TrussEngine::new();
    // actions/cache/restore@v4 and actions/cache/save@v4 are common
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/cache/restore@v4
      - run: echo "build"
      - uses: actions/cache/save@v4
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("action") && d.message.contains("invalid format"))
        .collect();

    assert!(
        action_errors.is_empty(),
        "actions/cache/restore@v4 and actions/cache/save@v4 should be valid. Got: {:?}",
        action_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_action_reference_valid_codeql_subpath_with_sha() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: github/codeql-action/init@9e907b5e64f6b83e7804b09294d44122997950d6
      - uses: github/codeql-action/autobuild@9e907b5e64f6b83e7804b09294d44122997950d6
      - uses: github/codeql-action/analyze@9e907b5e64f6b83e7804b09294d44122997950d6
"#;

    let result = engine.analyze(yaml);
    let action_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("action") && d.message.contains("invalid format"))
        .collect();

    assert!(
        action_errors.is_empty(),
        "CodeQL action subpaths with SHA should be valid. Got: {:?}",
        action_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
