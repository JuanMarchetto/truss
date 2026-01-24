# Multifile Adversarial Testing Framework

This directory contains the testing framework for comparing Truss against competitors (actionlint, yamllint, yaml-language-server) in complex multifile scenarios.

## Overview

The test suite evaluates both **correctness** (which errors each tool finds) and **performance** (execution time) across real-world GitHub repositories with known workflow issues.

## Directory Structure

```
test-suite/
├── repos/                    # Cloned test repositories
├── results/                  # Validation results from all tools
│   ├── truss/
│   ├── actionlint/
│   ├── yamllint/
│   └── yaml-language-server/
├── comparison/               # Comparison analysis and reports
│   ├── coverage.json        # Error detection coverage
│   ├── performance.json     # Timing data
│   └── reports/             # HTML/text reports
└── repos.json               # Test repository configuration
```

## Quick Start

### 1. Setup Test Repositories

Clone test repositories:

```bash
# Clone all repositories
bash scripts/setup-test-repos.sh

# Clone only high-priority repositories
bash scripts/setup-test-repos.sh high

# List available repositories
bash scripts/manage-repos.sh list
```

### 2. Run Full Test Suite

Run validation on all repositories:

```bash
bash scripts/run-full-suite.sh
```

Or use the justfile command:

```bash
just test-multifile
```

### 3. Run on Single Repository

Test a specific repository:

```bash
bash scripts/run-validation.sh test-suite/repos/rust-lang-rust
```

Or use the justfile command:

```bash
just test-repo rust-lang-rust
```

### 4. View Results

After running the test suite, view the generated reports:

```bash
# Markdown report
cat test-suite/comparison/reports/summary.md

# HTML report (open in browser)
open test-suite/comparison/reports/summary.html

# JSON comparison data
cat test-suite/comparison/coverage.json | jq
```

## Manual Steps

### Discover Workflows

Find all workflow files in a repository:

```bash
bash scripts/discover-workflows.sh test-suite/repos/rust-lang-rust
```

### Run Individual Tool

Run a specific tool on a file:

```bash
# Truss
./target/release/truss validate --json workflow.yml

# actionlint
bash competitors/actionlint/capture.sh workflow.yml

# yamllint
bash competitors/yamllint/capture.sh workflow.yml

# yaml-language-server
bash competitors/yaml-language-server/capture.sh workflow.yml
```

### Compare Results

Compare results from different tools:

```bash
bash scripts/compare-results.py test-suite/results test-suite/comparison/coverage.json
```

### Generate Reports

Generate reports from comparison data:

```bash
bash scripts/generate-report.py test-suite/comparison/coverage.json test-suite/comparison/reports
```

## Repository Management

Manage test repositories:

```bash
# Clone a repository
bash scripts/manage-repos.sh clone owner/repo

# Update a repository
bash scripts/manage-repos.sh update repo-name

# List all repositories
bash scripts/manage-repos.sh list

# Clean old repositories (older than 30 days)
bash scripts/manage-repos.sh clean 30
```

## Test Repositories

The following repositories are configured in `repos.json`:

1. **rust-lang/rust** - Complex dynamic matrices, large workflows
2. **microsoft/TypeScript** - Multiple workflows, complex job dependencies
3. **actions/checkout** - Simple but diverse workflow patterns
4. **facebook/react** - Multiple workflow files, various triggers
5. **pytorch/pytorch** - Complex matrix strategies
6. **tensorflow/tensorflow** - Large number of workflow files
7. **kubernetes/kubernetes** - Large-scale CI/CD patterns

## Results Format

### Tool Results (JSON)

Each tool outputs results in the following format:

```json
{
  "file": "path/to/workflow.yml",
  "valid": false,
  "diagnostics": [
    {
      "message": "Missing required field 'on:'",
      "severity": "error",
      "location": {
        "file": "path/to/workflow.yml",
        "line": 1,
        "column": 1
      }
    }
  ],
  "duration_ms": 12.5,
  "metadata": {
    "file_size": 1024,
    "lines": 45
  }
}
```

### Comparison Results (JSON)

The comparison engine produces:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "files_analyzed": 15,
  "tools": {
    "truss": {
      "errors_found": 23,
      "files_analyzed": 15,
      "avg_time_ms": 11.2,
      "total_time_ms": 168.0
    }
  },
  "coverage_analysis": {
    "actionlint": {
      "total_errors": 28,
      "truss_found": 23,
      "coverage": 0.82
    }
  },
  "summary": {
    "total_files": 15,
    "total_errors_truss": 23,
    "total_errors_actionlint": 28,
    "coverage_truss": 0.82,
    "avg_time_truss_ms": 11.2,
    "avg_time_actionlint_ms": 165.7
  }
}
```

## Success Metrics

The test suite evaluates:

1. **Coverage**: Truss finds ≥90% of errors found by actionlint
2. **Performance**: Truss maintains 10-15x speed advantage
3. **Unique Value**: Truss finds errors others miss (document cases)
4. **Reliability**: Comparison runs complete without errors
5. **Documentation**: Clear reports showing strengths/weaknesses

## Troubleshooting

### No workflow files found

Ensure repositories are cloned:

```bash
bash scripts/setup-test-repos.sh
bash scripts/manage-repos.sh list
```

### Tool not found errors

Install missing tools:

```bash
# actionlint
brew install actionlint
# or
go install github.com/rhymond/actionlint@latest

# yamllint
pip install yamllint
# or
brew install yamllint

# yaml-language-server
npm install -g yaml-language-server
```

### Permission denied

Make scripts executable:

```bash
chmod +x scripts/*.sh
chmod +x competitors/*/capture.sh
```

## Continuous Integration

To run comparison tests in CI, add to `.github/workflows/test-comparison.yml`:

```yaml
name: Test Comparison

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  compare:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup
        run: |
          cargo build --release
          bash scripts/setup-test-repos.sh high
      - name: Run comparison
        run: bash scripts/run-full-suite.sh
      - name: Upload reports
        uses: actions/upload-artifact@v3
        with:
          name: comparison-reports
          path: test-suite/comparison/reports/
```

## Future Enhancements

- Machine learning for error message normalization
- Automated false positive detection
- Integration with GitHub API to find repos with workflow issues
- Historical tracking of comparison results
- Web dashboard for results visualization

