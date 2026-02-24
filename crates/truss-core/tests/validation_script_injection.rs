//! Tests for ScriptInjectionRule
//!
//! **Status:** Rule implemented and tested
//!
//! Detects potential script injection vulnerabilities in run scripts.

use truss_core::Severity;
use truss_core::TrussEngine;

#[test]
fn test_script_injection_pr_title() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.title }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("script injection")
                && d.message.contains("untrusted")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !injection_warnings.is_empty(),
        "Using github.event.pull_request.title in run script should trigger script injection warning"
    );
}

#[test]
fn test_script_injection_pr_body() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.pull_request.body }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("script injection")
                && d.message.contains("untrusted")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !injection_warnings.is_empty(),
        "Using github.event.pull_request.body in run script should trigger script injection warning"
    );
}

#[test]
fn test_script_injection_head_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.head_ref }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("script injection")
                && d.message.contains("untrusted")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !injection_warnings.is_empty(),
        "Using github.head_ref in run script should trigger script injection warning"
    );
}

#[test]
fn test_script_injection_issue_body() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: issues
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.issue.body }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("script injection")
                && d.message.contains("untrusted")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !injection_warnings.is_empty(),
        "Using github.event.issue.body in run script should trigger script injection warning"
    );
}

#[test]
fn test_script_injection_comment_body() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: issue_comment
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.comment.body }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("script injection")
                && d.message.contains("untrusted")
                && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        !injection_warnings.is_empty(),
        "Using github.event.comment.body in run script should trigger script injection warning"
    );
}

#[test]
fn test_no_injection_safe_context() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.sha }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("script injection") && d.severity == Severity::Warning)
        .collect();

    assert!(
        injection_warnings.is_empty(),
        "Using github.sha in run script should NOT trigger script injection warning"
    );
}

#[test]
fn test_no_injection_env_var() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ env.MY_VAR }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("script injection") && d.severity == Severity::Warning)
        .collect();

    assert!(
        injection_warnings.is_empty(),
        "Using env.MY_VAR in run script should NOT trigger script injection warning"
    );
}

#[test]
fn test_no_injection_secrets() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ secrets.TOKEN }}"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("script injection") && d.severity == Severity::Warning)
        .collect();

    assert!(
        injection_warnings.is_empty(),
        "Using secrets.TOKEN in run script should NOT trigger script injection warning"
    );
}

#[test]
fn test_injection_in_env_value_safe() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: pull_request
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Safe usage via env
        env:
          PR_TITLE: ${{ github.event.pull_request.title }}
        run: echo "$PR_TITLE"
"#;

    let result = engine.analyze(yaml);
    let injection_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("script injection") && d.severity == Severity::Warning)
        .collect();

    assert!(
        injection_warnings.is_empty(),
        "Using untrusted input in env value (not in run block) should NOT trigger script injection warning"
    );
}
