use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};

/// Validates workflow_call outputs are properly defined.
pub struct WorkflowCallOutputsRule;

impl ValidationRule for WorkflowCallOutputsRule {
    fn name(&self) -> &str {
        "workflow_call_outputs"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => return diagnostics,
        };

        let on_to_check = utils::unwrap_node(on_value);

        // Find workflow_call
        let workflow_call_value = utils::find_value_for_key(on_to_check, source, "workflow_call");

        if workflow_call_value.is_none() {
            return diagnostics;
        }

        let workflow_call = workflow_call_value.unwrap();
        let call_to_check = utils::unwrap_node(workflow_call);

        // Collect all job names and their outputs for reference validation
        let jobs_value = utils::find_value_for_key(root, source, "jobs");
        let (job_names, job_outputs) = if let Some(jobs_node) = jobs_value {
            (
                collect_job_names(jobs_node, source),
                collect_job_outputs(jobs_node, source),
            )
        } else {
            (HashSet::new(), std::collections::HashMap::new())
        };

        // Extract defined outputs
        let outputs_value = utils::find_value_for_key(call_to_check, source, "outputs");

        if let Some(outputs_node) = outputs_value {
            let outputs_to_check = utils::unwrap_node(outputs_node);

            // Find all job output references in output value expressions
            let output_refs = find_job_output_references(outputs_to_check, source);

            // Also find all expressions to validate invalid ones
            let all_expressions = find_all_expressions(outputs_to_check, source);

            // Check for invalid expressions (expressions that don't match jobs.X.outputs.Y pattern)
            for (expr_text, span) in all_expressions {
                let expr_text_str: &str = &expr_text;
                let expr_lower = expr_text_str.to_lowercase();
                // Check if this expression contains a jobs.X.outputs.Y pattern
                let has_valid_pattern =
                    expr_lower.contains("jobs.") && expr_lower.contains(".outputs.");
                if !has_valid_pattern && !expr_text_str.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "workflow_call output has invalid expression: '{}'. Output value must reference a job output using 'jobs.<job_id>.outputs.<output_name>'.",
                            expr_text_str.trim()
                        ),
                        severity: Severity::Error,
                        span,
                    });
                }
            }

            // Validate each reference
            for (job_name, output_name, span) in output_refs {
                if !job_names.contains(&job_name) {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "workflow_call output references non-existent job: 'jobs.{}.outputs.{}'",
                            job_name, output_name
                        ),
                        severity: Severity::Error,
                        span,
                    });
                } else {
                    // Check if the job output exists
                    if let Some(outputs) = job_outputs.get(&job_name) {
                        if !outputs.contains(&output_name) {
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "workflow_call output references non-existent job output: 'jobs.{}.outputs.{}'. Available outputs: {}",
                                    job_name,
                                    output_name,
                                    if outputs.is_empty() {
                                        "none".to_string()
                                    } else {
                                        outputs.iter().cloned().collect::<Vec<_>>().join(", ")
                                    }
                                ),
                                severity: Severity::Error,
                                span,
                            });
                        }
                    } else {
                        // Job exists but has no outputs defined
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "workflow_call output references job '{}' which has no outputs defined.",
                                job_name
                            ),
                            severity: Severity::Error,
                            span,
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

fn collect_job_names(jobs_node: Node, source: &str) -> HashSet<String> {
    let mut job_names = HashSet::new();

    fn collect(node: Node, source: &str, names: &mut HashSet<String>) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');
                    names.insert(key_cleaned.to_string());
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    collect(child, source, names);
                }
            }
        }
    }

    collect(jobs_node, source, &mut job_names);
    job_names
}

fn collect_job_outputs(jobs_node: Node, source: &str) -> HashMap<String, HashSet<String>> {
    let mut job_outputs = HashMap::new();

    fn collect(node: Node, source: &str, outputs: &mut HashMap<String, HashSet<String>>) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let job_name = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':')
                        .to_string();

                    let job_value = utils::get_pair_value(node);

                    if let Some(job_value_raw) = job_value {
                        let job_value = utils::unwrap_node(job_value_raw);

                        if job_value.kind() == "block_mapping" || job_value.kind() == "flow_mapping"
                        {
                            // Find outputs in this job
                            let outputs_value =
                                utils::find_value_for_key(job_value, source, "outputs");
                            if let Some(outputs_node_raw) = outputs_value {
                                let outputs_node = utils::unwrap_node(outputs_node_raw);

                                let mut output_names = HashSet::new();
                                collect_output_names(outputs_node, source, &mut output_names);
                                if !output_names.is_empty() {
                                    outputs.insert(job_name, output_names);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    collect(child, source, outputs);
                }
            }
        }
    }

    collect(jobs_node, source, &mut job_outputs);
    job_outputs
}

fn collect_output_names(node: Node, source: &str, names: &mut HashSet<String>) {
    match node.kind() {
        "block_mapping_pair" | "flow_pair" => {
            if let Some(key_node) = node.child(0) {
                let key_text = utils::node_text(key_node, source);
                let key_cleaned = key_text
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .trim_end_matches(':');
                names.insert(key_cleaned.to_string());
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_output_names(child, source, names);
            }
        }
    }
}

fn find_job_output_references(outputs_node: Node, source: &str) -> Vec<(String, String, Span)> {
    let mut references = Vec::new();
    let node_text = utils::node_text(outputs_node, source);

    let source_bytes = node_text.as_bytes();
    let mut i = 0;
    let node_start = outputs_node.start_byte();

    while i < source_bytes.len() {
        if i + 3 < source_bytes.len()
            && source_bytes[i] == b'$'
            && source_bytes[i + 1] == b'{'
            && source_bytes[i + 2] == b'{'
        {
            let mut j = i + 3;
            let mut brace_count = 2;
            let mut found_closing = false;

            while j < source_bytes.len() {
                if j + 1 < source_bytes.len()
                    && source_bytes[j] == b'}'
                    && source_bytes[j + 1] == b'}'
                {
                    brace_count -= 2;
                    if brace_count == 0 {
                        found_closing = true;
                        j += 2;
                        break;
                    }
                    j += 2;
                } else if source_bytes[j] == b'{' {
                    brace_count += 1;
                    j += 1;
                } else if source_bytes[j] == b'}' {
                    brace_count -= 1;
                    j += 1;
                } else {
                    j += 1;
                }
            }

            if found_closing {
                let expr_start = i + 3;
                let expr_end = j - 2;

                if expr_start < expr_end && expr_end <= source_bytes.len() {
                    let expr_text = &node_text[expr_start..expr_end];
                    let expr_lower = expr_text.to_lowercase();
                    let mut search_pos = 0;

                    while let Some(pos) = expr_lower[search_pos..].find("jobs.") {
                        let actual_pos = search_pos + pos;
                        let after_jobs = &expr_text[actual_pos + 5..];

                        let job_name_end = after_jobs
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
                            .unwrap_or(after_jobs.len());

                        let job_name = &after_jobs[..job_name_end.min(after_jobs.len())];

                        if !job_name.is_empty() {
                            let after_job = &after_jobs[job_name_end..];

                            // Check if followed by .outputs.
                            if let Some(after_outputs) = after_job.strip_prefix(".outputs") {
                                let after_outputs_trimmed = after_outputs.trim();

                                // Extract output name after .outputs.
                                if let Some(output_name) = after_outputs_trimmed.strip_prefix('.') {
                                    let output_name_end = output_name
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
                                                || c == '['
                                        })
                                        .unwrap_or(output_name.len());

                                    let output_name_cleaned = &output_name[..output_name_end];

                                    if !output_name_cleaned.is_empty() {
                                        // Calculate span relative to the original source
                                        let expr_offset_in_source = node_start + i + 3;
                                        let jobs_pos_in_expr = actual_pos;
                                        let job_name_start =
                                            expr_offset_in_source + jobs_pos_in_expr + 5; // +5 for "jobs."

                                        references.push((
                                            job_name.to_string(),
                                            output_name_cleaned.to_string(),
                                            Span {
                                                start: job_name_start,
                                                end: job_name_start + job_name.len(),
                                            },
                                        ));
                                    }
                                }
                            }
                        }

                        search_pos = actual_pos + 5 + job_name_end;
                    }
                }

                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    references
}

fn find_all_expressions(outputs_node: Node, source: &str) -> Vec<(String, Span)> {
    let mut expressions = Vec::new();
    let node_text = utils::node_text(outputs_node, source);

    let source_bytes = node_text.as_bytes();
    let mut i = 0;
    let node_start = outputs_node.start_byte();

    while i < source_bytes.len() {
        if i + 3 < source_bytes.len()
            && source_bytes[i] == b'$'
            && source_bytes[i + 1] == b'{'
            && source_bytes[i + 2] == b'{'
        {
            let mut j = i + 3;
            let mut brace_count = 2;
            let mut found_closing = false;

            while j < source_bytes.len() {
                if j + 1 < source_bytes.len()
                    && source_bytes[j] == b'}'
                    && source_bytes[j + 1] == b'}'
                {
                    brace_count -= 2;
                    if brace_count == 0 {
                        found_closing = true;
                        j += 2;
                        break;
                    }
                    j += 2;
                } else if source_bytes[j] == b'{' {
                    brace_count += 1;
                    j += 1;
                } else if source_bytes[j] == b'}' {
                    brace_count -= 1;
                    j += 1;
                } else {
                    j += 1;
                }
            }

            if found_closing {
                let expr_start = i + 3;
                let expr_end = j - 2;

                if expr_start < expr_end && expr_end <= source_bytes.len() {
                    let expr_text = &node_text[expr_start..expr_end];
                    expressions.push((
                        expr_text.to_string(),
                        Span {
                            start: node_start + i,
                            end: node_start + j,
                        },
                    ));
                }

                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    expressions
}
