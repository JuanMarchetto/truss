use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates timeout-minutes at step level.
pub struct StepTimeoutRule;

impl ValidationRule for StepTimeoutRule {
    fn name(&self) -> &str {
        "step_timeout"
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

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn process_jobs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(_key_node) = node.child(0) {
                        if let Some(job_value_raw) = utils::get_pair_value(node) {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                // Find steps in this job
                                let steps_value =
                                    utils::find_value_for_key(job_value, source, "steps");
                                if let Some(steps_node_raw) = steps_value {
                                    let steps_node = utils::unwrap_node(steps_node_raw);
                                    validate_steps_in_node(steps_node, source, diagnostics);
                                }
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        process_jobs(child, source, diagnostics);
                    }
                }
            }
        }

        fn validate_steps_in_node(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_sequence" | "flow_sequence" => {
                    let mut cursor = node.walk();
                    for step_node in node.children(&mut cursor) {
                        // Each step in a sequence might be wrapped in block_node
                        let step_to_validate = utils::unwrap_node(step_node);
                        validate_step_timeout(step_to_validate, source, diagnostics);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        validate_steps_in_node(child, source, diagnostics);
                    }
                }
            }
        }

        fn validate_step_timeout(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let step_to_check = utils::unwrap_node(step_node);

            // Step can be block_mapping, flow_mapping, or we need to search for it
            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let timeout_value =
                    utils::find_value_for_key(step_to_check, source, "timeout-minutes");

                if let Some(timeout_node) = timeout_value {
                    validate_timeout_value(timeout_node, source, diagnostics);
                }
            } else {
                // Try to find timeout-minutes field by searching recursively
                fn find_timeout_recursive(
                    node: Node,
                    source: &str,
                    diagnostics: &mut Vec<Diagnostic>,
                ) {
                    match node.kind() {
                        "block_mapping_pair" | "flow_pair" => {
                            if let Some(key_node) = node.child(0) {
                                let key_text = utils::node_text(key_node, source);
                                let key_cleaned = key_text
                                    .trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    })
                                    .trim_end_matches(':');
                                if key_cleaned == "timeout-minutes" {
                                    if let Some(timeout_node) = utils::get_pair_value(node) {
                                        validate_timeout_value(timeout_node, source, diagnostics);
                                    }
                                }
                            }
                        }
                        _ => {
                            let mut cursor = node.walk();
                            for child in node.children(&mut cursor) {
                                find_timeout_recursive(child, source, diagnostics);
                            }
                        }
                    }
                }
                find_timeout_recursive(step_to_check, source, diagnostics);
            }
        }

        fn validate_timeout_value(
            timeout_node: Node,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let timeout_text = utils::node_text(timeout_node, source);
            let timeout_cleaned =
                timeout_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

            // Check if it's an expression
            if timeout_cleaned.starts_with("${{") {
                // Expressions are valid
                return;
            }

            // Check if it's a string (quoted) - check both the original text and cleaned text
            let timeout_trimmed = timeout_text.trim();
            let is_string = timeout_trimmed.starts_with('"')
                && timeout_trimmed.ends_with('"')
                && timeout_trimmed.len() >= 2
                || timeout_trimmed.starts_with('\'')
                    && timeout_trimmed.ends_with('\'')
                    && timeout_trimmed.len() >= 2;

            if is_string {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Step has invalid timeout-minutes: '{}'. Timeout must be a number, not a string.",
                        timeout_cleaned
                    ),
                    severity: Severity::Error,
                    span: Span {
                        start: timeout_node.start_byte(),
                        end: timeout_node.end_byte(),
                    },
                });
                return;
            }

            // Try to parse as number
            match timeout_cleaned.parse::<f64>() {
                Ok(value) => {
                    if value < 0.0 {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step has invalid timeout-minutes: '{}'. Timeout must be a positive number.",
                                timeout_cleaned
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: timeout_node.start_byte(),
                                end: timeout_node.end_byte(),
                            },
                        });
                    } else if value == 0.0 {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step has invalid timeout-minutes: '{}'. Timeout must be a positive number (greater than zero).",
                                timeout_cleaned
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: timeout_node.start_byte(),
                                end: timeout_node.end_byte(),
                            },
                        });
                    }
                }
                Err(_) => {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Step has invalid timeout-minutes: '{}'. Timeout must be a number or expression.",
                            timeout_cleaned
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: timeout_node.start_byte(),
                            end: timeout_node.end_byte(),
                        },
                    });
                }
            }
        }

        process_jobs(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}
