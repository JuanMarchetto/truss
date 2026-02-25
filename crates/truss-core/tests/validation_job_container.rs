//! Tests for JobContainerRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates container and services configurations in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_job_container_valid_image() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: node:18
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("image"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Valid container with image should not produce errors"
    );
}

#[test]
fn test_job_container_valid_with_ports() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: node:18
      ports:
        - 80:8080
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("ports"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Valid container with ports should not produce errors"
    );
}

#[test]
fn test_job_container_valid_with_env() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: node:18
      env:
        NODE_ENV: production
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("env"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Valid container with env variables should not produce errors"
    );
}

#[test]
fn test_job_container_valid_services() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("services"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Valid services configuration should not produce errors"
    );
}

#[test]
fn test_job_container_error_invalid_port_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: node:18
      ports:
        - invalid-port-format
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("ports"))
                && (d.message.contains("invalid") || d.message.contains("format"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !container_errors.is_empty(),
        "Invalid port mapping format should produce error"
    );
}

#[test]
fn test_job_container_error_invalid_image_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: ""
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("image"))
                && (d.message.contains("invalid") || d.message.contains("empty"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !container_errors.is_empty(),
        "Invalid container image format should produce error"
    );
}

// === Regression tests for container as string/expression (Bug #4) ===

#[test]
fn test_job_container_valid_string_image() {
    let mut engine = TrussEngine::new();
    // Container can be a plain string (the image name)
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container: node:18
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("image"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Container as plain string (node:18) should be valid. Got: {:?}",
        container_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_job_container_valid_expression() {
    let mut engine = TrussEngine::new();
    // Container from matrix expression
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        container: [ubuntu:20.04, ubuntu:22.04]
    container: ${{ matrix.container }}
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("container")
                && d.message.contains("missing")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Container as expression (${{ matrix.container }}) should be valid. Got: {:?}",
        container_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_job_container_valid_registry_string() {
    let mut engine = TrussEngine::new();
    // Container as full registry path string
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    container: quay.io/pypa/manylinux_2_28_x86_64
    steps:
      - run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let container_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("container") || d.message.contains("image"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        container_errors.is_empty(),
        "Container as registry path string should be valid. Got: {:?}",
        container_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
