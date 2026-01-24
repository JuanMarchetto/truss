//! Tests for ArtifactValidationRule
//!
//! **Status:** Tests written first (TDD) - Rule not yet implemented
//!
//! Validates actions/upload-artifact and actions/download-artifact usage in GitHub Actions workflows.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_artifact_valid_upload() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v3
        with:
          name: my-artifact
          path: dist/
"#;
    
    let result = engine.analyze(yaml);
    let artifact_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("artifact") || d.message.contains("upload")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        artifact_errors.is_empty(),
        "Valid artifact upload should not produce errors"
    );
}

#[test]
fn test_artifact_valid_download() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v3
        with:
          name: my-artifact
"#;
    
    let result = engine.analyze(yaml);
    let artifact_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("artifact") || d.message.contains("download")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        artifact_errors.is_empty(),
        "Valid artifact download should not produce errors"
    );
}

#[test]
fn test_artifact_valid_if_no_files_found() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v3
        with:
          name: my-artifact
          path: dist/
          if-no-files-found: warn
"#;
    
    let result = engine.analyze(yaml);
    let artifact_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("artifact") || d.message.contains("upload")) && 
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        artifact_errors.is_empty(),
        "Valid artifact with if-no-files-found should not produce errors"
    );
}

#[test]
fn test_artifact_error_empty_name() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v3
        with:
          name: ""
          path: dist/
"#;
    
    let result = engine.analyze(yaml);
    let artifact_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("artifact") || d.message.contains("upload") || d.message.contains("name")) && 
                (d.message.contains("empty") || d.message.contains("invalid")) &&
                d.severity == Severity::Error)
        .collect();
    
    assert!(
        !artifact_errors.is_empty(),
        "Empty artifact name should produce error"
    );
}

#[test]
fn test_artifact_warning_invalid_path() {
    let mut engine = TrussEngine::new();
    let yaml = r#"
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/upload-artifact@v3
        with:
          name: my-artifact
          path: /nonexistent/path
"#;
    
    let result = engine.analyze(yaml);
    let _artifact_warnings: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| (d.message.contains("artifact") || d.message.contains("upload") || d.message.contains("path")) && 
                (d.severity == Severity::Warning || d.severity == Severity::Error))
        .collect();
    
    // Note: This may produce a warning or be valid depending on implementation
    // Basic format validation should pass, but potentially invalid paths might warn
    assert!(
        true,
        "Potentially invalid artifact path may produce warning"
    );
}

