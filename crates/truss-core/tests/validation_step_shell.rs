//! Tests for StepShellRule
//!
//! **Status:** Rule implemented and tested
//!
//! Validates shell field values in GitHub Actions workflows.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_step_shell_valid_bash() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: bash
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid shell: bash should not produce errors"
    );
}

#[test]
fn test_step_shell_valid_pwsh() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: windows-latest
    steps:
      - shell: pwsh
        run: Write-Host "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid shell: pwsh should not produce errors"
    );
}

#[test]
fn test_step_shell_valid_python() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: python
        run: print("Test")
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid shell: python should not produce errors"
    );
}

#[test]
fn test_step_shell_valid_sh() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: sh
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid shell: sh should not produce errors"
    );
}

#[test]
fn test_step_shell_valid_cmd() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: windows-latest
    steps:
      - shell: cmd
        run: echo Test
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid shell: cmd should not produce errors"
    );
}

#[test]
fn test_step_shell_valid_custom() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: /usr/bin/zsh {0}
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        shell_errors.is_empty(),
        "Valid custom shell should not produce errors"
    );
}

#[test]
fn test_step_shell_error_invalid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: invalid-shell
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && (d.message.contains("invalid") || d.message.contains("unknown"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !shell_errors.is_empty(),
        "Invalid shell value should produce error"
    );
}

#[test]
fn test_step_shell_error_empty() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - shell: ""
        run: echo "Test"
"#;

    let result = engine.analyze(yaml);
    let shell_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("shell") || d.message.contains("step"))
                && (d.message.contains("empty") || d.message.contains("invalid"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !shell_errors.is_empty(),
        "Empty shell value should produce error"
    );
}
