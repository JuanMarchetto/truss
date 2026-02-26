# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 41 validation rules across 5 categories (core, job, step, workflow, expression/security)
- Tree-sitter YAML parsing with incremental re-parsing support
- Parallel rule execution via rayon
- CLI with glob patterns, directory recursion, stdin input, JSON output, severity filtering
- Per-rule filtering: `--ignore-rules` and `--only-rules`
- `.truss.yml` project configuration (ignore paths, enable/disable rules)
- Unique rule IDs on all diagnostics
- LSP server with real-time diagnostics and incremental parsing
- VS Code extension
- WASM bindings and browser playground
- Zero false positives on 271 production workflow files from 7 major OSS projects
- Cross-platform release workflow (Linux, macOS, Windows)
- GitHub Pages deployment for WASM playground
- Code coverage via cargo-tarpaulin in CI

[Unreleased]: https://github.com/JuanMarchetto/truss/commits/main
