use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::HashSet;
use tree_sitter::{Node, Tree};

/// Validates if condition expressions in jobs.
pub struct JobIfExpressionRule;

impl ValidationRule for JobIfExpressionRule {
    fn name(&self) -> &str {
        "job_if_expression"
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

        // Collect all job names for reference validation
        let job_names: HashSet<String> = collect_job_names(jobs_value, source);

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn check_job_if(
            node: Node,
            source: &str,
            job_names: &HashSet<String>,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let job_name = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':')
                            .to_string();

                        if let Some(job_value_raw) = utils::get_pair_value(node) {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                let if_value = utils::find_value_for_key(job_value, source, "if");

                                if let Some(if_node) = if_value {
                                    let if_text = utils::node_text(if_node, source);
                                    let if_cleaned = if_text.trim();

                                    // GitHub Actions auto-wraps if: conditions in ${{ }}.
                                    // Both bare expressions and explicitly wrapped ones are valid.
                                    let inner = if if_cleaned.starts_with("${{")
                                        && if_cleaned.ends_with("}}")
                                    {
                                        // Explicitly wrapped: extract inner expression
                                        if_cleaned[3..if_cleaned.len() - 2].trim()
                                    } else {
                                        // Bare expression: GitHub Actions auto-wraps this
                                        if_cleaned
                                    };

                                    {
                                        // Validate expression syntax
                                        if !utils::is_valid_expression_syntax(inner) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has invalid 'if' expression syntax: '{}'",
                                                    job_name, inner
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: if_node.start_byte(),
                                                    end: if_node.end_byte(),
                                                },
                                            });
                                        }

                                        // Check for obviously undefined context variables
                                        if inner.contains("github.nonexistent")
                                            || inner.contains("nonexistent.property")
                                        {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' 'if' expression may reference undefined context variable: '{}'",
                                                    job_name, inner
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: if_node.start_byte(),
                                                    end: if_node.end_byte(),
                                                },
                                            });
                                        }

                                        // Check for potentially always-true/false conditions
                                        if utils::is_potentially_always_true(inner) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' 'if' expression may always evaluate to true: '{}'",
                                                    job_name, inner
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: if_node.start_byte(),
                                                    end: if_node.end_byte(),
                                                },
                                            });
                                        } else if utils::is_potentially_always_false(inner) {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' 'if' expression may always evaluate to false: '{}'",
                                                    job_name, inner
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: if_node.start_byte(),
                                                    end: if_node.end_byte(),
                                                },
                                            });
                                        }

                                        // Check for references to non-existent jobs
                                        if inner.contains("jobs.") {
                                            let jobs_prefix = "jobs.";
                                            let mut search_pos = 0;
                                            while let Some(pos) =
                                                inner[search_pos..].find(jobs_prefix)
                                            {
                                                let actual_pos =
                                                    search_pos + pos + jobs_prefix.len();
                                                let after_jobs = &inner[actual_pos..];

                                                // Find where the job name ends
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

                                                let referenced_job = &after_jobs
                                                    [..job_name_end.min(after_jobs.len())];

                                                if !referenced_job.is_empty()
                                                    && !job_names.contains(referenced_job)
                                                {
                                                    let expr_start =
                                                        if_node.start_byte() + 3 + actual_pos
                                                            - jobs_prefix.len();
                                                    let expr_end = expr_start
                                                        + jobs_prefix.len()
                                                        + referenced_job.len();

                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Job '{}' 'if' expression references non-existent job: 'jobs.{}'",
                                                            job_name, referenced_job
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: expr_start,
                                                            end: expr_end,
                                                        },
                                                    });
                                                }

                                                search_pos = actual_pos + job_name_end;
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
                        check_job_if(child, source, job_names, diagnostics);
                    }
                }
            }
        }

        check_job_if(jobs_to_process, source, &job_names, &mut diagnostics);

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

