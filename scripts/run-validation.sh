#!/usr/bin/env bash
# Run validation on a single repository
# Usage: run-validation.sh <repo-path> [results-dir]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_PATH="${1:-}"
RESULTS_DIR="${2:-$SCRIPT_DIR/../test-suite/results}"

if [ -z "$REPO_PATH" ]; then
    echo "Usage: $0 <repo-path> [results-dir]" >&2
    exit 1
fi

if [ ! -d "$REPO_PATH" ]; then
    echo "Error: Repository path not found: $REPO_PATH" >&2
    exit 1
fi

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"/{truss,actionlint,yamllint,yaml-language-server}

# Check if truss is built
if [ ! -f "$SCRIPT_DIR/../target/release/truss" ]; then
    echo "Error: Truss not built. Please run 'cargo build --release'" >&2
    exit 1
fi

TRUSS_CMD="$SCRIPT_DIR/../target/release/truss"

# Discover workflow files in this repo
echo "Discovering workflow files in $REPO_PATH..."
WORKFLOW_FILES=$("$SCRIPT_DIR/discover-workflows.sh" "$REPO_PATH" | python3 -c "
import json, sys
data = json.load(sys.stdin)
for item in data:
    print(item['absolute_path'])
")

if [ -z "$WORKFLOW_FILES" ]; then
    echo "No workflow files found in $REPO_PATH"
    exit 0
fi

WORKFLOW_COUNT=$(echo "$WORKFLOW_FILES" | wc -l | tr -d ' ')
echo "Found $WORKFLOW_COUNT workflow files"

# Run validation tools
for tool in truss actionlint yamllint yaml-language-server; do
    echo "Running $tool..."
    TOOL_COUNT=0
    
    for file in $WORKFLOW_FILES; do
        if [ ! -f "$file" ]; then
            continue
        fi
        
        safe_name=$(echo "$file" | sed 's/[^a-zA-Z0-9]/_/g')
        output_file="$RESULTS_DIR/$tool/${safe_name}.json"
        
        case "$tool" in
            truss)
                "$TRUSS_CMD" validate --json --quiet "$file" > "$output_file" 2>&1 || true
                ;;
            actionlint)
                "$SCRIPT_DIR/../competitors/actionlint/capture.sh" "$file" > "$output_file" 2>&1 || true
                ;;
            yamllint)
                "$SCRIPT_DIR/../competitors/yamllint/capture.sh" "$file" > "$output_file" 2>&1 || true
                ;;
            yaml-language-server)
                "$SCRIPT_DIR/../competitors/yaml-language-server/capture.sh" "$file" > "$output_file" 2>&1 || true
                ;;
        esac
        
        TOOL_COUNT=$((TOOL_COUNT + 1))
    done
    
    echo "$tool: Validated $TOOL_COUNT files"
done

echo "Validation complete. Results in: $RESULTS_DIR"

