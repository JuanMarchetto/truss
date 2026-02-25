use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates shell field values.
pub struct StepShellRule;

impl ValidationRule for StepShellRule {
    fn name(&self) -> &str {
        "step_shell"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

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
                        validate_step_shell(step_to_validate, source, diagnostics);
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

        fn validate_step_shell(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let step_to_check = utils::unwrap_node(step_node);

            // Step can be block_mapping, flow_mapping, or we need to search for it
            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let shell_value = utils::find_value_for_key(step_to_check, source, "shell");

                if let Some(shell_node) = shell_value {
                    validate_shell_value(shell_node, source, diagnostics);
                }
            } else {
                // Try to find shell field by searching recursively
                fn find_shell_recursive(
                    node: Node,
                    source: &str,
                    diagnostics: &mut Vec<Diagnostic>,
                ) {
                    match node.kind() {
                        "block_mapping_pair" | "flow_pair" => {
                            if let Some(key_node) = node.child(0) {
                                let key_cleaned = utils::clean_key(key_node, source);
                                if key_cleaned == "shell" {
                                    if let Some(shell_node) = utils::get_pair_value(node) {
                                        validate_shell_value(shell_node, source, diagnostics);
                                    }
                                }
                            }
                        }
                        _ => {
                            let mut cursor = node.walk();
                            for child in node.children(&mut cursor) {
                                find_shell_recursive(child, source, diagnostics);
                            }
                        }
                    }
                }
                find_shell_recursive(step_to_check, source, diagnostics);
            }
        }

        fn validate_shell_value(shell_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let shell_text = utils::node_text(shell_node, source);

            // Check if it's an expression first
            let shell_trimmed = shell_text.trim();
            if shell_trimmed.starts_with("${{") {
                // Expressions are valid
                return;
            }

            // Check if empty - handle various empty string representations
            // Empty can be: "", '', or whitespace-only after trimming quotes
            let shell_cleaned =
                shell_trimmed.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

            // Check if empty: either cleaned is empty, or it's just quotes
            if shell_cleaned.is_empty() {
                diagnostics.push(Diagnostic {
                    message: "Step has empty shell value. Shell must be a valid shell name or custom command.".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: shell_node.start_byte(),
                        end: shell_node.end_byte(),
                    },
                    rule_id: String::new(),
                });
                return;
            }

            // Known valid shells
            let known_shells = ["bash", "pwsh", "python", "sh", "cmd", "powershell"];
            let is_known = known_shells
                .iter()
                .any(|s| s.eq_ignore_ascii_case(shell_cleaned));

            // Check for custom shell format (should contain {0} placeholder)
            let is_custom = shell_cleaned.contains("{0}");

            if !is_known && !is_custom {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Step has invalid shell: '{}'. Valid shells are: bash, pwsh, python, sh, cmd, powershell, or a custom command with {{0}} placeholder.",
                        shell_cleaned
                    ),
                    severity: Severity::Error,
                    span: Span {
                        start: shell_node.start_byte(),
                        end: shell_node.end_byte(),
                    },
                    rule_id: String::new(),
                });
            }
        }

        process_jobs(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
