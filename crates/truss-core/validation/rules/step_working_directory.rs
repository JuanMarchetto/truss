use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates working-directory paths.
pub struct StepWorkingDirectoryRule;

impl ValidationRule for StepWorkingDirectoryRule {
    fn name(&self) -> &str {
        "step_working_directory"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_cleaned = utils::clean_key(key_node, source);
                        if key_cleaned == "steps" {
                            if let Some(steps_value_raw) = utils::get_pair_value(node) {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_sequence"
                                    || steps_value.kind() == "flow_sequence"
                                {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_working_directory(
                                            step_node,
                                            source,
                                            diagnostics,
                                        );
                                    }
                                }
                            }
                        }
                        if let Some(value_node) = utils::get_pair_value(node) {
                            find_steps(value_node, source, diagnostics);
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps(child, source, diagnostics);
                    }
                }
            }
        }

        fn validate_step_working_directory(
            step_node: Node,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let step_to_check = utils::unwrap_node(step_node);

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let working_dir_value =
                    utils::find_value_for_key(step_to_check, source, "working-directory");

                if let Some(working_dir_node) = working_dir_value {
                    let working_dir_text = utils::node_text(working_dir_node, source);
                    let working_dir_cleaned = working_dir_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

                    // Check if it's an expression
                    if working_dir_cleaned.starts_with("${{") {
                        // Expressions are valid
                        return;
                    }

                    // Basic format validation
                    if working_dir_cleaned.is_empty() {
                        diagnostics.push(Diagnostic {
                            message: "Step has empty working-directory. working-directory must be a valid path.".to_string(),
                            severity: Severity::Error,
                            span: Span {
                                start: working_dir_node.start_byte(),
                                end: working_dir_node.end_byte(),
                            },
                            rule_id: String::new(),
                        });
                    } else {
                        // Basic path format validation
                        // Warn about potentially invalid paths
                        if working_dir_cleaned.starts_with("/")
                            && !working_dir_cleaned.starts_with("/home")
                            && !working_dir_cleaned.starts_with("/github")
                        {
                            // Absolute paths that don't look like standard GitHub Actions paths
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "Step working-directory '{}' is an absolute path that may not exist in the GitHub Actions runner environment. Consider using a relative path.",
                                    working_dir_cleaned
                                ),
                                severity: Severity::Warning,
                                span: Span {
                                    start: working_dir_node.start_byte(),
                                    end: working_dir_node.end_byte(),
                                },
                                rule_id: String::new(),
                            });
                        }

                        // Warn about paths with invalid characters (basic check)
                        if working_dir_cleaned.contains("..")
                            && working_dir_cleaned != ".."
                            && !working_dir_cleaned.starts_with("../")
                        {
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "Step working-directory '{}' contains '..' in an unusual position. Verify the path is correct.",
                                    working_dir_cleaned
                                ),
                                severity: Severity::Warning,
                                span: Span {
                                    start: working_dir_node.start_byte(),
                                    end: working_dir_node.end_byte(),
                                },
                                rule_id: String::new(),
                            });
                        }
                    }
                    // Note: Full path validation requires filesystem access
                    // We only do basic format validation here
                }
            }
        }

        find_steps(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
