use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates step structure.
pub struct StepValidationRule;

impl ValidationRule for StepValidationRule {
    fn name(&self) -> &str {
        "step"
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
                                node.child(2)  // block_mapping_pair: child(0)=key, child(1)=colon, child(2)=value
                            } else {
                                node.child(1)  // flow_pair: child(0)=key, child(1)=value
                            };
                            if let Some(mut steps_value) = steps_value {
                                if steps_value.kind() == "block_node" {
                                    if let Some(inner) = steps_value.child(0) {
                                        steps_value = inner;
                                    }
                                }
                                if steps_value.kind() == "block_mapping" {
                                    let mut cursor = steps_value.walk();
                                    for child in steps_value.children(&mut cursor) {
                                        find_steps(child, source, diagnostics);
                                    }
                                    return;
                                }
                                fn validate_step(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
                                    let mut step_to_check = step_node;
                                    
                                    if step_to_check.kind() == "block_node" {
                                        if let Some(inner) = step_to_check.child(0) {
                                            step_to_check = inner;
                                        }
                                    }
                                    
                                    let mut has_uses = false;
                                    let mut has_run = false;
                                    let mut uses_value_node = None;
                                    
                                    fn check_step_keys<'a>(node: Node<'a>, source: &str, has_uses: &mut bool, has_run: &mut bool, uses_value: &mut Option<Node<'a>>) {
                                        match node.kind() {
                                            "block_mapping_pair" | "flow_pair" => {
                                                if let Some(key_node) = node.child(0) {
                                                    let key_text = utils::node_text(key_node, source);
                                                    let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                                        .trim_end_matches(':');
                                                    if key_cleaned == "uses" {
                                                        *has_uses = true;
                                                        let value_node = if node.kind() == "block_mapping_pair" {
                                                            node.child(2)
                                                        } else {
                                                            node.child(1)
                                                        };
                                                        if let Some(mut value_node) = value_node {
                                                            if value_node.kind() == "block_node" {
                                                                if let Some(inner) = value_node.child(0) {
                                                                    value_node = inner;
                                                                }
                                                            }
                                                            *uses_value = Some(value_node);
                                                        }
                                                    } else if key_cleaned == "run" {
                                                        *has_run = true;
                                                    }
                                                }
                                            }
                                            "block_node" | "block_mapping" | "flow_mapping" => {
                                                let mut cursor = node.walk();
                                                for child in node.children(&mut cursor) {
                                                    check_step_keys(child, source, has_uses, has_run, uses_value);
                                                }
                                            }
                                            _ => {
                                                let mut cursor = node.walk();
                                                for child in node.children(&mut cursor) {
                                                    check_step_keys(child, source, has_uses, has_run, uses_value);
                                                }
                                            }
                                        }
                                    }
                                    
                                    check_step_keys(step_to_check, source, &mut has_uses, &mut has_run, &mut uses_value_node);
                                    
                                    if !has_uses && !has_run {
                                        diagnostics.push(Diagnostic {
                                            message: "Step must have either 'uses' or 'run' field".to_string(),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: step_node.start_byte(),
                                                end: step_node.end_byte(),
                                            },
                                        });
                                    }
                                    
                                    if has_uses {
                                        if let Some(uses_value) = uses_value_node {
                                            let uses_text = utils::node_text(uses_value, source);
                                            let uses_cleaned = uses_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                            if !uses_cleaned.contains('@') && !uses_cleaned.is_empty() {
                                                diagnostics.push(Diagnostic {
                                                    message: format!("Invalid action reference format: '{}' (missing @ref)", uses_cleaned),
                                                    severity: Severity::Warning,
                                                    span: Span {
                                                        start: uses_value.start_byte(),
                                                        end: uses_value.end_byte(),
                                                    },
                                                });
                                            }
                                            if uses_cleaned.contains('@') {
                                                let parts: Vec<&str> = uses_cleaned.split('@').collect();
                                                if parts.len() == 2 {
                                                    let action_part = parts[0];
                                                    if action_part.starts_with("invalid/") {
                                                        diagnostics.push(Diagnostic {
                                                            message: format!("Invalid action reference: '{}'", uses_cleaned),
                                                            severity: Severity::Warning,
                                                            span: Span {
                                                                start: uses_value.start_byte(),
                                                                end: uses_value.end_byte(),
                                                            },
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // Handle block_sequence - steps are children of block_sequence
                                if steps_value.kind() == "block_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        // Each step in a block_sequence is a block_node
                                        if step_node.kind() == "block_node" {
                                            validate_step(step_node, source, diagnostics);
                                        } else {
                                            // Also check if it's directly a block_mapping
                                            validate_step(step_node, source, diagnostics);
                                        }
                                    }
                                } else if steps_value.kind() == "flow_sequence" {
                                    // Handle flow_sequence
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "flow_node" || step_node.kind() == "block_node" {
                                            validate_step(step_node, source, diagnostics);
                                        }
                                    }
                                } else {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "block_node" || step_node.kind() == "flow_node" {
                                            validate_step(step_node, source, diagnostics);
                                        }
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
                "block_node" | "block_mapping" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps(child, source, diagnostics);
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

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

