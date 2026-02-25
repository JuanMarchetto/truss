use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates matrix strategy syntax in GitHub Actions workflows.
pub struct MatrixStrategyRule;

impl ValidationRule for MatrixStrategyRule {
    fn name(&self) -> &str {
        "matrix_strategy"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();

        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        fn find_matrix_nodes<'a>(
            node: Node<'a>,
            source: &str,
            matrices: &mut Vec<(Node<'a>, Span)>,
            depth: usize,
        ) {
            if depth > 10 {
                return; // Prevent infinite recursion
            }

            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_cleaned = utils::clean_key(key_node, source);

                        // Get value: last non-comment, non-":" child
                        let value_node_opt = {
                            let mut found = None;
                            for i in (1..node.child_count()).rev() {
                                if let Some(child) = node.child(i) {
                                    if child.kind() != "comment" && child.kind() != ":" {
                                        found = Some(child);
                                        break;
                                    }
                                }
                            }
                            found
                        };

                        if key_cleaned == "matrix" {
                            if let Some(value_node) = value_node_opt {
                                matrices.push((
                                    value_node,
                                    Span {
                                        start: node.start_byte(),
                                        end: node.end_byte(),
                                    },
                                ));
                            }
                        } else if let Some(value_node) = value_node_opt {
                            find_matrix_nodes(value_node, source, matrices, depth + 1);
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
            let matrix_to_check = utils::unwrap_node(matrix_node);

            let is_empty = {
                let node_kind = matrix_to_check.kind();
                let matrix_text = utils::node_text(matrix_to_check, source);
                let trimmed = matrix_text.trim();
                let no_whitespace: String =
                    trimmed.chars().filter(|c| !c.is_whitespace()).collect();

                let text_empty =
                    no_whitespace == "{}" || no_whitespace == "[]" || trimmed.is_empty();

                if text_empty {
                    true
                } else {
                    match node_kind {
                        "block_mapping" | "flow_mapping" => {
                            let mut cursor = matrix_to_check.walk();
                            let pair_count = matrix_to_check
                                .children(&mut cursor)
                                .filter(|child| {
                                    matches!(child.kind(), "block_mapping_pair" | "flow_pair")
                                })
                                .count();
                            pair_count == 0
                        }
                        "block_sequence" | "flow_sequence" => {
                            let mut cursor = matrix_to_check.walk();
                            let has_children =
                                matrix_to_check.children(&mut cursor).next().is_some();
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

            fn check_matrix_structure(
                node: Node,
                source: &str,
                has_include: &mut bool,
                has_exclude: &mut bool,
                has_matrix_keys: &mut bool,
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                match node.kind() {
                    "block_mapping" | "flow_mapping" => {
                        *has_matrix_keys = true;
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_matrix_structure(
                                child,
                                source,
                                has_include,
                                has_exclude,
                                has_matrix_keys,
                                diagnostics,
                            );
                        }
                    }
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_cleaned = utils::clean_key(key_node, source);

                            if key_cleaned == "include" {
                                *has_include = true;
                            } else if key_cleaned == "exclude" {
                                *has_exclude = true;
                            } else {
                                *has_matrix_keys = true;

                                // Validate matrix key name format
                                if !is_valid_matrix_key_name(key_cleaned) {
                                    diagnostics.push(Diagnostic {
                                        message: format!(
                                            "Invalid matrix key name: '{}'. Matrix keys must contain only alphanumeric characters, hyphens, and underscores.",
                                            key_cleaned
                                        ),
                                        severity: Severity::Error,
                                        span: Span {
                                            start: key_node.start_byte(),
                                            end: key_node.end_byte(),
                                        },
                                        rule_id: String::new(),
                                    });
                                }

                                // Get value node: last non-comment, non-":" child after key
                                let value_node_opt = {
                                    let mut found = None;
                                    for i in (1..node.child_count()).rev() {
                                        if let Some(child) = node.child(i) {
                                            if child.kind() != "comment" && child.kind() != ":" {
                                                found = Some(child);
                                                break;
                                            }
                                        }
                                    }
                                    found
                                };

                                // Validate matrix value must be an array (not a scalar)
                                if let Some(value_node) = value_node_opt {
                                    let value_to_check = utils::unwrap_node(value_node);

                                    let value_kind = value_to_check.kind();
                                    let is_array = value_kind == "block_sequence"
                                        || value_kind == "flow_sequence";

                                    if !is_array {
                                        let value_text = utils::node_text(value_to_check, source);
                                        if !value_text.contains("${{") {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Matrix key '{}' has invalid value: '{}'. Matrix values must be arrays, not scalars. Use '[{}]' instead.",
                                                    key_cleaned, value_text.trim(), value_text.trim()
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: value_to_check.start_byte(),
                                                    end: value_to_check.end_byte(),
                                                },
                                                rule_id: String::new(),
                                            });
                                        }
                                    } else {
                                        // Validate array elements (basic type validation)
                                        validate_matrix_array_elements(
                                            value_to_check,
                                            source,
                                            key_cleaned,
                                            diagnostics,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_matrix_structure(
                                child,
                                source,
                                has_include,
                                has_exclude,
                                has_matrix_keys,
                                diagnostics,
                            );
                        }
                    }
                }
            }

            check_matrix_structure(
                matrix_to_check,
                source,
                &mut has_include,
                &mut has_exclude,
                &mut has_matrix_keys,
                &mut diagnostics,
            );

            let is_valid_structure = has_matrix_keys || has_include || has_exclude;

            if !is_valid_structure {
                diagnostics.push(Diagnostic {
                    message: "Invalid matrix syntax: matrix must contain keys or include/exclude"
                        .to_string(),
                    severity: Severity::Error,
                    span,
                });
            }

            fn validate_include_exclude(
                node: Node,
                source: &str,
                key_name: &str,
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let key_cleaned = utils::clean_key(key_node, source);

                            if key_cleaned == key_name {
                                // Get value node: last non-comment, non-":" child
                                let value_node_opt = {
                                    let mut found = None;
                                    for i in (1..node.child_count()).rev() {
                                        if let Some(child) = node.child(i) {
                                            if child.kind() != "comment" && child.kind() != ":" {
                                                found = Some(child);
                                                break;
                                            }
                                        }
                                    }
                                    found
                                };

                                if let Some(value_node) = value_node_opt {
                                    let value_to_check = utils::unwrap_node(value_node);

                                    let value_kind = value_to_check.kind();
                                    let is_array = value_kind == "block_sequence"
                                        || value_kind == "flow_sequence";

                                    if !is_array {
                                        let value_text = utils::node_text(value_to_check, source);
                                        if !value_text.contains("${{") {
                                            diagnostics.push(Diagnostic {
                                                message: format!(
                                                    "Invalid {} syntax: must be an array",
                                                    key_name
                                                ),
                                                severity: Severity::Error,
                                                span: Span {
                                                    start: value_to_check.start_byte(),
                                                    end: value_to_check.end_byte(),
                                                },
                                                rule_id: String::new(),
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
                            validate_include_exclude(child, source, key_name, diagnostics);
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            validate_include_exclude(child, source, key_name, diagnostics);
                        }
                    }
                }
            }

            validate_include_exclude(matrix_to_check, source, "include", &mut diagnostics);
            validate_include_exclude(matrix_to_check, source, "exclude", &mut diagnostics);
        }

        diagnostics
    }
}

/// Validates that a matrix key name follows the correct format.
/// Matrix keys must contain only alphanumeric characters, hyphens, and underscores.
fn is_valid_matrix_key_name(key_name: &str) -> bool {
    if key_name.is_empty() {
        return false;
    }

    key_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validates matrix array elements (basic type validation)
fn validate_matrix_array_elements(
    array_node: Node,
    source: &str,
    key_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cursor = array_node.walk();
    for child in array_node.children(&mut cursor) {
        // Skip bracket nodes in flow sequences: "[" and "]"
        let child_kind = child.kind();
        if child_kind == "[" || child_kind == "]" {
            continue;
        }

        let mut element_to_check = child;

        // Unwrap block_node and flow_node to get to the actual element
        while matches!(element_to_check.kind(), "block_node" | "flow_node") {
            if let Some(inner) = element_to_check.child(0) {
                element_to_check = inner;
            } else {
                break;
            }
        }

        // Unwrap block_sequence_item to get its content (skip "-" and comments)
        if element_to_check.kind() == "block_sequence_item" {
            for i in 0..element_to_check.child_count() {
                if let Some(seq_child) = element_to_check.child(i) {
                    if seq_child.kind() != "-" && seq_child.kind() != "comment" {
                        element_to_check = seq_child;
                        break;
                    }
                }
            }
            // Continue unwrapping block_node/flow_node after sequence item
            while matches!(element_to_check.kind(), "block_node" | "flow_node") {
                if let Some(inner) = element_to_check.child(0) {
                    element_to_check = inner;
                } else {
                    break;
                }
            }
        }

        match element_to_check.kind() {
            "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" | "block_scalar" => {
                // Scalar values are valid
            }
            "block_mapping" | "flow_mapping" => {
                // Objects in arrays are valid (for include/exclude)
            }
            "block_sequence" | "flow_sequence" => {
                // Nested arrays are valid
            }
            _ => {
                // Other types might be invalid
                let value_text = utils::node_text(element_to_check, source);
                if !value_text.contains("${{") && !value_text.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Matrix key '{}' has potentially invalid array element type. Matrix values should be strings, numbers, or expressions.",
                            key_name
                        ),
                        severity: Severity::Warning,
                        span: Span {
                            start: element_to_check.start_byte(),
                            end: element_to_check.end_byte(),
                        },
                        rule_id: String::new(),
                    });
                }
            }
        }
    }
}
