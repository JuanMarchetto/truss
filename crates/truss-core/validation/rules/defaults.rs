use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates defaults configuration at workflow and job levels.
pub struct DefaultsValidationRule;

impl ValidationRule for DefaultsValidationRule {
    fn name(&self) -> &str {
        "defaults"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();

        // Check workflow-level defaults
        let defaults_value = utils::find_value_for_key(root, source, "defaults");
        if let Some(defaults_node) = defaults_value {
            validate_defaults(defaults_node, source, "workflow", &mut diagnostics);
        }

        // Check job-level defaults
        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        fn check_job_defaults(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::clean_key(key_node, source).to_string();

                        if let Some(job_value_raw) = utils::get_pair_value(node) {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                let defaults_value =
                                    utils::find_value_for_key(job_value, source, "defaults");
                                if let Some(defaults_node) = defaults_value {
                                    validate_defaults(
                                        defaults_node,
                                        source,
                                        &format!("job '{}'", job_name),
                                        diagnostics,
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        check_job_defaults(child, source, diagnostics);
                    }
                }
            }
        }

        check_job_defaults(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}

fn validate_defaults(
    defaults_node: Node,
    source: &str,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let defaults_to_check = utils::unwrap_node(defaults_node);

    // Check defaults.run.shell
    let run_value = utils::find_value_for_key(defaults_to_check, source, "run");
    if let Some(run_value_raw) = run_value {
        let run_node = utils::unwrap_node(run_value_raw);

        let shell_value = utils::find_value_for_key(run_node, source, "shell");
        if let Some(shell_node) = shell_value {
            let shell_text = utils::node_text(shell_node, source);
            let shell_cleaned =
                shell_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

            if !shell_cleaned.starts_with("${{") {
                let known_shells = ["bash", "pwsh", "python", "sh", "cmd", "powershell"];
                let is_known = known_shells.contains(&shell_cleaned.to_lowercase().as_str());
                let is_custom = shell_cleaned.contains("{0}");

                if !is_known && !is_custom && !shell_cleaned.is_empty() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "{} defaults.run.shell has invalid value: '{}'. Valid shells are: bash, pwsh, python, sh, cmd, powershell, or a custom command with {{0}} placeholder.",
                            context, shell_cleaned
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: shell_node.start_byte(),
                            end: shell_node.end_byte(),
                        },
                    });
                }
            }
        }

        // Check defaults.run.working-directory
        let working_dir_value = utils::find_value_for_key(run_node, source, "working-directory");
        if let Some(working_dir_node) = working_dir_value {
            let working_dir_text = utils::node_text(working_dir_node, source);
            let working_dir_cleaned =
                working_dir_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

            if !working_dir_cleaned.starts_with("${{") && working_dir_cleaned.is_empty() {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "{} defaults.run.working-directory is empty. working-directory must be a valid path.",
                        context
                    ),
                    severity: Severity::Error,
                    span: Span {
                        start: working_dir_node.start_byte(),
                        end: working_dir_node.end_byte(),
                    },
                });
            }
        }
    }
}
