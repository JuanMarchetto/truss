use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// All valid GitHub Actions event types.
const VALID_EVENTS: &[&str] = &[
    "push",
    "pull_request",
    "pull_request_target",
    "pull_request_review",
    "pull_request_review_comment",
    "issues",
    "issue_comment",
    "label",
    "milestone",
    "project",
    "project_card",
    "project_column",
    "repository_dispatch",
    "workflow_dispatch",
    "workflow_call",
    "workflow_run",
    "schedule",
    "watch",
    "fork",
    "create",
    "delete",
    "deployment",
    "deployment_status",
    "page_build",
    "public",
    "registry_package",
    "release",
    "status",
    "check_run",
    "check_suite",
    "discussion",
    "discussion_comment",
    "gollum",
    "merge_group",
    "branch_protection_rule",
];

/// Validates workflow trigger configuration.
pub struct WorkflowTriggerRule;

impl ValidationRule for WorkflowTriggerRule {
    fn name(&self) -> &str {
        "workflow_trigger"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();

        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => {
                diagnostics.push(Diagnostic {
                    message: "Workflow must have an 'on' field".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: 0,
                        end: source.len().min(100),
                    },
                });
                return diagnostics;
            }
        };

        let on_to_check = utils::unwrap_node(on_value);

        let on_text = utils::node_text(on_to_check, source);
        if on_text.contains(", ]") || on_text.contains(",]") {
            diagnostics.push(Diagnostic {
                message: "Invalid trigger syntax: empty array item".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: on_to_check.start_byte(),
                    end: on_to_check.end_byte(),
                },
            });
        }

        let event_node = utils::unwrap_node(on_to_check);

        let event_text = if event_node.kind() == "plain_scalar"
            || event_node.kind() == "double_quoted_scalar"
            || event_node.kind() == "single_quoted_scalar"
        {
            Some(
                utils::node_text(event_node, source)
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .to_lowercase(),
            )
        } else {
            let text = utils::node_text(event_node, source).trim().to_lowercase();
            if !text.is_empty() && !text.contains('\n') {
                Some(text)
            } else {
                None
            }
        };

        // Validate event types - check all possible event types in the on: mapping
        fn validate_event_types(
            node: tree_sitter::Node,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let event_type = utils::clean_key(key_node, source).to_lowercase();

                        if !VALID_EVENTS.contains(&event_type.as_str()) && !event_type.is_empty() {
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "Invalid event type: '{}'. Valid event types include: push, pull_request, workflow_dispatch, schedule, workflow_call, and others.",
                                    event_type
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
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        validate_event_types(child, source, diagnostics);
                    }
                }
            }
        }

        // Validate event types in the on: mapping
        validate_event_types(on_to_check, source, &mut diagnostics);

        // Also validate simple event text if present (scalar on: value)
        if let Some(event_text) = event_text {
            if !VALID_EVENTS.contains(&event_text.as_str())
                && !event_text.is_empty()
                && !event_text.contains(':')
                && !event_text.contains('[')
            {
                diagnostics.push(Diagnostic {
                    message: format!("Invalid event type: '{}'", event_text),
                    severity: Severity::Error,
                    span: Span {
                        start: event_node.start_byte(),
                        end: event_node.end_byte(),
                    },
                });
            }
        }

        diagnostics
    }
}
