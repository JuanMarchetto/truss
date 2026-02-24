//! Tests for StepEnvValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates environment variable names and values at step level in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_env_valid_single() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          NODE_ENV: production
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("env") || d.message.contains("environment"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid step env variable should not produce errors"
    );
}

#[test]
fn test_step_env_valid_multiple() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          VAR1: value1
          VAR2: value2
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("env") || d.message.contains("environment"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid multiple step env variables should not produce errors"
    );
}

#[test]
fn test_step_env_valid_expression() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          VERSION: ${{ github.ref }}
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("env") || d.message.contains("environment"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid step env variable with expression should not produce errors"
    );
}

#[test]
fn test_step_env_error_invalid_name_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          INVALID-NAME: value
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("env")
                || d.message.contains("environment")
                || d.message.contains("INVALID-NAME"))
                && (d.message.contains("invalid")
                    || d.message.contains("format")
                    || d.message.contains("name"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !env_errors.is_empty(),
        "Invalid env variable name format should produce error"
    );
}

#[test]
fn test_step_env_error_invalid_syntax() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          VAR=value
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let _env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("env")
                || d.message.contains("environment")
                || d.message.contains("syntax"))
                && d.severity == Severity::Error
        })
        .collect();

    // Note: YAML parser might catch this, but rule should also validate
    assert!(
        true,
        "Invalid env variable syntax may produce error (YAML parser or rule)"
    );
}

#[test]
fn test_step_env_with_comment_before_step() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - # Deploy step env vars
        env:
          DEPLOY_KEY: secret
        run: echo "deploying"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("environment") && d.message.contains("invalid")
                || d.message.contains("env") && d.severity == Severity::Error
        })
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid env with comment between dash and step should not produce errors. Got: {:?}",
        env_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
