//! Regression tests for expression validation edge cases.
//!
//! These tests cover patterns found in real-world workflows that previously
//! caused false positives in expression validation rules.

use truss_core::{Severity, TrussEngine};

// ---------------------------------------------------------------------------
// fromJSON() function
// ---------------------------------------------------------------------------

#[test]
fn expression_fromjson_is_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  setup:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.set-matrix.outputs.matrix }}
    steps:
      - id: set-matrix
        run: echo 'matrix={"os":["ubuntu-latest"]}' >> $GITHUB_OUTPUT
  build:
    needs: setup
    strategy:
      matrix:
        include: ${{ fromJSON(needs.setup.outputs.matrix) }}
    runs-on: ${{ matrix.os }}
    steps:
      - run: echo "build"
"#;

    let result = engine.analyze(yaml);
    let fromjson_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("fromJSON") || d.message.contains("Unknown function"))
        .collect();

    assert!(
        fromjson_errors.is_empty(),
        "fromJSON() should be recognized as a valid function. Got: {:?}",
        fromjson_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn expression_tojson_is_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo '${{ toJson(matrix) }}'
      - run: echo '${{ toJSON(github.event) }}'
"#;

    let result = engine.analyze(yaml);
    let tojson_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("toJson")
                || d.message.contains("toJSON")
                || d.message.contains("Unknown function")
        })
        .collect();

    assert!(
        tojson_errors.is_empty(),
        "toJson/toJSON should be recognized as valid functions. Got: {:?}",
        tojson_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Bare if: expressions (without ${{ }} wrapper)
// ---------------------------------------------------------------------------

#[test]
fn bare_job_if_github_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  deploy:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: echo "deploy"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("if") && d.message.contains("${{"))
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare job if: expression should be valid (GitHub auto-wraps). Got: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn bare_step_if_github_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - if: github.ref == 'refs/heads/main'
        run: echo "main"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("if") && d.message.contains("${{"))
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare step if: expression should be valid (GitHub auto-wraps). Got: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn bare_if_with_event_name() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: [push, pull_request]
jobs:
  deploy:
    if: github.event_name == 'push'
    runs-on: ubuntu-latest
    steps:
      - run: echo "deploy"
  coverage:
    if: github.event_name != 'merge_group'
    runs-on: ubuntu-latest
    steps:
      - run: echo "coverage"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("if") || d.message.contains("expression"))
                && d.message.contains("${{")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare if: with event_name comparison should be valid. Got: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn bare_if_always() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  required:
    if: always()
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - run: echo "always runs"
"#;

    let result = engine.analyze(yaml);
    let if_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("if") && d.message.contains("${{") && d.severity == Severity::Error
        })
        .collect();

    assert!(
        if_errors.is_empty(),
        "Bare if: always() should be valid. Got: {:?}",
        if_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Complex expression patterns from real workflows
// ---------------------------------------------------------------------------

#[test]
fn expression_contains_fromjson_nested() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  setup:
    runs-on: ubuntu-latest
    outputs:
      run_type: ${{ steps.calc.outputs.run_type }}
    steps:
      - id: calc
        run: echo "run_type=auto" >> $GITHUB_OUTPUT
  outcome:
    needs: [setup]
    if: ${{ !cancelled() && contains(fromJSON('["auto", "try"]'), needs.setup.outputs.run_type) }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "outcome"
"#;

    let result = engine.analyze(yaml);
    let expression_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            (d.message.contains("fromJSON") || d.message.contains("Unknown function"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        expression_errors.is_empty(),
        "Nested fromJSON in contains() should be valid. Got: {:?}",
        expression_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn expression_ternary_environment() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: ${{ (github.repository == 'org/repo' && github.ref == 'refs/heads/main') && 'production' || '' }}
    steps:
      - run: echo "deploy"
"#;

    let result = engine.analyze(yaml);
    let expression_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("expression") && d.severity == Severity::Error)
        .collect();

    assert!(
        expression_errors.is_empty(),
        "Ternary-style expression should be valid. Got: {:?}",
        expression_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Runner label edge cases
// ---------------------------------------------------------------------------

#[test]
fn self_hosted_runner_array() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on:
      - 'self-hosted'
      - '1ES.Pool=TypeScript-1ES-GitHub-Large'
      - '1ES.ImageOverride=azure-linux-3'
    steps:
      - run: echo "build"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("runner") && d.severity == Severity::Error)
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Self-hosted runner array should not produce errors. Got: {:?}",
        runner_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn modern_arm_runners() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  arm:
    runs-on: ubuntu-24.04-arm
    steps:
      - run: echo "arm"
  arm64:
    runs-on: ubuntu-latest-arm64
    steps:
      - run: echo "arm64"
  windows_arm:
    runs-on: windows-latest-arm64
    steps:
      - run: echo "win arm"
  macos_xlarge:
    runs-on: macos-latest-xlarge
    steps:
      - run: echo "xlarge"
"#;

    let result = engine.analyze(yaml);
    let runner_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("unknown runner"))
        .collect();

    assert!(
        runner_warnings.is_empty(),
        "Modern ARM and xlarge runners should be recognized. Got: {:?}",
        runner_warnings
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn expression_in_runs_on() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
name: CI
on: push
jobs:
  build:
    runs-on: "${{ matrix.os }}"
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    steps:
      - run: echo "build"
"#;

    let result = engine.analyze(yaml);
    let runner_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("runner") && d.severity == Severity::Error)
        .collect();

    assert!(
        runner_errors.is_empty(),
        "Expression in runs-on should not produce errors. Got: {:?}",
        runner_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
