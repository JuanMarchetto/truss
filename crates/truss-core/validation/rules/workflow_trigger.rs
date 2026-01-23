use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;
use super::super::ValidationRule;
use super::super::utils;

/// Validates workflow trigger configuration.
pub struct WorkflowTriggerRule;

impl ValidationRule for WorkflowTriggerRule {
    fn name(&self) -> &str {
        "workflow_trigger"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

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

        let mut on_to_check = on_value;
        if on_to_check.kind() == "block_node" {
            if let Some(inner) = on_to_check.child(0) {
                on_to_check = inner;
            }
        }

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

        let mut event_node = on_to_check;
        if event_node.kind() == "block_node" {
            if let Some(inner) = event_node.child(0) {
                event_node = inner;
            }
        }

        let event_text = if event_node.kind() == "plain_scalar" || event_node.kind() == "double_quoted_scalar" || event_node.kind() == "single_quoted_scalar" {
            Some(utils::node_text(event_node, source).trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace()).to_lowercase())
        } else {
            let text = utils::node_text(event_node, source).trim().to_lowercase();
            if !text.is_empty() && !text.contains('\n') {
                Some(text)
            } else {
                None
            }
        };
        
        if let Some(event_text) = event_text {
            let valid_events = ["push", "pull_request", "workflow_dispatch", "schedule", "repository_dispatch", "workflow_run", "workflow_call"];
            if !valid_events.contains(&event_text.as_str()) && !event_text.is_empty() {
                if !event_text.contains(":") && !event_text.contains("[") {
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
        }

        diagnostics
    }
}

