use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates environment variable names and values at step level.
pub struct StepEnvValidationRule;

impl ValidationRule for StepEnvValidationRule {
    fn name(&self) -> &str {
        "step_env"
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

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            let steps_value = utils::get_pair_value(node);
                            if let Some(steps_value_raw) = steps_value {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_sequence" || steps_value.kind() == "flow_sequence" {
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
            
            // Handle block_sequence_item: child(0) is "-" marker, child(1) is actual content
            if step_to_check.kind() == "block_sequence_item" {
                if let Some(inner) = step_to_check.child(1) {
                    step_to_check = inner;
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
                                    let key_text = utils::node_text(key_node, source);
                                    let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                        .trim_end_matches(':');
                                    
                                    // Validate env var name format: [A-Z_][A-Z0-9_]*
                                    // GitHub Actions allows uppercase letters, numbers, and underscores
                                    if !key_cleaned.is_empty() {
                                        let is_valid = key_cleaned.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_');
                                        let starts_valid = key_cleaned.chars().next().map(|c| c.is_uppercase() || c == '_').unwrap_or(false);
                                        
                                        if !is_valid || !starts_valid {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Invalid environment variable name: '{}'. Environment variable names must start with an uppercase letter or underscore and contain only uppercase letters, numbers, and underscores.",
                                                    key_cleaned
                                                ),
                                                severity: Severity::Error,
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

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

