//! Tests for JobStrategyValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates strategy field syntax and constraints (max-parallel, fail-fast) in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_job_strategy_valid_max_parallel() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      max-parallel: 2
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("max-parallel"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        strategy_errors.is_empty(),
        "Valid max-parallel value should not produce errors"
    );
}

#[test]
fn test_job_strategy_valid_fail_fast() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      fail-fast: true
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("fail-fast"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        strategy_errors.is_empty(),
        "Valid fail-fast boolean value should not produce errors"
    );
}

#[test]
fn test_job_strategy_valid_both_fields() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      max-parallel: 2
      fail-fast: false
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy")
                || d.message.contains("max-parallel")
                || d.message.contains("fail-fast"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        strategy_errors.is_empty(),
        "Valid strategy with both max-parallel and fail-fast should not produce errors"
    );
}

#[test]
fn test_job_strategy_error_max_parallel_negative() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      max-parallel: -1
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("max-parallel"))
                && (d.message.contains("negative")
                    || d.message.contains("positive")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !strategy_errors.is_empty(),
        "Negative max-parallel value should produce error"
    );
}

#[test]
fn test_job_strategy_error_max_parallel_zero() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      max-parallel: 0
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("max-parallel"))
                && (d.message.contains("zero")
                    || d.message.contains("positive")
                    || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !strategy_errors.is_empty(),
        "Zero max-parallel value should produce error"
    );
}

#[test]
fn test_job_strategy_error_max_parallel_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      max-parallel: "3"
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("max-parallel"))
                && (d.message.contains("string")
                    || d.message.contains("number")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !strategy_errors.is_empty(),
        "String max-parallel value should produce error"
    );
}

#[test]
fn test_job_strategy_error_fail_fast_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      fail-fast: "true"
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("fail-fast"))
                && (d.message.contains("string")
                    || d.message.contains("boolean")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !strategy_errors.is_empty(),
        "String fail-fast value should produce error"
    );
}

#[test]
fn test_job_strategy_error_fail_fast_number() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
      fail-fast: 1
"#;

    let result = engine.analyze(yaml);
    let strategy_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("strategy") || d.message.contains("fail-fast"))
                && (d.message.contains("number")
                    || d.message.contains("boolean")
                    || d.message.contains("type"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !strategy_errors.is_empty(),
        "Number fail-fast value should produce error"
    );
}
