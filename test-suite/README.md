# Multifile Adversarial Testing Framework

This is where we pit Truss against the competition -- actionlint, yamllint, and yaml-language-server -- across real GitHub repositories with known workflow issues.

We're testing two things: **correctness** (does each tool catch the right errors?) and **performance** (how fast does it do it?).

## Directory Structure

```
test-suite/
├── repos/                    # Cloned test repositories
├── results/                  # Validation results from all tools
│   ├── truss/
│   ├── actionlint/
│   ├── yamllint/
│   └── yaml-language-server/
├── comparison/               # Analysis and reports
│   ├── coverage.json        # Error detection coverage
│   ├── performance.json     # Timing data
│   └── reports/             # HTML and text reports
└── repos.json               # Test repository configuration
```

## Getting Started

### 1. Clone the test repos

```bash
# All of them
bash scripts/setup-test-repos.sh

# Just the important ones
bash scripts/setup-test-repos.sh high

# See what's there
bash scripts/manage-repos.sh list
```

### 2. Run the full suite

```bash
bash scripts/run-full-suite.sh
```

Or via just:

```bash
just test-multifile
```

### 3. Test a single repo

```bash
bash scripts/run-validation.sh test-suite/repos/rust-lang-rust
```

Or:

```bash
just test-repo rust-lang-rust
```

### 4. Check the results

```bash
# Markdown report
cat test-suite/comparison/reports/summary.md

# HTML report
open test-suite/comparison/reports/summary.html

# Raw JSON
cat test-suite/comparison/coverage.json | jq
```

## Doing Things Manually

### Find workflows in a repo

```bash
bash scripts/discover-workflows.sh test-suite/repos/rust-lang-rust
```

### Run a single tool on a file

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

### Compare and report

```bash
# Compare results across tools
bash scripts/compare-results.py test-suite/results test-suite/comparison/coverage.json

# Generate human-readable reports
bash scripts/generate-report.py test-suite/comparison/coverage.json test-suite/comparison/reports
```

## Managing Test Repos

```bash
bash scripts/manage-repos.sh clone owner/repo   # Clone a new one
bash scripts/manage-repos.sh update repo-name    # Pull latest
bash scripts/manage-repos.sh list                # See what's cloned
bash scripts/manage-repos.sh clean 30            # Remove repos older than 30 days
```

## Test Repositories

These are configured in `repos.json`:

1. **rust-lang/rust** -- Complex dynamic matrices, large workflows
2. **microsoft/TypeScript** -- Multiple workflows, complex job dependencies
3. **actions/checkout** -- Simple but diverse workflow patterns
4. **facebook/react** -- Multiple workflow files, various triggers
5. **pytorch/pytorch** -- Complex matrix strategies
6. **tensorflow/tensorflow** -- Lots of workflow files
7. **kubernetes/kubernetes** -- Large-scale CI/CD patterns

## Output Formats

### Per-tool results (JSON)

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

### Comparison results (JSON)

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

## What We're Looking For

1. **Coverage** -- Truss catches at least 90% of the errors actionlint finds
2. **Performance** -- Truss maintains a 10-15x speed advantage
3. **Unique finds** -- Errors Truss catches that others miss (we document these)
4. **Reliability** -- The comparison pipeline runs cleanly end to end
5. **Clarity** -- Reports that make strengths and weaknesses obvious

## Troubleshooting

### No workflow files found

Make sure the repos are actually cloned:

```bash
bash scripts/setup-test-repos.sh
bash scripts/manage-repos.sh list
```

### Tool not found

Install whatever's missing:

```bash
# actionlint
brew install actionlint
# or: go install github.com/rhymond/actionlint@latest

# yamllint
pip install yamllint
# or: brew install yamllint

# yaml-language-server
npm install -g yaml-language-server
```

### Permission denied

```bash
chmod +x scripts/*.sh
chmod +x competitors/*/capture.sh
```

## Running in CI

Here's a workflow you can drop into `.github/workflows/test-comparison.yml`:

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

## Ideas for Later

- Smarter error message normalization (maybe ML-based)
- Automated false positive detection
- GitHub API integration to find repos with broken workflows
- Historical tracking of results over time
- A web dashboard for browsing comparison data
