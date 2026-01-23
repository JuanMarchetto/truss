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

        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

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
                            if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                matrices.push((value_node, Span {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
                                }));
                            }
                        } else {
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
            let mut matrix_to_check = matrix_node;
            while matches!(matrix_to_check.kind(), "block_node" | "flow_node") {
                if let Some(inner) = matrix_to_check.child(0) {
                    matrix_to_check = inner;
                } else {
                    break;
                }
            }

            let is_empty = {
                let node_kind = matrix_to_check.kind();
                let matrix_text = utils::node_text(matrix_to_check, source);
                let trimmed = matrix_text.trim();
                let no_whitespace: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();

                let text_empty = no_whitespace == "{}" 
                    || no_whitespace == "[]" 
                    || trimmed.is_empty();
                
                if text_empty {
                    true
                } else {
                    match node_kind {
                        "block_mapping" | "flow_mapping" => {
                            let mut cursor = matrix_to_check.walk();
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
                                *has_matrix_keys = true;

                                if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                    let value_kind = value_node.kind();
                                    if value_kind != "block_sequence" && value_kind != "flow_sequence" {
                                        let value_text = utils::node_text(value_node, source);
                                        if !value_text.contains("${{") {
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

            let is_valid_structure = has_matrix_keys || has_include || has_exclude;

            if !is_valid_structure {
                diagnostics.push(Diagnostic {
                    message: "Invalid matrix syntax: matrix must contain keys or include/exclude".to_string(),
                    severity: Severity::Error,
                    span,
                });
            }

            fn validate_include_exclude(node: Node, source: &str, key_name: &str, diagnostics: &mut Vec<Diagnostic>, parent_span: Span) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_text = utils::node_text(key_node, source);
                            let key_cleaned = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                                .trim_end_matches(':');
                            
                            if key_cleaned == key_name {
                                if let Some(value_node) = node.child(2).or_else(|| node.child(1)) {
                                    let mut value_to_check = value_node;
                                    while value_to_check.kind() == "block_node" {
                                        if let Some(inner) = value_to_check.child(0) {
                                            value_to_check = inner;
                                        } else {
                                            break;
                                        }
                                    }

                                    let value_kind = value_to_check.kind();
                                    let is_array = value_kind == "block_sequence" || value_kind == "flow_sequence";

                                    if !is_array {
                                        let value_text = utils::node_text(value_to_check, source);
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

