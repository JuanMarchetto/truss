use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// GitHub event properties that can be controlled by external users and are
/// therefore untrusted inputs. Using these directly in `run:` scripts via
/// `${{ }}` creates a script injection vulnerability.
const UNTRUSTED_INPUTS: &[&str] = &[
    "github.event.issue.title",
    "github.event.issue.body",
    "github.event.pull_request.title",
    "github.event.pull_request.body",
    "github.event.pull_request.head.ref",
    "github.event.pull_request.head.label",
    "github.event.comment.body",
    "github.event.review.body",
    "github.event.review_comment.body",
    "github.event.discussion.title",
    "github.event.discussion.body",
    "github.event.pages.*.page_name",
    "github.event.commits.*.message",
    "github.event.commits.*.author.name",
    "github.event.commits.*.author.email",
    "github.event.head_commit.message",
    "github.event.head_commit.author.name",
    "github.event.head_commit.author.email",
    "github.head_ref",
];

/// Detects potential script injection vulnerabilities where untrusted inputs
/// are used directly in `run:` scripts via `${{ }}` expressions.
pub struct ScriptInjectionRule;

impl ValidationRule for ScriptInjectionRule {
    fn name(&self) -> &str {
        "script_injection"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

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
                        check_script_injection(
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

fn check_script_injection(
    run_text: &str,
    start_byte: usize,
    end_byte: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expressions = utils::find_expressions(run_text);

    for expr in &expressions {
        let inner = expr.inner.trim();
        for untrusted in UNTRUSTED_INPUTS {
            // Check for exact match or property access on the untrusted input
            if inner == *untrusted || inner.starts_with(&format!("{}.", untrusted)) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Potential script injection: untrusted input '{}' is used directly in a 'run' script. \
                         Use an environment variable instead: env: MY_VAR: ${{{{ {} }}}}",
                        untrusted, inner
                    ),
                    severity: Severity::Warning,
                    span: Span {
                        start: start_byte,
                        end: end_byte,
                    },
                });
                break;
            }
            // Also check for patterns like github.event.pull_request.title
            // inside a longer expression
            if inner.contains(untrusted) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Potential script injection: expression contains untrusted input '{}'. \
                         Consider passing untrusted values through environment variables.",
                        untrusted
                    ),
                    severity: Severity::Warning,
                    span: Span {
                        start: start_byte,
                        end: end_byte,
                    },
                });
                break;
            }
        }
    }
}
