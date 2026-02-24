use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Deprecated workflow commands and their replacements.
const DEPRECATED_COMMANDS: &[(&str, &str)] = &[
    (
        "::set-output",
        "Use `echo \"name=value\" >> $GITHUB_OUTPUT` instead",
    ),
    (
        "::save-state",
        "Use `echo \"name=value\" >> $GITHUB_STATE` instead",
    ),
    (
        "::set-env",
        "Use `echo \"name=value\" >> $GITHUB_ENV` instead",
    ),
    ("::add-path", "Use `echo \"path\" >> $GITHUB_PATH` instead"),
];

/// Detects deprecated workflow commands in `run:` scripts.
pub struct DeprecatedCommandsRule;

impl ValidationRule for DeprecatedCommandsRule {
    fn name(&self) -> &str {
        "deprecated_commands"
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

        let jobs_to_process = utils::unwrap_node(jobs_value);
        find_run_steps(jobs_to_process, source, &mut diagnostics);

        diagnostics
    }
}

fn find_run_steps(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    match node.kind() {
        "block_mapping_pair" | "flow_pair" => {
            if let Some(key_node) = node.child(0) {
                let key_text = utils::node_text(key_node, source);
                let key_cleaned = key_text
                    .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                    .trim_end_matches(':');
                if key_cleaned == "run" {
                    if let Some(value_node) = utils::get_pair_value(node) {
                        let run_text = utils::node_text(value_node, source);
                        check_deprecated_commands(
                            &run_text,
                            value_node.start_byte(),
                            value_node.end_byte(),
                            diagnostics,
                        );
                    }
                }
                if let Some(value_node) = utils::get_pair_value(node) {
                    find_run_steps(value_node, source, diagnostics);
                }
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_run_steps(child, source, diagnostics);
            }
        }
    }
}

fn check_deprecated_commands(
    run_text: &str,
    start_byte: usize,
    end_byte: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (command, replacement) in DEPRECATED_COMMANDS {
        if run_text.contains(command) {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Deprecated workflow command '{}' detected. {}",
                    command, replacement
                ),
                severity: Severity::Warning,
                span: Span {
                    start: start_byte,
                    end: end_byte,
                },
            });
        }
    }
}
