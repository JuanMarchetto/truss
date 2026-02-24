use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::{Node, Tree};

/// Validates event-specific fields in on: triggers.
pub struct EventPayloadValidationRule;

impl ValidationRule for EventPayloadValidationRule {
    fn name(&self) -> &str {
        "event_payload"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let root = tree.root_node();
        let on_value = match utils::find_value_for_key(root, source, "on") {
            Some(v) => v,
            None => return diagnostics,
        };

        let on_to_check = utils::unwrap_node(on_value);

        // Validate push event fields
        let push_value = utils::find_value_for_key(on_to_check, source, "push");
        if let Some(push_node) = push_value {
            validate_push_event(push_node, source, &mut diagnostics);
        }

        // Validate pull_request event fields
        let pr_value = utils::find_value_for_key(on_to_check, source, "pull_request");
        if let Some(pr_node) = pr_value {
            validate_pull_request_event(pr_node, source, &mut diagnostics);
        }

        // Validate schedule event fields
        let schedule_value = utils::find_value_for_key(on_to_check, source, "schedule");
        if let Some(schedule_node) = schedule_value {
            validate_schedule_event(schedule_node, source, &mut diagnostics);
        }

        // Validate workflow_dispatch event fields
        let workflow_dispatch_value =
            utils::find_value_for_key(on_to_check, source, "workflow_dispatch");
        if let Some(wd_node) = workflow_dispatch_value {
            validate_workflow_dispatch_event(wd_node, source, &mut diagnostics);
        }

        // Validate workflow_call event fields
        let workflow_call_value = utils::find_value_for_key(on_to_check, source, "workflow_call");
        if let Some(wc_node) = workflow_call_value {
            validate_workflow_call_event(wc_node, source, &mut diagnostics);
        }

        // Validate issues event fields
        let issues_value = utils::find_value_for_key(on_to_check, source, "issues");
        if let Some(issues_node) = issues_value {
            validate_issues_event(issues_node, source, &mut diagnostics);
        }

        diagnostics
    }
}

fn validate_push_event(push_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let push_to_check = utils::unwrap_node(push_node);

    // Valid fields for push: branches, branches-ignore, paths, paths-ignore, tags, tags-ignore
    let valid_fields = [
        "branches",
        "branches-ignore",
        "paths",
        "paths-ignore",
        "tags",
        "tags-ignore",
    ];

    // Check for branches + branches-ignore conflict
    let has_branches_plain = utils::find_value_for_key(push_to_check, source, "branches").is_some();
    let has_branches_ignore =
        utils::find_value_for_key(push_to_check, source, "branches-ignore").is_some();
    if has_branches_plain && has_branches_ignore {
        if let Some(bi_node) = utils::find_value_for_key(push_to_check, source, "branches-ignore") {
            diagnostics.push(Diagnostic {
                message: "Cannot use both 'branches' and 'branches-ignore' on the same event. They are mutually exclusive.".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: bi_node.start_byte(),
                    end: bi_node.end_byte(),
                },
            });
        }
    }

    // Check for tags + tags-ignore conflict
    let has_tags_plain = utils::find_value_for_key(push_to_check, source, "tags").is_some();
    let has_tags_ignore = utils::find_value_for_key(push_to_check, source, "tags-ignore").is_some();
    if has_tags_plain && has_tags_ignore {
        if let Some(ti_node) = utils::find_value_for_key(push_to_check, source, "tags-ignore") {
            diagnostics.push(Diagnostic {
                message: "Cannot use both 'tags' and 'tags-ignore' on the same event. They are mutually exclusive.".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: ti_node.start_byte(),
                    end: ti_node.end_byte(),
                },
            });
        }
    }

    // Check for paths + paths-ignore conflict
    let has_paths = utils::find_value_for_key(push_to_check, source, "paths").is_some();
    let has_paths_ignore =
        utils::find_value_for_key(push_to_check, source, "paths-ignore").is_some();
    if has_paths && has_paths_ignore {
        if let Some(pi_node) = utils::find_value_for_key(push_to_check, source, "paths-ignore") {
            diagnostics.push(Diagnostic {
                message: "Cannot use both 'paths' and 'paths-ignore' on the same event. They are mutually exclusive.".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: pi_node.start_byte(),
                    end: pi_node.end_byte(),
                },
            });
        }
    }

    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for push event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(push_to_check, source, &valid_fields, diagnostics);
}

fn validate_pull_request_event(pr_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let pr_to_check = utils::unwrap_node(pr_node);

    // Check for branches + branches-ignore conflict
    let has_branches_plain = utils::find_value_for_key(pr_to_check, source, "branches").is_some();
    let has_branches_ignore =
        utils::find_value_for_key(pr_to_check, source, "branches-ignore").is_some();
    if has_branches_plain && has_branches_ignore {
        if let Some(bi_node) = utils::find_value_for_key(pr_to_check, source, "branches-ignore") {
            diagnostics.push(Diagnostic {
                message: "Cannot use both 'branches' and 'branches-ignore' on the same event. They are mutually exclusive.".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: bi_node.start_byte(),
                    end: bi_node.end_byte(),
                },
            });
        }
    }

    // Check for paths + paths-ignore conflict
    let has_paths = utils::find_value_for_key(pr_to_check, source, "paths").is_some();
    let has_paths_ignore = utils::find_value_for_key(pr_to_check, source, "paths-ignore").is_some();
    if has_paths && has_paths_ignore {
        if let Some(pi_node) = utils::find_value_for_key(pr_to_check, source, "paths-ignore") {
            diagnostics.push(Diagnostic {
                message: "Cannot use both 'paths' and 'paths-ignore' on the same event. They are mutually exclusive.".to_string(),
                severity: Severity::Error,
                span: Span {
                    start: pi_node.start_byte(),
                    end: pi_node.end_byte(),
                },
            });
        }
    }

    // Valid fields for pull_request: types, branches, branches-ignore, paths, paths-ignore
    let valid_fields = [
        "types",
        "branches",
        "branches-ignore",
        "paths",
        "paths-ignore",
    ];

    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for pull_request event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }

                    // Validate types field values
                    if key_cleaned == "types" {
                        if let Some(value_node) = utils::get_pair_value(node) {
                            validate_pr_types(value_node, source, diagnostics);
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(pr_to_check, source, &valid_fields, diagnostics);
}

fn validate_pr_types(types_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let valid_types = [
        "opened",
        "closed",
        "synchronize",
        "reopened",
        "assigned",
        "unassigned",
        "labeled",
        "unlabeled",
        "review_requested",
        "review_request_removed",
        "edited",
        "ready_for_review",
        "converted_to_draft",
        "auto_merge_enabled",
        "auto_merge_disabled",
        "enqueued",
        "dequeued",
        "milestoned",
        "demilestoned",
        "locked",
        "unlocked",
    ];

    fn check_type(
        node: Node,
        source: &str,
        valid_types: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                let type_text = utils::node_text(node, source);
                let type_cleaned =
                    type_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                if !valid_types.contains(&type_cleaned) {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Invalid pull_request type: '{}'. Valid types are: {}",
                            type_cleaned,
                            valid_types.join(", ")
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
                    check_type(child, source, valid_types, diagnostics);
                }
            }
        }
    }

    check_type(types_node, source, &valid_types, diagnostics);
}

fn validate_cron_expression(
    cron_cleaned: &str,
    cron_node: Node,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if cron_cleaned.starts_with("${{") {
        return;
    }

    let parts: Vec<&str> = cron_cleaned.split_whitespace().collect();
    if parts.len() != 5 {
        diagnostics.push(Diagnostic {
            message: format!(
                "Invalid cron expression: '{}'. Cron expression must have 5 space-separated fields (minute hour day month weekday).",
                cron_cleaned
            ),
            severity: Severity::Error,
            span: Span {
                start: cron_node.start_byte(),
                end: cron_node.end_byte(),
            },
        });
        return;
    }

    // Validate field ranges: minute(0-59), hour(0-23), day(1-31), month(1-12), weekday(0-6)
    let field_specs: &[(&str, u32, u32)] = &[
        ("minute", 0, 59),
        ("hour", 0, 23),
        ("day of month", 1, 31),
        ("month", 1, 12),
        ("day of week", 0, 6),
    ];

    for (i, (field_name, min, max)) in field_specs.iter().enumerate() {
        if let Some(err) = validate_cron_field(parts[i], field_name, *min, *max) {
            diagnostics.push(Diagnostic {
                message: format!(
                    "Invalid cron {}: '{}' in '{}'. {}",
                    field_name, parts[i], cron_cleaned, err
                ),
                severity: Severity::Error,
                span: Span {
                    start: cron_node.start_byte(),
                    end: cron_node.end_byte(),
                },
            });
        }
    }
}

/// Validate a single cron field (e.g., "*/15", "1-5", "0,30", "MON-FRI").
/// Returns None if valid, Some(error_message) if invalid.
fn validate_cron_field(field: &str, name: &str, min: u32, max: u32) -> Option<String> {
    if field == "*" {
        return None;
    }

    // Handle step values: */N or range/N
    if let Some(base_and_step) = field.strip_prefix("*/") {
        return match base_and_step.parse::<u32>() {
            Ok(0) => Some(format!("Step value must be greater than 0 for {}", name)),
            Ok(_) => None,
            Err(_) => Some(format!(
                "Invalid step value '{}' for {}",
                base_and_step, name
            )),
        };
    }

    // Split by comma for lists: "1,2,3"
    for part in field.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Some(format!("Empty value in {} field", name));
        }

        // Handle step in range: "1-5/2"
        let (range_part, step_part) = if let Some(idx) = part.find('/') {
            (&part[..idx], Some(&part[idx + 1..]))
        } else {
            (part, None)
        };

        if let Some(step_str) = step_part {
            match step_str.parse::<u32>() {
                Ok(0) => return Some(format!("Step value must be greater than 0 for {}", name)),
                Ok(_) => {}
                Err(_) => return Some(format!("Invalid step value '{}' for {}", step_str, name)),
            }
        }

        // Handle ranges: "1-5"
        if range_part.contains('-') {
            let bounds: Vec<&str> = range_part.splitn(2, '-').collect();
            if bounds.len() == 2 {
                match (bounds[0].parse::<u32>(), bounds[1].parse::<u32>()) {
                    (Ok(lo), Ok(hi)) => {
                        if lo < min || lo > max {
                            return Some(format!(
                                "Value {} is out of range ({}-{}) for {}",
                                lo, min, max, name
                            ));
                        }
                        if hi < min || hi > max {
                            return Some(format!(
                                "Value {} is out of range ({}-{}) for {}",
                                hi, min, max, name
                            ));
                        }
                    }
                    _ => {
                        // Could be month/weekday names — skip validation
                    }
                }
            }
        } else {
            // Single value
            match range_part.parse::<u32>() {
                Ok(val) => {
                    if val < min || val > max {
                        return Some(format!(
                            "Value {} is out of range ({}-{}) for {}",
                            val, min, max, name
                        ));
                    }
                }
                Err(_) => {
                    // Could be month/weekday name (JAN, MON, etc.) — skip validation
                }
            }
        }
    }

    None
}

fn validate_schedule_event(schedule_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let schedule_to_check = utils::unwrap_node(schedule_node);

    // GitHub Actions schedule is a list of objects: [{cron: '...'}, {cron: '...'}]
    // Check if it's a sequence and validate each item has a cron field
    if schedule_to_check.kind() == "block_sequence" || schedule_to_check.kind() == "flow_sequence" {
        let mut cursor = schedule_to_check.walk();
        let mut found_any_cron = false;
        for item in schedule_to_check.children(&mut cursor) {
            let item_content = utils::unwrap_node(item);
            // block_sequence_item children: "-" token, then content (skip comments)
            let actual_content = if item_content.kind() == "block_sequence_item" {
                let mut content = item_content;
                for i in 1..item_content.child_count() {
                    if let Some(child) = item_content.child(i) {
                        if child.kind() != "comment" {
                            content = child;
                            break;
                        }
                    }
                }
                utils::unwrap_node(content)
            } else {
                item_content
            };

            if actual_content.kind() == "block_mapping" || actual_content.kind() == "flow_mapping" {
                let cron_in_item = utils::find_value_for_key(actual_content, source, "cron");
                if let Some(cron_node) = cron_in_item {
                    found_any_cron = true;
                    let cron_text = utils::node_text(cron_node, source);
                    let cron_cleaned = cron_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                    validate_cron_expression(cron_cleaned, cron_node, diagnostics);
                }
            }
        }
        if found_any_cron {
            return;
        }
    }

    // Fallback: try to find cron directly (for non-sequence structures)
    let cron_value = utils::find_value_for_key(schedule_to_check, source, "cron");
    if cron_value.is_none() {
        diagnostics.push(Diagnostic {
            message: "schedule event is missing required 'cron' field.".to_string(),
            severity: Severity::Error,
            span: Span {
                start: schedule_to_check.start_byte(),
                end: schedule_to_check.end_byte(),
            },
        });
    } else if let Some(cron_node) = cron_value {
        let cron_text = utils::node_text(cron_node, source);
        let cron_cleaned =
            cron_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
        validate_cron_expression(cron_cleaned, cron_node, diagnostics);
    }

    // Valid fields for schedule: cron
    let valid_fields = ["cron"];
    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for schedule event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(schedule_to_check, source, &valid_fields, diagnostics);
}

fn validate_workflow_dispatch_event(
    wd_node: Node,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let wd_to_check = utils::unwrap_node(wd_node);

    // Valid fields for workflow_dispatch: inputs
    let valid_fields = ["inputs"];
    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for workflow_dispatch event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(wd_to_check, source, &valid_fields, diagnostics);
}

fn validate_workflow_call_event(wc_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let wc_to_check = utils::unwrap_node(wc_node);

    // Valid fields for workflow_call: inputs, secrets, outputs
    let valid_fields = ["inputs", "secrets", "outputs"];
    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for workflow_call event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(wc_to_check, source, &valid_fields, diagnostics);
}

fn validate_issues_event(issues_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let issues_to_check = utils::unwrap_node(issues_node);

    // Valid fields for issues: types
    let valid_fields = ["types"];

    fn check_fields(
        node: Node,
        source: &str,
        valid_fields: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "block_mapping_pair" | "flow_pair" => {
                if let Some(key_node) = node.child(0) {
                    let key_text = utils::node_text(key_node, source);
                    let key_cleaned = key_text
                        .trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace())
                        .trim_end_matches(':');

                    if !valid_fields.contains(&key_cleaned) {
                        diagnostics.push(Diagnostic {
                            message: format!(
                                "Invalid field '{}' for issues event. Valid fields are: {}",
                                key_cleaned,
                                valid_fields.join(", ")
                            ),
                            severity: Severity::Error,
                            span: Span {
                                start: key_node.start_byte(),
                                end: key_node.end_byte(),
                            },
                        });
                    }

                    // Validate types field values
                    if key_cleaned == "types" {
                        if let Some(value_node) = utils::get_pair_value(node) {
                            validate_issues_types(value_node, source, diagnostics);
                        }
                    }
                }
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    check_fields(child, source, valid_fields, diagnostics);
                }
            }
        }
    }

    check_fields(issues_to_check, source, &valid_fields, diagnostics);
}

fn validate_issues_types(types_node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let valid_types = [
        "opened",
        "edited",
        "deleted",
        "closed",
        "reopened",
        "assigned",
        "unassigned",
        "labeled",
        "unlabeled",
        "locked",
        "unlocked",
        "transferred",
        "milestoned",
        "demilestoned",
        "pinned",
        "unpinned",
    ];

    fn check_type(
        node: Node,
        source: &str,
        valid_types: &[&str],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "plain_scalar" | "double_quoted_scalar" | "single_quoted_scalar" => {
                let type_text = utils::node_text(node, source);
                let type_cleaned =
                    type_text.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
                if !valid_types.contains(&type_cleaned) {
                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Invalid issues activity type: '{}'. Valid types are: {}",
                            type_cleaned,
                            valid_types.join(", ")
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
                    check_type(child, source, valid_types, diagnostics);
                }
            }
        }
    }

    check_type(types_node, source, &valid_types, diagnostics);
}
