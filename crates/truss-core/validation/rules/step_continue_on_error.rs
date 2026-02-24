use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates continue-on-error is a boolean.
pub struct StepContinueOnErrorRule;

impl ValidationRule for StepContinueOnErrorRule {
    fn name(&self) -> &str {
        "step_continue_on_error"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_cleaned = utils::clean_key(key_node, source);
                        if key_cleaned == "steps" {
                            if let Some(steps_value_raw) = utils::get_pair_value(node) {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_mapping" {
                                    let mut cursor = steps_value.walk();
                                    for child in steps_value.children(&mut cursor) {
                                        find_steps(child, source, diagnostics);
                                    }
                                    return;
                                }
                                // Handle block_sequence - steps are children of block_sequence
                                if steps_value.kind() == "block_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        // Each step in a block_sequence can be block_node or block_sequence_item
                                        if step_node.kind() == "block_node"
                                            || step_node.kind() == "block_sequence_item"
                                        {
                                            validate_step_continue_on_error(
                                                step_node,
                                                source,
                                                diagnostics,
                                            );
                                        } else if step_node.kind() == "block_mapping" {
                                            // Direct block_mapping (unlikely but possible)
                                            validate_step_continue_on_error(
                                                step_node,
                                                source,
                                                diagnostics,
                                            );
                                        }
                                    }
                                } else if steps_value.kind() == "flow_sequence" {
                                    // Handle flow_sequence
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "flow_node"
                                            || step_node.kind() == "block_node"
                                        {
                                            validate_step_continue_on_error(
                                                step_node,
                                                source,
                                                diagnostics,
                                            );
                                        }
                                    }
                                } else {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "block_node"
                                            || step_node.kind() == "flow_node"
                                        {
                                            validate_step_continue_on_error(
                                                step_node,
                                                source,
                                                diagnostics,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(value_node) = utils::get_pair_value(node) {
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

        fn validate_step_continue_on_error(
            step_node: Node,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let mut step_to_check = step_node;

            // Handle different node types that can represent a step
            // block_sequence_item -> block_node -> block_mapping
            // block_sequence_item has: child(0) = "-" marker, child(1) = actual content
            if step_to_check.kind() == "block_sequence_item" {
                // child(0) is the "-" marker, content follows (skip comments)
                let mut found = false;
                for i in 1..step_to_check.child_count() {
                    if let Some(child) = step_to_check.child(i) {
                        if child.kind() != "comment" {
                            step_to_check = child;
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    return;
                }
            }
            let step_to_check = utils::unwrap_node(step_to_check);

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let continue_on_error_value =
                    utils::find_value_for_key(step_to_check, source, "continue-on-error");

                if let Some(continue_node) = continue_on_error_value {
                    let continue_text = utils::node_text(continue_node, source);
                    let continue_cleaned = continue_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

                    // Check if it's an expression
                    if continue_cleaned.starts_with("${{") {
                        // Expressions are valid
                        return;
                    }

                    // Check if it's a string (quoted) - match the exact pattern from job_strategy.rs and timeout.rs
                    // Check the original text before any processing
                    if continue_text.trim().starts_with('"')
                        || continue_text.trim().starts_with('\'')
                    {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step has invalid continue-on-error: '{}'. continue-on-error must be a boolean (true or false), not a string.",
                                continue_cleaned
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: continue_node.start_byte(),
                                end: continue_node.end_byte(),
                            },
                        });
                    } else {
                        // Check if it's a boolean
                        let is_bool = continue_cleaned == "true" || continue_cleaned == "false";
                        if !is_bool {
                            // Check if it's a number
                            if continue_cleaned.parse::<f64>().is_ok() {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Step has invalid continue-on-error: '{}'. continue-on-error must be a boolean (true or false), not a number.",
                                        continue_cleaned
                                    ),
                                    severity: Severity::Error,
                                    span: Span {
                                        start: continue_node.start_byte(),
                                        end: continue_node.end_byte(),
                                    },
                                });
                            } else {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Step has invalid continue-on-error: '{}'. continue-on-error must be a boolean (true or false).",
                                        continue_cleaned
                                    ),
                                    severity: Severity::Error,
                                    span: Span {
                                        start: continue_node.start_byte(),
                                        end: continue_node.end_byte(),
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }

        find_steps(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
