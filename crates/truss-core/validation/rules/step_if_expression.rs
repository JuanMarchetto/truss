use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates if condition expressions in steps.
pub struct StepIfExpressionRule;

impl ValidationRule for StepIfExpressionRule {
    fn name(&self) -> &str {
        "step_if_expression"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            if let Some(steps_value_raw) = utils::get_pair_value(node) {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_sequence"
                                    || steps_value.kind() == "flow_sequence"
                                {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_if(step_node, source, diagnostics);
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

        fn validate_step_if(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let mut step_to_check = utils::unwrap_node(step_node);

            // Handle block_sequence_item - find the value child (skip dash and comments)
            if step_to_check.kind() == "block_sequence_item" {
                let mut found = false;
                for i in 1..step_to_check.child_count() {
                    if let Some(child) = step_to_check.child(i) {
                        if child.kind() != "comment" {
                            step_to_check = utils::unwrap_node(child);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    return;
                }
            }

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let if_value = utils::find_value_for_key(step_to_check, source, "if");

                if let Some(if_node) = if_value {
                    let if_text = utils::node_text(if_node, source);
                    let if_cleaned = if_text.trim();

                    // GitHub Actions auto-wraps if: conditions in ${{ }}.
                    // Both bare expressions and explicitly wrapped ones are valid.
                    let inner = if if_cleaned.starts_with("${{") && if_cleaned.ends_with("}}") {
                        // Explicitly wrapped: extract inner expression
                        if_cleaned[3..if_cleaned.len() - 2].trim()
                    } else {
                        // Bare expression: GitHub Actions auto-wraps this
                        if_cleaned
                    };

                    // Validate expression syntax
                    if !utils::is_valid_expression_syntax(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!("Invalid step 'if' expression syntax: '{}'", inner),
                            severity: Severity::Error,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    }

                    // Check for potentially always-true/false conditions
                    if utils::is_potentially_always_true(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step 'if' expression may always evaluate to true: '{}'",
                                inner
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    } else if utils::is_potentially_always_false(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step 'if' expression may always evaluate to false: '{}'",
                                inner
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
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
