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
        let text = source.get(node.start_byte()..node.end_byte())?;
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

    // A GitHub Actions workflow must have `on` or `jobs` at the top level.
    // `name` alone is not sufficient — many YAML files have a `name:` key.
    let has_on = top_level_keys
        .iter()
        .any(|key| key.to_lowercase().trim() == "on");
    let has_jobs = top_level_keys
        .iter()
        .any(|key| key.to_lowercase().trim() == "jobs");

    has_on || has_jobs
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
                let key_text = source
                    .get(key_node.start_byte()..key_node.end_byte())
                    .unwrap_or("");
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
                let key_text = source
                    .get(key_node.start_byte()..key_node.end_byte())
                    .unwrap_or("");
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

/// Helper to extract text from a node.
///
/// Returns an empty string if the byte offsets fall outside the source
/// or land on a non-UTF-8 boundary.
pub(crate) fn node_text(node: Node, source: &str) -> String {
    source
        .get(node.start_byte()..node.end_byte())
        .unwrap_or("")
        .to_string()
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

/// An extracted `${{ ... }}` expression with its byte offsets in the source.
pub(crate) struct Expression<'a> {
    /// The inner text between `${{` and `}}`, untrimmed.
    pub inner: &'a str,
    /// Byte offset of the `$` in `${{` within the source string.
    pub start: usize,
    /// Byte offset just past the closing `}}` within the source string.
    pub end: usize,
}

/// Find all `${{ ... }}` expressions in a source string using brace-counting.
///
/// Unlike simple `find("${{")` + `find("}}")`, this correctly handles nested
/// braces in format strings (e.g., `${{ format('{0}', matrix.os) }}`).
///
/// Skips expressions inside YAML comments (`# ... ${{ expr }}`).
pub(crate) fn find_expressions(source: &str) -> Vec<Expression<'_>> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i + 2 < bytes.len() {
        if bytes[i] == b'$' && bytes[i + 1] == b'{' && bytes[i + 2] == b'{' {
            // Skip expressions inside YAML comments
            let line_start = source[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line_before = &source[line_start..i];
            if line_before.trim_start().starts_with('#') {
                i += 3;
                continue;
            }

            let mut j = i + 3;
            let mut brace_count: i32 = 2;
            let mut found_closing = false;

            while j < bytes.len() {
                if j + 1 < bytes.len() && bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    brace_count -= 2;
                    if brace_count == 0 {
                        found_closing = true;
                        j += 2;
                        break;
                    }
                    j += 2;
                } else if bytes[j] == b'{' {
                    brace_count += 1;
                    j += 1;
                } else if bytes[j] == b'}' {
                    brace_count -= 1;
                    j += 1;
                } else {
                    j += 1;
                }
            }

            if found_closing {
                let expr_start = i + 3;
                let expr_end = j - 2;
                if expr_start < expr_end && expr_end <= bytes.len() {
                    if let Some(inner) = source.get(expr_start..expr_end) {
                        results.push(Expression {
                            inner,
                            start: i,
                            end: j,
                        });
                    }
                }
                i = j;
            } else {
                // Unclosed expression — record it with end at source length
                // so callers can emit an "unclosed expression" diagnostic.
                let expr_start = i + 3;
                if let Some(inner) = source.get(expr_start..source.len()) {
                    results.push(Expression {
                        inner,
                        start: i,
                        end: source.len(),
                    });
                }
                break;
            }
        } else {
            i += 1;
        }
    }

    results
}

/// Check if expression may always evaluate to false.
pub(crate) fn is_potentially_always_false(expr: &str) -> bool {
    let expr_lower = expr.to_lowercase();
    expr_lower == "false"
        || expr_lower == "!true"
        || expr_lower.contains("&& false")
        || expr_lower.contains("false &&")
}
