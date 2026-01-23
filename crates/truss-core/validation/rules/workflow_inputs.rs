use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Tree, Node};
use super::super::ValidationRule;
use super::super::utils;
use std::collections::HashMap;

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
        let mut on_to_check = on_value;
        if on_to_check.kind() == "block_node" {
            if let Some(inner) = on_to_check.child(0) {
                on_to_check = inner;
            }
        }

        // Find workflow_dispatch
        let workflow_dispatch_value = utils::find_value_for_key(on_to_check, source, "workflow_dispatch");
        
        if workflow_dispatch_value.is_none() {
            // No workflow_dispatch, nothing to validate
            return diagnostics;
        }

        let workflow_dispatch = workflow_dispatch_value.unwrap();
        let mut dispatch_to_check = workflow_dispatch;
        if dispatch_to_check.kind() == "block_node" {
            if let Some(inner) = dispatch_to_check.child(0) {
                dispatch_to_check = inner;
            }
        }

        // Extract defined inputs and their types
        let inputs_value = utils::find_value_for_key(dispatch_to_check, source, "inputs");
        
        let mut defined_inputs: HashMap<String, (String, Span)> = HashMap::new(); // input_name -> (type, span)
        
        if let Some(inputs_node) = inputs_value {
            let mut inputs_to_check = inputs_node;
            if inputs_to_check.kind() == "block_node" {
                if let Some(inner) = inputs_to_check.child(0) {
                    inputs_to_check = inner;
                }
            }

            // Collect all input definitions
            self.collect_input_definitions(inputs_to_check, source, &mut defined_inputs);
        }

        // Validate input types
        for (input_name, (input_type, type_span)) in &defined_inputs {
            if !self.is_valid_input_type(input_type) {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Invalid input type '{}' for input '{}'. Valid types are: string, choice, boolean, environment",
                        input_type, input_name
                    ),
                    severity: Severity::Error,
                    span: *type_span,
                });
            }
        }

        // Find all inputs.* references in expressions
        let input_references = self.find_input_references(source);
        
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
                            defined_inputs.keys().cloned().collect::<Vec<_>>().join(", ")
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
    fn collect_input_definitions(&self, node: Node, source: &str, inputs: &mut HashMap<String, (String, Span)>) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let input_name = key_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':')
                        .to_string();
                    
                    // Get the input value node
                    let input_value = if node.kind() == "block_mapping_pair" {
                        node.child(2)
                    } else {
                        node.child(1)
                    };
                    
                    if let Some(mut input_value) = input_value {
                        if input_value.kind() == "block_node" {
                            if let Some(inner) = input_value.child(0) {
                                input_value = inner;
                            }
                        }
                        
                        // Find the type field
                        let type_value = utils::find_value_for_key(input_value, source, "type");
                        if let Some(type_node) = type_value {
                            let type_text = utils::node_text(type_node, source);
                            let type_cleaned = type_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                            inputs.insert(input_name, (type_cleaned.to_string(), Span {
                                start: type_node.start_byte(),
                                end: type_node.end_byte(),
                            }));
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
        matches!(input_type, "string" | "choice" | "boolean" | "environment")
    }

    fn find_input_references(&self, source: &str) -> Vec<(String, Span)> {
        let mut references = Vec::new();
        let source_bytes = source.as_bytes();
        let mut i = 0;
        
        while i < source_bytes.len() {
            // Look for ${{ pattern
            if i + 3 < source_bytes.len() 
                && source_bytes[i] == b'$' 
                && source_bytes[i + 1] == b'{' 
                && source_bytes[i + 2] == b'{' {
                
                // Find the closing }}
                let mut j = i + 3;
                let mut brace_count = 2;
                let mut found_closing = false;
                
                while j < source_bytes.len() {
                    if j + 1 < source_bytes.len() && source_bytes[j] == b'}' && source_bytes[j + 1] == b'}' {
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
                        let expr_text = &source[expr_start..expr_end];
                        
                        // Look for inputs.* references
                        let expr_lower = expr_text.to_lowercase();
                        let mut search_pos = 0;
                        
                        while let Some(pos) = expr_lower[search_pos..].find("inputs.") {
                            let actual_pos = search_pos + pos;
                            let after_inputs = &expr_text[actual_pos + 7..]; // Skip "inputs."
                            
                            // Find where the input name ends
                            let name_end = after_inputs
                                .find(|c: char| c.is_whitespace() || c == '}' || c == ')' || c == ']' || 
                                      c == '&' || c == '|' || c == '=' || c == '!' || c == '<' || c == '>' || c == '.')
                                .unwrap_or(after_inputs.len());
                            
                            let input_name = &after_inputs[..name_end.min(after_inputs.len())];
                            
                            if !input_name.is_empty() {
                                references.push((
                                    input_name.to_string(),
                                    Span {
                                        start: i + 3 + actual_pos + 7,
                                        end: i + 3 + actual_pos + 7 + name_end,
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

