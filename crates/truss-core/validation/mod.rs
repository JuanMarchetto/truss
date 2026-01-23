//! Validation rule framework.
//! Rules are independent and can run in parallel.

use crate::{Diagnostic, TrussResult};
use tree_sitter::Tree;

pub mod rules;
pub mod utils;

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

    /// Run all validation rules in parallel.
    ///
    /// Rules are independent and can run concurrently.
    /// Results are merged deterministically.
    pub fn validate_parallel(&self, tree: &Tree, source: &str) -> TrussResult {
        use rayon::prelude::*;

        let all_diagnostics: Vec<Diagnostic> = self
            .rules
            .par_iter()
            .flat_map(|rule| rule.validate(tree, source))
            .collect();

        let mut diagnostics = all_diagnostics;
        diagnostics.sort_by_key(|d| (d.span.start, d.severity));

        TrussResult { diagnostics }
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export all rules for backward compatibility
pub use rules::*;

