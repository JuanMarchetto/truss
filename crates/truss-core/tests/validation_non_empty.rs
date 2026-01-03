//! Integration tests for NonEmptyRule
//!
//! These tests verify empty document validation through the public TrussEngine API.

use truss_core::TrussEngine;
use truss_core::Severity;

#[test]
fn test_non_empty_rule_empty_string() {
    let mut engine = TrussEngine::new();
    let empty_yaml = "";
    
    let result = engine.analyze(empty_yaml);
    let empty_warnings: Vec<_> = result.diagnostics
        .iter()
        .filter(|d| d.message.to_lowercase().contains("empty"))
        .collect();
    
    assert!(
        !empty_warnings.is_empty(),
        "Empty string should produce at least one warning about being empty"
    );
    assert!(
        empty_warnings.iter().any(|d| d.severity == Severity::Warning),
        "Empty document should be a warning, not an error"
    );
}

#[test]
fn test_non_empty_rule_whitespace_only() {
    let mut engine = TrussEngine::new();
    let whitespace_yamls = vec![
        "   ",
        "\n\n\n",
        "\t\t",
        "  \n  \t  \n  ",
    ];
    
    for yaml in whitespace_yamls {
        let result = engine.analyze(yaml);
        let empty_warnings: Vec<_> = result.diagnostics
            .iter()
            .filter(|d| d.message.to_lowercase().contains("empty"))
            .collect();
        
        assert!(
            !empty_warnings.is_empty(),
            "Whitespace-only document should produce warning: {:?}",
            yaml
        );
        assert!(
            empty_warnings.iter().any(|d| d.severity == Severity::Warning),
            "Whitespace-only should be a warning"
        );
    }
}

#[test]
fn test_non_empty_rule_valid_content() {
    let mut engine = TrussEngine::new();
    let valid_yamls = vec![
        "name: test",
        "key: value",
        "jobs:\n  build:\n    runs-on: ubuntu-latest",
    ];
    
    for yaml in valid_yamls {
        let result = engine.analyze(yaml);
        let empty_warnings: Vec<_> = result.diagnostics
            .iter()
            .filter(|d| d.message.to_lowercase().contains("empty"))
            .collect();
        
        assert!(
            empty_warnings.is_empty(),
            "Non-empty YAML should not produce empty warnings: {}",
            yaml
        );
    }
}

#[test]
fn test_non_empty_rule_deterministic() {
    let mut engine = TrussEngine::new();
    let empty_yaml = "";
    
    let result1 = engine.analyze(empty_yaml);
    let result2 = engine.analyze(empty_yaml);
    
    let empty1: Vec<_> = result1.diagnostics.iter()
        .filter(|d| d.message.to_lowercase().contains("empty"))
        .collect();
    let empty2: Vec<_> = result2.diagnostics.iter()
        .filter(|d| d.message.to_lowercase().contains("empty"))
        .collect();
    
    assert_eq!(
        empty1.len(),
        empty2.len(),
        "NonEmpty rule should be deterministic"
    );
}

