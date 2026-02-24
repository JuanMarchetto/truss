use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates environment variable names and values at step level.
pub struct StepEnvValidationRule;

impl ValidationRule for StepEnvValidationRule {
    fn name(&self) -> &str {
        "step_env"
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
                            let steps_value = utils::get_pair_value(node);
                            if let Some(steps_value_raw) = steps_value {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_sequence"
                                    || steps_value.kind() == "flow_sequence"
                                {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_env(step_node, source, diagnostics);
                                    }
                                }
                            }
                        }
                        let value_node = utils::get_pair_value(node);
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

        fn validate_step_env(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let mut step_to_check = step_node;

            // Handle block_sequence_item: child(0) is "-" marker, content follows (skip comments)
            if step_to_check.kind() == "block_sequence_item" {
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

            step_to_check = utils::unwrap_node(step_to_check);

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let env_value = utils::find_value_for_key(step_to_check, source, "env");
                if let Some(env_node_raw) = env_value {
                    let env_node = utils::unwrap_node(env_node_raw);

                    // Validate env variable names
                    fn check_env_vars(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
                        match node.kind() {
                            "block_mapping_pair" | "flow_pair" => {
                                if let Some(key_node) = node.child(0) {
                                    let key_cleaned = utils::clean_key(key_node, source);

                                    // Validate env var name format
                                    // GitHub Actions allows letters (upper and lower), numbers, and underscores
                                    // Names must start with a letter or underscore
                                    if !key_cleaned.is_empty() {
                                        let is_valid = key_cleaned
                                            .chars()
                                            .all(|c| c.is_alphanumeric() || c == '_');
                                        let starts_valid = key_cleaned
                                            .chars()
                                            .next()
                                            .map(|c| c.is_alphabetic() || c == '_')
                                            .unwrap_or(false);

                                        if !is_valid || !starts_valid {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Invalid environment variable name: '{}'. Environment variable names must start with a letter or underscore and contain only letters, numbers, and underscores.",
                                                    key_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: key_node.start_byte(),
                                                    end: key_node.end_byte(),
                                                },
                                            });
                                        }

                                        // Check for reserved GITHUB_ prefix
                                        if key_cleaned.starts_with("GITHUB_") {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Environment variable '{}' uses reserved prefix 'GITHUB_'. Variables starting with 'GITHUB_' are reserved by GitHub Actions.",
                                                    key_cleaned
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: key_node.start_byte(),
                                                    end: key_node.end_byte(),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                            _ => {
                                let mut cursor = node.walk();
                                for child in node.children(&mut cursor) {
                                    check_env_vars(child, source, diagnostics);
                                }
                            }
                        }
                    }

                    check_env_vars(env_node, source, diagnostics);
                }
            }
        }

        find_steps(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
