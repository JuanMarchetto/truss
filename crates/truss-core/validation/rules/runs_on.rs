use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates that `runs-on` is required for all jobs.
pub struct RunsOnRequiredRule;

impl ValidationRule for RunsOnRequiredRule {
    fn name(&self) -> &str {
        "runs_on_required"
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

        // Process jobs and check for runs-on directly
        fn check_job_runs_on(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');

                        // Get the job value node
                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value_raw) = job_value {
                            let job_value = utils::unwrap_node(job_value_raw);

                            // Only consider it a job if the value is a mapping (job definition)
                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                // This is a job definition, check for runs-on
                                let runs_on_value =
                                    utils::find_value_for_key(job_value, source, "runs-on");

                                match runs_on_value {
                                    Some(value_node) => {
                                        // runs-on exists, check if it's empty
                                        let text = utils::node_text(value_node, source);
                                        let cleaned = text.trim_matches(|c: char| {
                                            c == '"' || c == '\'' || c.is_whitespace()
                                        });
                                        if cleaned.is_empty() {
                                            diagnostics.push(Diagnostic {
                                                message: format!("Job '{}' has empty 'runs-on' field. 'runs-on' is required and cannot be empty.", key_cleaned),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: value_node.start_byte(),
                                                    end: value_node.end_byte(),
                                                },
                                            });
                                        }
                                    }
                                    None => {
                                        // runs-on is missing
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' is missing required 'runs-on' field.",
                                                key_cleaned
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: key_node.start_byte(),
                                                end: job_value.end_byte(),
                                            },
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Continue traversing
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        check_job_runs_on(child, source, diagnostics);
                    }
                }
            }
        }

        let jobs_to_process = utils::unwrap_node(jobs_value);

        check_job_runs_on(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}
