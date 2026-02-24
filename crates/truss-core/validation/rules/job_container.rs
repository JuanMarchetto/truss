use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates container and services configurations.
pub struct JobContainerRule;

impl ValidationRule for JobContainerRule {
    fn name(&self) -> &str {
        "job_container"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let jobs_node = match utils::get_jobs_node(tree, source) {
            Some(n) => n,
            None => return diagnostics,
        };

        fn check_job_container(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
            match node.kind() {
                "block_mapping_pair" | "flow_pair" => {
                    if let Some(key_node) = node.child(0) {
                        let job_name = utils::clean_key(key_node, source).to_string();

                        let job_value = utils::get_pair_value(node);

                        if let Some(job_value_raw) = job_value {
                            let job_value = utils::unwrap_node(job_value_raw);

                            if job_value.kind() == "block_mapping"
                                || job_value.kind() == "flow_mapping"
                            {
                                // Check container
                                let container_value =
                                    utils::find_value_for_key(job_value, source, "container");
                                if let Some(container_node) = container_value {
                                    validate_container(
                                        container_node,
                                        source,
                                        &job_name,
                                        diagnostics,
                                    );
                                }

                                // Check services
                                let services_value =
                                    utils::find_value_for_key(job_value, source, "services");
                                if let Some(services_node) = services_value {
                                    validate_services(
                                        services_node,
                                        source,
                                        &job_name,
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
                        check_job_container(child, source, diagnostics);
                    }
                }
            }
        }

        fn validate_container(
            container_node: Node,
            source: &str,
            job_name: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let container_to_check = utils::unwrap_node(container_node);

            // Check image field (required)
            let image_value = utils::find_value_for_key(container_to_check, source, "image");
            if image_value.is_none() {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "Job '{}' container is missing required 'image' field. Container must specify an image.",
                        job_name
                    ),
                    severity: Severity::Error,
                    span: Span {
                        start: container_to_check.start_byte(),
                        end: container_to_check.end_byte(),
                    },
                });
            } else if let Some(image_node) = image_value {
                let image_text = utils::node_text(image_node, source);
                let image_cleaned =
                    image_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

                if image_cleaned.is_empty() {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Job '{}' container has empty image field. Container image is required.",
                            job_name
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: image_node.start_byte(),
                            end: image_node.end_byte(),
                        },
                    });
                }
            }

            // Check ports format
            let ports_value = utils::find_value_for_key(container_to_check, source, "ports");
            if let Some(ports_node) = ports_value {
                validate_ports(ports_node, source, job_name, diagnostics);
            }
        }

        fn validate_services(
            services_node: Node,
            source: &str,
            job_name: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            // Services is a mapping of service names to container configs
            // Validate each service container
            fn check_service(
                node: Node,
                source: &str,
                job_name: &str,
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                match node.kind() {
                    "block_mapping_pair" | "flow_pair" => {
                        if let Some(key_node) = node.child(0) {
                            let _service_name = utils::node_text(key_node, source);

                            let service_value = utils::get_pair_value(node);

                            if let Some(service_value_raw) = service_value {
                                let service_value = utils::unwrap_node(service_value_raw);

                                validate_container(service_value, source, job_name, diagnostics);
                            }
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_service(child, source, job_name, diagnostics);
                        }
                    }
                }
            }

            check_service(services_node, source, job_name, diagnostics);
        }

        fn validate_ports(
            ports_node: Node,
            source: &str,
            job_name: &str,
            diagnostics: &mut Vec<Diagnostic>,
        ) {
            let _ports_text = utils::node_text(ports_node, source);

            // Ports can be an array of strings in format "host:container"
            // Basic validation: check if format looks correct
            fn check_port_format(
                node: Node,
                source: &str,
                job_name: &str,
                diagnostics: &mut Vec<Diagnostic>,
            ) {
                match node.kind() {
                    "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                        let port_text = utils::node_text(node, source);
                        let port_cleaned = port_text
                            .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());

                        // Check if it's in format "host:container"
                        if !port_cleaned.contains(':') && !port_cleaned.is_empty() {
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "Job '{}' container has invalid port format: '{}'. Ports should be in format 'host:container'.",
                                    job_name, port_cleaned
                                ),
                                severity: Severity::Error,
                                span: Span {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
                                },
                            });
                        }
                    }
                    _ => {
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            check_port_format(child, source, job_name, diagnostics);
                        }
                    }
                }
            }

            check_port_format(ports_node, source, job_name, diagnostics);
        }

        check_job_container(jobs_node, source, &mut diagnostics);

        diagnostics
    }
}
