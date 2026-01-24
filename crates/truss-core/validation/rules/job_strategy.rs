use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates strategy field syntax and constraints (max-parallel, fail-fast).
pub struct JobStrategyValidationRule;

impl ValidationRule for JobStrategyValidationRule {
    fn name(&self) -> &str {
        "job_strategy"
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

        fn check_job_strategy(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
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
                                // Check for strategy in this job
                                let strategy_value = utils::find_value_for_key(job_value, source, "strategy");
                                
                                if let Some(mut strategy_node) = strategy_value {
                                    if strategy_node.kind() == "block_node" {
                                        if let Some(inner) = strategy_node.child(0) {
                                            strategy_node = inner;
                                        }
                                    }
                                    
                                    // Validate strategy structure: if strategy is defined, matrix should exist
                                    let matrix_value = utils::find_value_for_key(strategy_node, source, "matrix");
                                    if matrix_value.is_none() {
                                        // Check if strategy has any other fields (max-parallel, fail-fast)
                                        let has_max_parallel = utils::find_value_for_key(strategy_node, source, "max-parallel").is_some();
                                        let has_fail_fast = utils::find_value_for_key(strategy_node, source, "fail-fast").is_some();
                                        
                                        if !has_max_parallel && !has_fail_fast {
                                            // Strategy is empty or only has invalid fields
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has a 'strategy' field but no 'matrix' field. Strategy requires a matrix to be defined.",
                                                    job_name
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: strategy_node.start_byte(),
                                                    end: strategy_node.end_byte(),
                                                },
                                            });
                                        } else {
                                            // Strategy has max-parallel or fail-fast but no matrix - this is valid but unusual
                                            // GitHub Actions allows this, but it's better to have a matrix
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has a 'strategy' field with 'max-parallel' or 'fail-fast' but no 'matrix' field. Consider adding a matrix for better job distribution.",
                                                    job_name
                                                ),
                                                severity: Severity::Warning,
                                                span: Span {
                                                    start: strategy_node.start_byte(),
                                                    end: strategy_node.end_byte(),
                                                },
                                            });
                                        }
                                    }
                                    
                                    // Check max-parallel
                                    let max_parallel_value = utils::find_value_for_key(strategy_node, source, "max-parallel");
                                    if let Some(max_parallel_node) = max_parallel_value {
                                        let max_parallel_text = utils::node_text(max_parallel_node, source);
                                        let max_parallel_cleaned = max_parallel_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                        
                                        // Check if it's an expression
                                        if max_parallel_cleaned.starts_with("${{") {
                                            // Expressions are valid
                                        } else if max_parallel_text.trim().starts_with('"') || max_parallel_text.trim().starts_with('\'') {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has invalid max-parallel: '{}'. max-parallel must be a number, not a string.",
                                                    job_name, max_parallel_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: max_parallel_node.start_byte(),
                                                    end: max_parallel_node.end_byte(),
                                                },
                                            });
                                        } else {
                                            match max_parallel_cleaned.parse::<i64>() {
                                                Ok(value) => {
                                                    if value < 0 {
                                                        diagnostics.push(Diagnostic {
                                                            message: format!(
                                                                "Job '{}' has invalid max-parallel: '{}'. max-parallel must be a positive integer.",
                                                                job_name, max_parallel_cleaned
                                                            ),
                                                            severity: Severity::Error,
                                                            span: Span {
                                                                start: max_parallel_node.start_byte(),
                                                                end: max_parallel_node.end_byte(),
                                                            },
                                                        });
                                                    } else if value == 0 {
                                                        diagnostics.push(Diagnostic {
                                                            message: format!(
                                                                "Job '{}' has invalid max-parallel: '{}'. max-parallel must be a positive integer (greater than zero).",
                                                                job_name, max_parallel_cleaned
                                                            ),
                                                            severity: Severity::Error,
                                                            span: Span {
                                                                start: max_parallel_node.start_byte(),
                                                                end: max_parallel_node.end_byte(),
                                                            },
                                                        });
                                                    }
                                                }
                                                Err(_) => {
                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Job '{}' has invalid max-parallel: '{}'. max-parallel must be a number or expression.",
                                                            job_name, max_parallel_cleaned
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: max_parallel_node.start_byte(),
                                                            end: max_parallel_node.end_byte(),
                                                        },
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Check fail-fast
                                    let fail_fast_value = utils::find_value_for_key(strategy_node, source, "fail-fast");
                                    if let Some(fail_fast_node) = fail_fast_value {
                                        let fail_fast_text = utils::node_text(fail_fast_node, source);
                                        let fail_fast_cleaned = fail_fast_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                                        
                                        // Check if it's an expression
                                        if fail_fast_cleaned.starts_with("${{") {
                                            // Expressions are valid
                                        } else if fail_fast_text.trim().starts_with('"') || fail_fast_text.trim().starts_with('\'') {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Job '{}' has invalid fail-fast: '{}'. fail-fast must be a boolean, not a string.",
                                                    job_name, fail_fast_cleaned
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: fail_fast_node.start_byte(),
                                                    end: fail_fast_node.end_byte(),
                                                },
                                            });
                                        } else {
                                            // Check if it's a boolean
                                            let is_bool = fail_fast_cleaned == "true" || fail_fast_cleaned == "false";
                                            if !is_bool {
                                                // Check if it's a number
                                                if fail_fast_cleaned.parse::<f64>().is_ok() {
                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Job '{}' has invalid fail-fast: '{}'. fail-fast must be a boolean (true or false), not a number.",
                                                            job_name, fail_fast_cleaned
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: fail_fast_node.start_byte(),
                                                            end: fail_fast_node.end_byte(),
                                                        },
                                                    });
                                                } else {
                                                    diagnostics.push(Diagnostic {
                                                        message: format!(
                                                            "Job '{}' has invalid fail-fast: '{}'. fail-fast must be a boolean (true or false).",
                                                            job_name, fail_fast_cleaned
                                                        ),
                                                        severity: Severity::Error,
                                                        span: Span {
                                                            start: fail_fast_node.start_byte(),
                                                            end: fail_fast_node.end_byte(),
                                                        },
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
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        check_job_strategy(child, source, diagnostics);
                    }
                }
            }
        }

        check_job_strategy(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

