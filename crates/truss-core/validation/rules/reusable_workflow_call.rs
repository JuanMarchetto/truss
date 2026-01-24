use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates uses: workflow calls reference valid reusable workflows.
pub struct ReusableWorkflowCallRule;

impl ValidationRule for ReusableWorkflowCallRule {
    fn name(&self) -> &str {
        "reusable_workflow_call"
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
                                // Check for uses: field (reusable workflow call)
                                let uses_value = utils::find_value_for_key(job_value, source, "uses");
                                
                                if let Some(uses_node) = uses_value {
                                    let uses_text = utils::node_text(uses_node, source);
                                    let uses_cleaned = uses_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                    
                                    // Check if it looks like a reusable workflow call (has @ and owner/repo format)
                                    // but is missing the .github/workflows/ path
                                    if uses_cleaned.contains('@') && !uses_cleaned.contains(".github/workflows/") {
                                        let parts: Vec<&str> = uses_cleaned.split('@').collect();
                                        if parts.len() == 2 {
                                            let path_part = parts[0];
                                            // Check if it looks like owner/repo (has exactly one slash, no path)
                                            if path_part.matches('/').count() == 1 && !path_part.contains("/.github/") {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' reusable workflow call '{}' has invalid format: missing path. Format: owner/repo/.github/workflows/file.yml@ref",
                                                        job_name, uses_cleaned
                                                    ),
                                                    severity: Severity::Error,
                                                    span: Span {
                                                        start: uses_node.start_byte(),
                                                        end: uses_node.end_byte(),
                                                    },
                                                });
                                            }
                                        }
                                    }
                                    
                                    // Check if it's a reusable workflow call (contains .github/workflows/)
                                    if uses_cleaned.contains(".github/workflows/") {
                                        // Validate format: owner/repo/.github/workflows/file.yml@ref
                                        if !uses_cleaned.contains('@') {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' reusable workflow call '{}' is missing @ref. Format: owner/repo/.github/workflows/file.yml@ref",
                                                    job_name, uses_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: uses_node.start_byte(),
                                                    end: uses_node.end_byte(),
                                                },
                                            });
                                        } else {
                                            // Check if path is valid
                                            let parts: Vec<&str> = uses_cleaned.split('@').collect();
                                            if parts.len() == 2 {
                                                let path = parts[0];
                                                if !path.contains("/.github/workflows/") {
                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Job '{}' reusable workflow call has invalid path: '{}'. Path must contain '/.github/workflows/'",
                                                            job_name, path
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: uses_node.start_byte(),
                                                            end: uses_node.start_byte() + path.len().min(uses_node.end_byte() - uses_node.start_byte()),
                                                        },
                                                    });
                                                } else {
                                                    // Validate that with: and secrets: fields are properly structured
                                                    // Note: Full validation of required inputs/secrets would require parsing the referenced workflow file
                                                    let with_value = utils::find_value_for_key(job_value, source, "with");
                                                    let secrets_value = utils::find_value_for_key(job_value, source, "secrets");
                                                    
                                                    if let Some(with_node) = with_value {
                                                        let with_text = utils::node_text(with_node, source);
                                                        // Basic check - with should be a mapping, not a scalar
                                                        if with_text.trim().is_empty() {
                                                            diagnostics.push(Diagnostic {
                                                                message: format!(
                                                                    "Job '{}' reusable workflow call has empty 'with:' field. Remove it or provide input values.",
                                                                    job_name
                                                                ),
                                                                severity: Severity::Warning,
                                                                span: Span {
                                                                    start: with_node.start_byte(),
                                                                    end: with_node.end_byte(),
                                                                },
                                                            });
                                                        }
                                                    }
                                                    
                                                    if let Some(secrets_node) = secrets_value {
                                                        let secrets_text = utils::node_text(secrets_node, source);
                                                        // Basic check - secrets should be a mapping, not a scalar
                                                        if secrets_text.trim().is_empty() {
                                                            diagnostics.push(Diagnostic {
                                                                message: format!(
                                                                    "Job '{}' reusable workflow call has empty 'secrets:' field. Remove it or provide secret values.",
                                                                    job_name
                                                                ),
                                                                severity: Severity::Warning,
                                                                span: Span {
                                                                    start: secrets_node.start_byte(),
                                                                    end: secrets_node.end_byte(),
                                                                },
                                                            });
                                                        }
                                                    }
                                                    
                                                    // Note: Full validation of required inputs/secrets would require:
                                                    // 1. Parsing the referenced workflow file
                                                    // 2. Extracting workflow_call inputs/secrets definitions
                                                    // 3. Checking if required inputs/secrets are provided
                                                    // This is beyond the scope of static analysis without filesystem access
                                                }
                                            }
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
                        process_jobs(child, source, diagnostics);
                    }
                }
            }
        }

        process_jobs(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}
