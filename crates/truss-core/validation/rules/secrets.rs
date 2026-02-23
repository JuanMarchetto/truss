use super::super::utils;
use super::super::ValidationRule;
use crate::{Diagnostic, Severity, Span};
use tree_sitter::Tree;

/// Validates secrets.* references in GitHub Actions workflows.
pub struct SecretsValidationRule;

impl ValidationRule for SecretsValidationRule {
    fn name(&self) -> &str {
        "secrets_validation"
    }

    fn validate(&self, tree: &Tree, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if !utils::is_github_actions_workflow(tree, source) {
            return diagnostics;
        }

        // Find all expressions in the source
        // Expressions are in the format ${{ ... }}
        let source_bytes = source.as_bytes();
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
                    // Extract the expression content (between ${{ and }})
                    let expr_start = i + 3;
                    let expr_end = j - 2;

                    if expr_start < expr_end && expr_end <= source_bytes.len() {
                        let expr_text = &source[expr_start..expr_end];

                        // Check for secret references
                        self.check_secret_references(expr_text, i, j, &mut diagnostics);
                    }

                    i = j;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        diagnostics
    }
}

impl SecretsValidationRule {
    fn check_secret_references(
        &self,
        expr: &str,
        expr_start: usize,
        _expr_end: usize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Look for secret references in the expression
        // Valid: secrets.SECRET_NAME
        // Invalid: secret.SECRET_NAME (singular)
        // Invalid: secretsSECRET_NAME (missing dot)

        let expr_lower = expr.to_lowercase();
        let mut search_pos = 0;

        while let Some(pos) = expr_lower[search_pos..].find("secret") {
            let actual_pos = search_pos + pos;
            let remaining = &expr[actual_pos..];

            // Check if it's "secret" (singular) - should be "secrets" (plural)
            if remaining.starts_with("secret.") && !remaining.starts_with("secrets.") {
                // Found "secret." instead of "secrets."
                // Find the end of this reference
                let secret_ref_start = actual_pos;
                let after_secret = &remaining[7..]; // Skip "secret."

                // Find where the secret name ends (space, operator, closing brace, etc.)
                let secret_name_end = after_secret
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
                    })
                    .unwrap_or(after_secret.len());

                let secret_name = &after_secret[..secret_name_end.min(after_secret.len())];
                let secret_ref_end = actual_pos + 7 + secret_name_end;

                diagnostics.push(Diagnostic {
                    message: format!(
                        "Invalid secret reference: 'secret.{}' should be 'secrets.{}' (use plural 'secrets')",
                        secret_name, secret_name
                    ),
                    severity: Severity::Error,
                    span: Span {
                        start: expr_start + 3 + secret_ref_start,
                        end: expr_start + 3 + secret_ref_end,
                    },
                });

                search_pos = secret_ref_end;
            }
            // Check if it's "secrets" followed by something that's not a dot
            else if remaining.starts_with("secrets") && remaining.len() > 7 {
                let after_secrets = &remaining[7..];
                // Check if next character is not a dot and not whitespace/operator
                let first_char = after_secrets.chars().next();
                let is_whitespace_or_op = first_char
                    .map(|c| {
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
                    })
                    .unwrap_or(false);

                if !after_secrets.starts_with('.') && !is_whitespace_or_op {
                    // Missing dot after "secrets"
                    // Find where the identifier ends
                    let identifier_end = after_secrets
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
                        .unwrap_or(after_secrets.len());

                    let identifier = &after_secrets[..identifier_end.min(after_secrets.len())];

                    diagnostics.push(Diagnostic {
                        message: format!(
                            "Invalid secret reference: 'secrets{}' should be 'secrets.{}' (missing dot)",
                            identifier, identifier
                        ),
                        severity: Severity::Error,
                        span: Span {
                            start: expr_start + 3 + actual_pos,
                            end: expr_start + 3 + actual_pos + 7 + identifier_end,
                        },
                    });

                    search_pos = actual_pos + 7 + identifier_end;
                } else {
                    // Valid "secrets." reference, validate secret name format
                    search_pos = actual_pos + 7;
                    if let Some(after_dot) = after_secrets.strip_prefix('.') {
                        // Find end of secret name
                        let name_end = after_dot
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
                            .unwrap_or(after_dot.len());

                        let secret_name = &after_dot[..name_end.min(after_dot.len())];

                        // Validate secret name format
                        if !is_valid_secret_name_format(secret_name) {
                            diagnostics.push(Diagnostic {
                                message: format!(
                                    "Invalid secret name format: '{}'. Secret names should contain only alphanumeric characters, hyphens, and underscores, and start with a letter or underscore.",
                                    secret_name
                                ),
                                severity: Severity::Warning,
                                span: Span {
                                    start: expr_start + 3 + actual_pos + 7 + 1,
                                    end: expr_start + 3 + actual_pos + 7 + 1 + name_end,
                                },
                            });
                        }

                        search_pos += 1 + name_end;
                    }
                }
            } else {
                search_pos = actual_pos + 6; // Skip past "secret"
            }
        }
    }
}

/// Validates that a secret name follows a reasonable format.
/// Secret names should contain only alphanumeric characters, hyphens, and underscores,
/// and should start with a letter or underscore.
fn is_valid_secret_name_format(secret_name: &str) -> bool {
    if secret_name.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    if let Some(first_char) = secret_name.chars().next() {
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }
    }

    // All characters must be alphanumeric, hyphen, or underscore
    secret_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
