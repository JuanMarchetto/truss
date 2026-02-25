use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates that `timeout-minutes` is a positive number.
pub struct TimeoutRule;

impl ValidationRule for TimeoutRule {
    fn name(&self) -> &str {
        "timeout"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        // Process jobs and check for timeout-minutes
        fn check_job_timeout(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_cleaned = utils::clean_key(key_node, source);

                        // Get the job value node
                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value_raw) = job_value {
                            let job_value = utils::unwrap_node(job_value_raw);

                            // Only consider it a job if the value is a mapping (job definition)
                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                // Check for timeout-minutes in this job
                                let timeout_value =
                                    utils::find_value_for_key(job_value, source, "timeout-minutes");

                                if let Some(timeout_node) = timeout_value {
                                    let timeout_text = utils::node_text(timeout_node, source);
                                    let timeout_cleaned = timeout_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });

                                    // Check if it's an expression (starts with ${{)
                                    if timeout_cleaned.starts_with("${{") {
                                        // Expressions are valid, skip validation
                                        return;
                                    }

                                    // Check if it's a string (quoted)
                                    if timeout_text.trim().starts_with('"')
                                        || timeout_text.trim().starts_with('\'')
                                    {
                                        // String value is invalid
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' has invalid timeout-minutes: '{}'. Timeout must be a number, not a string.",
                                                key_cleaned, timeout_cleaned
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: timeout_node.start_byte(),
                                                end: timeout_node.end_byte(),
                                            },
                                            rule_id: String::new(),
                                        });
                                        return;
                                    }

                                    // Try to parse as number
                                    match timeout_cleaned.parse::<f64>() {
                                        Ok(value) => {
                                            if value < 0.0 {
                                                // Negative value
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' has invalid timeout-minutes: '{}'. Timeout must be a positive number.",
                                                        key_cleaned, timeout_cleaned
                                                    ),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: timeout_node.start_byte(),
                                                        end: timeout_node.end_byte(),
                                                    },
                                                    rule_id: String::new(),
                                                });
                                            } else if value == 0.0 {
                                                // Zero value
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' has invalid timeout-minutes: '{}'. Timeout must be a positive number (greater than zero).",
                                                        key_cleaned, timeout_cleaned
                                                    ),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: timeout_node.start_byte(),
                                                        end: timeout_node.end_byte(),
                                                    },
                                                    rule_id: String::new(),
                                                });
                                            }
                                            // Positive values (including decimals) are valid
                                        }
                                        Err(_) => {
                                            // Not a valid number and not an expression
                                            // Could be a string without quotes or invalid format
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has invalid timeout-minutes: '{}'. Timeout must be a number or expression.",
                                                    key_cleaned, timeout_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: timeout_node.start_byte(),
                                                    end: timeout_node.end_byte(),
                                                },
                                                rule_id: String::new(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Continue traversing
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        check_job_timeout(child, source, diagnostics);
                    }
                }
            }
        }

        check_job_timeout(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
