use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates concurrency syntax.
pub struct ConcurrencyRule;

impl ValidationRule for ConcurrencyRule {
    fn name(&self) -> &str {
        "concurrency"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        let root = tree.root_node();
        
        // Check workflow-level concurrency
        let workflow_concurrency = utils::find_value_for_key(root, source, "concurrency");
        if let Some(concurrency_node) = workflow_concurrency {
            validate_concurrency_node(concurrency_node, source, "workflow", &mut diagnostics);
        }
        
        // Check job-level concurrency
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        let mut jobs_to_process = jobs_value;
        if jobs_to_process.kind() == "block_node" {
            if let Some(inner) = jobs_to_process.child(0) {
                jobs_to_process = inner;
            }
        }

        fn process_jobs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let job_name = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();
                        
                        let job_value = if node.kind() == "block_mapping_pair" {
                            node.child(2)
                        } else {
                            node.child(1)
                        };
                        
                        if let Some(mut job_value) = job_value {
                            if job_value.kind() == "block_node" {
                                if let Some(inner) = job_value.child(0) {
                                    job_value = inner;
                                }
                            }
                            
                            if job_value.kind() == "block_mapping" || job_value.kind() == "flow_mapping" {
                                let concurrency_node = utils::find_value_for_key(job_value, source, "concurrency");
                                if let Some(concurrency) = concurrency_node {
                                    validate_concurrency_node(concurrency, source, &format!("job '{}'", job_name), diagnostics);
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

        process_jobs(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

fn validate_concurrency_node(concurrency_node: Node, source: &str, context: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut concurrency_to_check = concurrency_node;
    if concurrency_to_check.kind() == "block_node" {
        if let Some(inner) = concurrency_to_check.child(0) {
            concurrency_to_check = inner;
        }
    }
    
    // Check if group is present and valid
    let group_value = utils::find_value_for_key(concurrency_to_check, source, "group");
    
    if group_value.is_none() {
        diagnostics.push(Diagnostic {
            message: format!(
                "Concurrency at {} level is missing required 'group' field.",
                context
            ),
            severity: Severity::Error,
            span: Span {
                start: concurrency_to_check.start_byte(),
                end: concurrency_to_check.end_byte(),
            },
        });
        return;
    }
    
    let group_node = group_value.unwrap();
    let group_text = utils::node_text(group_node, source);
    let group_cleaned = group_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
    
    // Group should be string or expression (starts with ${{)
    if !group_cleaned.starts_with("${{") {
        // Try to parse as number - if it succeeds, it's invalid
        if group_cleaned.parse::<f64>().is_ok() && !group_cleaned.contains('.') {
            // It's a number, which is invalid
            diagnostics.push(Diagnostic {
                message: format!(
                    "Concurrency 'group' at {} level must be a string or expression, not a number.",
                    context
                ),
                severity: Severity::Error,
                span: Span {
                    start: group_node.start_byte(),
                    end: group_node.end_byte(),
                },
            });
        }
    }
    
    // Check cancel-in-progress if present
    let cancel_value = utils::find_value_for_key(concurrency_to_check, source, "cancel-in-progress");
    
    if let Some(cancel_node) = cancel_value {
        let cancel_text = utils::node_text(cancel_node, source);
        
        // Check if it's a quoted scalar (string) - this is invalid
        // Check both node kind and if text starts/ends with quotes
        let is_quoted_string = cancel_node.kind() == "double_quoted_scalar" || 
                              cancel_node.kind() == "single_quoted_scalar" ||
                              (cancel_text.trim().starts_with('"') && cancel_text.trim().ends_with('"')) ||
                              (cancel_text.trim().starts_with('\'') && cancel_text.trim().ends_with('\''));
        
        if is_quoted_string {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Concurrency 'cancel-in-progress' at {} level must be a boolean (true/false), not a string.",
                    context
                ),
                severity: Severity::Error,
                span: Span {
                    start: cancel_node.start_byte(),
                    end: cancel_node.end_byte(),
                },
            });
        } else {
            // Check the actual value
            let cancel_cleaned = cancel_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
            
            // cancel-in-progress should be boolean (true/false)
            if cancel_cleaned != "true" && cancel_cleaned != "false" {
                // Try to parse as boolean
                if cancel_cleaned.parse::<bool>().is_err() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Concurrency 'cancel-in-progress' at {} level must be a boolean (true/false).",
                            context
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: cancel_node.start_byte(),
                            end: cancel_node.end_byte(),
                        },
                    });
                }
            }
        }
    }
}

