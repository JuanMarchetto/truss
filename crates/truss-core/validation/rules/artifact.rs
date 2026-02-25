use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates actions/upload-artifact and actions/download-artifact usage.
pub struct ArtifactValidationRule;

impl ValidationRule for ArtifactValidationRule {
    fn name(&self) -> &str {
        "artifact"
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
                            let steps_value = utils::get_pair_value(node);
                            if let Some(steps_value) = steps_value {
                                let steps_value = utils::unwrap_node(steps_value);
                                if steps_value.kind() == "block_sequence"
                                    || steps_value.kind() == "flow_sequence"
                                {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_artifact_step(step_node, source, diagnostics);
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

        fn validate_artifact_step(
            step_node: Node,
            source: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let mut step_to_check = utils::unwrap_node(step_node);

            // Handle block_sequence_item - the mapping is a child
            // In YAML, block_sequence_item has children: ["-", block_node] where block_node contains the mapping
            if step_to_check.kind() == "block_sequence_item" {
                // Try to find the mapping - it could be at different child indices
                for i in 0..step_to_check.child_count() {
                    if let Some(child) = step_to_check.child(i) {
                        let candidate = utils::unwrap_node(child);
                        if candidate.kind() == "block_mapping" || candidate.kind() == "flow_mapping"
                        {
                            step_to_check = candidate;
                            break;
                        }
                    }
                }
            }

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let uses_value = utils::find_value_for_key(step_to_check, source, "uses");

                if let Some(uses_node) = uses_value {
                    let uses_text = utils::node_text(uses_node, source);
                    let uses_cleaned = uses_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

                    // Check if it's an artifact action
                    if uses_cleaned.starts_with("actions/upload-artifact")
                        || uses_cleaned.starts_with("actions/download-artifact")
                    {
                        // Check for name field in with mapping
                        let with_value = utils::find_value_for_key(step_to_check, source, "with");

                        if let Some(with_value) = with_value {
                            let with_node = utils::unwrap_node(with_value);

                            // Find name field - search recursively in the with mapping
                            let name_node = utils::find_value_for_key(with_node, source, "name");

                            if let Some(name_value) = name_node {
                                let name_node = utils::unwrap_node(name_value);

                                // Get name text
                                let name_text = utils::node_text(name_node, source);

                                // Check if it's an expression - expressions are valid
                                let trimmed_name = name_text.trim();
                                let is_expression = trimmed_name.starts_with("${{");

                                if !is_expression {
                                    // Clean the name text (remove quotes and whitespace)
                                    let name_cleaned = name_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });

                                    // Check if empty: For YAML `name: ""`, we need to detect empty strings
                                    // The most reliable check is if cleaned text is empty after removing quotes
                                    let is_empty = name_cleaned.is_empty();

                                    if is_empty {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Artifact action '{}' has empty name. Artifact name cannot be empty.",
                                                uses_cleaned
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: name_node.start_byte(),
                                                end: name_node.end_byte(),
                                            },
                                            rule_id: String::new(),
                                        });
                                    } else {
                                        // Validate artifact name format (basic validation)
                                        if !is_valid_artifact_name_format(name_cleaned) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Artifact action '{}' has invalid name format: '{}'. Artifact names should contain only alphanumeric characters, hyphens, underscores, dots, and spaces.",
                                                    uses_cleaned, name_cleaned
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: name_node.start_byte(),
                                                    end: name_node.end_byte(),
                                                },
                                                rule_id: String::new(),
                                            });
                                        }
                                    }
                                }
                            }

                            // Validate path field (for upload-artifact)
                            if uses_cleaned.starts_with("actions/upload-artifact") {
                                let path_value =
                                    utils::find_value_for_key(with_node, source, "path");
                                if let Some(path_node) = path_value {
                                    let path_text = utils::node_text(path_node, source);
                                    let path_cleaned = path_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });

                                    if !path_cleaned.starts_with("${{") && path_cleaned.is_empty() {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Artifact action '{}' has empty path. Path is required for upload-artifact.",
                                                uses_cleaned
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: path_node.start_byte(),
                                                end: path_node.end_byte(),
                                            },
                                            rule_id: String::new(),
                                        });
                                    }
                                }
                            }

                            // Validate retention-days (must be a number between 1 and 90)
                            let retention_days_value =
                                utils::find_value_for_key(with_node, source, "retention-days");
                            if let Some(retention_node) = retention_days_value {
                                let retention_text = utils::node_text(retention_node, source);
                                let retention_cleaned = retention_text.trim_matches(|c: char| {
                                    c == '"' || c == '\'' || c.is_whitespace()
                                });

                                if !retention_cleaned.starts_with("${{") {
                                    match retention_cleaned.parse::<i64>() {
                                        Ok(value) => {
                                            if !(1..=90).contains(&value) {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Artifact action '{}' has invalid retention-days: '{}'. retention-days must be between 1 and 90.",
                                                        uses_cleaned, retention_cleaned
                                                    ),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: retention_node.start_byte(),
                                                        end: retention_node.end_byte(),
                                                    },
                                                    rule_id: String::new(),
                                                });
                                            }
                                        }
                                        Err(_) => {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Artifact action '{}' has invalid retention-days: '{}'. retention-days must be a number between 1 and 90.",
                                                    uses_cleaned, retention_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: retention_node.start_byte(),
                                                    end: retention_node.end_byte(),
                                                },
                                                rule_id: String::new(),
                                            });
                                        }
                                    }
                                }
                            }

                            // Validate compression-level (must be a number between 0 and 9, or 'fastest', 'fast', 'default', 'best')
                            let compression_level_value =
                                utils::find_value_for_key(with_node, source, "compression-level");
                            if let Some(compression_node) = compression_level_value {
                                let compression_text = utils::node_text(compression_node, source);
                                let compression_cleaned =
                                    compression_text.trim_matches(|c: char| {
                                        c == '"' || c == '\'' || c.is_whitespace()
                                    });

                                if !compression_cleaned.starts_with("${{") {
                                    let valid_levels = ["fastest", "fast", "default", "best"];
                                    let is_valid_string = valid_levels
                                        .iter()
                                        .any(|l| l.eq_ignore_ascii_case(compression_cleaned));

                                    if !is_valid_string {
                                        match compression_cleaned.parse::<i64>() {
                                            Ok(value) => {
                                                if !(0..=9).contains(&value) {
                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Artifact action '{}' has invalid compression-level: '{}'. compression-level must be between 0 and 9, or one of: fastest, fast, default, best.",
                                                            uses_cleaned, compression_cleaned
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: compression_node.start_byte(),
                                                            end: compression_node.end_byte(),
                                                        },
                                                        rule_id: String::new(),
                                                    });
                                                }
                                            }
                                            Err(_) => {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Artifact action '{}' has invalid compression-level: '{}'. compression-level must be between 0 and 9, or one of: fastest, fast, default, best.",
                                                        uses_cleaned, compression_cleaned
                                                    ),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: compression_node.start_byte(),
                                                        end: compression_node.end_byte(),
                                                    },
                                                    rule_id: String::new(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        find_steps(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}

/// Validates that an artifact name follows a reasonable format.
/// Artifact names should contain only alphanumeric characters, hyphens, underscores, and dots.
fn is_valid_artifact_name_format(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ')
}
