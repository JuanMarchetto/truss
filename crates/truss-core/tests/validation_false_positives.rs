//! False positive regression tests
//!
//! Each test pair covers a real-world pattern that previously produced false positives,
//! plus a genuinely invalid counterpart to ensure real errors are still caught.

use truss_core::Severity;
use truss_core::TrussEngine;

// ============================================================================
// 1. Reusable workflow calls — local paths don't need @ref
// ============================================================================

#[test]
fn test_local_reusable_workflow_no_ref_required() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-build:
    uses: ./.github/workflows/build.yml
  call-test:
    uses: ./.github/workflows/test.yml
    with:
      node-version: 18
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("missing @ref") && d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "Local workflow references (./) should not require @ref, got: {:?}",
        errors
    );
}

#[test]
fn test_remote_reusable_workflow_requires_ref() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  call-build:
    uses: owner/repo/.github/workflows/build.yml
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("missing @ref") && d.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Remote workflow references without @ref should produce an error"
    );
}

// ============================================================================
// 2. Matrix — object elements in arrays are valid
// ============================================================================

#[test]
fn test_matrix_array_objects_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        config:
          - { os: ubuntu-latest, arch: x64 }
          - { os: windows-latest, arch: x64 }
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("invalid array element type") && d.severity == Severity::Warning
        })
        .collect();

    assert!(
        errors.is_empty(),
        "Matrix arrays with object elements should not produce warnings, got: {:?}",
        errors
    );
}

#[test]
fn test_matrix_include_only_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            node: 18
          - os: windows-latest
            node: 20
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("matrix")
                && d.message.contains("must contain keys")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        errors.is_empty(),
        "Include-only matrix should not produce 'must contain keys' error, got: {:?}",
        errors
    );
}

#[test]
fn test_matrix_fromjson_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix: ${{ fromJSON(needs.setup.outputs.matrix) }}
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let matrix_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("matrix")
                && (d.message.contains("Invalid") || d.message.contains("must contain"))
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        matrix_errors.is_empty(),
        "fromJSON() matrix should not produce errors, got: {:?}",
        matrix_errors
    );
}

#[test]
fn test_matrix_scalar_value_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: ubuntu-latest
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("Matrix key") && d.message.contains("must be arrays"))
        .collect();

    assert!(
        !errors.is_empty(),
        "Scalar matrix value (not array) should still produce an error"
    );
}

#[test]
fn test_matrix_empty_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix: {}
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("matrix") && d.message.contains("empty"))
        .collect();

    assert!(
        !errors.is_empty(),
        "Empty matrix should still produce an error"
    );
}

// ============================================================================
// 3. GITHUB_* env vars — passing through is valid
// ============================================================================

#[test]
fn test_github_env_var_passthrough_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build
        run: echo "building"
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITHUB_RUN_ID: ${{ github.run_id }}
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("GITHUB_") && d.message.contains("reserved"))
        .collect();

    assert!(
        warnings.is_empty(),
        "Passing GITHUB_* env vars should not produce warnings, got: {:?}",
        warnings
    );
}

#[test]
fn test_invalid_env_var_name_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build
        run: echo "building"
        env:
          INVALID-NAME: value
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("Invalid environment variable name") && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !errors.is_empty(),
        "Env var with hyphen should still produce an error"
    );
}

// ============================================================================
// 4. Artifact names with spaces
// ============================================================================

#[test]
fn test_artifact_name_with_spaces_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v4
        with:
          name: SARIF file
          path: results.sarif
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("invalid name format"))
        .collect();

    assert!(
        warnings.is_empty(),
        "Artifact names with spaces should be valid, got: {:?}",
        warnings
    );
}

#[test]
fn test_artifact_empty_name_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v4
        with:
          name: ""
          path: results.sarif
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("empty name") && d.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Empty artifact name should still produce an error"
    );
}

// ============================================================================
// 5. Self-hosted / custom runner labels
// ============================================================================

#[test]
fn test_self_hosted_runner_labels_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: linux.2xlarge
    steps:
      - run: echo "test"
  build2:
    runs-on: linux.24_04.4x
    steps:
      - run: echo "test"
  build3:
    runs-on: buildjet-4vcpu-ubuntu-2204
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("unknown runner label"))
        .collect();

    assert!(
        warnings.is_empty(),
        "Custom/self-hosted runner labels should not produce warnings, got: {:?}",
        warnings
    );
}

#[test]
fn test_empty_runner_label_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ""
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("empty runs-on") && d.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Empty runner label should still produce an error"
    );
}

// ============================================================================
// 6. GITHUB_TOKEN in workflow_call
// ============================================================================

#[test]
fn test_github_token_in_workflow_call_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      NPM_TOKEN:
        required: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Publish
        run: echo "publish"
        env:
          NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("GITHUB_TOKEN")
                && (d.message.contains("not defined") || d.message.contains("no secrets defined"))
        })
        .collect();

    assert!(
        errors.is_empty(),
        "secrets.GITHUB_TOKEN should always be implicitly available, got: {:?}",
        errors
    );
}

#[test]
fn test_secrets_inherit_pattern() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets: inherit
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: echo "deploy"
        env:
          MY_SECRET: ${{ secrets.MY_SECRET }}
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("secrets")
                && d.message.contains("not defined")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        errors.is_empty(),
        "secrets: inherit should allow all secret references, got: {:?}",
        errors
    );
}

#[test]
fn test_undefined_secret_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    secrets:
      NPM_TOKEN:
        required: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: echo "deploy"
        env:
          MY_SECRET: ${{ secrets.MY_UNDEFINED_SECRET }}
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("MY_UNDEFINED_SECRET")
                && d.message.contains("undefined")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !errors.is_empty(),
        "Reference to undefined secret should still produce an error"
    );
}

// ============================================================================
// 7. Expression always-true/false — complex expressions are not always-true
// ============================================================================

#[test]
fn test_expression_not_always_true() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    if: inputs.force == true || steps.check.outputs.result
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("always evaluate to true"))
        .collect();

    assert!(
        warnings.is_empty(),
        "Complex expression with 'true' in comparison should not be flagged as always-true, got: {:?}",
        warnings
    );
}

#[test]
fn test_expression_bare_true_still_warns() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    if: true
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("always evaluate to true"))
        .collect();

    assert!(
        !warnings.is_empty(),
        "Bare 'true' expression should still produce a warning"
    );
}

// ============================================================================
// 8. cancel-in-progress with expression value
// ============================================================================

#[test]
fn test_cancel_in_progress_expression_valid() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name != 'push' }}
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("cancel-in-progress")
                && d.message.contains("boolean")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        errors.is_empty(),
        "Expression cancel-in-progress should be valid, got: {:?}",
        errors
    );
}

#[test]
fn test_cancel_in_progress_string_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: "yes"
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("cancel-in-progress")
                && d.message.contains("boolean")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        !errors.is_empty(),
        "String cancel-in-progress should still produce an error"
    );
}

// ============================================================================
// 9. Expression operator in format strings
// ============================================================================

#[test]
fn test_expression_operator_in_format_string() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Build
        run: echo ${{ format('--tagVersion={0}', inputs.tag) }}
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("assignment operator") || d.message.contains("invalid operator")
        })
        .collect();

    assert!(
        warnings.is_empty(),
        "= inside format string literal should not be flagged, got: {:?}",
        warnings
    );
}

#[test]
fn test_expression_triple_equals_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    if: ${{ github.ref === 'refs/heads/main' }}
    steps:
      - run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("===") && d.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "=== operator should still produce an error"
    );
}

// ============================================================================
// 10. Unclosed expression
// ============================================================================

#[test]
fn test_unclosed_expression_still_errors() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: ${{ github.ref
        run: echo "test"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("unclosed expression") && d.severity == Severity::Error)
        .collect();

    assert!(
        !errors.is_empty(),
        "Unclosed expression should still produce an error"
    );
}

// ============================================================================
// 11. workflow_call + workflow_dispatch dual trigger inputs
// ============================================================================

#[test]
fn test_workflow_call_and_dispatch_dual_inputs() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on:
  workflow_call:
    inputs:
      environment:
        type: string
        required: true
  workflow_dispatch:
    inputs:
      debug_enabled:
        type: boolean
        default: false
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: echo "deploying to ${{ inputs.environment }}"
      - name: Debug
        if: inputs.debug_enabled
        run: echo "debug"
"#;

    let result = engine.analyze(yaml);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.message.contains("undefined")
                && d.message.contains("input")
                && d.severity == Severity::Error
        })
        .collect();

    assert!(
        errors.is_empty(),
        "Inputs from both workflow_call and workflow_dispatch should be valid, got: {:?}",
        errors
    );
}

// ============================================================================
// 12. Unknown function false positive from format string parsing
// ============================================================================

#[test]
fn test_no_false_unknown_function_from_format() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Tag
        run: echo ${{ format('{0}-{1}', github.sha, github.run_number) }}
"#;

    let result = engine.analyze(yaml);
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("Unknown function"))
        .collect();

    assert!(
        warnings.is_empty(),
        "format() with brace placeholders should not produce unknown function warnings, got: {:?}",
        warnings
    );
}
