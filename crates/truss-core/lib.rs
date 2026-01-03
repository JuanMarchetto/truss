//! Truss Core
//!
//! Core validation and analysis engine for CI/CD pipelines.
//! This crate is editor-agnostic and fully deterministic.

mod ast;
mod parser;
mod validation;

use std::fmt;
use parser::YamlParser;
use validation::{RuleSet, ValidationRule, NonEmptyRule, GitHubActionsSchemaRule, SyntaxRule};

/// Entry point for the Truss validation engine.
pub struct TrussEngine {
    parser: YamlParser,
    rules: RuleSet,
}

impl TrussEngine {
    /// Creates a new engine instance with default validation rules.
    ///
    /// The engine maintains parser state for incremental parsing,
    /// but validation rules are stateless and reusable.
    pub fn new() -> Self {
        let mut rules = RuleSet::new();
        rules.add_rule(SyntaxRule);
        rules.add_rule(NonEmptyRule);
        rules.add_rule(GitHubActionsSchemaRule);

        Self {
            parser: YamlParser::new(),
            rules,
        }
    }

    /// Analyze a YAML document and return diagnostics.
    ///
    /// This function:
    /// - Parses YAML using tree-sitter
    /// - Runs validation rules in parallel
    /// - Returns deterministic results
    /// - Is cheap to call repeatedly
    pub fn analyze(&mut self, source: &str) -> TrussResult {
        // Parse YAML
        let tree = match self.parser.parse(source) {
            Ok(tree) => tree,
            Err(_) => {
                // Parse failed, return syntax error
                return TrussResult {
                    diagnostics: vec![Diagnostic {
                        message: "Failed to parse YAML".to_string(),
                        severity: Severity::Error,
                        span: Span {
                            start: 0,
                            end: source.len().min(100),
                        },
                    }],
                };
            }
        };

        // Run validation rules in parallel
        // Automatically uses parallel when beneficial (many rules)
        if self.rules.rules().len() > 3 {
            self.rules.validate_parallel(&tree, source)
        } else {
            self.rules.validate_sequential(&tree, source)
        }
    }

    /// Analyze with incremental parsing support.
    ///
    /// If an old_tree is provided, uses incremental parsing for better performance.
    pub fn analyze_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&tree_sitter::Tree>,
    ) -> TrussResult {
        // Parse YAML incrementally if possible
        let tree = match old_tree {
            Some(old) => self.parser.parse_incremental(source, Some(old)),
            None => self.parser.parse(source),
        };

        let tree = match tree {
            Ok(tree) => tree,
            Err(_) => {
                return TrussResult {
                    diagnostics: vec![Diagnostic {
                        message: "Failed to parse YAML".to_string(),
                        severity: Severity::Error,
                        span: Span {
                            start: 0,
                            end: source.len().min(100),
                        },
                    }],
                };
            }
        };

        // Run validation rules
        if self.rules.rules().len() > 3 {
            self.rules.validate_parallel(&tree, source)
        } else {
            self.rules.validate_sequential(&tree, source)
        }
    }

    /// Add a custom validation rule.
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.add_rule(rule);
    }
}

impl Default for TrussEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a Truss analysis pass.
#[derive(Debug)]
pub struct TrussResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl TrussResult {
    /// Returns true if no errors were found.
    pub fn is_ok(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// A diagnostic produced by the engine.
#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub span: Span,
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Text span associated with a diagnostic.
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] {} ({}..{})",
            self.severity, self.message, self.span.start, self.span.end
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_produces_warning() {
        let mut engine = TrussEngine::new();
        let result = engine.analyze("");

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("empty"));
    }

    #[test]
    fn non_empty_document_is_ok() {
        let mut engine = TrussEngine::new();
        let input = "name: test\non: push";

        let result = engine.analyze(input);

        assert!(result.is_ok());
        // May have diagnostics from validation rules, but should be ok (no errors)
    }

    #[test]
    fn analysis_is_deterministic() {
        let mut engine = TrussEngine::new();
        let input = "name: test\non: push";

        let result_a = engine.analyze(input);
        let result_b = engine.analyze(input);

        assert_eq!(result_a.diagnostics.len(), result_b.diagnostics.len());
    }

    #[test]
    fn engine_can_be_reused_multiple_times() {
        let mut engine = TrussEngine::new();

        let first = engine.analyze("name: first\non: push");
        let second = engine.analyze("name: second\non: push");

        assert!(first.is_ok());
        assert!(second.is_ok());
    }
}
