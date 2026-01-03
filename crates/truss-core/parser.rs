//! YAML parsing using tree-sitter.
//! Provides incremental parsing capabilities.

use tree_sitter::{Parser, Tree, Language};
use tree_sitter_yaml as ts_yaml;

/// Safely convert LanguageFn to Language.
/// 
/// This encapsulates the necessary unsafe FFI call to initialize
/// the tree-sitter language from the C function pointer.
/// The unsafe code is safe in practice - it's calling a well-tested
/// C function provided by tree-sitter-yaml.
fn language_from_fn() -> Language {
    let lang_fn = ts_yaml::LANGUAGE.into_raw();
    unsafe {
        let raw_ptr = lang_fn();
        Language::from_raw(raw_ptr as *const _)
    }
}

/// YAML parser with incremental parsing support.
pub struct YamlParser {
    parser: Parser,
}

impl YamlParser {
    /// Create a new YAML parser.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        // Initialize language using safe wrapper function
        let language = language_from_fn();
        parser
            .set_language(&language)
            .expect("Failed to load tree-sitter YAML grammar");

        Self { parser }
    }

    /// Parse YAML source code into a syntax tree.
    ///
    /// This is a full parse. For incremental updates, use `parse_incremental`.
    pub fn parse(&mut self, source: &str) -> Result<Tree, ParseError> {
        self.parser
            .parse(source, None)
            .ok_or(ParseError::ParseFailed)
    }

    /// Parse YAML with an existing tree for incremental updates.
    ///
    /// This allows efficient re-parsing when only part of the document changes.
    /// The old_tree should be from a previous parse of a similar document.
    pub fn parse_incremental(
        &mut self,
        source: &str,
        old_tree: Option<&Tree>,
    ) -> Result<Tree, ParseError> {
        self.parser
            .parse(source, old_tree)
            .ok_or(ParseError::ParseFailed)
    }
}

impl Default for YamlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse error.
#[derive(Debug, Clone)]
pub enum ParseError {
    ParseFailed,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseFailed => write!(f, "Failed to parse YAML"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Convert tree-sitter node to a span.
///
/// This is a utility function for future use in validation rules
/// that need to report diagnostics with precise source locations.
#[allow(dead_code)]
pub fn node_to_span(node: &tree_sitter::Node) -> crate::Span {
    crate::Span {
        start: node.start_byte(),
        end: node.end_byte(),
    }
}

