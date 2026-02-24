use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::HashMap;
use tree_sitter::{Node, Tree};

/// Validates workflow_dispatch inputs.
pub struct WorkflowInputsRule;

impl ValidationRule for WorkflowInputsRule {
    fn name(&self) -> &str {
        "workflow_inputs"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        let root = tree.root_node();
        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => return diagnostics,
        };

        // Check if workflow_dispatch is present
        let on_to_check = utils::unwrap_node(on_value);

        // Find workflow_dispatch
        let workflow_dispatch_value =
            utils::find_value_for_key(on_to_check, source, "workflow_dispatch");

        if workflow_dispatch_value.is_none() {
            // No workflow_dispatch, nothing to validate
            return diagnostics;
        }

        let workflow_dispatch = workflow_dispatch_value.unwrap();
        let dispatch_to_check = utils::unwrap_node(workflow_dispatch);

        // Extract defined inputs and their types
        let inputs_value = utils::find_value_for_key(dispatch_to_check, source, "inputs");

        let mut defined_inputs: HashMap<String, (String, Span)> = HashMap::new(); // input_name -> (type, span)

        if let Some(inputs_node) = inputs_value {
            let inputs_to_check = utils::unwrap_node(inputs_node);

            // Collect all input definitions
            self.collect_input_definitions(inputs_to_check, source, &mut defined_inputs);
        }

        // Validate input types and properties
        for (input_name, (input_type, type_span)) in &defined_inputs {
            if !self.is_valid_input_type(input_type) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Invalid input type '{}' for input '{}'. Valid types are: string, number, choice, boolean, environment",
                        input_type, input_name
                    ),
                    severity: Severity::Error,
                    span: *type_span,
                });
            }
        }

        // Validate input properties (default, required, description)
        if let Some(inputs_node) = inputs_value {
            let inputs_to_check = utils::unwrap_node(inputs_node);
            self.validate_input_properties(inputs_to_check, source, &mut diagnostics);
        }

        // Find all inputs.* references in expressions (but exclude the inputs definition section itself)
        // We need to find references in jobs, not in the inputs definition
        let jobs_value = utils::find_value_for_key(root, source, "jobs");
        let input_references = if let Some(jobs_node) = jobs_value {
            // Only search for input references in the jobs section
            self.find_input_references_in_node(jobs_node, source)
        } else {
            Vec::new()
        };

        // Validate that all referenced inputs are defined
        for (input_name, span) in input_references {
            if !defined_inputs.contains_key(&input_name) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Reference to undefined input '{}'. Available inputs: {}",
                        input_name,
                        if defined_inputs.is_empty() {
                            "none".to_string()
                        } else {
                            defined_inputs
                                .keys()
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ),
                    severity: Severity::Error,
                    span,
                });
            }
        }

        diagnostics
    }
}

impl WorkflowInputsRule {
    fn collect_input_definitions(
        &self,
        node: Node,
        source: &str,
        inputs: &mut HashMap<String, (String, Span)>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let input_name = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':')
                        .trim()
                        .to_string();

                    // Get the input value node
                    let input_value = utils::get_pair_value(node);

                    if let Some(input_value_raw) = input_value {
                        let input_value = utils::unwrap_node(input_value_raw);

                        // Find the type field
                        let type_value = utils::find_value_for_key(input_value, source, "type");
                        if let Some(type_node) = type_value {
                            let type_text = utils::node_text(type_node, source);
                            let type_cleaned = type_text
                                .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                            inputs.insert(
                                input_name,
                                (
                                    type_cleaned.to_string(),
                                    Span {
                                        start: type_node.start_byte(),
                                        end: type_node.end_byte(),
                                    },
                                ),
                            );
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.collect_input_definitions(child, source, inputs);
                }
            }
        }
    }

    fn is_valid_input_type(&self, input_type: &str) -> bool {
        matches!(
            input_type,
            "string" | "choice" | "boolean" | "environment" | "number"
        )
    }

    fn validate_input_properties(
        &self,
        node: Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let input_name = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':')
                        .to_string();

                    let input_value = utils::get_pair_value(node);

                    if let Some(input_value_raw) = input_value {
                        let input_value = utils::unwrap_node(input_value_raw);

                        // Validate required field (must be boolean)
                        let required_value =
                            utils::find_value_for_key(input_value, source, "required");
                        if let Some(required_node) = required_value {
                            let required_text = utils::node_text(required_node, source);
                            let required_cleaned = required_text
                                .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                            if required_cleaned != "true"
                                && required_cleaned != "false"
                                && !required_cleaned.starts_with("${{")
                            {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Input '{}' has invalid 'required' value: '{}'. 'required' must be a boolean (true or false).",
                                        input_name, required_cleaned
                                    ),
                                    severity: Severity::Error,
                                    span: Span {
                                        start: required_node.start_byte(),
                                        end: required_node.end_byte(),
                                    },
                                });
                            }
                        }

                        // Validate default value (type-specific validation)
                        let default_value =
                            utils::find_value_for_key(input_value, source, "default");
                        if let Some(default_node) = default_value {
                            let type_value = utils::find_value_for_key(input_value, source, "type");
                            if let Some(type_node) = type_value {
                                let type_text = utils::node_text(type_node, source);
                                let type_cleaned = type_text.trim_matches(|c: char| {
                                    c == '"' || c == '\'' || c.is_whitespace()
                                });
                                let default_text = utils::node_text(default_node, source);

                                // Basic validation - default should match input type
                                if !default_text.starts_with("${{") {
                                    match type_cleaned {
                                        "boolean" => {
                                            let default_cleaned =
                                                default_text.trim_matches(|c: char| {
                                                    c == '"' || c == '\'' || c.is_whitespace()
                                                });
                                            if default_cleaned != "true"
                                                && default_cleaned != "false"
                                            {
                                                diagnostics.push(Diagnostic {
                                                    message: format!(
                                                        "Input '{}' has invalid default value for boolean type: '{}'. Default must be 'true' or 'false'.",
                                                        input_name, default_cleaned
                                                    ),
                                                    severity: Severity::Warning,
                                                    span: Span {
                                                        start: default_node.start_byte(),
                                                        end: default_node.end_byte(),
                                                    },
                                                });
                                            }
                                        }
                                        "choice" => {
                                            // Choice defaults should be validated against choices array
                                            // This is a basic check - full validation would require checking choices array
                                        }
                                        _ => {
                                            // String and environment types accept any string default
                                        }
                                    }
                                }
                            }
                        }

                        // Validate description (should be a string)
                        let description_value =
                            utils::find_value_for_key(input_value, source, "description");
                        if let Some(description_node) = description_value {
                            let description_text = utils::node_text(description_node, source);
                            // Description should be a string (basic validation)
                            if description_text.trim().is_empty()
                                && !description_text.starts_with("${{")
                            {
                                diagnostics.push(Diagnostic {
                                    message: format!(
                                        "Input '{}' has empty description. Consider adding a description to document the input.",
                                        input_name
                                    ),
                                    severity: Severity::Warning,
                                    span: Span {
                                        start: description_node.start_byte(),
                                        end: description_node.end_byte(),
                                    },
                                });
                            }
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.validate_input_properties(child, source, diagnostics);
                }
            }
        }
    }

    fn find_input_references_in_node(&self, node: Node, source: &str) -> Vec<(String, Span)> {
        let mut references = Vec::new();
        let node_text = utils::node_text(node, source);
        let node_start = node.start_byte();
        let source_bytes = node_text.as_bytes();
        let mut i = 0;

        while i < source_bytes.len() {
            // Look for ${{ pattern
            if i + 3 < source_bytes.len()
                && source_bytes[i] == b'$'
                && source_bytes[i + 1] == b'{'
                && source_bytes[i + 2] == b'{'
            {
                // Find the closing }}
                let mut j = i + 3;
                let mut brace_count = 2;
                let mut found_closing = false;

                while j < source_bytes.len() {
                    if j + 1 < source_bytes.len()
                        && source_bytes[j] == b'}'
                        && source_bytes[j + 1] == b'}'
                    {
                        brace_count -= 2;
                        if brace_count == 0 {
                            found_closing = true;
                            j += 2;
                            break;
                        }
                        j += 2;
                    } else if source_bytes[j] == b'{' {
                        brace_count += 1;
                        j += 1;
                    } else if source_bytes[j] == b'}' {
                        brace_count -= 1;
                        j += 1;
                    } else {
                        j += 1;
                    }
                }

                if found_closing {
                    // Extract the expression content
                    let expr_start = i + 3;
                    let expr_end = j - 2;

                    if expr_start < expr_end && expr_end <= source_bytes.len() {
                        let expr_text = &node_text[expr_start..expr_end];

                        // Look for inputs.* references
                        let expr_lower = expr_text.to_lowercase();
                        let mut search_pos = 0;

                        while let Some(pos) = expr_lower[search_pos..].find("inputs.") {
                            let actual_pos = search_pos + pos;
                            let after_inputs = &expr_text[actual_pos + 7..]; // Skip "inputs."

                            // Find where the input name ends
                            let name_end = after_inputs
                                .find(|c: char| {
                                    c.is_whitespace()
                                        || c == '}'
                                        || c == ')'
                                        || c == ']'
                                        || c == '&'
                                        || c == '|'
                                        || c == '='
                                        || c == '!'
                                        || c == '<'
                                        || c == '>'
                                        || c == '.'
                                })
                                .unwrap_or(after_inputs.len());

                            let input_name = &after_inputs[..name_end.min(after_inputs.len())];
                            // Trim any whitespace from input name
                            let input_name_cleaned = input_name.trim();

                            if !input_name_cleaned.is_empty() {
                                references.push((
                                    input_name_cleaned.to_string(),
                                    Span {
                                        start: node_start + i + 3 + actual_pos + 7,
                                        end: node_start + i + 3 + actual_pos + 7 + name_end,
                                    },
                                ));
                            }

                            search_pos = actual_pos + 7 + name_end;
                        }
                    }

                    i = j;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        references
    }
}
