# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Community infrastructure: issue templates, security policy, code of conduct
- Release workflow for cross-platform binary distribution

## [0.1.0] - 2025-01-15

### Added
- Initial release of Truss GitHub Actions workflow validator
- 41 validation rules covering:
  - Workflow structure and syntax validation
  - Job configuration (needs, outputs, strategy, containers)
  - Step validation (shell, timeout, env, working directory)
  - Expression validation and script injection detection
  - Action reference validation and deprecated command detection
  - Permissions, secrets, and environment validation
  - Reusable workflow call validation (inputs, outputs, secrets)
  - Matrix strategy and concurrency configuration
  - Artifact and event payload validation
  - Runner label validation
- Tree-sitter based YAML parsing for accurate AST analysis
- Parallel rule execution via rayon
- CLI with glob pattern support, directory recursion, stdin input
- JSON output format for CI/CD integration
- Severity filtering (error, warning, info)
- LSP server for real-time editor integration
- Zero false positives on real-world workflow testing
- Performance: sub-millisecond single-file validation

[Unreleased]: https://github.com/JuanMarchetto/truss/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/JuanMarchetto/truss/releases/tag/v0.1.0
