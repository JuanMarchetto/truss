//! Truss Core
//!
//! Core validation and analysis engine for CI/CD pipelines.
//! This crate is editor-agnostic and fully deterministic.

use std::fmt;

/// Entry point for the Truss validation engine.
#[derive(Default)]
pub struct TrussEngine;

impl TrussEngine {
    /// Creates a new engine instance.
    ///
    /// The engine is stateless for now, but this allows
    /// future configuration without breaking the API.
    pub fn new() -> Self {
        Self
    }

    /// Analyze a YAML document and return diagnostics.
    ///
    /// This function must:
    /// - Be deterministic
    /// - Avoid side effects
    /// - Be cheap to call repeatedly
    pub fn analyze(&self, source: &str) -> TrussResult {
        // Placeholder parse logic.
        // Real implementation will use tree-sitter.
        if source.trim().is_empty() {
            return TrussResult {
                diagnostics: vec![Diagnostic {
                    message: "Document is empty".to_string(),
                    severity: Severity::Warning,
                    span: Span::default(),
                }],
            };
        }

        TrussResult {
            diagnostics: Vec::new(),
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let engine = TrussEngine::new();
        let result = engine.analyze("");

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("empty"));
    }

    #[test]
    fn non_empty_document_is_ok() {
        let engine = TrussEngine::new();
        let input = "name: test\non: push";

        let result = engine.analyze(input);

        assert!(result.is_ok());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn analysis_is_deterministic() {
        let engine = TrussEngine::new();
        let input = "name: test\non: push";

        let result_a = engine.analyze(input);
        let result_b = engine.analyze(input);

        assert_eq!(result_a.diagnostics.len(), result_b.diagnostics.len());
    }

    #[test]
    fn engine_can_be_reused_multiple_times() {
        let engine = TrussEngine::new();

        let first = engine.analyze("name: first");
        let second = engine.analyze("name: second");

        assert!(first.is_ok());
        assert!(second.is_ok());
    }
}
