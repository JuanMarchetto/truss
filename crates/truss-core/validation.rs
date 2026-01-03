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

    fn validate(&self, _tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check if this looks like a GitHub Actions workflow
        // Basic check: should have 'name' or 'on' at top level
        let has_name = source.contains("name:");
        
        // Check for 'on:' at start of line (not 'runs-on:' etc)
        let has_on = source.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("on:") || trimmed.starts_with("\"on\":") || trimmed.starts_with("'on':")
        });

        if !has_name && !has_on {
            // Might not be a GitHub Actions workflow, skip validation
            return diagnostics;
        }

        // Check for required 'on' field for GitHub Actions
        if !has_on {
            diagnostics.push(Diagnostic {
                message: "GitHub Actions workflow must have an 'on' field".to_string(),
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

/// Validates YAML syntax using tree-sitter parse errors.
pub struct SyntaxRule;

impl ValidationRule for SyntaxRule {
    fn name(&self) -> &str {
        "syntax"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root_node = tree.root_node();

        // Check for parse errors (tree-sitter marks errors in the tree)
        if root_node.has_error() {
            // Walk the tree to find error nodes
            let mut cursor = root_node.walk();
            let mut has_errors = false;

            for child in root_node.children(&mut cursor) {
                if child.is_error() || child.is_missing() {
                    has_errors = true;
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Syntax error: {}",
                            source[child.start_byte()..child.end_byte().min(source.len())]
                                .chars()
                                .take(50)
                                .collect::<String>()
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: child.start_byte(),
                            end: child.end_byte(),
                        },
                    });
                }
            }

            if has_errors && diagnostics.is_empty() {
                // General syntax error if we couldn't pinpoint it
                diagnostics.push(Diagnostic {
                    message: "YAML syntax error detected".to_string(),
                    severity: Severity::Error,
                    span: Span {
                        start: 0,
                        end: source.len().min(100),
                    },
                });
            }
        }

        diagnostics
    }
}

