//! YAML parsing using tree-sitter.
//! Provides incremental parsing capabilities.

use tree_sitter::{Parser, Tree};
use tree_sitter_yaml as ts_yaml;

/// YAML parser with incremental parsing support.
pub struct YamlParser {
    parser: Parser,
}

impl YamlParser {
    /// Create a new YAML parser.
    ///
    /// # Panics
    ///
    /// Panics if the compiled-in tree-sitter YAML grammar fails to load.
    /// This should never happen in practice since the grammar is statically
    /// linked at compile time.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = ts_yaml::language();
        parser
            .set_language(&language)
            .expect("Failed to load tree-sitter YAML grammar — this is a build-time bug");

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
