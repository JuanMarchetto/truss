use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates action reference format (owner/repo@ref).
pub struct ActionReferenceRule;

impl ValidationRule for ActionReferenceRule {
    fn name(&self) -> &str {
        "action_reference"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn find_steps_with_uses(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');

                        if key_cleaned == "steps" {
                            let steps_value = utils::get_pair_value(node);

                            if let Some(steps_value_raw) = steps_value {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                fn process_steps_sequence(
                                    seq_node: Node,
                                    source: &str,
                                    diagnostics: &mut Vec<Diagnostic>,
                                ) {
                                    let mut cursor = seq_node.walk();
                                    for step_item in seq_node.children(&mut cursor) {
                                        if step_item.kind() == "block_mapping"
                                            || step_item.kind() == "flow_mapping"
                                        {
                                            let uses_value = utils::find_value_for_key(
                                                step_item, source, "uses",
                                            );
                                            if let Some(uses_node) = uses_value {
                                                validate_action_reference(
                                                    uses_node,
                                                    source,
                                                    diagnostics,
                                                );
                                            }
                                        } else {
                                            process_steps_sequence(step_item, source, diagnostics);
                                        }
                                    }
                                }

                                if steps_value.kind() == "block_sequence"
                                    || steps_value.kind() == "flow_sequence"
                                {
                                    process_steps_sequence(steps_value, source, diagnostics);
                                }
                            }
                        } else {
                            let value_node = utils::get_pair_value(node);

                            if let Some(value_node) = value_node {
                                find_steps_with_uses(value_node, source, diagnostics);
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps_with_uses(child, source, diagnostics);
                    }
                }
            }
        }

        find_steps_with_uses(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

fn validate_action_reference(uses_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let uses_text = utils::node_text(uses_node, source);
    let uses_cleaned = uses_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

    // Exceptions: local paths and docker actions don't need @ref
    if uses_cleaned.starts_with("./")
        || uses_cleaned.starts_with("../")
        || uses_cleaned.starts_with("/")
    {
        return;
    }

    if uses_cleaned.starts_with("docker://") {
        return;
    }

    if !uses_cleaned.contains('@') {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' is missing required '@ref'. Remote actions must specify a version, branch, or SHA (e.g., owner/repo@v1).",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
        return;
    }

    let parts: Vec<&str> = uses_cleaned.split('@').collect();
    if parts.len() != 2 {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' has invalid format. Expected format: owner/repo@ref",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
        return;
    }

    let owner_repo = parts[0];
    let _ref = parts[1];

    if !owner_repo.contains('/') {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' is missing owner. Expected format: owner/repo@ref (e.g., actions/checkout@v3)",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
        return;
    }

    let owner_repo_parts: Vec<&str> = owner_repo.split('/').collect();
    if owner_repo_parts.len() != 2 {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' has invalid format. Expected format: owner/repo@ref",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
        return;
    }

    let owner = owner_repo_parts[0];
    let repo = owner_repo_parts[1];

    if owner.contains(' ') || owner.is_empty() {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' has invalid owner format. Owner cannot contain spaces or be empty.",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
    }

    if repo.is_empty() {
        diagnostics.push(Diagnostic {
            message: format!(
                "action reference '{}' has invalid format. Repository name cannot be empty.",
                uses_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: uses_node.start_byte(),
                end: uses_node.end_byte(),
            },
        });
    }
}
