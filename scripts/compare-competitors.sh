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
# Only include competitors that are actually available (tools installed)
AVAILABLE_COMPETITORS=()
for competitor_dir in "$COMPETITORS_DIR"/*/; do
    if [ ! -d "$competitor_dir" ]; then
        continue
    fi
    
    competitor_name=$(basename "$competitor_dir")
    # Skip custom-scripts directory
    if [ "$competitor_name" = "custom-scripts" ]; then
        continue
    fi
    
    # Remove trailing slash to avoid double slashes
    competitor_dir="${competitor_dir%/}"
    run_script="$competitor_dir/run.sh"
    
    if [ -f "$run_script" ] && [ -x "$run_script" ]; then
        # Test if the competitor tool is actually available
        # Run the script and capture both stdout and stderr
        # If it outputs "not found" or "Error:" about missing tool, skip it
        # Use a subshell to avoid affecting the main script's error handling
        TEST_OUTPUT=$("$run_script" "$FIXTURE_FILE" 2>&1 || true)
        
        # Check if the output indicates the tool is not installed
        if echo "$TEST_OUTPUT" | grep -qiE "(not found|Error:.*not found|Please install|No .* found)" >/dev/null 2>&1; then
            echo "Skipping $competitor_name: tool not installed" >&2
            continue
        fi
        
        # Tool is available (either it validated successfully or failed validation, but tool exists)
        # Exit code 0 = success, exit code 1 = validation failed (but tool exists)
        # Both are fine for benchmarking
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
    "--ignore-failure"  # Ignore failures from competitors (some may fail validation but still be benchmarkable)
    "--shell=none"       # More accurate timing without shell overhead
)

# Add each command as a separate argument to hyperfine
for cmd in "${COMMANDS[@]}"; do
    HYPERFINE_ARGS+=("$cmd")
done

# Run hyperfine
hyperfine "${HYPERFINE_ARGS[@]}"

