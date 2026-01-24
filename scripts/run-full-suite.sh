#!/usr/bin/env bash
# Run full test suite across all repos and tools
# Usage: run-full-suite.sh [repos-dir] [results-dir] [comparison-dir]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOS_DIR="${1:-$SCRIPT_DIR/../test-suite/repos}"
RESULTS_DIR="${2:-$SCRIPT_DIR/../test-suite/results}"
COMPARISON_DIR="${3:-$SCRIPT_DIR/../test-suite/comparison}"

# Ensure directories exist
mkdir -p "$RESULTS_DIR"/{truss,actionlint,yamllint,yaml-language-server}
mkdir -p "$COMPARISON_DIR"/reports

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Running Full Test Suite ===${NC}"
echo "Repos directory: $REPOS_DIR"
echo "Results directory: $RESULTS_DIR"
echo "Comparison directory: $COMPARISON_DIR"
echo ""

# Check if truss is built
if [ ! -f "$SCRIPT_DIR/../target/release/truss" ]; then
    echo -e "${YELLOW}Warning: Truss not built. Building now...${NC}"
    (cd "$SCRIPT_DIR/.." && cargo build --release)
fi

TRUSS_CMD="$SCRIPT_DIR/../target/release/truss"

# Discover all workflow files
echo -e "${GREEN}Discovering workflow files...${NC}"
WORKFLOW_INVENTORY=$(mktemp)
"$SCRIPT_DIR/discover-workflows.sh" "$REPOS_DIR" > "$WORKFLOW_INVENTORY" || {
    echo -e "${RED}Error: Failed to discover workflows${NC}" >&2
    exit 1
}

WORKFLOW_COUNT=$(python3 -c "import json, sys; data=json.load(sys.stdin); print(len(data))" < "$WORKFLOW_INVENTORY")
echo "Found $WORKFLOW_COUNT workflow files"

if [ "$WORKFLOW_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}No workflow files found. Please clone test repositories first.${NC}"
    echo "Run: $SCRIPT_DIR/setup-test-repos.sh"
    exit 1
fi

# Extract workflow file paths to temp file to avoid shell issues
WORKFLOW_FILES_TMP=$(mktemp)
python3 -c "
import json, sys
data = json.load(sys.stdin)
for item in data:
    print(item['absolute_path'])
" < "$WORKFLOW_INVENTORY" > "$WORKFLOW_FILES_TMP"

# Function to run validation tool
run_tool() {
    local tool=$1
    local file=$2
    local output_file=$3
    
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
        *)
            echo "Unknown tool: $tool" >&2
            return 1
            ;;
    esac
}

# Run Truss on all files
echo -e "\n${GREEN}Running Truss validation...${NC}"
TRUSS_COUNT=0
while IFS= read -r file; do
    [ -z "$file" ] && continue
    if [ ! -f "$file" ]; then
        continue
    fi
    
    # Create safe filename for output
    safe_name=$(echo "$file" | sed 's/[^a-zA-Z0-9]/_/g')
    output_file="$RESULTS_DIR/truss/${safe_name}.json"
    
    run_tool "truss" "$file" "$output_file"
    TRUSS_COUNT=$((TRUSS_COUNT + 1))
    
    if [ $((TRUSS_COUNT % 10)) -eq 0 ]; then
        echo "  Processed $TRUSS_COUNT files..."
    fi
done < "$WORKFLOW_FILES_TMP"
echo "Truss: Validated $TRUSS_COUNT files"

# Run competitors on all files
for tool in actionlint yamllint yaml-language-server; do
    echo -e "\n${GREEN}Running $tool validation...${NC}"
    TOOL_COUNT=0
    
    while IFS= read -r file; do
        [ -z "$file" ] && continue
        if [ ! -f "$file" ]; then
            continue
        fi
        
        safe_name=$(echo "$file" | sed 's/[^a-zA-Z0-9]/_/g')
        output_file="$RESULTS_DIR/$tool/${safe_name}.json"
        
        run_tool "$tool" "$file" "$output_file"
        
        # Verify JSON was written correctly
        if [ -f "$output_file" ] && [ -s "$output_file" ]; then
            if ! python3 -m json.tool "$output_file" > /dev/null 2>&1; then
                echo "    Warning: Invalid JSON in $output_file (first 100 chars):" >&2
                head -c 100 "$output_file" >&2
                echo "" >&2
            fi
        fi
        
        TOOL_COUNT=$((TOOL_COUNT + 1))
        
        if [ $((TOOL_COUNT % 10)) -eq 0 ]; then
            echo "  Processed $TOOL_COUNT files..."
        fi
    done < "$WORKFLOW_FILES_TMP"
    echo "$tool: Validated $TOOL_COUNT files"
done

# Compare results
echo -e "\n${GREEN}Comparing results...${NC}"
COMPARISON_JSON="$COMPARISON_DIR/coverage.json"
"$SCRIPT_DIR/compare-results.py" "$RESULTS_DIR" "$COMPARISON_JSON" || {
    echo -e "${RED}Error: Failed to compare results${NC}" >&2
    exit 1
}

# Generate reports
echo -e "\n${GREEN}Generating reports...${NC}"
"$SCRIPT_DIR/generate-report.py" "$COMPARISON_JSON" "$COMPARISON_DIR/reports" || {
    echo -e "${YELLOW}Warning: Failed to generate reports${NC}" >&2
}

# Print summary
echo -e "\n${GREEN}=== Test Suite Complete ===${NC}"
echo "Results: $RESULTS_DIR"
echo "Comparison: $COMPARISON_JSON"
echo "Reports: $COMPARISON_DIR/reports/"
echo ""
echo "View reports:"
echo "  - Markdown: $COMPARISON_DIR/reports/summary.md"
echo "  - HTML: $COMPARISON_DIR/reports/summary.html"

# Clean up
rm -f "$WORKFLOW_INVENTORY" "$WORKFLOW_FILES_TMP"

