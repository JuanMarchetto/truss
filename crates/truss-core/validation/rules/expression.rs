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

        let mut pos = 0;
        while let Some(start) = source[pos..].find("${{") {
            let actual_start = pos + start;
            let after_start = actual_start + 3;

            // Skip expressions inside YAML comments (# ... ${{ expr }})
            let line_start = source[..actual_start]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let line_before = &source[line_start..actual_start];
            if line_before.trim_start().starts_with('#') {
                pos = after_start;
                continue;
            }

            if let Some(end_offset) = source[after_start..].find("}}") {
                let end = after_start + end_offset + 2;
                let expr = &source[actual_start..end];

                let inner = expr[3..expr.len() - 2].trim();
                if inner.is_empty() {
                    diagnostics.push(Diagnostic {
                        message: "Empty expression".to_string(),
                        severity: Severity::Error,
                        span: Span {
                            start: actual_start,
                            end,
                        },
                    });
                } else if !utils::is_valid_expression_syntax(inner) {
                    diagnostics.push(Diagnostic {
                        message: format!("Invalid expression syntax: '{}'", inner),
                        severity: Severity::Error,
                        span: Span {
                            start: actual_start,
                            end,
                        },
                    });
                }

                if inner.contains("github.nonexistent") || inner.contains("nonexistent.property") {
                    diagnostics.push(Diagnostic {
                        message: format!("Undefined context variable: {}", inner),
                        severity: Severity::Warning,
                        span: Span {
                            start: actual_start,
                            end,
                        },
                    });
                }

                // Validate operators
                validate_expression_operators(inner, actual_start, end, &mut diagnostics);

                // Validate function calls
                validate_expression_functions(inner, actual_start, end, &mut diagnostics);

                pos = end;
            } else {
                diagnostics.push(Diagnostic {
                    message: "unclosed expression".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: actual_start,
                        end: source.len().min(actual_start + 50),
                    },
                });
                break;
            }
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
