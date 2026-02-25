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
    let mut has_on = false;
    let mut has_jobs = false;

    fn check_top_level_keys(
        node: Node,
        source: &str,
        has_on: &mut bool,
        has_jobs: &mut bool,
        depth: usize,
    ) {
        if depth > 4 || (*has_on && *has_jobs) {
            return;
        }

        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let text = source
                        .get(key_node.start_byte()..key_node.end_byte())
                        .unwrap_or("");
                    let cleaned = text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');
                    if cleaned.eq_ignore_ascii_case("on") {
                        *has_on = true;
                    } else if cleaned.eq_ignore_ascii_case("jobs") {
                        *has_jobs = true;
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_top_level_keys(child, source, has_on, has_jobs, depth + 1);
                }
            }
        }
    }

    check_top_level_keys(root, source, &mut has_on, &mut has_jobs, 0);

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

/// Extract and clean a YAML key name from a node.
///
/// Strips surrounding quotes, whitespace, and trailing colon. This is the
/// standard key cleaning pattern used throughout validation rules.
pub(crate) fn clean_key<'a>(node: Node, source: &'a str) -> &'a str {
    node_text(node, source)
        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
        .trim_end_matches(':')
}

/// Get the unwrapped `jobs:` mapping node from a workflow tree.
///
/// Combines `find_value_for_key` + `unwrap_node` for the common pattern
/// used by 31+ validation rules.
pub(crate) fn get_jobs_node<'a>(tree: &'a Tree, source: &'a str) -> Option<Node<'a>> {
    let root = tree.root_node();
    let jobs_value = find_value_for_key(root, source, "jobs")?;
    Some(unwrap_node(jobs_value))
}

/// Helper to extract text from a node.
///
/// Returns an empty string if the byte offsets fall outside the source
/// or land on a non-UTF-8 boundary.
pub(crate) fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    source.get(node.start_byte()..node.end_byte()).unwrap_or("")
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

    let has_context = KNOWN_CONTEXTS.iter().any(|ctx| {
        expr.len() > ctx.len() && expr.as_bytes()[ctx.len()] == b'.' && expr.starts_with(ctx)
    });

    let has_function = expr.contains("contains(")
        || expr.contains("startsWith(")
        || expr.contains("endsWith(")
        || expr.contains("format(")
        || expr.contains("join(")
        || contains_ignore_ascii_case(expr, "tojson(")
        || contains_ignore_ascii_case(expr, "fromjson(")
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
    expr.eq_ignore_ascii_case("true")
        || expr.eq_ignore_ascii_case("!false")
        || contains_ignore_ascii_case(expr, "|| true")
        || contains_ignore_ascii_case(expr, "true ||")
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
    expr.eq_ignore_ascii_case("false")
        || expr.eq_ignore_ascii_case("!true")
        || contains_ignore_ascii_case(expr, "&& false")
        || contains_ignore_ascii_case(expr, "false &&")
}

/// Case-insensitive substring search without allocating a new String.
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Case-insensitive substring search returning the byte offset, without allocating.
///
/// Used to replace `expr.to_lowercase().find("keyword")` patterns across rules.
pub(crate) fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.len() > haystack.len() {
        return None;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
