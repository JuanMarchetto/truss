use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;
use super::super::ValidationRule;
use super::super::utils;

/// Validates that GitHub Actions workflows have required top-level fields.
pub struct GitHubActionsSchemaRule;

impl ValidationRule for GitHubActionsSchemaRule {
    fn name(&self) -> &str {
        "github_actions_schema"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Use strict workflow detection
        if !utils::is_github_actions_workflow(tree, source) {
            // Not a GitHub Actions workflow, skip validation
            return diagnostics;
        }

        // Check for required 'on' field for GitHub Actions
        // We need to check if 'on' key exists in the top-level mapping
        let root = tree.root_node();
        
        fn find_key_in_tree(node: tree_sitter::Node, source: &str, target_key: &str) -> bool {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = &source[key_node.start_byte()..key_node.end_byte()];
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == target_key {
                            return true;
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if find_key_in_tree(child, source, target_key) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        
        let has_on = find_key_in_tree(root, source, "on");

        if !has_on {
            diagnostics.push(Diagnostic {
                message: "GitHub Actions workflow must have an 'on' field".to_string(),
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

