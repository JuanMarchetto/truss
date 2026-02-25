use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// Validates workflow name field.
pub struct WorkflowNameRule;

impl ValidationRule for WorkflowNameRule {
    fn name(&self) -> &str {
        "workflow_name"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let name_value = match utils::find_value_for_key(root, source, "name") {
            Some(v) => v,
            None => return diagnostics,
        };

        let name_text = utils::node_text(name_value, source);
        let name_cleaned =
            name_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

        if name_cleaned.is_empty() || name_cleaned == "\"\"" || name_cleaned == "''" {
            diagnostics.push(Diagnostic {
                message: "Workflow name cannot be empty".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: name_value.start_byte(),
                    end: name_value.end_byte(),
                },
                rule_id: String::new(),
            });
        }

        if !name_cleaned.contains("${{") && name_cleaned.len() > 255 {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Workflow name is too long ({} characters, maximum is 255)",
                    name_cleaned.len()
                ),
                severity: Severity::Error,
                span: Span {
                    start: name_value.start_byte(),
                    end: name_value.end_byte(),
                },
                rule_id: String::new(),
            });
        }

        diagnostics
    }
}
