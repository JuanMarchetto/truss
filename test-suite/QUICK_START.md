# Quick Start

Don't have `just` installed? No worries -- here are the plain bash commands.

## Before You Start

**Build Truss:**

```bash
cargo build --release
```

**Install the competitor tools** (optional -- you only need these for full comparison):

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

## Step 1: Grab the Test Repos

```bash
# Clone all test repositories
bash scripts/setup-test-repos.sh

# Or just the high-priority ones
bash scripts/setup-test-repos.sh high

# Check what got cloned
bash scripts/manage-repos.sh list
```

If you don't have network access or just want to try things out quickly, skip ahead to "Testing Without Cloning Repos" below.

## Step 2: Run the Tests

```bash
# Run everything (all repos, all tools)
bash scripts/run-full-suite.sh

# Or just one repo
bash scripts/run-validation.sh test-suite/repos/rust-lang-rust
```

## Step 3: See What Happened

```bash
# Markdown report
cat test-suite/comparison/reports/summary.md

# HTML report
open test-suite/comparison/reports/summary.html
# On Linux:
xdg-open test-suite/comparison/reports/summary.html

# Raw JSON
cat test-suite/comparison/coverage.json | jq
```

## Testing Without Cloning Repos

If you can't clone repositories, you can still exercise the framework using the existing benchmark fixtures:

```bash
bash scripts/test-framework.sh
```

This runs Truss (and any installed competitor tools) against the bundled fixtures, then does the comparison and report generation. Results end up in `test-suite/results-test/` and `test-suite/comparison-test/`.

## Trying Things by Hand

### Truss JSON output

```bash
# One file
./target/release/truss validate --json benchmarks/fixtures/simple.yml

# A bunch of files
./target/release/truss validate --json benchmarks/fixtures/*.yml
```

### Competitor capture scripts

```bash
bash competitors/actionlint/capture.sh benchmarks/fixtures/simple.yml
bash competitors/yamllint/capture.sh benchmarks/fixtures/simple.yml
bash competitors/yaml-language-server/capture.sh benchmarks/fixtures/simple.yml
```

### Comparison and reporting

```bash
python3 scripts/compare-results.py test-suite/results test-suite/comparison/coverage.json
python3 scripts/generate-report.py test-suite/comparison/coverage.json test-suite/comparison/reports
```

## Common Issues

**"just: command not found"** -- Use the bash scripts directly:
- `just test-multifile` becomes `bash scripts/run-full-suite.sh`
- `just test-repo <repo>` becomes `bash scripts/run-validation.sh test-suite/repos/<repo>`
- `just setup-test-repos` becomes `bash scripts/setup-test-repos.sh`

**"No such file or directory" for reports** -- Reports only exist after you run the suite. Either run `bash scripts/run-full-suite.sh` or try `bash scripts/test-framework.sh` first.

**Permission denied** -- Make the scripts executable:

```bash
chmod +x scripts/*.sh
chmod +x competitors/*/capture.sh
```

**Tool not found** -- That's fine. The framework skips tools that aren't installed. If you want full comparison, install the missing ones (see "Before You Start" above).

## What You Should See Afterward

```
test-suite/
├── repos/                    # Cloned repositories (if you ran setup)
├── results/                  # Raw validation results
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
