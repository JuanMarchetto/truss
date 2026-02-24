//! Truss Core
//!
//! Core validation and analysis engine for CI/CD pipelines.
//! This crate is editor-agnostic and fully deterministic.

mod parser;
mod validation;

use parser::{ParseError, YamlParser};
use serde::{Deserialize, Serialize};
use std::fmt;
use validation::{
    ActionReferenceRule, ArtifactValidationRule, ConcurrencyRule, DefaultsValidationRule,
    EnvironmentRule, EventPayloadValidationRule, ExpressionValidationRule, GitHubActionsSchemaRule,
    JobContainerRule, JobIfExpressionRule, JobNameRule, JobNeedsRule, JobOutputsRule,
    JobStrategyValidationRule, MatrixStrategyRule, NonEmptyRule, PermissionsRule,
    ReusableWorkflowCallRule, RuleSet, RunnerLabelRule, RunsOnRequiredRule, SecretsValidationRule,
    StepContinueOnErrorRule, StepEnvValidationRule, StepIdUniquenessRule, StepIfExpressionRule,
    StepNameRule, StepOutputReferenceRule, StepShellRule, StepTimeoutRule, StepValidationRule,
    StepWorkingDirectoryRule, SyntaxRule, TimeoutRule, ValidationRule, WorkflowCallInputsRule,
    WorkflowCallOutputsRule, WorkflowCallSecretsRule, WorkflowInputsRule, WorkflowNameRule,
    WorkflowTriggerRule,
};

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
        rules.add_rule(WorkflowTriggerRule);
        rules.add_rule(JobNameRule);
        rules.add_rule(JobNeedsRule);
        rules.add_rule(StepValidationRule);
        rules.add_rule(ExpressionValidationRule);
        rules.add_rule(PermissionsRule);
        rules.add_rule(EnvironmentRule);
        rules.add_rule(WorkflowNameRule);
        rules.add_rule(MatrixStrategyRule);
        rules.add_rule(RunsOnRequiredRule);
        rules.add_rule(SecretsValidationRule);
        rules.add_rule(TimeoutRule);
        rules.add_rule(WorkflowInputsRule);
        rules.add_rule(JobOutputsRule);
        rules.add_rule(ConcurrencyRule);
        rules.add_rule(ActionReferenceRule);
        rules.add_rule(StepIdUniquenessRule);
        rules.add_rule(StepOutputReferenceRule);
        rules.add_rule(JobStrategyValidationRule);
        rules.add_rule(StepIfExpressionRule);
        rules.add_rule(JobIfExpressionRule);
        rules.add_rule(WorkflowCallInputsRule);
        rules.add_rule(WorkflowCallSecretsRule);
        rules.add_rule(ReusableWorkflowCallRule);
        rules.add_rule(WorkflowCallOutputsRule);
        rules.add_rule(StepContinueOnErrorRule);
        rules.add_rule(StepTimeoutRule);
        rules.add_rule(StepShellRule);
        rules.add_rule(StepWorkingDirectoryRule);
        rules.add_rule(ArtifactValidationRule);
        rules.add_rule(EventPayloadValidationRule);
        rules.add_rule(RunnerLabelRule);
        rules.add_rule(StepEnvValidationRule);
        rules.add_rule(JobContainerRule);
        rules.add_rule(StepNameRule);
        rules.add_rule(DefaultsValidationRule);

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
        let tree = match self.parser.parse(source) {
            Ok(tree) => tree,
            Err(_) => return Self::parse_error_result(source),
        };

        self.rules.validate_parallel(&tree, source)
    }

    /// Analyze with incremental parsing support.
    ///
    /// If an old_tree is provided, uses incremental parsing for better performance.
    pub fn analyze_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&tree_sitter::Tree>,
    ) -> TrussResult {
        let tree = match self.parse_maybe_incremental(source, old_tree) {
            Ok(tree) => tree,
            Err(_) => return Self::parse_error_result(source),
        };

        self.rules.validate_parallel(&tree, source)
    }

    /// Analyze a YAML document and return both diagnostics and the parsed tree.
    ///
    /// This is useful for LSP implementations that need to store the tree
    /// for incremental parsing on subsequent edits.
    pub fn analyze_with_tree(&mut self, source: &str) -> (TrussResult, tree_sitter::Tree) {
        let tree = match self.parser.parse(source) {
            Ok(tree) => tree,
            Err(_) => return (Self::parse_error_result(source), self.dummy_tree()),
        };

        let result = self.rules.validate_parallel(&tree, source);
        (result, tree)
    }

    /// Analyze with incremental parsing and return both diagnostics and the parsed tree.
    ///
    /// This is useful for LSP implementations that need to store the tree
    /// for incremental parsing on subsequent edits.
    pub fn analyze_incremental_with_tree(
        &mut self,
        source: &str,
        old_tree: Option<&tree_sitter::Tree>,
    ) -> (TrussResult, tree_sitter::Tree) {
        let tree = match self.parse_maybe_incremental(source, old_tree) {
            Ok(tree) => tree,
            Err(_) => return (Self::parse_error_result(source), self.dummy_tree()),
        };

        let result = self.rules.validate_parallel(&tree, source);
        (result, tree)
    }

    fn parse_maybe_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&tree_sitter::Tree>,
    ) -> Result<tree_sitter::Tree, ParseError> {
        match old_tree {
            Some(old) => self.parser.parse_incremental(source, Some(old)),
            None => self.parser.parse(source),
        }
    }

    fn parse_error_result(source: &str) -> TrussResult {
        TrussResult {
            diagnostics: vec![Diagnostic {
                message: "Failed to parse YAML".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: 0,
                    end: source.len().min(100),
                },
            }],
        }
    }

    fn dummy_tree(&mut self) -> tree_sitter::Tree {
        self.parser
            .parse("")
            .expect("BUG: tree-sitter failed to parse empty string; parser may be misconfigured")
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub span: Span,
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Text span associated with a diagnostic.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
