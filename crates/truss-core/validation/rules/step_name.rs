use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates step name field format.
pub struct StepNameRule;

impl ValidationRule for StepNameRule {
    fn name(&self) -> &str {
        "step_name"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        let root = tree.root_node();
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        let mut jobs_to_process = jobs_value;
        if jobs_to_process.kind() == "block_node" {
            if let Some(inner) = jobs_to_process.child(0) {
                jobs_to_process = inner;
            }
        }

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            let steps_value = if node.kind() == "block_mapping_pair" {
                                node.child(2)
                            } else {
                                node.child(1)
                            };
                            if let Some(mut steps_value) = steps_value {
                                if steps_value.kind() == "block_node" {
                                    if let Some(inner) = steps_value.child(0) {
                                        steps_value = inner;
                                    }
                                }
                                if steps_value.kind() == "block_sequence" || steps_value.kind() == "flow_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_name(step_node, source, diagnostics);
                                    }
                                }
                            }
                        }
                        let value_node = if node.kind() == "block_mapping_pair" {
                            node.child(2)
                        } else {
                            node.child(1)
                        };
                        if let Some(value_node) = value_node {
                            find_steps(value_node, source, diagnostics);
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps(child, source, diagnostics);
                    }
                }
            }
        }

        fn validate_step_name(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let mut step_to_check = step_node;
            
            // Handle block_sequence_item: child(0) is "-" marker, child(1) is actual content
            if step_to_check.kind() == "block_sequence_item" {
                if let Some(inner) = step_to_check.child(1) {
                    step_to_check = inner;
                }
            }
            
            // Handle block_node wrapper
            if step_to_check.kind() == "block_node" {
                if let Some(inner) = step_to_check.child(0) {
                    step_to_check = inner;
                }
            }
            
            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let name_value = utils::find_value_for_key(step_to_check, source, "name");
                
                if let Some(name_node) = name_value {
                    let name_text = utils::node_text(name_node, source);
                    let name_cleaned = name_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    
                    // Check if it's an expression
                    if name_cleaned.starts_with("${{") {
                        // Expressions are valid
                        return;
                    }
                    
                    // Warn if empty
                    if name_cleaned.is_empty() {
                        diagnostics.push(Diagnostic {
                            message: "Step has empty name. Consider providing a descriptive name for better workflow visibility.".to_string(),
                            severity: Severity::Warning,
                            span: Span {
                                start: name_node.start_byte(),
                                end: name_node.end_byte(),
                            },
                        });
                    } else if name_cleaned.len() > 100 {
                        // Warn if very long
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step name is very long ({} characters). Consider using a shorter, more concise name.",
                                name_cleaned.len()
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: name_node.start_byte(),
                                end: name_node.end_byte(),
                            },
                        });
                    }
                }
            }
        }

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

