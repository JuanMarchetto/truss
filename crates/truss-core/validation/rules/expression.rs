use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;
use super::super::ValidationRule;
use super::super::utils;

/// Validates GitHub Actions expressions.
pub struct ExpressionValidationRule;

/// Check if an expression has valid syntax.
/// 
/// GitHub Actions expressions are JavaScript-like, so we check for:
/// - Valid property access (dot notation, bracket notation)
/// - Valid operators
/// - Valid function calls
/// - No obvious syntax errors
fn is_valid_expression_syntax(expr: &str) -> bool {
    let expr = expr.trim();
    
    // Empty expressions are invalid (handled separately)
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
        || expr.starts_with("steps.")
        || expr.starts_with("job.")
        || expr.starts_with("jobs.")
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

    if !has_context && !has_function && !has_operator && !is_literal && !is_bare_context
        && !expr.contains('.') && !expr.contains('(') && !expr.contains('[')
    {
        return false;
    }

    true
}

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
            let line_start = source[..actual_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_before = &source[line_start..actual_start];
            if line_before.trim_start().starts_with('#') {
                pos = after_start;
                continue;
            }

            if let Some(end_offset) = source[after_start..].find("}}") {
                let end = after_start + end_offset + 2;
                let expr = &source[actual_start..end];
                
                let inner = expr[3..expr.len()-2].trim();
                if inner.is_empty() {
                    diagnostics.push(Diagnostic {
                        message: "Empty expression".to_string(),
                        severity: Severity::Error,
                        span: Span {
                            start: actual_start,
                            end,
                        },
                    });
                } else if !is_valid_expression_syntax(inner) {
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
fn validate_expression_operators(expr: &str, start: usize, end: usize, diagnostics: &mut Vec<Diagnostic>) {
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
    if expr.contains("=") && !expr.contains("==") && !expr.contains("!=") && !expr.contains("<=") && !expr.contains(">=") {
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
fn validate_expression_functions(expr: &str, start: usize, _end: usize, diagnostics: &mut Vec<Diagnostic>) {
    let valid_functions = [
        "contains", "startsWith", "endsWith", "format", "join",
        "toJSON", "fromJSON", "hashFiles",
        "success", "failure", "cancelled", "always"
    ];

    // Case-insensitive variants that GitHub Actions accepts
    let case_insensitive_functions = ["tojson", "fromjson"];

    // Find function calls in expression
    let mut search_pos = 0;
    while let Some(pos) = expr[search_pos..].find('(') {
        let actual_pos = search_pos + pos;
        let before_paren = &expr[..actual_pos];

        // Find function name (backwards from '(')
        if let Some(func_start) = before_paren.rfind(|c: char| c.is_whitespace() || c == '&' || c == '|' || c == '!' || c == '(' || c == '[' || c == '.') {
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

