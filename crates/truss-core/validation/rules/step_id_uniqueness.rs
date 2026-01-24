use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;
use std::collections::HashSet;

/// Validates that step IDs are unique within a job.
pub struct StepIdUniquenessRule;

impl ValidationRule for StepIdUniquenessRule {
    fn name(&self) -> &str {
        "step_id_uniqueness"
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
                                // Collect step IDs from this job
                                let step_ids = collect_step_ids(job_value, source);
                                
                                // Check for duplicates and validate format
                                let mut seen = HashSet::new();
                                for (step_id, span) in &step_ids {
                                    // Validate step ID format (alphanumeric, hyphens, underscores)
                                    if !is_valid_step_id_format(step_id) {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' has step ID '{}' with invalid format. Step IDs must contain only alphanumeric characters, hyphens, and underscores.",
                                                job_name, step_id
                                            ),
                                            severity: Severity::Warning,
                                            span: *span,
                                        });
                                    }
                                    
                                    // Check for duplicates
                                    if seen.contains(step_id) {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' has duplicate step ID: '{}'. Step IDs must be unique within a job.",
                                                job_name, step_id
                                            ),
                                            severity: Severity::Error,
                                            span: *span,
                                        });
                                    } else {
                                        seen.insert(step_id.clone());
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

fn collect_step_ids(job_node: Node, source: &str) -> Vec<(String, Span)> {
    let mut step_ids = Vec::new();
    
    // Find steps in this job
    let steps_value = utils::find_value_for_key(job_node, source, "steps");
    
    if let Some(mut steps_node) = steps_value {
        if steps_node.kind() == "block_node" {
            if let Some(inner) = steps_node.child(0) {
                steps_node = inner;
            }
        }
        
        // Traverse steps to find step IDs
        fn collect_from_steps(node: Node, source: &str, step_ids: &mut Vec<(String, Span)>) {
            match node.kind() {
                "block_sequence" | "flow_sequence" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, step_ids);
                    }
                }
                "block_mapping" | "flow_mapping" => {
                    // This is a step object
                    let id_value = utils::find_value_for_key(node, source, "id");
                    if let Some(id_node) = id_value {
                        let id_text = utils::node_text(id_node, source);
                        let id_cleaned = id_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                        if !id_cleaned.is_empty() {
                            step_ids.push((
                                id_cleaned.to_string(),
                                Span {
                                    start: id_node.start_byte(),
                                    end: id_node.end_byte(),
                                },
                            ));
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, step_ids);
                    }
                }
            }
        }
        
        collect_from_steps(steps_node, source, &mut step_ids);
    }
    
    step_ids
}

/// Validates that a step ID follows the correct format.
/// Step IDs must contain only alphanumeric characters, hyphens, and underscores.
fn is_valid_step_id_format(step_id: &str) -> bool {
    if step_id.is_empty() {
        return false;
    }
    
    step_id.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_'
    })
}

