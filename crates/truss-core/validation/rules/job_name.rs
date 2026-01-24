use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates job names.
pub struct JobNameRule;

impl ValidationRule for JobNameRule {
    fn name(&self) -> &str {
        "job_name"
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

        let mut job_names = Vec::new();
        
        fn collect_job_names(node: Node, source: &str, names: &mut Vec<(String, Span)>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        names.push((key_cleaned.to_string(), Span {
                            start: key_node.start_byte(),
                            end: key_node.end_byte(),
                        }));
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_job_names(child, source, names);
                    }
                }
            }
        }

        collect_job_names(jobs_value, source, &mut job_names);

        let mut seen = std::collections::HashSet::new();
        for (name, span) in &job_names {
            if seen.contains(name) {
                diagnostics.push(Diagnostic {
                    message: format!("duplicate job name: '{}'", name),
                    severity: Severity::Error,
                    span: *span,
                });
            } else {
                seen.insert(name.clone());
            }
        }

        let reserved_names = ["if", "else", "elif", "for", "while", "with"];
        for (name, span) in &job_names {
            let name_trimmed = name.trim();
            
            // Validate job name length (GitHub Actions has practical limits)
            if name_trimmed.len() > 100 {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Job name '{}' is too long ({} characters). Consider using a shorter name (recommended: < 50 characters).",
                        name_trimmed, name_trimmed.len()
                    ),
                    severity: Severity::Warning,
                    span: *span,
                });
            }
            
            // Validate job name character set (alphanumeric, hyphens, underscores)
            if !is_valid_job_name_format(name_trimmed) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Invalid job name: '{}'. Job names must contain only alphanumeric characters, hyphens, and underscores.",
                        name_trimmed
                    ),
                    severity: Severity::Error,
                    span: *span,
                });
            }
            
            if reserved_names.contains(&name_trimmed.to_lowercase().as_str()) {
                diagnostics.push(Diagnostic {
                    message: format!("Reserved name cannot be used as job name: '{}'", name_trimmed),
                    severity: Severity::Error,
                    span: *span,
                });
            }
        }

        diagnostics
    }
}

/// Validates that a job name follows the correct format.
/// Job names must contain only alphanumeric characters, hyphens, and underscores.
fn is_valid_job_name_format(job_name: &str) -> bool {
    if job_name.is_empty() {
        return false;
    }
    
    job_name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_'
    })
}

