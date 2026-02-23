use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;
use std::collections::{HashSet, HashMap};

/// Validates that step output references (steps.<step_id>.outputs.<output_name>) reference valid outputs.
pub struct StepOutputReferenceRule;

impl ValidationRule for StepOutputReferenceRule {
    fn name(&self) -> &str {
        "step_output_reference"
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

        let mut processed_jobs = HashSet::new();
        
        // First pass: collect all step IDs with their job names across all jobs
        let mut all_step_ids_by_job: HashMap<String, HashSet<String>> = HashMap::new();
        
        fn collect_all_step_ids(node: Node, source: &str, step_ids_by_job: &mut HashMap<String, HashSet<String>>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let job_name = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();

                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value_raw) = job_value {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping" || job_value.kind() == "flow_mapping" {
                                let step_ids_vec = collect_step_ids(job_value, source);
                                let step_ids_set: HashSet<String> = step_ids_vec
                                    .into_iter()
                                    .map(|(id, _)| id)
                                    .collect();
                                if !step_ids_set.is_empty() {
                                    step_ids_by_job.insert(job_name, step_ids_set);
                                }
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_all_step_ids(child, source, step_ids_by_job);
                    }
                }
            }
        }
        
        collect_all_step_ids(jobs_to_process, source, &mut all_step_ids_by_job);

        fn process_jobs(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>, processed_jobs: &mut HashSet<String>, all_step_ids_by_job: &HashMap<String, HashSet<String>>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let job_name = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();
                        
                        // Skip if we've already processed this job
                        if processed_jobs.contains(&job_name) {
                            return;
                        }
                        processed_jobs.insert(job_name.clone());
                        
                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value_raw) = job_value {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping" || job_value.kind() == "flow_mapping" {
                                // Collect step IDs from this job (with their step indices for better error messages)
                                let step_ids_vec = collect_step_ids(job_value, source);
                                
                                let step_ids_map: HashSet<String> = step_ids_vec
                                    .into_iter()
                                    .map(|(id, _)| id)
                                    .collect();
                                
                                
                                // Collect steps without IDs for better error messages
                                let steps_without_ids = collect_steps_without_ids(job_value, source);
                                
                                
                                // Collect outputs set by each step (for validation)
                                let step_outputs_map = collect_step_outputs(job_value, source);
                                
                                // Find all step output references in this job (recursively search all fields)
                                let output_refs = find_step_output_references_recursive(job_value, source);
                                
                                
                                // Validate each reference
                                for (step_id, output_name, span) in output_refs {
                                    
                                    
                                    // Direct error production: if step exists and has outputs, check if output name exists
                                    // Produce error immediately if conditions are met
                                    if step_ids_map.contains(&step_id) && step_outputs_map.contains_key(&step_id) {
                                        if let Some(outputs) = step_outputs_map.get(&step_id) {
                                            if !outputs.contains(&output_name) {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' references step output 'steps.{}.outputs.{}' but output '{}' is not found. Available outputs: {}",
                                                        job_name,
                                                        step_id,
                                                        output_name,
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
                                                continue; // Skip rest of validation
                                            }
                                        }
                                    }
                                    
                                    // Check if step exists in map
                                    let step_exists_in_map = step_ids_map.contains(&step_id);
                                    
                                    
                                    if !step_exists_in_map {
                                        // Check if step exists in a different job
                                        let mut found_in_other_job = false;
                                        for (other_job_name, other_step_ids) in all_step_ids_by_job {
                                            if other_job_name != &job_name && other_step_ids.contains(&step_id) {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' references step output 'steps.{}.outputs.{}' but step '{}' is in job '{}'. Step outputs can only be referenced within the same job.",
                                                        job_name, step_id, output_name, step_id, other_job_name
                                                    ),
                                                    severity: Severity::Error,
                                                    span,
                                                });
                                                found_in_other_job = true;
                                                break;
                                            }
                                        }
                                        
                                        if !found_in_other_job {
                                            // If there are steps without IDs, assume the reference is to a step that lacks an 'id' field
                                            // This handles the case where someone references steps.build.outputs.result but no step has id: build
                                            if !steps_without_ids.is_empty() {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' references step output 'steps.{}.outputs.{}' but step '{}' does not have an 'id' field. Steps must have an 'id' field to be referenced.",
                                                        job_name, step_id, output_name, step_id
                                                    ),
                                                    severity: Severity::Error,
                                                    span,
                                                });
                                            } else {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' references step output 'steps.{}.outputs.{}' but step '{}' does not exist in this job.",
                                                        job_name, step_id, output_name, step_id
                                                    ),
                                                    severity: Severity::Error,
                                                    span,
                                                });
                                            }
                                        }
                                    } else {
                                        // Step exists, validate output name
                                        if let Some(outputs) = step_outputs_map.get(&step_id) {
                                            if !outputs.contains(&output_name) {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Job '{}' references step output 'steps.{}.outputs.{}' but output '{}' is not found. Available outputs: {}",
                                                        job_name,
                                                        step_id,
                                                        output_name,
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
                                        }
                                        // Also validate output name format
                                        if !is_valid_output_name_format(&output_name) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' references step output 'steps.{}.outputs.{}' with potentially invalid output name format. Output names should contain only alphanumeric characters, hyphens, and underscores.",
                                                    job_name, step_id, output_name
                                                ),
                                                severity: Severity::Warning,
                                                span,
                                            });
                                        }
                                    }
                                }
                            }
                            // Don't recursively process children of job pairs - we've already handled everything
                        }
                    }
                    // Don't recursively process children of job pairs - we've already handled everything
                }
                "block_mapping" | "flow_mapping" => {
                    // Actually, we DO need to recurse into children to find job pairs!
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        process_jobs(child, source, diagnostics, processed_jobs, all_step_ids_by_job);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        process_jobs(child, source, diagnostics, processed_jobs, all_step_ids_by_job);
                    }
                }
            }
        }
        
        process_jobs(jobs_to_process, source, &mut diagnostics, &mut processed_jobs, &all_step_ids_by_job);

        diagnostics
    }
}

fn collect_step_ids(job_node: Node, source: &str) -> Vec<(String, Span)> {
    let mut step_ids = Vec::new();
    
    let steps_value = utils::find_value_for_key(job_node, source, "steps");
    
    if let Some(steps_node_raw) = steps_value {
        let steps_node = utils::unwrap_node(steps_node_raw);

        fn collect_from_steps(node: Node, source: &str, step_ids: &mut Vec<(String, Span)>) {
            match node.kind() {
                "block_sequence" | "flow_sequence" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, step_ids);
                    }
                }
                "block_mapping" | "flow_mapping" => {
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

/// Collects step names/identifiers that don't have IDs (for better error messages)
/// This is a heuristic - we collect step names from 'name' fields or use indices
fn collect_steps_without_ids(job_node: Node, source: &str) -> HashSet<String> {
    let mut steps_without_ids = HashSet::new();
    
    let steps_value = utils::find_value_for_key(job_node, source, "steps");
    
    if let Some(steps_node_raw) = steps_value {
        let steps_node = utils::unwrap_node(steps_node_raw);

        fn collect_from_steps(node: Node, source: &str, steps_without_ids: &mut HashSet<String>, step_index: &mut usize) {
            match node.kind() {
                "block_sequence" | "flow_sequence" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, steps_without_ids, step_index);
                    }
                }
                "block_mapping" | "flow_mapping" => {
                    // Check if this step has an ID
                    let id_value = utils::find_value_for_key(node, source, "id");
                    if id_value.is_none() {
                        // Try to get step name for better error message
                        let name_value = utils::find_value_for_key(node, source, "name");
                        if let Some(name_node) = name_value {
                            let name_text = utils::node_text(name_node, source);
                            let name_cleaned = name_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                            if !name_cleaned.is_empty() {
                                steps_without_ids.insert(name_cleaned.to_string());
                            }
                        }
                        // Also add index-based identifier
                        steps_without_ids.insert(format!("step_{}", step_index));
                    }
                    *step_index += 1;
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, steps_without_ids, step_index);
                    }
                }
            }
        }
        
        let mut step_index = 0;
        collect_from_steps(steps_node, source, &mut steps_without_ids, &mut step_index);
    }
    
    steps_without_ids
}

/// Validates that an output name follows a reasonable format.
/// Output names should contain only alphanumeric characters, hyphens, and underscores.
fn is_valid_output_name_format(output_name: &str) -> bool {
    if output_name.is_empty() {
        return false;
    }
    
    output_name.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_'
    })
}

/// Recursively finds all step output references in a job node
fn find_step_output_references_recursive(node: Node, source: &str) -> Vec<(String, String, Span)> {
    let mut references = Vec::new();
    
    fn search_node(node: Node, source: &str, references: &mut Vec<(String, String, Span)>) {
        // Search in scalar nodes (which contain the actual text)
        if node.kind() == "plain_scalar" || node.kind() == "double_quote_scalar" || node.kind() == "single_quote_scalar" {
            let node_text = utils::node_text(node, source);
            let node_start = node.start_byte();
            
            // Find all ${{ ... }} expressions
            let source_bytes = node_text.as_bytes();
            let mut i = 0;
            
            while i < source_bytes.len() {
                if i + 3 < source_bytes.len() 
                    && source_bytes[i] == b'$' 
                    && source_bytes[i + 1] == b'{' 
                    && source_bytes[i + 2] == b'{' {
                    
                    let mut j = i + 3;
                    let mut brace_count = 2;
                    let mut found_closing = false;
                    
                    while j < source_bytes.len() {
                        if j + 1 < source_bytes.len() && source_bytes[j] == b'}' && source_bytes[j + 1] == b'}' {
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
                            
                            // Look for steps.*.outputs.* references
                            let expr_lower = expr_text.to_lowercase();
                            let mut search_pos = 0;
                            
                            while let Some(pos) = expr_lower[search_pos..].find("steps.") {
                                let actual_pos = search_pos + pos;
                                let after_steps = &expr_text[actual_pos + 6..];
                                
                                // Find where the step ID ends
                                let step_id_end = after_steps
                                    .find(|c: char| c.is_whitespace() || c == '.' || c == '}' || c == ')' || c == ']' || 
                                          c == '&' || c == '|' || c == '=' || c == '!' || c == '<' || c == '>' || c == '[')
                                    .unwrap_or(after_steps.len());
                                
                                let step_id = &after_steps[..step_id_end.min(after_steps.len())];
                                
                                if !step_id.is_empty() {
                                    // Check if this is followed by .outputs
                                    let after_step_id = &after_steps[step_id_end..];
                                    
                                    if after_step_id.starts_with(".outputs") {
                                        // Check if there's a property access after .outputs
                                        // ".outputs" is 8 characters (., o, u, t, p, u, t, s)
                                        // So we need to skip 8 characters to get to the character after ".outputs"
                                        let after_outputs = if after_step_id.len() > 8 {
                                            &after_step_id[8..]
                                        } else {
                                            ""
                                        };
                                        let after_outputs_trimmed_raw = after_outputs.trim();
                                        
                                        // Handle the case where we might get "s.result" instead of ".result"
                                        // This happens when the slice is off by one character
                                        let after_outputs_trimmed = if after_outputs_trimmed_raw.starts_with("s.") && after_outputs_trimmed_raw.len() > 2 {
                                            // Skip the "s" and use the rest
                                            &after_outputs_trimmed_raw[1..]
                                        } else {
                                            after_outputs_trimmed_raw
                                        };
                                        
                                        // Check if after .outputs we have a property access
                                        // Extract output name - handle both ".result" and "s.result" cases
                                        // After normalization, after_outputs_trimmed should start with "."
                                        let output_name = if after_outputs_trimmed.starts_with(".") {
                                            // Extract output name after the dot
                                            let after_dot = &after_outputs_trimmed[1..];
                                            let output_name_end = after_dot
                                                .find(|c: char| c.is_whitespace() || c == '.' || c == '}' || c == ')' || c == ']' || 
                                                      c == '&' || c == '|' || c == '=' || c == '!' || c == '<' || c == '>' || c == '[')
                                                .unwrap_or(after_dot.len());
                                            &after_dot[..output_name_end]
                                        } else if after_outputs_trimmed.starts_with("s.") {
                                            // Handle the case where normalization didn't work (shouldn't happen but handle it)
                                            let after_s_dot = &after_outputs_trimmed[2..];
                                            let output_name_end = after_s_dot
                                                .find(|c: char| c.is_whitespace() || c == '.' || c == '}' || c == ')' || c == ']' || 
                                                      c == '&' || c == '|' || c == '=' || c == '!' || c == '<' || c == '>' || c == '[')
                                                .unwrap_or(after_s_dot.len());
                                            &after_s_dot[..output_name_end]
                                        } else {
                                            // Fallback: try to extract even if it doesn't start with "."
                                            // This handles edge cases
                                            let output_name_end = after_outputs_trimmed
                                                .find(|c: char| c.is_whitespace() || c == '.' || c == '}' || c == ')' || c == ']' || 
                                                      c == '&' || c == '|' || c == '=' || c == '!' || c == '<' || c == '>' || c == '[')
                                                .unwrap_or(after_outputs_trimmed.len());
                                            &after_outputs_trimmed[..output_name_end]
                                        };
                                        
                                        if !output_name.is_empty() {
                                            let span_start = node_start + i + 3 + actual_pos + 6;
                                            let span_end = span_start + step_id.len();
                                            
                                            references.push((
                                                step_id.to_string(),
                                                output_name.to_string(),
                                                Span {
                                                    start: span_start,
                                                    end: span_end,
                                                },
                                            ));
                                        } else if after_outputs_trimmed.starts_with("[") {
                                            // Bracket notation - extract output name
                                            if let Some(close_bracket) = after_outputs_trimmed.find(']') {
                                                let output_name = &after_outputs_trimmed[1..close_bracket];
                                                let output_name_cleaned = output_name.trim_matches(|c: char| c == '"' || c == '\'');
                                                
                                                if !output_name_cleaned.is_empty() {
                                                    let span_start = node_start + i + 3 + actual_pos + 6;
                                                    let span_end = span_start + step_id.len();
                                                    
                                                    references.push((
                                                        step_id.to_string(),
                                                        output_name_cleaned.to_string(),
                                                        Span {
                                                            start: span_start,
                                                            end: span_end,
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                search_pos = actual_pos + 6 + step_id_end;
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
        }
        
        // Recursively search children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            search_node(child, source, references);
        }
    }
    
    search_node(node, source, &mut references);
    
    references
}

/// Collects outputs set by each step by parsing run commands
fn collect_step_outputs(job_node: Node, source: &str) -> HashMap<String, HashSet<String>> {
    let mut step_outputs = HashMap::new();
    
    let steps_value = utils::find_value_for_key(job_node, source, "steps");
    
    if let Some(steps_node_raw) = steps_value {
        let steps_node = utils::unwrap_node(steps_node_raw);

        fn collect_from_steps(node: Node, source: &str, step_outputs: &mut HashMap<String, HashSet<String>>) {
            match node.kind() {
                "block_sequence" | "flow_sequence" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, step_outputs);
                    }
                }
                "block_mapping" | "flow_mapping" => {
                    // This is a step object
                    let id_value = utils::find_value_for_key(node, source, "id");
                    if let Some(id_node) = id_value {
                        let id_text = utils::node_text(id_node, source);
                        let id_cleaned = id_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                        
                        if !id_cleaned.is_empty() {
                            let mut outputs = HashSet::new();
                            
                            // Look for outputs set via $GITHUB_OUTPUT
                            let run_value = utils::find_value_for_key(node, source, "run");
                            if let Some(run_node) = run_value {
                                let run_text = utils::node_text(run_node, source);
                                
                                // Look for patterns like: echo "name=value" >> $GITHUB_OUTPUT
                                // or: echo "name=value" >> "$GITHUB_OUTPUT"
                                // or: echo name=value >> $GITHUB_OUTPUT
                                for line in run_text.lines() {
                                    // Pattern: echo "name=value" >> $GITHUB_OUTPUT or echo name=value >> $GITHUB_OUTPUT
                                    if line.contains("$GITHUB_OUTPUT") || line.contains("\"$GITHUB_OUTPUT\"") {
                                        // Try to extract output name from echo commands
                                        if let Some(echo_start) = line.find("echo") {
                                            let after_echo = &line[echo_start + 4..].trim();
                                            
                                            // First try quoted strings
                                            for quote_char in ['"', '\''] {
                                                if let Some(quote_start) = after_echo.find(quote_char) {
                                                    if let Some(quote_end) = after_echo[quote_start + 1..].find(quote_char) {
                                                        let quoted = &after_echo[quote_start + 1..quote_start + 1 + quote_end];
                                                        if let Some(equals_pos) = quoted.find('=') {
                                                            let output_name = &quoted[..equals_pos];
                                                            if !output_name.is_empty() {
                                                                outputs.insert(output_name.to_string());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Also try unquoted patterns: name=value
                                            // Find the part before >> or | or && or ; or end of line
                                            let before_redirect = after_echo
                                                .split(">>")
                                                .next()
                                                .and_then(|s| s.split("|").next())
                                                .and_then(|s| s.split("&&").next())
                                                .and_then(|s| s.split(";").next())
                                                .unwrap_or(after_echo);
                                            
                                            // Look for name=value pattern
                                            for part in before_redirect.split_whitespace() {
                                                if let Some(equals_pos) = part.find('=') {
                                                    let output_name = &part[..equals_pos];
                                                    // Remove quotes if present
                                                    let output_name = output_name.trim_matches(|c: char| c == '"' || c == '\'');
                                                    if !output_name.is_empty() && output_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                                                        outputs.insert(output_name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Pattern: ::set-output name=name::value (legacy)
                                    if line.contains("::set-output") {
                                        if let Some(name_start) = line.find("name=") {
                                            let after_name = &line[name_start + 5..];
                                            if let Some(colon_pos) = after_name.find("::") {
                                                let output_name = &after_name[..colon_pos];
                                                let output_name = output_name.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                                if !output_name.is_empty() {
                                                    outputs.insert(output_name.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if !outputs.is_empty() {
                                step_outputs.insert(id_cleaned.to_string(), outputs.clone());
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_from_steps(child, source, step_outputs);
                    }
                }
            }
        }
        
        collect_from_steps(steps_node, source, &mut step_outputs);
    }
    
    step_outputs
}
