# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Truss, please report it responsibly.

**Do NOT open a public issue for security vulnerabilities.**

Instead, please open a private security advisory via GitHub's "Report a vulnerability" button on the Security tab.

### What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response timeline

- **Acknowledgment:** within 48 hours
- **Initial assessment:** within 1 week
- **Fix & disclosure:** coordinated with reporter

## Scope

Truss is a static analysis tool that reads YAML files. Security concerns include:

- **Path traversal:** Truss should not read files outside the specified paths
- **Denial of service:** Maliciously crafted YAML causing excessive resource consumption
- **Code execution:** Truss should never execute workflow commands, only analyze them
- **Dependency vulnerabilities:** Issues in tree-sitter or other dependencies

## Security measures

- Truss runs `cargo audit` in CI to check for known dependency vulnerabilities
- All parsing is done via tree-sitter (memory-safe, sandboxed)
- No network access during validation
- No code execution or shell invocation
