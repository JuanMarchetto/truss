//! Tests for EnvironmentRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates environment references in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_environment_valid_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: production
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid environment string should not produce errors"
    );
}

#[test]
fn test_environment_valid_object() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: production
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid environment object should not produce errors"
    );
}

#[test]
fn test_environment_valid_with_url() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: production
      url: https://example.com
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    assert!(
        env_errors.is_empty(),
        "Valid environment with URL should not produce errors"
    );
}

#[test]
fn test_environment_valid_workflow_level() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
env:
  NODE_ENV: production
jobs:
  test:
    runs-on: ubuntu-latest
"#;

    let result = engine.analyze(yaml);
    // This is workflow-level env (environment variables), not environment protection
    // Should not produce errors
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    assert!(
        env_errors.is_empty(),
        "Workflow-level env variables should not produce environment errors"
    );
}

#[test]
fn test_environment_invalid_name_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: "invalid name with spaces"
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("environment")
                && (d.message.contains("name") || d.message.contains("format"))
        })
        .collect();

    assert!(
        !env_errors.is_empty(),
        "Invalid environment name format should produce error"
    );
}

#[test]
fn test_environment_valid_step_level() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying"
        env:
          DEPLOY_ENV: production
"#;

    let result = engine.analyze(yaml);
    // Step-level env is environment variables, not environment protection
    // Should not produce errors
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    assert!(
        env_errors.is_empty(),
        "Step-level env variables should not produce environment errors"
    );
}

#[test]
fn test_environment_valid_with_protection_rules() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: production
      protection_rules:
        - type: required_reviewers
"#;

    let result = engine.analyze(yaml);
    let env_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("environment") && d.severity == Severity::Error)
        .collect();

    // If protection_rules is considered invalid in workflow YAML, we expect an error
    assert!(
        !env_errors.is_empty(),
        "Environment protection rules should be validated"
    );
}
