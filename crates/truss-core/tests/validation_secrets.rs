//! Tests for SecretsValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates secrets.* references in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_secrets_valid_github_token() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.GITHUB_TOKEN }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("secret") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid secrets.GITHUB_TOKEN reference should not produce errors"
    );
}

#[test]
fn test_secrets_valid_custom_secret() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.MY_SECRET }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("secret") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid custom secret reference should not produce errors"
    );
}

#[test]
fn test_secrets_valid_in_env() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      API_KEY: ${{ secrets.API_KEY }}
    steps:
      - run: echo "Building"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("secret") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid secret reference in env should not produce errors"
    );
}

#[test]
fn test_secrets_valid_multiple_secrets() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "${{ secrets.USERNAME }}"
          echo "${{ secrets.PASSWORD }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("secret") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid multiple secret references should not produce errors"
    );
}

#[test]
fn test_secrets_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secret.MY_SECRET }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("syntax"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    // Should be 'secrets' (plural), not 'secret' (singular)
    assert!(
        !secret_errors.is_empty() || result.diagnostics.iter().any(|d| d.message.contains("expression")),
        "Invalid secret syntax (singular 'secret' instead of 'secrets') should produce error or expression error"
    );
}

#[test]
fn test_secrets_invalid_missing_dot() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secretsMY_SECRET }}"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("secret") || d.message.contains("syntax"))
                && (d.severity == Severity::Error || d.severity == Severity::Warning)
        })
        .collect();

    // Missing dot between 'secrets' and secret name
    assert!(
        !secret_errors.is_empty()
            || result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("expression")),
        "Invalid secret syntax (missing dot) should produce error or expression error"
    );
}

#[test]
fn test_secrets_valid_in_conditional() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: ${{ secrets.DEPLOY_ENABLED == 'true' }}
        run: echo "Deploying"
"#;

    let result = engine.analyze(yaml);
    let secret_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("secret") && d.severity == Severity::Error)
        .collect();

    assert!(
        secret_errors.is_empty(),
        "Valid secret reference in conditional should not produce errors"
    );
}
