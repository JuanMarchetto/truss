use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates runs-on labels are valid GitHub-hosted runners or self-hosted runner groups.
pub struct RunnerLabelRule;

impl ValidationRule for RunnerLabelRule {
    fn name(&self) -> &str {
        "runner_label"
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

        let mut jobs_to_process = jobs_value;
        if jobs_to_process.kind() == "block_node" {
            if let Some(inner) = jobs_to_process.child(0) {
                jobs_to_process = inner;
            }
        }

        fn check_job_runs_on(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
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
                                let runs_on_value = utils::find_value_for_key(job_value, source, "runs-on");
                                
                                if let Some(runs_on_node) = runs_on_value {
                                    let runs_on_text = utils::node_text(runs_on_node, source);
                                    let runs_on_cleaned = runs_on_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                    
                                    // Check if it's an expression
                                    if runs_on_cleaned.starts_with("${{") {
                                        // Expressions are valid
                                        return;
                                    }
                                    
                                    // Known GitHub-hosted runners
                                    let known_runners = [
                                        "ubuntu-latest", "ubuntu-22.04", "ubuntu-20.04",
                                        "windows-latest", "windows-2022", "windows-2019",
                                        "macos-latest", "macos-13", "macos-12",
                                    ];
                                    
                                    let is_known = known_runners.iter().any(|&r| r == runs_on_cleaned);
                                    let is_self_hosted = runs_on_cleaned == "self-hosted" || runs_on_cleaned.starts_with("self-hosted[");
                                    
                                    if !is_known && !is_self_hosted {
                                        // Basic format validation - warn on potentially invalid labels
                                        if runs_on_cleaned.is_empty() {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has empty runs-on label.",
                                                    job_name
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: runs_on_node.start_byte(),
                                                    end: runs_on_node.end_byte(),
                                                },
                                            });
                                        } else {
                                            // Warn on unknown runner labels (might be valid self-hosted or custom)
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' uses unknown runner label: '{}'. This may be a valid self-hosted runner or custom label.",
                                                    job_name, runs_on_cleaned
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: runs_on_node.start_byte(),
                                                    end: runs_on_node.end_byte(),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        check_job_runs_on(child, source, diagnostics);
                    }
                }
            }
        }

        check_job_runs_on(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

