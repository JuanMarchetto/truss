use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates permissions configuration.
pub struct PermissionsRule;

impl ValidationRule for PermissionsRule {
    fn name(&self) -> &str {
        "permissions"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        let root = tree.root_node();
        
        let valid_scopes = [
            "actions", "checks", "contents", "deployments", "id-token", "issues",
            "discussions", "packages", "pages", "pull-requests", "repository-projects",
            "security-events", "statuses", "write-all", "read-all", "none"
        ];
        
        let valid_values = ["read", "write", "none"];
        
        fn validate_permissions_node(node: Node, source: &str, valid_scopes: &[&str], valid_values: &[&str], diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                    let text = utils::node_text(node, source);
                    let cleaned = text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    if cleaned != "read-all" && cleaned != "write-all" && cleaned != "none" {
                        diagnostics.push(Diagnostic {
                            message: format!("Invalid permission value: '{}' (must be 'read-all', 'write-all', or 'none')", cleaned),
                            severity: Severity::Error,
                            span: Span {
                                start: node.start_byte(),
                                end: node.end_byte(),
                            },
                        });
                    }
                }
                "block_mapping" | "flow_mapping" | "block_node" => {
                    let mut node_to_process = node;
                    if node.kind() == "block_node" {
                        if let Some(inner) = node.child(0) {
                            node_to_process = inner;
                        }
                    }
                    
                    let mut cursor = node_to_process.walk();
                    for child in node_to_process.children(&mut cursor) {
                        if child.kind() == "block_mapping_pair" || child.kind() == "flow_pair" {
                            if let Some(key_node) = child.child(0) {
                                let scope = utils::node_text(key_node, source);
                                let scope_cleaned = scope.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                    .trim_end_matches(':');
                                if !valid_scopes.iter().any(|&s| s == scope_cleaned) {
                                    diagnostics.push(Diagnostic {
                                        message: format!("Invalid permission scope: '{}'", scope_cleaned),
                                        severity: Severity::Error,
                                        span: Span {
                                            start: key_node.start_byte(),
                                            end: key_node.end_byte(),
                                        },
                                    });
                                }
                                
                                let value_node = if child.kind() == "block_mapping_pair" {
                                    child.child(2)  // block_mapping_pair: child(0)=key, child(1)=colon, child(2)=value
                                } else {
                                    child.child(1)  // flow_pair: child(0)=key, child(1)=value
                                };
                                if let Some(mut value_node) = value_node {
                                    if value_node.kind() == "block_node" {
                                        if let Some(inner) = value_node.child(0) {
                                            value_node = inner;
                                        }
                                    }
                                    let value_text = utils::node_text(value_node, source);
                                    let value_cleaned = value_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                    if !valid_values.iter().any(|&v| v == value_cleaned) {
                                        diagnostics.push(Diagnostic {
                                            message: format!("Invalid permission value: '{}' (must be 'read', 'write', or 'none')", value_cleaned),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: value_node.start_byte(),
                                                end: value_node.end_byte(),
                                            },
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        if let Some(permissions_value) = utils::find_value_for_key(root, source, "permissions") {
            validate_permissions_node(permissions_value, source, &valid_scopes, &valid_values, &mut diagnostics);
        }
        
        if let Some(jobs_value) = utils::find_value_for_key(root, source, "jobs") {
            fn find_job_permissions(node: Node, source: &str, valid_scopes: &[&str], valid_values: &[&str], diagnostics: &mut Vec<Diagnostic>) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_text = utils::node_text(key_node, source);
                            let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                .trim_end_matches(':');
                            if key_cleaned == "permissions" {
                                if let Some(perm_value) = node.child(1) {
                                    validate_permissions_node(perm_value, source, valid_scopes, valid_values, diagnostics);
                                }
                            }
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            find_job_permissions(child, source, valid_scopes, valid_values, diagnostics);
                        }
                    }
                }
            }
            
            find_job_permissions(jobs_value, source, &valid_scopes, &valid_values, &mut diagnostics);
        }

        diagnostics
    }
}

