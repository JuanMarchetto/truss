use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates working-directory paths.
pub struct StepWorkingDirectoryRule;

impl ValidationRule for StepWorkingDirectoryRule {
    fn name(&self) -> &str {
        "step_working_directory"
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

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            let steps_value = if node.kind() == "block_mapping_pair" {
                                node.child(2)
                            } else {
                                node.child(1)
                            };
                            if let Some(mut steps_value) = steps_value {
                                if steps_value.kind() == "block_node" {
                                    if let Some(inner) = steps_value.child(0) {
                                        steps_value = inner;
                                    }
                                }
                                if steps_value.kind() == "block_sequence" || steps_value.kind() == "flow_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_working_directory(step_node, source, diagnostics);
                                    }
                                }
                            }
                        }
                        let value_node = if node.kind() == "block_mapping_pair" {
                            node.child(2)
                        } else {
                            node.child(1)
                        };
                        if let Some(value_node) = value_node {
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

        fn validate_step_working_directory(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let mut step_to_check = step_node;
            
            if step_to_check.kind() == "block_node" {
                if let Some(inner) = step_to_check.child(0) {
                    step_to_check = inner;
                }
            }
            
            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let working_dir_value = utils::find_value_for_key(step_to_check, source, "working-directory");
                
                if let Some(working_dir_node) = working_dir_value {
                    let working_dir_text = utils::node_text(working_dir_node, source);
                    let working_dir_cleaned = working_dir_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    
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
                        });
                    } else {
                        // Basic path format validation
                        // Warn about potentially invalid paths
                        if working_dir_cleaned.starts_with("/") && !working_dir_cleaned.starts_with("/home") && !working_dir_cleaned.starts_with("/github") {
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
                            });
                        }
                        
                        // Warn about paths with invalid characters (basic check)
                        if working_dir_cleaned.contains("..") && working_dir_cleaned != ".." && !working_dir_cleaned.starts_with("../") {
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
                            });
                        }
                    }
                    // Note: Full path validation requires filesystem access
                    // We only do basic format validation here
                }
            }
        }

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

