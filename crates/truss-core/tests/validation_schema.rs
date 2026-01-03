//! Integration tests for GitHubActionsSchemaRule
//!
//! These tests verify GitHub Actions schema validation through the public TrussEngine API.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_schema_rule_valid_workflow_with_on() {
    let mut engine = TrussEngine::new();
    let valid_workflows = vec![
        "on: push",
        "name: CI\non: push",
        "on:\n  push:\n    branches: [main]",
        "name: Test\non: [push, pull_request]",
    ];
    
    for yaml in valid_workflows {
        let result = engine.analyze(yaml);
        let schema_errors: Vec<_> = result.diagnostics
            .iter()
            .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
            .collect();
        
        assert!(
            schema_errors.is_empty(),
            "Valid GitHub Actions workflow should not produce schema errors: {}",
            yaml
        );
    }
}

#[test]
fn test_schema_rule_missing_on_field() {
    let mut engine = TrussEngine::new();
    let yaml = "name: My Workflow\njobs:\n  test:\n    runs-on: ubuntu-latest";
    
    let result = engine.analyze(yaml);
    let schema_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("on") && d.severity == Severity::Error)
        .collect();
    
    // GitHubActionsSchemaRule detects missing 'on' field when 'name:' is present
    assert!(
        !schema_errors.is_empty(),
        "Missing 'on' field should produce error"
    );
    assert!(
        schema_errors.iter().any(|d| d.message.contains("on")),
        "Error message should mention missing 'on' field"
    );
}

#[test]
fn test_schema_rule_non_github_actions_yaml() {
    let mut engine = TrussEngine::new();
    // These are clearly not GitHub Actions workflows (no 'name:' or 'jobs:' or 'on:')
    let non_workflow_yamls = vec![
        "key: value",
        "database:\n  host: localhost\n  port: 5432",
    ];
    
    for yaml in non_workflow_yamls {
        let result = engine.analyze(yaml);
        let schema_errors: Vec<_> = result.diagnostics
            .iter()
            .filter(|d| d.message.contains("GitHub Actions") || 
                    (d.message.contains("on") && d.severity == Severity::Error))
            .collect();
        
        // Rule should skip validation if it doesn't look like a workflow
        // (no 'name:' or 'on:' at top level)
        assert!(
            schema_errors.is_empty(),
            "Non-GitHub Actions YAML should not produce schema errors: {}",
            yaml
        );
    }
}

#[test]
fn test_schema_rule_deterministic() {
    let mut engine = TrussEngine::new();
    let yaml = "name: Test\non: push";
    
    let result1 = engine.analyze(yaml);
    let result2 = engine.analyze(yaml);
    
    let schema1: Vec<_> = result1.diagnostics.iter()
        .filter(|d| d.message.contains("on"))
        .collect();
    let schema2: Vec<_> = result2.diagnostics.iter()
        .filter(|d| d.message.contains("on"))
        .collect();
    
    assert_eq!(
        schema1.len(),
        schema2.len(),
        "Schema rule should be deterministic"
    );
}

#[test]
fn test_schema_rule_error_severity() {
    let mut engine = TrussEngine::new();
    let yaml = "name: CI"; // Missing 'on'
    
    let result = engine.analyze(yaml);
    let schema_errors: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.contains("on"))
        .collect();
    
    for diagnostic in schema_errors {
        assert_eq!(
            diagnostic.severity,
            Severity::Error,
            "Missing required field should be Error severity"
        );
    }
}

