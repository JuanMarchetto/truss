use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::HashSet;
use tree_sitter::{Node, Tree};

/// Validates workflow_call secrets and their usage.
pub struct WorkflowCallSecretsRule;

impl ValidationRule for WorkflowCallSecretsRule {
    fn name(&self) -> &str {
        "workflow_call_secrets"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => return diagnostics,
        };

        let on_to_check = utils::unwrap_node(on_value);

        // Only validate secrets for reusable workflows (with workflow_call)
        // Regular workflows can use secrets.GITHUB_TOKEN, secrets.MY_SECRET, etc. without errors
        if !utils::key_exists(on_to_check, source, "workflow_call") {
            return diagnostics;
        }

        // Extract defined secrets (workflow_call may exist with or without a value)
        let mut defined_secrets: HashSet<String> = HashSet::new();

        if let Some(workflow_call) = utils::find_value_for_key(on_to_check, source, "workflow_call")
        {
            let call_to_check = utils::unwrap_node(workflow_call);
            let secrets_value = utils::find_value_for_key(call_to_check, source, "secrets");

            if let Some(secrets_node) = secrets_value {
                let secrets_to_check = utils::unwrap_node(secrets_node);
                self.collect_secret_definitions(secrets_to_check, source, &mut defined_secrets);
                self.validate_secret_properties(secrets_to_check, source, &mut diagnostics);
            }
        }

        // Find all secrets.* references in expressions
        let secret_references = self.find_secret_references(source);

        // Validate that all referenced secrets are defined
        for (secret_name, span) in secret_references {
            // If no secrets are defined but secrets are referenced, that's an error
            if defined_secrets.is_empty() {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Secret '{}' is referenced but workflow_call has no secrets defined.",
                        secret_name
                    ),
                    severity: Severity::Error,
                    span,
                });
            } else if !defined_secrets.contains(&secret_name) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Reference to undefined workflow_call secret '{}'. Available secrets: {}",
                        secret_name,
                        defined_secrets
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    severity: Severity::Error,
                    span,
                });
            }
        }

        diagnostics
    }
}

impl WorkflowCallSecretsRule {
    fn collect_secret_definitions(&self, node: Node, source: &str, secrets: &mut HashSet<String>) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let secret_name = utils::clean_key(key_node, source).to_string();
                    secrets.insert(secret_name);
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.collect_secret_definitions(child, source, secrets);
                }
            }
        }
    }

    fn validate_secret_properties(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let secret_name = utils::clean_key(key_node, source).to_string();

                    let secret_value = utils::get_pair_value(node);

                    if let Some(secret_value_raw) = secret_value {
                        let secret_value = utils::unwrap_node(secret_value_raw);

                        // Validate required field (must be boolean)
                        let required_value =
                            utils::find_value_for_key(secret_value, source, "required");
                        if let Some(required_node) = required_value {
                            let required_text = utils::node_text(required_node, source);
                            let required_cleaned = required_text
                                .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                            if required_cleaned != "true"
                                && required_cleaned != "false"
                                && !required_cleaned.starts_with("${{")
                            {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Secret '{}' has invalid 'required' value: '{}'. 'required' must be a boolean (true or false).",
                                        secret_name, required_cleaned
                                    ),
                                    severity: Severity::Error,
                                    span: Span {
                                        start: required_node.start_byte(),
                                        end: required_node.end_byte(),
                                    },
                                    rule_id: String::new(),
                                });
                            }
                        }

                        // Validate description (should be a string)
                        let description_value =
                            utils::find_value_for_key(secret_value, source, "description");
                        if let Some(description_node) = description_value {
                            let description_text = utils::node_text(description_node, source);
                            // Description should be a string (basic validation)
                            if description_text.trim().is_empty()
                                && !description_text.starts_with("${{")
                            {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Secret '{}' has empty description. Consider adding a description to document the secret.",
                                        secret_name
                                    ),
                                    severity: Severity::Warning,
                                    span: Span {
                                        start: description_node.start_byte(),
                                        end: description_node.end_byte(),
                                    },
                                    rule_id: String::new(),
                                });
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.validate_secret_properties(child, source, diagnostics);
                }
            }
        }
    }

    fn find_secret_references(&self, source: &str) -> Vec<(String, Span)> {
        let mut references = Vec::new();

        for expr in utils::find_expressions(source) {
            let mut search_pos = 0;

            while let Some(pos) =
                utils::find_ignore_ascii_case(&expr.inner[search_pos..], "secrets.")
            {
                let actual_pos = search_pos + pos;
                let after_secrets = &expr.inner[actual_pos + 8..];

                let name_end = after_secrets
                    .find(|c: char| {
                        c.is_whitespace()
                            || c == '}'
                            || c == ')'
                            || c == ']'
                            || c == '&'
                            || c == '|'
                            || c == '='
                            || c == '!'
                            || c == '<'
                            || c == '>'
                            || c == '.'
                    })
                    .unwrap_or(after_secrets.len());

                let secret_name = &after_secrets[..name_end.min(after_secrets.len())];

                if !secret_name.is_empty() {
                    let name_start = expr.start + 3 + actual_pos + 8;
                    references.push((
                        secret_name.to_string(),
                        Span {
                            start: name_start,
                            end: name_start + name_end,
                        },
                    ));
                }

                search_pos = actual_pos + 8 + name_end;
            }
        }

        references
    }
}
