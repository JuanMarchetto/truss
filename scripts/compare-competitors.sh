#!/usr/bin/env bash
# Compare Truss against all available competitors using hyperfine
# Automatically discovers all competitors in competitors/ directory

set -euo pipefail

FIXTURE_FILE="${1:-benchmarks/fixtures/complex-dynamic.yml}"
WARMUP="${2:-5}"
OUTPUT_FILE="${3:-benchmarks/hyperfine/compare.md}"

# Ensure truss is built
if [ ! -f "./target/release/truss" ]; then
    echo "Error: Truss not built. Please run 'just build' or 'cargo build --release'" >&2
    exit 1
fi

# Discover all competitors
COMPETITORS_DIR="competitors"
if [ ! -d "$COMPETITORS_DIR" ]; then
    echo "Error: Competitors directory not found: $COMPETITORS_DIR" >&2
    exit 1
fi

# Build command list
COMMANDS=()
COMMANDS+=("./target/release/truss validate --quiet $FIXTURE_FILE")

# Discover all competitors with run.sh scripts
AVAILABLE_COMPETITORS=()
for competitor_dir in "$COMPETITORS_DIR"/*/; do
    if [ ! -d "$competitor_dir" ]; then
        continue
    fi
    
    competitor_name=$(basename "$competitor_dir")
    # Remove trailing slash to avoid double slashes
    competitor_dir="${competitor_dir%/}"
    run_script="$competitor_dir/run.sh"
    
    if [ -f "$run_script" ] && [ -x "$run_script" ]; then
        COMMANDS+=("$run_script $FIXTURE_FILE")
        AVAILABLE_COMPETITORS+=("$competitor_name")
    fi
done

# Check if we have any competitors
if [ ${#AVAILABLE_COMPETITORS[@]} -eq 0 ]; then
    echo "Warning: No competitors found in $COMPETITORS_DIR/" >&2
    echo "Only benchmarking Truss itself." >&2
fi

# Print status
echo "Comparing Truss against ${#AVAILABLE_COMPETITORS[@]} competitor(s): ${AVAILABLE_COMPETITORS[*]}" >&2
echo "Fixture: $FIXTURE_FILE" >&2
echo "Output: $OUTPUT_FILE" >&2
echo "" >&2

# Ensure output directory exists
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Build and run hyperfine command
# We need to properly quote each command for hyperfine
HYPERFINE_ARGS=(
    "--warmup" "$WARMUP"
    "--export-markdown" "$OUTPUT_FILE"
)

# Add each command as a separate argument to hyperfine
for cmd in "${COMMANDS[@]}"; do
    HYPERFINE_ARGS+=("$cmd")
done

# Run hyperfine
hyperfine "${HYPERFINE_ARGS[@]}"

