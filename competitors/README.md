# Competitors

Wrapper scripts for benchmarking Truss against other YAML and GitHub Actions validators.

## Layout

```
competitors/
├── competitor-name/
│   ├── run.sh          # Performance benchmarking wrapper
│   └── capture.sh      # Outputs JSON results for comparison
└── README.md
```

## Adding a New Competitor

1. Create a directory: `competitors/your-tool/`

2. Add a `run.sh` that takes a YAML file path as its first argument, validates it with the tool, and exits 0/non-zero. Suppress output for clean benchmarking. Handle the case where the tool isn't installed.

3. Add a `capture.sh` that takes a YAML file path, runs the tool, and outputs structured JSON (timing, diagnostics, metadata). Look at the existing `capture.sh` scripts for the expected format.

4. Make them executable: `chmod +x competitors/your-tool/*.sh`

Here's a minimal `run.sh` template:

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

if command -v your-tool &> /dev/null; then
    your-tool "$YAML_FILE" > /dev/null 2>&1
    exit $?
fi

echo "Error: your-tool not found. Please install it." >&2
exit 1
```

## Running Benchmarks

The benchmarking system auto-discovers all competitors:

```bash
just compare        # Compare against all competitors
just compare-smoke  # Quick smoke test with simple fixture
```

Or call the script directly:

```bash
scripts/compare-competitors.sh [fixture-file] [warmup-runs] [output-file]
```

## Current Competitors

- **actionlint** -- Static checker for GitHub Actions workflow files
- **yamllint** -- YAML linter for syntax and style
- **yaml-language-server** -- LSP server for YAML validation

Each has a `run.sh` for benchmarking and a `capture.sh` for structured result capture.

## JSON Output Format

The `capture.sh` scripts produce JSON like this:

```json
{
  "file": "path/to/workflow.yml",
  "valid": false,
  "diagnostics": [
    {
      "message": "Error message",
      "severity": "error",
      "location": {
        "file": "path/to/workflow.yml",
        "line": 10,
        "column": 5,
        "column_end": 10
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

The comparison engine in `scripts/compare-results.py` consumes this format to analyze results across tools.
