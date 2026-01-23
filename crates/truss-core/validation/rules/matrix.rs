use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;

/// Validates matrix strategy syntax in GitHub Actions workflows.
pub struct MatrixStrategyRule;

impl ValidationRule for MatrixStrategyRule {
    fn name(&self) -> &str {
        "matrix_strategy"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        let root = tree.root_node();
        
        // Find all jobs
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        // Find all matrix definitions within jobs
        fn find_matrix_nodes<'a>(node: Node<'a>, source: &str, matrices: &mut Vec<(Node<'a>, Span)>, depth: usize) {
            if depth > 10 {
                return; // Prevent infinite recursion
            }
            
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        
                        if key_cleaned == "matrix" {
                            // Find the value node - try child(2) first, then child(1)
                            // In tree-sitter YAML, block_mapping_pair structure is typically:
                            // child(0): key, child(1): value (for block mappings) or child(2): value
                            if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                matrices.push((value_node, Span {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
                                }));
                            }
                        } else {
                            // Not a matrix key, but we still need to recurse into its value
                            // to find matrix keys nested deeper (e.g., strategy -> matrix)
                            if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                find_matrix_nodes(value_node, source, matrices, depth + 1);
                            }
                        }
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_matrix_nodes(child, source, matrices, depth + 1);
                    }
                }
            }
        }

        let mut matrix_nodes = Vec::new();
        find_matrix_nodes(jobs_value, source, &mut matrix_nodes, 0);

        for (matrix_node, span) in matrix_nodes {
            // Handle block_node and flow_node wrappers - may need to unwrap multiple levels
            let mut matrix_to_check = matrix_node;
            while matches!(matrix_to_check.kind(), "block_node" | "flow_node") {
                if let Some(inner) = matrix_to_check.child(0) {
                    matrix_to_check = inner;
                } else {
                    break;
                }
            }

            // Check for empty matrix - check if it's a mapping/sequence with no children
            let is_empty = {
                let node_kind = matrix_to_check.kind();
                
                // First check text content (most reliable for flow_mapping like {})
                let matrix_text = utils::node_text(matrix_to_check, source);
                let trimmed = matrix_text.trim();
                // Remove all whitespace and check if it's just {} or []
                let no_whitespace: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
                
                // Check if it's just {} or []
                let text_empty = no_whitespace == "{}" 
                    || no_whitespace == "[]" 
                    || trimmed.is_empty();
                
                if text_empty {
                    true
                } else {
                    // Also check structure - see if there are any mapping pairs
                    match node_kind {
                        "block_mapping" | "flow_mapping" => {
                            let mut cursor = matrix_to_check.walk();
                            // Count actual mapping pairs, not just any children
                            let pair_count = matrix_to_check.children(&mut cursor)
                                .filter(|child| matches!(child.kind(), "block_mapping_pair" | "flow_pair"))
                                .count();
                            pair_count == 0
                        }
                        "block_sequence" | "flow_sequence" => {
                            let mut cursor = matrix_to_check.walk();
                            let has_children = matrix_to_check.children(&mut cursor).next().is_some();
                            !has_children
                        }
                        _ => false,
                    }
                }
            };

            if is_empty {
                diagnostics.push(Diagnostic {
                    message: "matrix cannot be empty".to_string(),
                    severity: Severity::Error,
                    span,
                });
                continue;
            }

            // Check if matrix is a mapping (object)
            let mut has_include = false;
            let mut has_exclude = false;
            let mut has_matrix_keys = false;

            fn check_matrix_structure(node: Node, source: &str, has_include: &mut bool, has_exclude: &mut bool, has_matrix_keys: &mut bool) {
                match node.kind() {
                    "block_mapping" | "flow_mapping" => {
                        *has_matrix_keys = true;
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_matrix_structure(child, source, has_include, has_exclude, has_matrix_keys);
                        }
                    }
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_text = utils::node_text(key_node, source);
                            let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                .trim_end_matches(':');
                            
                            if key_cleaned == "include" {
                                *has_include = true;
                            } else if key_cleaned == "exclude" {
                                *has_exclude = true;
                            } else {
                                // Regular matrix key (like os, node-version, etc.)
                                *has_matrix_keys = true;
                                
                                // Validate that the value is an array
                                if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                    let value_kind = value_node.kind();
                                    if value_kind != "block_sequence" && value_kind != "flow_sequence" {
                                        // Allow expressions in matrix values
                                        let value_text = utils::node_text(value_node, source);
                                        if !value_text.contains("${{") {
                                            // Not an expression, should be an array
                                            // This is a warning, not an error, as GitHub Actions allows expressions
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_matrix_structure(child, source, has_include, has_exclude, has_matrix_keys);
                        }
                    }
                }
            }

            check_matrix_structure(matrix_to_check, source, &mut has_include, &mut has_exclude, &mut has_matrix_keys);

            // Matrix must have either regular keys or include/exclude
            let is_valid_structure = has_matrix_keys || has_include || has_exclude;

            if !is_valid_structure {
                diagnostics.push(Diagnostic {
                    message: "Invalid matrix syntax: matrix must contain keys or include/exclude".to_string(),
                    severity: Severity::Error,
                    span,
                });
            }

            // Always validate include/exclude structure (even if structure check passed)
            // This ensures we catch invalid include/exclude values
            fn validate_include_exclude(node: Node, source: &str, key_name: &str, diagnostics: &mut Vec<Diagnostic>, parent_span: Span) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_text = utils::node_text(key_node, source);
                            let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                .trim_end_matches(':');
                            
                            if key_cleaned == key_name {
                                if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                    // Handle block_node wrapper for value
                                    let mut value_to_check = value_node;
                                    while value_to_check.kind() == "block_node" {
                                        if let Some(inner) = value_to_check.child(0) {
                                            value_to_check = inner;
                                        } else {
                                            break;
                                        }
                                    }
                                    
                                    let value_kind = value_to_check.kind();
                                    // include/exclude should be arrays (block_sequence or flow_sequence)
                                    let is_array = value_kind == "block_sequence" || value_kind == "flow_sequence";
                                    
                                    if !is_array {
                                        let value_text = utils::node_text(value_to_check, source);
                                        // Allow expressions, but otherwise must be an array
                                        if !value_text.contains("${{") {
                                            diagnostics.push(Diagnostic {
                                                message: format!("Invalid {} syntax: must be an array", key_name),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: value_to_check.start_byte(),
                                                    end: value_to_check.end_byte(),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "block_mapping" | "flow_mapping" => {
                        // Explicitly iterate through mapping pairs
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            validate_include_exclude(child, source, key_name, diagnostics, parent_span);
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            validate_include_exclude(child, source, key_name, diagnostics, parent_span);
                        }
                    }
                }
            }

            validate_include_exclude(matrix_to_check, source, "include", &mut diagnostics, span);
            validate_include_exclude(matrix_to_check, source, "exclude", &mut diagnostics, span);
        }

        diagnostics
    }
}

