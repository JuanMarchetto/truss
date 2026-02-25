use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates permissions configuration.
pub struct PermissionsRule;

impl ValidationRule for PermissionsRule {
    fn name(&self) -> &str {
        "permissions"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();

        let valid_scopes = [
            "actions",
            "attestations",
            "checks",
            "contents",
            "deployments",
            "id-token",
            "issues",
            "discussions",
            "packages",
            "pages",
            "pull-requests",
            "repository-projects",
            "security-events",
            "statuses",
            "workflows",
        ];

        let valid_values = ["read", "write", "none"];

        fn validate_permissions_node(
            node: Node,
            source: &str,
            valid_scopes: &[&str],
            valid_values: &[&str],
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            match node.kind() {
                "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                    let text = utils::node_text(node, source);
                    let cleaned =
                        text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    if cleaned != "read-all" && cleaned != "write-all" && cleaned != "none" {
                        diagnostics.push(Diagnostic {
                            message: format!("Invalid permission value: '{}' (must be 'read-all', 'write-all', or 'none')", cleaned),
                            severity: Severity::Error,
                            span: Span {
                                start: node.start_byte(),
                                end: node.end_byte(),
                            },
                            rule_id: String::new(),
                        });
                    }
                }
                "block_mapping" | "flow_mapping" | "block_node" => {
                    let node_to_process = utils::unwrap_node(node);

                    let mut cursor = node_to_process.walk();
                    for child in node_to_process.children(&mut cursor) {
                        if child.kind() == "block_mapping_pair" || child.kind() == "flow_pair" {
                            if let Some(key_node) = child.child(0) {
                                let scope = utils::node_text(key_node, source);
                                let scope_cleaned = scope
                                    .trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    })
                                    .trim_end_matches(':');
                                if !valid_scopes.contains(&scope_cleaned) {
                                    diagnostics.push(Diagnostic {
                                        message: format!(
                                            "Invalid permission scope: '{}'",
                                            scope_cleaned
                                        ),
                                        severity: Severity::Error,
                                        span: Span {
                                            start: key_node.start_byte(),
                                            end: key_node.end_byte(),
                                        },
                                        rule_id: String::new(),
                                    });
                                }

                                if let Some(value_node_raw) = utils::get_pair_value(child) {
                                    let value_node = utils::unwrap_node(value_node_raw);
                                    let value_text = utils::node_text(value_node, source);
                                    let value_cleaned = value_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });
                                    if !valid_values.contains(&value_cleaned) {
                                        diagnostics.push(Diagnostic {
                                            message: format!("Invalid permission value: '{}' (must be 'read', 'write', or 'none')", value_cleaned),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: value_node.start_byte(),
                                                end: value_node.end_byte(),
                                            },
                                            rule_id: String::new(),
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
            validate_permissions_node(
                permissions_value,
                source,
                &valid_scopes,
                &valid_values,
                &mut diagnostics,
            );
        }

        if let Some(jobs_value) = utils::find_value_for_key(root, source, "jobs") {
            fn find_job_permissions(
                node: Node,
                source: &str,
                valid_scopes: &[&str],
                valid_values: &[&str],
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_cleaned = utils::clean_key(key_node, source);
                            if key_cleaned == "permissions" {
                                if let Some(perm_value) = utils::get_pair_value(node) {
                                    validate_permissions_node(
                                        perm_value,
                                        source,
                                        valid_scopes,
                                        valid_values,
                                        diagnostics,
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            find_job_permissions(
                                child,
                                source,
                                valid_scopes,
                                valid_values,
                                diagnostics,
                            );
                        }
                    }
                }
            }

            find_job_permissions(
                jobs_value,
                source,
                &valid_scopes,
                &valid_values,
                &mut diagnostics,
            );
        }

        diagnostics
    }
}
