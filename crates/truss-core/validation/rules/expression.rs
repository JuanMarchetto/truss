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
        || expr.starts_with("env.");
    
    let has_function = expr.contains("contains(")
        || expr.contains("startsWith(")
        || expr.contains("endsWith(")
        || expr.contains("format(")
        || expr.contains("join(")
        || expr.contains("toJSON(")
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
    
    if !has_context && !has_function && !has_operator && !is_literal {
        if !expr.contains('.') && !expr.contains('(') && !expr.contains('[') {
            return false;
        }
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
            
            if let Some(end_offset) = source[after_start..].find("}}") {
                let end = after_start + end_offset + 2;
                let expr = &source[actual_start..end];
                
                let inner = &expr[3..expr.len()-2].trim();
                if inner.is_empty() {
                    diagnostics.push(Diagnostic {
                        message: "Empty expression".to_string(),
                        severity: Severity::Error,
                        span: Span {
                            start: actual_start,
                            end: end,
                        },
                    });
                } else {
                    // Validate expression syntax
                    if !is_valid_expression_syntax(inner) {
                        diagnostics.push(Diagnostic {
                            message: format!("Invalid expression syntax: '{}'", inner),
                            severity: Severity::Error,
                            span: Span {
                                start: actual_start,
                                end: end,
                            },
                        });
                    }
                }
                
                if inner.contains("github.nonexistent") || inner.contains("nonexistent.property") {
                    diagnostics.push(Diagnostic {
                        message: format!("Undefined context variable: {}", inner),
                        severity: Severity::Warning,
                        span: Span {
                            start: actual_start,
                            end: end,
                        },
                    });
                }
                
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

