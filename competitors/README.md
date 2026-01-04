# Competitors

This directory contains wrapper scripts for benchmarking Truss against other YAML/GitHub Actions validators and linters.

## Structure

Each competitor should have its own directory with a `run.sh` script:

```
competitors/
├── competitor-name/
│   └── run.sh          # Wrapper script that validates a YAML file
└── README.md           # This file
```

## Adding a New Competitor

1. Create a new directory: `competitors/your-competitor/`
2. Create a `run.sh` script that:
   - Accepts a YAML file path as the first argument
   - Validates the file using the competitor tool
   - Exits with code 0 on success, non-zero on failure
   - Suppresses output (redirect to `/dev/null`) for benchmarking
   - Handles cases where the tool is not installed

3. Make the script executable: `chmod +x competitors/your-competitor/run.sh`

## Example Wrapper Script

```bash
#!/usr/bin/env bash
set -euo pipefail

if [ $# -eq 0 ]; then
    echo "Usage: $0 <yaml-file>" >&2
    exit 1
fi

YAML_FILE="$1"

if [ ! -f "$YAML_FILE" ]; then
    echo "Error: File not found: $YAML_FILE" >&2
    exit 1
fi

# Check if tool is available and run it
if command -v your-tool &> /dev/null; then
    your-tool "$YAML_FILE" > /dev/null 2>&1
    exit $?
fi

echo "Error: your-tool not found. Please install it." >&2
exit 1
```

## Running Benchmarks

The benchmarking system automatically discovers all competitors:

```bash
just compare        # Compare against all competitors
just compare-smoke  # Quick smoke test with simple fixture
```

Or use the script directly:

```bash
scripts/compare-competitors.sh [fixture-file] [warmup-runs] [output-file]
```

## Current Competitors

- **actionlint**: Static checker for GitHub Actions workflow files
- **yamllint**: YAML linter that checks syntax and style
- **yaml-language-server**: LSP server for YAML validation


