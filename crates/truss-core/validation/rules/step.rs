use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates step structure.
pub struct StepValidationRule;

impl ValidationRule for StepValidationRule {
    fn name(&self) -> &str {
        "step"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let jobs_value = match utils::find_value_for_key(root, source, "jobs") {
            Some(v) => v,
            None => return diagnostics,
        };

        let jobs_to_process = utils::unwrap_node(jobs_value);

        fn find_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let key_text = utils::node_text(key_node, source);
                        let key_cleaned = key_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                            .trim_end_matches(':');
                        if key_cleaned == "steps" {
                            if let Some(steps_value_raw) = utils::get_pair_value(node) {
                                let steps_value = utils::unwrap_node(steps_value_raw);
                                if steps_value.kind() == "block_mapping" {
                                    let mut cursor = steps_value.walk();
                                    for child in steps_value.children(&mut cursor) {
                                        find_steps(child, source, diagnostics);
                                    }
                                    return;
                                }
                                fn validate_step(
                                    step_node: Node,
                                    source: &str,
                                    diagnostics: &mut Vec<Diagnostic>,
                                ) {
                                    let step_to_check = utils::unwrap_node(step_node);

                                    let mut has_uses = false;
                                    let mut has_run = false;

                                    fn check_step_keys(
                                        node: Node,
                                        source: &str,
                                        has_uses: &mut bool,
                                        has_run: &mut bool,
                                    ) {
                                        match node.kind() {
                                            "block_mapping_pair" | "flow_pair" => {
                                                if let Some(key_node) = node.child(0) {
                                                    let key_text =
                                                        utils::node_text(key_node, source);
                                                    let key_cleaned = key_text
                                                        .trim_matches(|c: char| {
                                                            c == '"'
                                                                || c == '\''
                                                                || c.is_whitespace()
                                                        })
                                                        .trim_end_matches(':');
                                                    if key_cleaned == "uses" {
                                                        *has_uses = true;
                                                    } else if key_cleaned == "run" {
                                                        *has_run = true;
                                                    }
                                                }
                                            }
                                            _ => {
                                                let mut cursor = node.walk();
                                                for child in node.children(&mut cursor) {
                                                    check_step_keys(
                                                        child, source, has_uses, has_run,
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    check_step_keys(
                                        step_to_check,
                                        source,
                                        &mut has_uses,
                                        &mut has_run,
                                    );

                                    if !has_uses && !has_run {
                                        diagnostics.push(Diagnostic {
                                            message: "Step must have either 'uses' or 'run' field"
                                                .to_string(),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: step_node.start_byte(),
                                                end: step_node.end_byte(),
                                            },
                                        });
                                    } else if has_uses && has_run {
                                        diagnostics.push(Diagnostic {
                                            message: "Step cannot have both 'uses' and 'run' fields. Each step must use one or the other."
                                                .to_string(),
                                            severity: Severity::Error,
                                            span: Span {
                                                start: step_node.start_byte(),
                                                end: step_node.end_byte(),
                                            },
                                        });
                                    }
                                }

                                // Handle block_sequence - steps are children of block_sequence
                                if steps_value.kind() == "block_sequence" {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        // Each step in a block_sequence is a block_node
                                        if step_node.kind() == "block_node" {
                                            validate_step(step_node, source, diagnostics);
                                        } else {
                                            // Also check if it's directly a block_mapping
                                            validate_step(step_node, source, diagnostics);
                                        }
                                    }
                                } else if steps_value.kind() == "flow_sequence" {
                                    // Handle flow_sequence
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "flow_node"
                                            || step_node.kind() == "block_node"
                                        {
                                            validate_step(step_node, source, diagnostics);
                                        }
                                    }
                                } else {
                                    let mut cursor = steps_value.walk();
                                    for step_node in steps_value.children(&mut cursor) {
                                        if step_node.kind() == "block_node"
                                            || step_node.kind() == "flow_node"
                                        {
                                            validate_step(step_node, source, diagnostics);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(value_node) = utils::get_pair_value(node) {
                            find_steps(value_node, source, diagnostics);
                        }
                    }
                }
                "block_node" | "block_mapping" => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps(child, source, diagnostics);
                    }
                }
                _ => {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        find_steps(child, source, diagnostics);
                    }
                }
            }
        }

        find_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}
