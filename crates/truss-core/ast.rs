//! Abstract Syntax Tree representation.
//! Provides incremental AST updates.

use tree_sitter::Tree;

/// Incremental AST that can be updated efficiently.
///
/// This is a placeholder for future incremental AST implementation.
/// It will be used by the LSP adapter for efficient document updates.
#[allow(dead_code)]
pub struct IncrementalAst {
    tree: Tree,
    source: String,
}

#[allow(dead_code)]
impl IncrementalAst {
    /// Create a new AST from a full parse.
    pub fn new(tree: Tree, source: String) -> Self {
        Self { tree, source }
    }

    /// Get the underlying tree-sitter tree.
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Get the source code.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Update the AST incrementally with new source.
    ///
    /// This is more efficient than a full re-parse when only part
    /// of the document has changed.
    pub fn update_incremental(
        &mut self,
        new_source: String,
        parser: &mut crate::parser::YamlParser,
    ) -> Result<(), crate::parser::ParseError> {
        let new_tree = parser.parse_incremental(&new_source, Some(&self.tree))?;
        self.tree = new_tree;
        self.source = new_source;
        Ok(())
    }
}

