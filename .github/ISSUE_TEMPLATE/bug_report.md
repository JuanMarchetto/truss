---
name: Bug Report
about: Report incorrect behavior or false positives/negatives
title: "[Bug] "
labels: bug
assignees: ''
---

## Describe the bug

A clear and concise description of the incorrect behavior.

## Workflow YAML

```yaml
# Minimal workflow that reproduces the issue
name: example
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
```

## Expected behavior

What diagnostics (or lack thereof) did you expect?

## Actual behavior

What diagnostics did Truss produce?

## Environment

- **Truss version:** (`truss validate --version`)
- **OS:** (e.g., macOS 14, Ubuntu 22.04, Windows 11)
- **Installation method:** (cargo install, binary release, VS Code extension)

## Additional context

- Is this a false positive (Truss flagged valid YAML) or a false negative (Truss missed an issue)?
- Does actionlint produce a different result for this workflow?
