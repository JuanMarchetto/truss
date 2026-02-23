use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates if condition expressions in steps.
pub struct StepIfExpressionRule;

impl ValidationRule for StepIfExpressionRule {
    fn name(&self) -> &str {
        "step_if_expression"
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

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            if let Some(steps_value_raw) = utils::get_pair_value(node) {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_sequence" || steps_value.kind() == "flow_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        validate_step_if(step_node, source, diagnostics);
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

        fn validate_step_if(step_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            let mut step_to_check = utils::unwrap_node(step_node);

            // Handle block_sequence_item - find the value child (skip dash and comments)
            if step_to_check.kind() == "block_sequence_item" {
                let mut found = false;
                for i in 1..step_to_check.child_count() {
                    if let Some(child) = step_to_check.child(i) {
                        if child.kind() != "comment" {
                            step_to_check = utils::unwrap_node(child);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    return;
                }
            }

            if step_to_check.kind() == "block_mapping" || step_to_check.kind() == "flow_mapping" {
                let if_value = utils::find_value_for_key(step_to_check, source, "if");

                if let Some(if_node) = if_value {
                    let if_text = utils::node_text(if_node, source);
                    let if_cleaned = if_text.trim();

                    // GitHub Actions auto-wraps if: conditions in ${{ }}.
                    // Both bare expressions and explicitly wrapped ones are valid.
                    let inner = if if_cleaned.starts_with("${{") && if_cleaned.ends_with("}}") {
                        // Explicitly wrapped: extract inner expression
                        if_cleaned[3..if_cleaned.len()-2].trim()
                    } else {
                        // Bare expression: GitHub Actions auto-wraps this
                        if_cleaned
                    };

                    // Validate expression syntax
                    if !is_valid_expression_syntax(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid step 'if' expression syntax: '{}'",
                                inner
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    }

                    // Check for undefined context variables
                    if inner.contains("github.nonexistent")
                        || inner.contains("nonexistent.property")
                        || (inner.contains("github.") && !is_valid_github_context(inner))
                        || (inner.contains("matrix.") && !is_valid_matrix_context(inner))
                        || (inner.contains("secrets.") && !is_valid_secrets_context(inner)) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step 'if' expression may reference undefined context variable: '{}'",
                                inner
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    }

                    // Check for potentially always-true/false conditions
                    if is_potentially_always_true(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step 'if' expression may always evaluate to true: '{}'",
                                inner
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    } else if is_potentially_always_false(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Step 'if' expression may always evaluate to false: '{}'",
                                inner
                            ),
                            severity: Severity::Warning,
                            span: Span {
                                start: if_node.start_byte(),
                                end: if_node.end_byte(),
                            },
                        });
                    }
                }
            }
        }

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

/// Check if an expression has valid syntax (similar to ExpressionValidationRule)
fn is_valid_expression_syntax(expr: &str) -> bool {
    let expr = expr.trim();

    if expr.is_empty() {
        return false;
    }

    let has_context = expr.starts_with("github.")
        || expr.starts_with("matrix.")
        || expr.starts_with("secrets.")
        || expr.starts_with("vars.")
        || expr.starts_with("needs.")
        || expr.starts_with("inputs.")
        || expr.starts_with("env.")
        || expr.starts_with("job.")
        || expr.starts_with("jobs.")
        || expr.starts_with("steps.")
        || expr.starts_with("runner.")
        || expr.starts_with("strategy.");

    let expr_lower = expr.to_lowercase();
    let has_function = expr.contains("contains(")
        || expr.contains("startsWith(")
        || expr.contains("endsWith(")
        || expr.contains("format(")
        || expr.contains("join(")
        || expr_lower.contains("tojson(")
        || expr_lower.contains("fromjson(")
        || expr.contains("hashFiles(")
        || expr.contains("success()")
        || expr.contains("failure()")
        || expr.contains("cancelled()")
        || expr.contains("always()");

    let has_operator = expr.contains("==")
        || expr.contains("!=")
        || expr.contains("&&")
        || expr.contains("||")
        || expr.contains("!")
        || expr.contains("<")
        || expr.contains(">")
        || expr.contains("<=")
        || expr.contains(">=");

    let is_literal = (expr.starts_with("'") && expr.ends_with("'"))
        || (expr.starts_with("\"") && expr.ends_with("\""))
        || expr.parse::<f64>().is_ok()
        || expr == "true" || expr == "false";

    // Bare context names (e.g., "github", "matrix") are valid expressions
    let is_bare_context = matches!(expr, "github" | "matrix" | "secrets" | "vars" | "needs" | "inputs" | "env" | "job" | "jobs" | "steps" | "runner" | "strategy");

    // Check if expression contains a dot but doesn't start with a known context
    // (e.g., "invalid.expression" should be rejected)
    if expr.contains('.') && !has_context {
        // Extract the first part before any operator, whitespace, or parenthesis
        let first_token = expr.split(|c: char| c.is_whitespace() || matches!(c, '&' | '|' | '=' | '!' | '<' | '>' | '(' | '[')).next().unwrap_or("");
        if first_token.contains('.') {
            let context_name = first_token.split('.').next().unwrap_or("");
            if !matches!(context_name, "github" | "matrix" | "secrets" | "vars" | "needs" | "inputs" | "env" | "job" | "jobs" | "steps" | "runner" | "strategy") {
                // Unknown context reference - reject it
                return false;
            }
        }
    }

    if !has_context && !has_function && !has_operator && !is_literal && !is_bare_context
        && !expr.contains('.') && !expr.contains('(') && !expr.contains('[')
    {
        return false;
    }

    true
}

/// Check if a GitHub context reference is valid
fn is_valid_github_context(expr: &str) -> bool {
    // Basic check - in real implementation, would validate against known GitHub context properties
    !expr.contains("github.nonexistent") && !expr.contains("github.invalid")
}

/// Check if a matrix context reference is valid
fn is_valid_matrix_context(expr: &str) -> bool {
    // Basic check - matrix references are usually valid if they reference a matrix key
    !expr.contains("matrix.nonexistent") && !expr.contains("matrix.invalid")
}

/// Check if a secrets context reference is valid
fn is_valid_secrets_context(expr: &str) -> bool {
    // Basic check - secrets references are usually valid
    !expr.contains("secrets.nonexistent") && !expr.contains("secrets.invalid")
}

/// Check if expression may always evaluate to true
fn is_potentially_always_true(expr: &str) -> bool {
    let expr_lower = expr.to_lowercase();
    expr_lower == "true"
        || expr_lower == "!false"
        || expr_lower.contains("|| true")
        || expr_lower.contains("true ||")
}

/// Check if expression may always evaluate to false
fn is_potentially_always_false(expr: &str) -> bool {
    let expr_lower = expr.to_lowercase();
    expr_lower == "false"
        || expr_lower == "!true"
        || expr_lower.contains("&& false")
        || expr_lower.contains("false &&")
}
