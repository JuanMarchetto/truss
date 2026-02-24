//! Tests for DeprecatedCommandsRule
//!
//! **Status:** Rule implemented and tested
//!
//! Detects deprecated workflow commands in run scripts.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_deprecated_set_output() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "::set-output name=value::result"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command")
                && d.message.contains("set-output")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !deprecated_warnings.is_empty(),
        "Deprecated ::set-output command should produce warning. Got diagnostics: {:?}",
        result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("GITHUB_OUTPUT")),
        "Warning should suggest using GITHUB_OUTPUT instead"
    );
}

#[test]
fn test_deprecated_save_state() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "::save-state name=value::result"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command")
                && d.message.contains("save-state")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !deprecated_warnings.is_empty(),
        "Deprecated ::save-state command should produce warning. Got diagnostics: {:?}",
        result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("GITHUB_STATE")),
        "Warning should suggest using GITHUB_STATE instead"
    );
}

#[test]
fn test_deprecated_set_env() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "::set-env name=MY_VAR::my_value"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command")
                && d.message.contains("set-env")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !deprecated_warnings.is_empty(),
        "Deprecated ::set-env command should produce warning. Got diagnostics: {:?}",
        result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("GITHUB_ENV")),
        "Warning should suggest using GITHUB_ENV instead"
    );
}

#[test]
fn test_deprecated_add_path() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "::add-path::/usr/local/bin"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command")
                && d.message.contains("add-path")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !deprecated_warnings.is_empty(),
        "Deprecated ::add-path command should produce warning. Got diagnostics: {:?}",
        result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("GITHUB_PATH")),
        "Warning should suggest using GITHUB_PATH instead"
    );
}

#[test]
fn test_no_deprecated_commands_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "value" >> $GITHUB_OUTPUT
      - run: echo "MY_VAR=my_value" >> $GITHUB_ENV
      - run: echo "/usr/local/bin" >> $GITHUB_PATH
      - run: echo "state_value" >> $GITHUB_STATE
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command") && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        deprecated_warnings.is_empty(),
        "Modern workflow using GITHUB_OUTPUT/ENV/PATH/STATE should not produce deprecated warnings. Got: {:?}",
        deprecated_warnings.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_deprecated_command_in_multiline_run() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "Building project"
          echo "::set-output name=result::success"
          echo "Done"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command")
                && d.message.contains("set-output")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !deprecated_warnings.is_empty(),
        "Deprecated command in multiline run block should produce warning. Got diagnostics: {:?}",
        result.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_multiple_deprecated_commands() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo "::set-output name=foo::bar"
          echo "::save-state name=baz::qux"
          echo "::set-env name=MY_VAR::my_value"
          echo "::add-path::/usr/local/bin"
"#;

    let result = engine.analyze(yaml);
    let deprecated_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Deprecated workflow command") && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        deprecated_warnings.len() >= 4,
        "Multiple deprecated commands should each produce a warning. Expected >= 4, got {}. Warnings: {:?}",
        deprecated_warnings.len(),
        deprecated_warnings.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("set-output")),
        "Should warn about set-output"
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("save-state")),
        "Should warn about save-state"
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("set-env")),
        "Should warn about set-env"
    );
    assert!(
        deprecated_warnings
            .iter()
            .any(|d| d.message.contains("add-path")),
        "Should warn about add-path"
    );
}
