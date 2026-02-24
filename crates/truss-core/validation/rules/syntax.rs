use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// Validates YAML syntax using tree-sitter parse errors.
pub struct SyntaxRule;

impl ValidationRule for SyntaxRule {
    fn name(&self) -> &str {
        "syntax"
    }

    fn requires_workflow(&self) -> bool {
        false
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let root_node = tree.root_node();

        if !root_node.has_error() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        let mut cursor = root_node.walk();

        for child in root_node.children(&mut cursor) {
            if child.is_error() || child.is_missing() {
                let start = child.start_byte();
                let end = child.end_byte().min(source.len());
                let error_snippet = if end > start && end <= source.len() {
                    source[start..end].chars().take(50).collect::<String>()
                } else {
                    String::new()
                };

                diagnostics.push(Diagnostic {
                    message: format!("Syntax error: {}", error_snippet),
                    severity: Severity::Error,
                    span: Span { start, end },
                });
            }
        }

        if diagnostics.is_empty() {
            diagnostics.push(Diagnostic {
                message: "YAML syntax error detected".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: 0,
                    end: source.len().min(100),
                },
            });
        }

        diagnostics
    }
}
