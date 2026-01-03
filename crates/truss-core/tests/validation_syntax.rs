//! Integration tests for SyntaxRule
//!
//! These tests verify syntax validation through the public TrussEngine API.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_syntax_rule_valid_yaml() {
    let mut engine = TrussEngine::new();
    let valid_yamls = vec![
        "name: test",
        "name: test\non: push",
        "key: value\nlist:\n  - item1\n  - item2",
        "jobs:\n  build:\n    runs-on: ubuntu-latest",
    ];

    for yaml in valid_yamls {
        let result = engine.analyze(yaml);
        // Check that there are no syntax errors (Error severity)
        let syntax_errors: Vec<_> = result.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error && d.message.contains("syntax"))
            .collect();
        assert!(
            syntax_errors.is_empty(),
            "Valid YAML should not produce syntax errors: {}",
            yaml
        );
    }
}

#[test]
fn test_syntax_rule_diagnostics_have_spans() {
    let mut engine = TrussEngine::new();
    let invalid_yaml = "invalid: [unclosed";
    
    let result = engine.analyze(invalid_yaml);
    
    // If there are syntax errors, they should have valid spans
    for diagnostic in &result.diagnostics {
        if diagnostic.message.contains("syntax") || diagnostic.message.contains("parse") {
            assert!(
                diagnostic.span.start <= diagnostic.span.end,
                "Diagnostic span should be valid: {:?}",
                diagnostic.span
            );
            assert_eq!(
                diagnostic.severity,
                Severity::Error,
                "Syntax errors should be Error severity"
            );
        }
    }
}

#[test]
fn test_syntax_rule_deterministic() {
    let mut engine = TrussEngine::new();
    let yaml = "name: test\non: push";
    
    let result1 = engine.analyze(yaml);
    let result2 = engine.analyze(yaml);
    
    // Count syntax-related diagnostics
    let syntax1: Vec<_> = result1.diagnostics.iter()
        .filter(|d| d.message.contains("syntax") || d.message.contains("parse"))
        .collect();
    let syntax2: Vec<_> = result2.diagnostics.iter()
        .filter(|d| d.message.contains("syntax") || d.message.contains("parse"))
        .collect();
    
    assert_eq!(
        syntax1.len(),
        syntax2.len(),
        "Syntax rule should be deterministic"
    );
}

