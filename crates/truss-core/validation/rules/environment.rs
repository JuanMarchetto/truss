use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates environment references.
pub struct EnvironmentRule;

impl ValidationRule for EnvironmentRule {
    fn name(&self) -> &str {
        "environment"
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

        fn find_environment_refs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "environment" {
                            let env_value = utils::get_pair_value(node);
                            if let Some(env_value_raw) = env_value {
                                let actual_env_value = utils::unwrap_node(env_value_raw);
                                {
                                    match actual_env_value.kind() {
                                        "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                                            let env_name = utils::node_text(actual_env_value, source);
                                            let env_name_cleaned = env_name.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                            if env_name_cleaned.contains(' ') && !env_name_cleaned.contains("${{") {
                                                diagnostics.push(Diagnostic {
                                                    message: format!("Invalid environment name format: '{}' (contains spaces)", env_name_cleaned),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: actual_env_value.start_byte(),
                                                        end: actual_env_value.end_byte(),
                                                    },
                                                });
                                            }
                                        }
                                        "block_mapping" | "flow_mapping" => {
                                            let mut cursor = actual_env_value.walk();
                                            for child in actual_env_value.children(&mut cursor) {
                                                if child.kind() == "block_mapping_pair" || child.kind() == "flow_pair" {
                                                    let field_key_node = child.child(0);
                                                    let field_value_node = utils::get_pair_value(child);
                                                    
                                                    if let Some(field_key_node) = field_key_node {
                                                        let field_name = utils::node_text(field_key_node, source);
                                                        let field_cleaned = field_name.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                                            .trim_end_matches(':');
                                                        
                                                        if field_cleaned == "protection_rules" {
                                                            diagnostics.push(Diagnostic {
                                                                message: "environment protection_rules is not supported in workflow YAML".to_string(),
                                                                severity: Severity::Error,
                                                                span: Span {
                                                                    start: field_key_node.start_byte(),
                                                                    end: field_key_node.end_byte(),
                                                                },
                                                            });
                                                        }
                                                        
                                                        if field_cleaned == "name" {
                                                            if let Some(name_value) = field_value_node {
                                                                let name_text = utils::node_text(name_value, source);
                                                                let name_cleaned = name_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                                                if name_cleaned.contains(' ') && !name_cleaned.contains("${{") {
                                                                    diagnostics.push(Diagnostic {
                                                                        message: format!("Invalid environment name format: '{}' (contains spaces)", name_cleaned),
                                                                        severity: Severity::Error,
                                                                        span: Span {
                                                                            start: name_value.start_byte(),
                                                                            end: name_value.end_byte(),
                                                                        },
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        if let Some(value_node) = utils::get_pair_value(node) {
                            find_environment_refs(value_node, source, diagnostics);
                        }
                    } else {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            find_environment_refs(child, source, diagnostics);
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_environment_refs(child, source, diagnostics);
                    }
                }
            }
        }

        find_environment_refs(jobs_value, source, &mut diagnostics);

        diagnostics
    }
}

