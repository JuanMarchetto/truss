use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::HashSet;
use tree_sitter::{Node, Tree};

/// Validates that job outputs reference valid step IDs.
pub struct JobOutputsRule;

impl ValidationRule for JobOutputsRule {
    fn name(&self) -> &str {
        "job_outputs"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        // Process each job
        fn process_jobs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::clean_key(key_node, source).to_string();

                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value) = job_value {
                            let job_value = utils::unwrap_node(job_value);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                // Collect step IDs from this job
                                let step_ids = collect_step_ids(job_value, source);

                                // Find outputs in this job
                                let outputs_value =
                                    utils::find_value_for_key(job_value, source, "outputs");

                                if let Some(outputs_node) = outputs_value {
                                    let outputs_to_check = utils::unwrap_node(outputs_node);

                                    // Find all step references in output expressions
                                    let (output_expressions, incomplete_refs) =
                                        find_output_expressions(outputs_to_check, source);

                                    // Validate each step reference
                                    for (step_id, span) in output_expressions {
                                        if !step_ids.contains(&step_id) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' output references step '{}' which does not exist. Available step IDs: {}",
                                                    job_name,
                                                    step_id,
                                                    if step_ids.is_empty() {
                                                        "none".to_string()
                                                    } else {
                                                        step_ids.iter().cloned().collect::<Vec<_>>().join(", ")
                                                    }
                                                ),
                                                severity: Severity::Error,
                                                span,
                                            });
                                        }
                                    }

                                    // Check for incomplete output references (steps.X.outputs without property name)
                                    for (span, step_id) in incomplete_refs {
                                        let msg = format!(
                                            "Job '{}' output has invalid syntax. Output reference 'steps.{}.outputs' is missing the output property name. Expected format: steps.{}.outputs.property_name",
                                            job_name, step_id, step_id
                                        );
                                        diagnostics.push(Diagnostic {
                                            message: msg,
                                            severity: Severity::Error,
                                            span,
                                        });
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

        process_jobs(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}

fn collect_step_ids(job_node: Node, source: &str) -> HashSet<String> {
    let mut step_ids = HashSet::new();

    // Find steps in this job
    let steps_value = utils::find_value_for_key(job_node, source, "steps");

    if let Some(steps_value) = steps_value {
        let steps_node = utils::unwrap_node(steps_value);

        // Traverse steps to find step IDs
        fn collect_from_steps(node: Node, source: &str, step_ids: &mut HashSet<String>) {
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
                        let id_cleaned = id_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                        if !id_cleaned.is_empty() {
                            step_ids.insert(id_cleaned.to_string());
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

#[allow(clippy::type_complexity)]
fn find_output_expressions(
    outputs_node: Node,
    source: &str,
) -> (Vec<(String, Span)>, Vec<(Span, String)>) {
    let mut references = Vec::new();
    let mut incomplete_refs = Vec::new();
    let outputs_text = utils::node_text(outputs_node, source);
    let node_start = outputs_node.start_byte();

    for expr in utils::find_expressions(outputs_text) {
        let mut search_pos = 0;

        while let Some(pos) = utils::find_ignore_ascii_case(&expr.inner[search_pos..], "steps.") {
            let actual_pos = search_pos + pos;
            let after_steps = &expr.inner[actual_pos + 6..];

            let step_id_end = after_steps
                .find(|c: char| {
                    c.is_whitespace()
                        || c == '.'
                        || c == '}'
                        || c == ')'
                        || c == ']'
                        || c == '&'
                        || c == '|'
                        || c == '='
                        || c == '!'
                        || c == '<'
                        || c == '>'
                })
                .unwrap_or(after_steps.len());

            let step_id = &after_steps[..step_id_end.min(after_steps.len())];

            if !step_id.is_empty() {
                let after_step_id = &after_steps[step_id_end..];

                if let Some(after_outputs) = after_step_id.strip_prefix(".outputs") {
                    let after_outputs_trimmed = after_outputs.trim();

                    let span_start = node_start + expr.start + 3 + actual_pos + 6;
                    let step_span = Span {
                        start: span_start,
                        end: span_start + step_id.len(),
                    };

                    let has_property_after = after_outputs_trimmed.starts_with(".")
                        || after_outputs_trimmed.starts_with("[");

                    let is_incomplete = after_outputs_trimmed.is_empty()
                        || after_outputs_trimmed
                            .chars()
                            .all(|c| c.is_whitespace() || c == '}' || c == ')' || c == ']');

                    let is_incomplete_ref = is_incomplete
                        || (!has_property_after
                            && after_outputs_trimmed.len() == 1
                            && !after_outputs_trimmed.starts_with(".")
                            && !after_outputs_trimmed.starts_with("["));

                    if is_incomplete_ref && !has_property_after {
                        incomplete_refs.push((
                            Span {
                                start: node_start + expr.start,
                                end: node_start + expr.end,
                            },
                            step_id.to_string(),
                        ));
                    } else {
                        references.push((step_id.to_string(), step_span));
                    }
                }
            }

            search_pos = actual_pos + 6 + step_id_end;
        }
    }

    (references, incomplete_refs)
}
