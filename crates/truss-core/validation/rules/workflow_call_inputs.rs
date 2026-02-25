use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use std::collections::HashMap;
use tree_sitter::{Node, Tree};

/// Validates workflow_call inputs and their usage.
pub struct WorkflowCallInputsRule;

impl ValidationRule for WorkflowCallInputsRule {
    fn name(&self) -> &str {
        "workflow_call_inputs"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => return diagnostics,
        };

        let on_to_check = utils::unwrap_node(on_value);

        // Find workflow_call
        let workflow_call_value = utils::find_value_for_key(on_to_check, source, "workflow_call");

        if workflow_call_value.is_none() {
            // No workflow_call — if workflow_dispatch is present, inputs.* references
            // are valid (GitHub Actions returns empty string for undeclared inputs).
            // Only flag input references when neither trigger is present.
            // Use key_exists because workflow_dispatch may have no value (null).
            if utils::key_exists(on_to_check, source, "workflow_dispatch") {
                // workflow_dispatch is present — inputs.* references are valid
                return diagnostics;
            }
            // No workflow_call and no workflow_dispatch — inputs.* references are invalid
            let input_references = self.find_input_references(source);
            for (input_name, span) in input_references {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Reference to input '{}' but workflow_call trigger is not defined",
                        input_name
                    ),
                    severity: Severity::Error,
                    span,
                    rule_id: String::new(),
                });
            }
            return diagnostics;
        }

        let workflow_call = workflow_call_value.unwrap();
        let call_to_check = utils::unwrap_node(workflow_call);

        // Extract defined inputs and their types
        let inputs_value = utils::find_value_for_key(call_to_check, source, "inputs");

        let mut defined_inputs: HashMap<String, (String, Span)> = HashMap::new();

        if let Some(inputs_node) = inputs_value {
            let inputs_to_check = utils::unwrap_node(inputs_node);

            self.collect_input_definitions(inputs_to_check, source, &mut defined_inputs);
        }

        // Also collect inputs from workflow_dispatch if it coexists —
        // inputs.* references are valid for either trigger's inputs.
        if let Some(dispatch_value) =
            utils::find_value_for_key(on_to_check, source, "workflow_dispatch")
        {
            let dispatch_to_check = utils::unwrap_node(dispatch_value);
            if let Some(dispatch_inputs) =
                utils::find_value_for_key(dispatch_to_check, source, "inputs")
            {
                let dispatch_inputs_node = utils::unwrap_node(dispatch_inputs);
                self.collect_input_definitions(dispatch_inputs_node, source, &mut defined_inputs);
            }
        }

        // Validate input types and properties
        for (input_name, (input_type, type_span)) in &defined_inputs {
            if !self.is_valid_input_type(input_type) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Invalid input type '{}' for workflow_call input '{}'. Valid types are: string, number, choice, boolean, environment",
                        input_type, input_name
                    ),
                    severity: Severity::Error,
                    span: *type_span,
                    rule_id: String::new(),
                });
            }
        }

        // Validate input properties (default, required, description)
        if let Some(inputs_node) = inputs_value {
            let inputs_to_check = utils::unwrap_node(inputs_node);
            self.validate_input_properties(inputs_to_check, source, &mut diagnostics);
        }

        // Find all inputs.* references in expressions
        let input_references = self.find_input_references(source);

        // Validate that all referenced inputs are defined
        for (input_name, span) in input_references {
            if !defined_inputs.contains_key(&input_name) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Reference to undefined workflow_call input '{}'. Available inputs: {}",
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
                    rule_id: String::new(),
                });
            }
        }

        diagnostics
    }
}

impl WorkflowCallInputsRule {
    fn collect_input_definitions(
        &self,
        node: Node,
        source: &str,
        inputs: &mut HashMap<String, (String, Span)>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let input_name = utils::clean_key(key_node, source).to_string();

                    let input_value = utils::get_pair_value(node);

                    if let Some(input_value_raw) = input_value {
                        let input_value = utils::unwrap_node(input_value_raw);

                        // Find the type field (optional — defaults to "string")
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
                        } else {
                            // Input without explicit type — defaults to "string" in GitHub Actions
                            inputs.insert(
                                input_name,
                                (
                                    "string".to_string(),
                                    Span {
                                        start: key_node.start_byte(),
                                        end: key_node.end_byte(),
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
                    let input_name = utils::clean_key(key_node, source).to_string();

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
                                    rule_id: String::new(),
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
                                                    rule_id: String::new(),
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
                                    rule_id: String::new(),
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

    fn find_input_references(&self, source: &str) -> Vec<(String, Span)> {
        let mut references = Vec::new();

        for expr in utils::find_expressions(source) {
            let mut search_pos = 0;

            while let Some(pos) =
                utils::find_ignore_ascii_case(&expr.inner[search_pos..], "inputs.")
            {
                let actual_pos = search_pos + pos;
                let after_inputs = &expr.inner[actual_pos + 7..];

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
                            || c == ','
                    })
                    .unwrap_or(after_inputs.len());

                let input_name = &after_inputs[..name_end.min(after_inputs.len())];

                if !input_name.is_empty() {
                    let name_start = expr.start + 3 + actual_pos + 7;
                    references.push((
                        input_name.to_string(),
                        Span {
                            start: name_start,
                            end: name_start + name_end,
                        },
                    ));
                }

                search_pos = actual_pos + 7 + name_end;
            }
        }

        references
    }
}
