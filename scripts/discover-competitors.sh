#!/usr/bin/env bash
# Discover all available competitors and build hyperfine command arguments
# This script finds all directories in competitors/ that have a run.sh script

set -euo pipefail

COMPETITORS_DIR="${1:-competitors}"
FIXTURE_FILE="${2:-benchmarks/fixtures/complex-dynamic.yml}"

if [ ! -d "$COMPETITORS_DIR" ]; then
    echo "Error: Competitors directory not found: $COMPETITORS_DIR" >&2
    exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
    echo "Error: Fixture file not found: $FIXTURE_FILE" >&2
    exit 1
fi

# Build the list of commands for hyperfine
# Always include truss first
COMMANDS=("./target/release/truss validate --quiet $FIXTURE_FILE")

# Discover all competitors with run.sh scripts
AVAILABLE_COMPETITORS=()
MISSING_COMPETITORS=()

for competitor_dir in "$COMPETITORS_DIR"/*/; do
    if [ ! -d "$competitor_dir" ]; then
        continue
    fi
    
    competitor_name=$(basename "$competitor_dir")
    # Remove trailing slash to avoid double slashes
    competitor_dir="${competitor_dir%/}"
    run_script="$competitor_dir/run.sh"
    
    if [ -f "$run_script" ] && [ -x "$run_script" ]; then
        # Test if the competitor actually works (doesn't error on missing tool)
        # We do a quick dry-run check by running with --help or similar
        # For now, we'll just check if the script exists and is executable
        # The actual validation will happen during benchmarking
        COMMANDS+=("$run_script $FIXTURE_FILE")
        AVAILABLE_COMPETITORS+=("$competitor_name")
    else
        MISSING_COMPETITORS+=("$competitor_name (no run.sh)")
    fi
done

# Print status
echo "Found ${#AVAILABLE_COMPETITORS[@]} competitor(s): ${AVAILABLE_COMPETITORS[*]}" >&2

if [ ${#MISSING_COMPETITORS[@]} -gt 0 ]; then
    echo "Warning: ${#MISSING_COMPETITORS[@]} competitor(s) without run.sh: ${MISSING_COMPETITORS[*]}" >&2
fi

# Output commands as space-separated, properly quoted for hyperfine
# This will be used to build the hyperfine command
for cmd in "${COMMANDS[@]}"; do
    echo "$cmd"
done

