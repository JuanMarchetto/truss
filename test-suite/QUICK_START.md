# Quick Start Guide

Since `just` may not be installed, here are the direct bash commands to use:

## Prerequisites

1. **Build Truss:**
   ```bash
   cargo build --release
   ```

2. **Install competitor tools (optional, for full comparison):**
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

## Step 1: Setup Test Repositories

```bash
# Clone all test repositories
bash scripts/setup-test-repos.sh

# Or clone only high-priority ones
bash scripts/setup-test-repos.sh high

# List what was cloned
bash scripts/manage-repos.sh list
```

**Note:** This requires network access and git. If you can't clone repos, you can test with existing fixtures (see below).

## Step 2: Run Test Suite

```bash
# Full test suite (all repos, all tools)
bash scripts/run-full-suite.sh

# Or test a single repository
bash scripts/run-validation.sh test-suite/repos/rust-lang-rust
```

## Step 3: View Results

```bash
# View Markdown report
cat test-suite/comparison/reports/summary.md

# View HTML report (open in browser)
open test-suite/comparison/reports/summary.html
# or
xdg-open test-suite/comparison/reports/summary.html  # Linux

# View JSON comparison data
cat test-suite/comparison/coverage.json | jq
```

## Testing Without Cloning Repos

If you can't clone repositories, test with existing benchmark fixtures:

```bash
# Test framework with existing fixtures
bash scripts/test-framework.sh
```

This will:
- Test Truss JSON output on existing fixtures
- Test competitor capture scripts (if tools are installed)
- Run comparison and report generation
- Show results in `test-suite/results-test/` and `test-suite/comparison-test/`

## Manual Testing

### Test Truss JSON Output

```bash
# Single file
./target/release/truss validate --json benchmarks/fixtures/simple.yml

# Multiple files
./target/release/truss validate --json benchmarks/fixtures/*.yml
```

### Test Competitor Capture Scripts

```bash
# actionlint
bash competitors/actionlint/capture.sh benchmarks/fixtures/simple.yml

# yamllint
bash competitors/yamllint/capture.sh benchmarks/fixtures/simple.yml

# yaml-language-server
bash competitors/yaml-language-server/capture.sh benchmarks/fixtures/simple.yml
```

### Run Comparison Manually

```bash
# Compare results
python3 scripts/compare-results.py test-suite/results test-suite/comparison/coverage.json

# Generate reports
python3 scripts/generate-report.py test-suite/comparison/coverage.json test-suite/comparison/reports
```

## Troubleshooting

### "just: command not found"

Use the bash scripts directly instead:
- `just test-multifile` → `bash scripts/run-full-suite.sh`
- `just test-repo <repo>` → `bash scripts/run-validation.sh test-suite/repos/<repo>`
- `just setup-test-repos` → `bash scripts/setup-test-repos.sh`

### "No such file or directory" for reports

The reports are only created after running the test suite. Run:
```bash
bash scripts/run-full-suite.sh
```

Or test with fixtures first:
```bash
bash scripts/test-framework.sh
```

### Permission denied

Make scripts executable:
```bash
chmod +x scripts/*.sh
chmod +x competitors/*/capture.sh
```

### Tool not found errors

The framework will skip tools that aren't installed. To get full comparison, install the missing tools (see Prerequisites above).

## Expected Output Structure

After running the test suite, you should have:

```
test-suite/
├── repos/                    # Cloned repositories (if setup was run)
├── results/                  # Validation results
│   ├── truss/
│   ├── actionlint/
│   ├── yamllint/
│   └── yaml-language-server/
└── comparison/
    ├── coverage.json         # Comparison data
    └── reports/
        ├── summary.md        # Markdown report
        └── summary.html      # HTML report
```

