#!/usr/bin/env bash
# Quick test of the framework using existing benchmark fixtures
# This doesn't require cloning repositories

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/../benchmarks/fixtures"
TEST_RESULTS_DIR="$SCRIPT_DIR/../test-suite/results-test"
TEST_COMPARISON_DIR="$SCRIPT_DIR/../test-suite/comparison-test"

echo "Testing framework with existing fixtures..."
echo "Fixtures: $FIXTURES_DIR"
echo "Results: $TEST_RESULTS_DIR"
echo ""

# Ensure directories exist
mkdir -p "$TEST_RESULTS_DIR"/{truss,actionlint,yamllint,yaml-language-server}
mkdir -p "$TEST_COMPARISON_DIR"/reports

# Check if truss is built
if [ ! -f "$SCRIPT_DIR/../target/release/truss" ]; then
    echo "Building Truss..."
    (cd "$SCRIPT_DIR/.." && cargo build --release)
fi

TRUSS_CMD="$SCRIPT_DIR/../target/release/truss"

# Find fixture files
FIXTURE_FILES=$(find "$FIXTURES_DIR" -name "*.yml" -o -name "*.yaml" 2>/dev/null | grep -v readme || true)

if [ -z "$FIXTURE_FILES" ]; then
    echo "No fixture files found in $FIXTURES_DIR"
    exit 1
fi

echo "Found fixture files:"
echo "$FIXTURE_FILES" | while read -r file; do
    echo "  - $file"
done
echo ""

# Test Truss JSON output
echo "Testing Truss JSON output..."
for file in $FIXTURE_FILES; do
    safe_name=$(basename "$file" | sed 's/[^a-zA-Z0-9]/_/g')
    output_file="$TEST_RESULTS_DIR/truss/${safe_name}.json"
    
    echo "  Validating: $file"
    "$TRUSS_CMD" validate --json --quiet "$file" > "$output_file" 2>&1 || true
    
    # Verify JSON is valid
    if python3 -m json.tool "$output_file" > /dev/null 2>&1; then
        echo "    ✓ Valid JSON"
    else
        echo "    ✗ Invalid JSON"
        cat "$output_file"
    fi
done

# Test competitor capture scripts (if tools are available)
for tool in actionlint yamllint yaml-language-server; do
    echo ""
    echo "Testing $tool capture script..."
    
    # Check if tool/capture script exists
    if [ ! -f "$SCRIPT_DIR/../competitors/$tool/capture.sh" ]; then
        echo "  ✗ capture.sh not found"
        continue
    fi
    
    # Test on first fixture
    first_file=$(echo "$FIXTURE_FILES" | head -1)
    if [ -n "$first_file" ]; then
        safe_name=$(basename "$first_file" | sed 's/[^a-zA-Z0-9]/_/g')
        output_file="$TEST_RESULTS_DIR/$tool/${safe_name}.json"
        
        echo "  Testing on: $first_file"
        if bash "$SCRIPT_DIR/../competitors/$tool/capture.sh" "$first_file" > "$output_file" 2>&1; then
            # Verify JSON is valid
            if python3 -m json.tool "$output_file" > /dev/null 2>&1; then
                echo "    ✓ Valid JSON output"
            else
                echo "    ✗ Invalid JSON output"
                cat "$output_file" | head -20
            fi
        else
            echo "    ⚠ Tool not available or error occurred"
            cat "$output_file" | head -5
        fi
    fi
done

# Test comparison script if we have results
echo ""
echo "Testing comparison script..."
if [ -n "$(find "$TEST_RESULTS_DIR/truss" -name "*.json" 2>/dev/null)" ]; then
    COMPARISON_JSON="$TEST_COMPARISON_DIR/coverage.json"
    if python3 "$SCRIPT_DIR/compare-results.py" "$TEST_RESULTS_DIR" "$COMPARISON_JSON" 2>&1; then
        echo "  ✓ Comparison completed"
        echo "  Results: $COMPARISON_JSON"
        
        # Show summary
        if [ -f "$COMPARISON_JSON" ]; then
            echo ""
            echo "Summary:"
            python3 -c "
import json, sys
with open('$COMPARISON_JSON') as f:
    data = json.load(f)
    summary = data.get('summary', {})
    print(f\"  Files analyzed: {summary.get('total_files', 0)}\")
    print(f\"  Truss errors: {summary.get('total_errors_truss', 0)}\")
    print(f\"  Truss avg time: {summary.get('avg_time_truss_ms', 0.0):.2f}ms\")
"
        fi
    else
        echo "  ✗ Comparison failed"
    fi
else
    echo "  ⚠ No results to compare"
fi

# Test report generation
echo ""
echo "Testing report generation..."
if [ -f "$TEST_COMPARISON_DIR/coverage.json" ]; then
    if python3 "$SCRIPT_DIR/generate-report.py" "$TEST_COMPARISON_DIR/coverage.json" "$TEST_COMPARISON_DIR/reports" 2>&1; then
        echo "  ✓ Reports generated"
        if [ -f "$TEST_COMPARISON_DIR/reports/summary.md" ]; then
            echo "  Markdown: $TEST_COMPARISON_DIR/reports/summary.md"
        fi
        if [ -f "$TEST_COMPARISON_DIR/reports/summary.html" ]; then
            echo "  HTML: $TEST_COMPARISON_DIR/reports/summary.html"
        fi
    else
        echo "  ✗ Report generation failed"
    fi
else
    echo "  ⚠ No comparison data to generate reports from"
fi

echo ""
echo "Framework test complete!"
echo "Test results: $TEST_RESULTS_DIR"
echo "Test reports: $TEST_COMPARISON_DIR/reports/"

