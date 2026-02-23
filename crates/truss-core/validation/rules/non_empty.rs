use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// Validates that the document is not empty.
pub struct NonEmptyRule;

impl ValidationRule for NonEmptyRule {
    fn name(&self) -> &str {
        "non_empty"
    }

    fn validate(&self, _tree: &Tree, source: &str) -> Vec<Diagnostic> {
        if source.trim().is_empty() {
            vec![Diagnostic {
                message: "Document is empty".to_string(),
                severity: Severity::Warning,
                span: Span::default(),
            }]
        } else {
            Vec::new()
        }
    }
}
