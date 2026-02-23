use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

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

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn check_job_runs_on(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let job_name = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();

                        // Get value node: last non-comment, non-":" child
                        let mut job_value_opt = None;
                        for i in (1..node.child_count()).rev() {
                            if let Some(child) = node.child(i) {
                                if child.kind() != "comment" && child.kind() != ":" {
                                    job_value_opt = Some(child);
                                    break;
                                }
                            }
                        }

                        if let Some(job_value_raw) = job_value_opt {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                let runs_on_value =
                                    utils::find_value_for_key(job_value, source, "runs-on");

                                if let Some(runs_on_node) = runs_on_value {
                                    let unwrapped = utils::unwrap_node(runs_on_node);

                                    // runs-on can be a sequence (array of labels) for self-hosted runners
                                    // e.g., runs-on: [self-hosted, linux, x64]
                                    if unwrapped.kind() == "block_sequence"
                                        || unwrapped.kind() == "flow_sequence"
                                    {
                                        // Array-style runs-on: check if any label is "self-hosted"
                                        let full_text = utils::node_text(unwrapped, source);
                                        if full_text.contains("self-hosted") {
                                            return; // Self-hosted runner arrays are always valid
                                        }
                                        // Custom runner group arrays are also valid
                                        return;
                                    }

                                    let runs_on_text = utils::node_text(runs_on_node, source);
                                    let runs_on_cleaned = runs_on_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });

                                    // Check if it's an expression
                                    if runs_on_cleaned.starts_with("${{") {
                                        // Expressions are valid
                                        return;
                                    }

                                    // Known GitHub-hosted runners (including ARM and latest versions)
                                    let known_runners = [
                                        "ubuntu-latest",
                                        "ubuntu-24.04",
                                        "ubuntu-22.04",
                                        "ubuntu-20.04",
                                        "ubuntu-latest-arm64",
                                        "ubuntu-24.04-arm",
                                        "ubuntu-22.04-arm",
                                        "windows-latest",
                                        "windows-2025",
                                        "windows-2022",
                                        "windows-2019",
                                        "windows-latest-arm64",
                                        "macos-latest",
                                        "macos-15",
                                        "macos-14",
                                        "macos-13",
                                        "macos-12",
                                        "macos-latest-xlarge",
                                        "macos-15-xlarge",
                                        "macos-14-xlarge",
                                        "macos-13-xlarge",
                                    ];

                                    let is_known = known_runners.contains(&runs_on_cleaned);
                                    let is_self_hosted = runs_on_cleaned == "self-hosted"
                                        || runs_on_cleaned.starts_with("self-hosted[");

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
