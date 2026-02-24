use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// Validates GitHub Actions expressions.
pub struct ExpressionValidationRule;

impl ValidationRule for ExpressionValidationRule {
    fn name(&self) -> &str {
        "expression"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        for expr in utils::find_expressions(source) {
            let inner = expr.inner.trim();

            // Detect unclosed expressions: find_expressions sets end to
            // source.len() when no closing }} is found.
            let is_closed =
                source.get(expr.end.saturating_sub(2)..expr.end) == Some("}}");
            if !is_closed {
                diagnostics.push(Diagnostic {
                    message: "unclosed expression".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: expr.start,
                        end: source.len().min(expr.start + 50),
                    },
                });
                continue;
            }

            if inner.is_empty() {
                diagnostics.push(Diagnostic {
                    message: "Empty expression".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: expr.start,
                        end: expr.end,
                    },
                });
            } else if !utils::is_valid_expression_syntax(inner) {
                diagnostics.push(Diagnostic {
                    message: format!("Invalid expression syntax: '{}'", inner),
                    severity: Severity::Error,
                    span: Span {
                        start: expr.start,
                        end: expr.end,
                    },
                });
            }

            // Validate operators
            validate_expression_operators(inner, expr.start, expr.end, &mut diagnostics);

            // Validate function calls
            validate_expression_functions(inner, expr.start, expr.end, &mut diagnostics);
        }

        diagnostics
    }
}

/// Validates expression operators
fn validate_expression_operators(
    expr: &str,
    start: usize,
    end: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Check for invalid operator combinations
    if expr.contains("===") || expr.contains("!==") {
        diagnostics.push(Diagnostic {
            message: format!(
                "Invalid operator in expression: '{}'. GitHub Actions expressions use '==' and '!=' for equality, not '===' or '!=='.",
                expr
            ),
            severity: Severity::Error,
            span: Span { start, end },
        });
    }

    // Check for invalid assignment operators (expressions are read-only)
    if expr.contains("=")
        && !expr.contains("==")
        && !expr.contains("!=")
        && !expr.contains("<=")
        && !expr.contains(">=")
    {
        // Might be assignment - warn
        if expr.matches('=').count() == 1 && !expr.contains("${{") {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Potentially invalid operator in expression: '{}'. Expressions are read-only and cannot use assignment operators.",
                    expr
                ),
                severity: Severity::Warning,
                span: Span {
                    start,
                    end,
                },
            });
        }
    }
}

/// Validates expression function calls
fn validate_expression_functions(
    expr: &str,
    start: usize,
    _end: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let valid_functions = [
        "contains",
        "startsWith",
        "endsWith",
        "format",
        "join",
        "toJSON",
        "fromJSON",
        "hashFiles",
        "success",
        "failure",
        "cancelled",
        "always",
    ];

    // Case-insensitive variants that GitHub Actions accepts
    let case_insensitive_functions = ["tojson", "fromjson"];

    // Find function calls in expression
    let mut search_pos = 0;
    while let Some(pos) = expr[search_pos..].find('(') {
        let actual_pos = search_pos + pos;
        let before_paren = &expr[..actual_pos];

        // Find function name (backwards from '(')
        if let Some(func_start) = before_paren.rfind(|c: char| {
            c.is_whitespace()
                || c == '&'
                || c == '|'
                || c == '!'
                || c == '('
                || c == '['
                || c == '.'
        }) {
            let func_name = expr[func_start + 1..actual_pos].trim();
            if !func_name.is_empty() {
                let is_valid = valid_functions.contains(&func_name)
                    || case_insensitive_functions.contains(&func_name.to_lowercase().as_str());
                if !is_valid {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Unknown function in expression: '{}'. Valid functions are: {}.",
                            func_name,
                            valid_functions.join(", ")
                        ),
                        severity: Severity::Warning,
                        span: Span {
                            start: start + func_start + 1,
                            end: start + actual_pos,
                        },
                    });
                }
            }
        }

        search_pos = actual_pos + 1;
    }
}
