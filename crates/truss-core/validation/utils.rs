//! Helper utilities for validation rules.

use tree_sitter::{Node, Tree};

/// Check if a YAML document is a GitHub Actions workflow by examining top-level keys.
///
/// This function walks the AST to find top-level mapping keys and checks for
/// GitHub Actions-specific keys: `on`, `jobs`, or `name`.
///
/// Returns `true` if the document appears to be a GitHub Actions workflow.
pub(crate) fn is_github_actions_workflow(tree: &Tree, source: &str) -> bool {
    let root = tree.root_node();
    let mut top_level_keys = Vec::new();

    fn extract_key_text(node: Node, source: &str) -> Option<String> {
        let text = &source[node.start_byte()..node.end_byte()];
        let cleaned = text
            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
            .trim_end_matches(':');
        Some(cleaned.to_string())
    }

    fn find_top_level_keys(node: Node, source: &str, keys: &mut Vec<String>, depth: usize) {
        if depth > 4 {
            return;
        }

        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    if let Some(key_text) = extract_key_text(key_node, source) {
                        keys.push(key_text);
                    }
                }
            }
            "block_mapping" | "flow_mapping" | "document" | "stream" | "block_node" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    find_top_level_keys(child, source, keys, depth + 1);
                }
            }
            _ => {
                if depth < 4 {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_top_level_keys(child, source, keys, depth + 1);
                    }
                }
            }
        }
    }

    find_top_level_keys(root, source, &mut top_level_keys, 0);

    top_level_keys.iter().any(|key| {
        let key_lower = key.to_lowercase();
        let key_trimmed = key_lower.trim();
        key_trimmed == "on" || key_trimmed == "jobs" || key_trimmed == "name"
    })
}

/// Unwrap a block_node or flow_node to get its content child, skipping comments.
///
/// In tree-sitter-yaml, comments are extras that can appear as children of any node.
/// When unwrapping block_node/flow_node, we must skip comment children to find the
/// actual content (block_mapping, block_sequence, block_scalar, etc.).
pub(crate) fn unwrap_node<'a>(node: Node<'a>) -> Node<'a> {
    let mut current = node;
    while let "block_node" | "flow_node" = current.kind() {
        let mut found_inner = false;
        for i in 0..current.child_count() {
            if let Some(child) = current.child(i) {
                if child.kind() != "comment" {
                    current = child;
                    found_inner = true;
                    break;
                }
            }
        }
        if !found_inner {
            break;
        }
    }
    current
}

/// Helper function to find a value node for a given key in the AST
pub(crate) fn find_value_for_key<'a>(
    node: Node<'a>,
    source: &'a str,
    target_key: &str,
) -> Option<Node<'a>> {
    match node.kind() {
        "block_mapping_pair" | "flow_pair" => {
            if let Some(key_node) = node.child(0) {
                let key_text = &source[key_node.start_byte()..key_node.end_byte()];
                let key_cleaned = key_text
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .trim_end_matches(':');
                if key_cleaned == target_key {
                    // Find value node: iterate from end, skip comments and ":"
                    for i in (1..node.child_count()).rev() {
                        if let Some(child) = node.child(i) {
                            let kind = child.kind();
                            if kind != "comment" && kind != ":" {
                                return Some(child);
                            }
                        }
                    }
                }
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(value) = find_value_for_key(child, source, target_key) {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// Extract the value node from a block_mapping_pair or flow_pair.
///
/// In tree-sitter-yaml, a `block_mapping_pair` has children: [key, ":", value]
/// but comments can appear as extra children at any position. This function
/// iterates from the end to find the last non-comment, non-":" child, which
/// is the value node.
///
/// Returns `None` if no value node is found (e.g., key-only pair).
pub(crate) fn get_pair_value<'a>(node: Node<'a>) -> Option<Node<'a>> {
    for i in (1..node.child_count()).rev() {
        if let Some(child) = node.child(i) {
            let kind = child.kind();
            if kind != "comment" && kind != ":" {
                return Some(child);
            }
        }
    }
    None
}

/// Check if a key exists in a mapping node, regardless of whether it has a value.
pub(crate) fn key_exists(node: Node, source: &str, target_key: &str) -> bool {
    match node.kind() {
        "block_mapping_pair" | "flow_pair" => {
            if let Some(key_node) = node.child(0) {
                let key_text = &source[key_node.start_byte()..key_node.end_byte()];
                let key_cleaned = key_text
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .trim_end_matches(':');
                if key_cleaned == target_key {
                    return true;
                }
            }
            false
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if key_exists(child, source, target_key) {
                    return true;
                }
            }
            false
        }
    }
}

/// Helper to extract text from a node
pub(crate) fn node_text(node: Node, source: &str) -> String {
    source[node.start_byte()..node.end_byte()].to_string()
}

/// Known GitHub Actions expression context names.
const KNOWN_CONTEXTS: &[&str] = &[
    "github", "matrix", "secrets", "vars", "needs", "inputs", "env", "job", "jobs", "steps",
    "runner", "strategy",
];

/// Check if an expression has valid GitHub Actions expression syntax.
///
/// Validates that the expression contains recognized contexts, functions,
/// operators, or literals. Also rejects unknown context references like
/// `invalid.property`.
pub(crate) fn is_valid_expression_syntax(expr: &str) -> bool {
    let expr = expr.trim();

    if expr.is_empty() {
        return false;
    }

    let has_context = KNOWN_CONTEXTS
        .iter()
        .any(|ctx| expr.starts_with(&format!("{}.", ctx)));

    let expr_lower = expr.to_lowercase();
    let has_function = expr.contains("contains(")
        || expr.contains("startsWith(")
        || expr.contains("endsWith(")
        || expr.contains("format(")
        || expr.contains("join(")
        || expr_lower.contains("tojson(")
        || expr_lower.contains("fromjson(")
        || expr.contains("hashFiles(")
        || expr.contains("success()")
        || expr.contains("failure()")
        || expr.contains("cancelled()")
        || expr.contains("always()");

    let has_operator = expr.contains("==")
        || expr.contains("!=")
        || expr.contains("&&")
        || expr.contains("||")
        || expr.contains("!")
        || expr.contains("<")
        || expr.contains(">")
        || expr.contains("<=")
        || expr.contains(">=");

    let is_literal = (expr.starts_with("'") && expr.ends_with("'"))
        || (expr.starts_with("\"") && expr.ends_with("\""))
        || expr.parse::<f64>().is_ok()
        || expr == "true"
        || expr == "false";

    let is_bare_context = KNOWN_CONTEXTS.contains(&expr);

    // Check if expression contains a dot but doesn't start with a known context
    // (e.g., "invalid.expression" should be rejected)
    if expr.contains('.') && !has_context {
        let first_token = expr
            .split(|c: char| {
                c.is_whitespace() || matches!(c, '&' | '|' | '=' | '!' | '<' | '>' | '(' | '[')
            })
            .next()
            .unwrap_or("");
        if first_token.contains('.') {
            let context_name = first_token.split('.').next().unwrap_or("");
            if !KNOWN_CONTEXTS.contains(&context_name) {
                return false;
            }
        }
    }

    if !has_context
        && !has_function
        && !has_operator
        && !is_literal
        && !is_bare_context
        && !expr.contains('.')
        && !expr.contains('(')
        && !expr.contains('[')
    {
        return false;
    }

    true
}

/// Check if expression may always evaluate to true.
pub(crate) fn is_potentially_always_true(expr: &str) -> bool {
    let expr_lower = expr.to_lowercase();
    expr_lower == "true"
        || expr_lower == "!false"
        || expr_lower.contains("|| true")
        || expr_lower.contains("true ||")
}

/// Check if expression may always evaluate to false.
pub(crate) fn is_potentially_always_false(expr: &str) -> bool {
    let expr_lower = expr.to_lowercase();
    expr_lower == "false"
        || expr_lower == "!true"
        || expr_lower.contains("&& false")
        || expr_lower.contains("false &&")
}
