//! Validation rule framework.
//! Rules are independent and can run in parallel.

use crate::{Diagnostic, Severity, Span, TrussResult};
use tree_sitter::Tree;

/// A validation rule that checks the AST.
///
/// Rules must be:
/// - Pure functions (same input → same output)
/// - Independent (no dependencies on other rules)
/// - Deterministic (no side effects)
pub trait ValidationRule: Send + Sync {
    /// Name of the validation rule.
    fn name(&self) -> &str;

    /// Validate the AST and return diagnostics.
    ///
    /// This function must be:
    /// - Pure (no side effects)
    /// - Deterministic (same AST → same diagnostics)
    /// - Independent (doesn't depend on other rules)
    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic>;
}

/// Collection of validation rules.
pub struct RuleSet {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl RuleSet {
    /// Create a new empty rule set.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a validation rule.
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Get all rules.
    pub fn rules(&self) -> &[Box<dyn ValidationRule>] {
        &self.rules
    }

    /// Run all validation rules in parallel.
    ///
    /// Rules are independent and can run concurrently.
    /// Results are merged deterministically.
    pub fn validate_parallel(&self, tree: &Tree, source: &str) -> TrussResult {
        use rayon::prelude::*;

        // Run rules in parallel
        let all_diagnostics: Vec<Diagnostic> = self
            .rules
            .par_iter()
            .flat_map(|rule| rule.validate(tree, source))
            .collect();

        // Sort by position for deterministic output order
        let mut diagnostics = all_diagnostics;
        diagnostics.sort_by_key(|d| (d.span.start, d.severity));

        TrussResult { diagnostics }
    }

    /// Run all validation rules sequentially.
    ///
    /// Useful for debugging or when parallel overhead isn't worth it.
    pub fn validate_sequential(&self, tree: &Tree, source: &str) -> TrussResult {
        let mut all_diagnostics = Vec::new();

        for rule in &self.rules {
            all_diagnostics.extend(rule.validate(tree, source));
        }

        // Sort by position for deterministic output order
        all_diagnostics.sort_by_key(|d| (d.span.start, d.severity));

        TrussResult { diagnostics: all_diagnostics }
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

// Example validation rules for GitHub Actions

/// Validates that the document is not empty.
pub struct NonEmptyRule;

impl ValidationRule for NonEmptyRule {
    fn name(&self) -> &str {
        "non_empty"
    }

    fn validate(&self, _tree: &Tree, source: &str) -> Vec<Diagnostic> {
        if source.trim().is_empty() {
            vec![Diagnostic {
                message: "Document is empty".to_string(),
                severity: Severity::Warning,
                span: Span::default(),
            }]
        } else {
            Vec::new()
        }
    }
}

/// Validates that GitHub Actions workflows have required top-level fields.
pub struct GitHubActionsSchemaRule;

impl ValidationRule for GitHubActionsSchemaRule {
    fn name(&self) -> &str {
        "github_actions_schema"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        // Optimize: Use AST instead of string searching
        // Walk the tree to find top-level keys
        let root_node = tree.root_node();
        let mut has_name = false;
        let mut has_on = false;

        // Quick check: if document is very small, might not be a workflow
        if source.trim().len() < 10 {
            return Vec::new();
        }

        // Walk the document node to find top-level mapping keys
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            // Look for mapping nodes at top level
            if child.kind() == "block_mapping" || child.kind() == "flow_mapping" {
                // Check children for key-value pairs
                let mut key_cursor = child.walk();
                for key_node in child.children(&mut key_cursor) {
                    if key_node.kind() == "block_mapping_pair" || key_node.kind() == "flow_pair" {
                        // Get the key
                        let key_start = key_node.start_byte();
                        let key_end = key_node.end_byte().min(source.len());
                        if key_start < key_end {
                            // Extract key text (simplified - just check first few chars)
                            let key_text = &source[key_start..key_end.min(key_start + 20)];
                            if key_text.starts_with("name") {
                                has_name = true;
                            } else if key_text.starts_with("on") && (key_text.len() == 2 || key_text.as_bytes().get(2).map_or(false, |&b| b == b':' || b == b' ')) {
                                has_on = true;
                            }
                        }
                    }
                }
            }
        }

        // Fallback to string search if AST walk didn't find anything (for edge cases)
        if !has_name && !has_on {
            // Quick string check as fallback
            if source.contains("name:") {
                has_name = true;
            }
            // Check for 'on:' at start of line (more precise than contains)
            if source.as_bytes().windows(3).any(|w| w == b"on:") {
                has_on = true;
            }
            
            // If still nothing, might not be a GitHub Actions workflow
            if !has_name && !has_on {
                return Vec::new();
            }
        }

        // Check for required 'on' field for GitHub Actions
        if !has_on {
            return vec![Diagnostic {
                message: "GitHub Actions workflow must have an 'on' field".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: 0,
                    end: source.len().min(100),
                },
            }];
        }

        Vec::new()
    }
}

/// Validates YAML syntax using tree-sitter parse errors.
pub struct SyntaxRule;

impl ValidationRule for SyntaxRule {
    fn name(&self) -> &str {
        "syntax"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let root_node = tree.root_node();

        // Early exit: if no errors, return empty immediately
        if !root_node.has_error() {
            return Vec::new();
        }

        // Only walk tree if there are errors
        let mut diagnostics = Vec::new();
        let mut cursor = root_node.walk();

        for child in root_node.children(&mut cursor) {
            if child.is_error() || child.is_missing() {
                // Optimize: avoid string allocation for error message when possible
                let start = child.start_byte();
                let end = child.end_byte().min(source.len());
                let error_snippet = if end > start && end <= source.len() {
                    source[start..end].chars().take(50).collect::<String>()
                } else {
                    String::new()
                };
                
                diagnostics.push(Diagnostic {
                    message: format!("Syntax error: {}", error_snippet),
                    severity: Severity::Error,
                    span: Span {
                        start,
                        end,
                    },
                });
            }
        }

        // If we found errors but couldn't pinpoint them, add a general error
        if diagnostics.is_empty() {
            diagnostics.push(Diagnostic {
                message: "YAML syntax error detected".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: 0,
                    end: source.len().min(100),
                },
            });
        }

        diagnostics
    }
}

