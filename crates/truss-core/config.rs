//! Configuration file support for Truss.
//!
//! Parses `.truss.yml` configuration files that allow users to customize
//! which rules are enabled, set severity overrides, and ignore file patterns.
//!
//! # Example `.truss.yml`
//!
//! ```yaml
//! rules:
//!   timeout:
//!     enabled: false
//!   script-injection:
//!     severity: error
//!
//! ignore:
//!   - "vendor/**"
//!   - ".github/workflows/generated-*.yml"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TrussConfig {
    /// Per-rule configuration overrides.
    pub rules: HashMap<String, RuleConfig>,

    /// File glob patterns to ignore during validation.
    pub ignore: Vec<String>,
}

/// Configuration for an individual rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleConfig {
    /// Whether this rule is enabled. Defaults to `true`.
    pub enabled: bool,

    /// Override the default severity for this rule.
    /// Valid values: "error", "warning", "info".
    pub severity: Option<String>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: None,
        }
    }
}

impl TrussConfig {
    /// Load configuration from a YAML file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::from_str(&content, path)
    }

    /// Parse configuration from a YAML string.
    fn from_str(content: &str, path: &Path) -> Result<Self, ConfigError> {
        serde_yaml::from_str(content).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    /// Auto-discover `.truss.yml` by walking up from the given directory.
    ///
    /// Searches the given directory, then each parent, stopping at the
    /// filesystem root. Returns `None` if no config file is found.
    pub fn discover(start_dir: &Path) -> Option<PathBuf> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(".truss.yml");
            if candidate.is_file() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Check if a file path should be ignored based on the `ignore` patterns.
    pub fn is_ignored(&self, path: &str) -> bool {
        for pattern in &self.ignore {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(path) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a rule is enabled.
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        match self.rules.get(rule_name) {
            Some(config) => config.enabled,
            None => true, // enabled by default
        }
    }

    /// Get the severity override for a rule, if any.
    pub fn rule_severity(&self, rule_name: &str) -> Option<&str> {
        self.rules
            .get(rule_name)
            .and_then(|c| c.severity.as_deref())
    }
}

/// Errors that can occur when loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error reading the config file.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// YAML parse error.
    Parse { path: PathBuf, message: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io { path, source } => {
                write!(f, "Failed to read config '{}': {}", path.display(), source)
            }
            ConfigError::Parse { path, message } => {
                write!(f, "Invalid config '{}': {}", path.display(), message)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_has_defaults() {
        let config: TrussConfig = serde_yaml::from_str("").unwrap();
        assert!(config.rules.is_empty());
        assert!(config.ignore.is_empty());
    }

    #[test]
    fn parse_rule_config() {
        let yaml = r#"
rules:
  timeout:
    enabled: false
  script-injection:
    severity: error
"#;
        let config: TrussConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.is_rule_enabled("timeout"));
        assert!(config.is_rule_enabled("script-injection"));
        assert_eq!(config.rule_severity("script-injection"), Some("error"));
    }

    #[test]
    fn parse_ignore_patterns() {
        let yaml = r#"
ignore:
  - "vendor/**"
  - "*.generated.yml"
"#;
        let config: TrussConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.ignore.len(), 2);
    }

    #[test]
    fn unknown_rule_is_enabled() {
        let config = TrussConfig::default();
        assert!(config.is_rule_enabled("nonexistent-rule"));
    }
}
