use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates job dependencies (needs).
pub struct JobNeedsRule;

impl ValidationRule for JobNeedsRule {
    fn name(&self) -> &str {
        "job_needs"
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

        let mut job_names = std::collections::HashSet::new();

        fn collect_job_names_set(
            node: Node,
            source: &str,
            names: &mut std::collections::HashSet<String>,
        ) {
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
                        collect_job_names_set(child, source, names);
                    }
                }
            }
        }

        collect_job_names_set(jobs_value, source, &mut job_names);

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn extract_needs_values(needs_node: Node, source: &str) -> Vec<String> {
            let mut values = Vec::new();
            let node = utils::unwrap_node(needs_node);

            match node.kind() {
                "flow_sequence" | "block_sequence" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "flow_node"
                            || child.kind() == "block_node"
                            || child.kind() == "plain_scalar"
                            || child.kind() == "double_quoted_scalar"
                            || child.kind() == "single_quoted_scalar"
                        {
                            let text = utils::node_text(child, source);
                            let cleaned = text.trim_matches(|c: char| {
                                c == '"'
                                    || c == '\''
                                    || c.is_whitespace()
                                    || c == '['
                                    || c == ']'
                                    || c == ','
                                    || c == '-'
                            });
                            if !cleaned.is_empty() {
                                values.push(cleaned.to_string());
                            }
                        }
                    }
                }
                "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                    let text = utils::node_text(node, source);
                    let cleaned =
                        text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    if !cleaned.is_empty() {
                        values.push(cleaned.to_string());
                    }
                }
                _ => {
                    let text = utils::node_text(node, source);
                    let cleaned =
                        text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    if !cleaned.is_empty() && !cleaned.contains('\n') {
                        values.push(cleaned.to_string());
                    }
                }
            }
            values
        }

        fn find_needs_references(
            node: Node,
            source: &str,
            job_name: &str,
            all_job_names: &std::collections::HashSet<String>,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "needs" {
                            if let Some(value_node_raw) = utils::get_pair_value(node) {
                                let value_node = utils::unwrap_node(value_node_raw);
                                let needs_values = extract_needs_values(value_node, source);
                                for need in needs_values {
                                    if !all_job_names.contains(&need) {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' references nonexistent job: '{}'",
                                                job_name, need
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: value_node.start_byte(),
                                                end: value_node.end_byte(),
                                            },
                                        });
                                    }

                                    if need == job_name {
                                        diagnostics.push(Diagnostic {
                                            message: format!(
                                                "Job '{}' cannot reference self in 'needs'",
                                                job_name
                                            ),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: value_node.start_byte(),
                                                end: value_node.end_byte(),
                                            },
                                        });
                                    }
                                }
                            }
                        } else if let Some(value_node) = utils::get_pair_value(node) {
                            find_needs_references(
                                value_node,
                                source,
                                job_name,
                                all_job_names,
                                diagnostics,
                            );
                        }
                    }
                }
                "block_node" | "block_mapping" => {
                    // Traverse into block_node or block_mapping
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_needs_references(child, source, job_name, all_job_names, diagnostics);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_needs_references(child, source, job_name, all_job_names, diagnostics);
                    }
                }
            }
        }

        fn process_job(
            node: Node,
            source: &str,
            all_job_names: &std::collections::HashSet<String>,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::node_text(key_node, source);
                        let job_name_cleaned = job_name
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();
                        if all_job_names.contains(&job_name_cleaned) {
                            if let Some(job_value_raw) = utils::get_pair_value(node) {
                                let job_value = utils::unwrap_node(job_value_raw);
                                find_needs_references(
                                    job_value,
                                    source,
                                    &job_name_cleaned,
                                    all_job_names,
                                    diagnostics,
                                );
                            }
                        }
                        if let Some(value_node) = utils::get_pair_value(node) {
                            process_job(value_node, source, all_job_names, diagnostics);
                        }
                    }
                }
                "block_node" | "block_mapping" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        process_job(child, source, all_job_names, diagnostics);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        process_job(child, source, all_job_names, diagnostics);
                    }
                }
            }
        }

        let mut dependencies: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        fn collect_dependencies(
            node: Node,
            source: &str,
            job_name: &str,
            deps: &mut std::collections::HashMap<String, Vec<String>>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "needs" {
                            if let Some(value_node_raw) = utils::get_pair_value(node) {
                                let value_node = utils::unwrap_node(value_node_raw);
                                let needs_values = extract_needs_values(value_node, source);
                                deps.insert(job_name.to_string(), needs_values);
                            }
                        } else if let Some(value_node) = utils::get_pair_value(node) {
                            collect_dependencies(value_node, source, job_name, deps);
                        }
                    }
                }
                "block_node" | "block_mapping" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_dependencies(child, source, job_name, deps);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_dependencies(child, source, job_name, deps);
                    }
                }
            }
        }

        fn collect_all_dependencies(
            node: Node,
            source: &str,
            all_job_names: &std::collections::HashSet<String>,
            deps: &mut std::collections::HashMap<String, Vec<String>>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::node_text(key_node, source);
                        let job_name_cleaned = job_name
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();
                        if all_job_names.contains(&job_name_cleaned) {
                            if let Some(job_value_raw) = utils::get_pair_value(node) {
                                let job_value = utils::unwrap_node(job_value_raw);
                                collect_dependencies(job_value, source, &job_name_cleaned, deps);
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        collect_all_dependencies(child, source, all_job_names, deps);
                    }
                }
            }
        }

        collect_all_dependencies(jobs_to_process, source, &job_names, &mut dependencies);

        fn has_cycle(
            job: &str,
            deps: &std::collections::HashMap<String, Vec<String>>,
            visited: &mut std::collections::HashSet<String>,
            rec_stack: &mut std::collections::HashSet<String>,
        ) -> bool {
            visited.insert(job.to_string());
            rec_stack.insert(job.to_string());

            if let Some(needs) = deps.get(job) {
                for need in needs {
                    if !visited.contains(need) {
                        if has_cycle(need, deps, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(need) {
                        return true;
                    }
                }
            }

            rec_stack.remove(job);
            false
        }

        let mut visited = std::collections::HashSet::new();
        for job in &job_names {
            if !visited.contains(job) {
                let mut rec_stack = std::collections::HashSet::new();
                if has_cycle(job, &dependencies, &mut visited, &mut rec_stack) {
                    diagnostics.push(Diagnostic {
                        message: format!("circular dependency detected involving job '{}'", job),
                        severity: Severity::Error,
                        span: Span {
                            start: 0,
                            end: source.len().min(100),
                        },
                    });
                }
            }
        }

        process_job(jobs_to_process, source, &job_names, &mut diagnostics);

        diagnostics
    }
}
