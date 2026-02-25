use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates concurrency syntax.
pub struct ConcurrencyRule;

impl ValidationRule for ConcurrencyRule {
    fn name(&self) -> &str {
        "concurrency"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();

        // Check workflow-level concurrency
        let workflow_concurrency = utils::find_value_for_key(root, source, "concurrency");
        if let Some(concurrency_node) = workflow_concurrency {
            validate_concurrency_node(concurrency_node, source, "workflow", &mut diagnostics);
        }

        // Check job-level concurrency
        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        fn process_jobs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::clean_key(key_node, source).to_string();

                        // Get value node: last non-comment, non-":" child after the key
                        let mut job_value_opt = None;
                        for i in (1..node.child_count()).rev() {
                            if let Some(child) = node.child(i) {
                                if child.kind() != "comment" && child.kind() != ":" {
                                    job_value_opt = Some(child);
                                    break;
                                }
                            }
                        }

                        if let Some(job_value_raw) = job_value_opt {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                let concurrency_node =
                                    utils::find_value_for_key(job_value, source, "concurrency");
                                if let Some(concurrency) = concurrency_node {
                                    validate_concurrency_node(
                                        concurrency,
                                        source,
                                        &format!("job '{}'", job_name),
                                        diagnostics,
                                    );
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

        process_jobs(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}

fn validate_concurrency_node(
    concurrency_node: Node,
    source: &str,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let concurrency_to_check = utils::unwrap_node(concurrency_node);

    // Concurrency can be a simple string (group name) or a mapping with group/cancel-in-progress.
    // String form: `concurrency: my-group` or `concurrency: ${{ github.ref }}`
    match concurrency_to_check.kind() {
        "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
            // String form is valid — the string IS the group name
            let text = utils::node_text(concurrency_to_check, source);
            let cleaned = text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
            if cleaned.is_empty() {
                diagnostics.push(Diagnostic {
                    message: format!("Concurrency at {} level has empty group name.", context),
                    severity: Severity::Error,
                    span: Span {
                        start: concurrency_to_check.start_byte(),
                        end: concurrency_to_check.end_byte(),
                    },
                    rule_id: String::new(),
                });
            }
            return;
        }
        _ => {}
    }

    // Mapping form: must have a `group` key
    let group_value = utils::find_value_for_key(concurrency_to_check, source, "group");

    if group_value.is_none() {
        diagnostics.push(Diagnostic {
            message: format!(
                "Concurrency at {} level is missing required 'group' field.",
                context
            ),
            severity: Severity::Error,
            span: Span {
                start: concurrency_to_check.start_byte(),
                end: concurrency_to_check.end_byte(),
            },
            rule_id: String::new(),
        });
        return;
    }

    let group_node = group_value.unwrap();
    let group_text = utils::node_text(group_node, source);
    let group_cleaned =
        group_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

    // Group should be a string or expression, not a bare number.
    // Skip values that start with ${{ (expressions) or with a known context prefix
    // (e.g., "github.ref" looks numeric-with-dot but is actually a valid context reference).
    if !group_cleaned.starts_with("${{") {
        let has_context_prefix = [
            "github.",
            "env.",
            "vars.",
            "inputs.",
            "secrets.",
            "matrix.",
            "needs.",
            "job.",
            "jobs.",
            "steps.",
            "runner.",
            "strategy.",
        ]
        .iter()
        .any(|prefix| group_cleaned.starts_with(prefix));

        if !has_context_prefix && group_cleaned.parse::<f64>().is_ok() {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Concurrency 'group' at {} level must be a string or expression, not a number.",
                    context
                ),
                severity: Severity::Error,
                span: Span {
                    start: group_node.start_byte(),
                    end: group_node.end_byte(),
                },
                rule_id: String::new(),
            });
        }
    }

    // Check cancel-in-progress if present
    let cancel_value =
        utils::find_value_for_key(concurrency_to_check, source, "cancel-in-progress");

    if let Some(cancel_node) = cancel_value {
        let cancel_text = utils::node_text(cancel_node, source);

        // Check if it's a quoted scalar (string) - this is invalid
        // Check both node kind and if text starts/ends with quotes
        let is_quoted_string = cancel_node.kind() == "double_quoted_scalar"
            || cancel_node.kind() == "single_quoted_scalar"
            || (cancel_text.trim().starts_with('"') && cancel_text.trim().ends_with('"'))
            || (cancel_text.trim().starts_with('\'') && cancel_text.trim().ends_with('\''));

        if is_quoted_string {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Concurrency 'cancel-in-progress' at {} level must be a boolean (true/false), not a string.",
                    context
                ),
                severity: Severity::Error,
                span: Span {
                    start: cancel_node.start_byte(),
                    end: cancel_node.end_byte(),
                },
                rule_id: String::new(),
            });
        } else {
            // Check the actual value
            let cancel_cleaned =
                cancel_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

            // cancel-in-progress should be boolean (true/false)
            if cancel_cleaned != "true" && cancel_cleaned != "false" {
                // Try to parse as boolean
                if cancel_cleaned.parse::<bool>().is_err() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Concurrency 'cancel-in-progress' at {} level must be a boolean (true/false).",
                            context
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: cancel_node.start_byte(),
                            end: cancel_node.end_byte(),
                        },
                        rule_id: String::new(),
                    });
                }
            }
        }
    }
}
