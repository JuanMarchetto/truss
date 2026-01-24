#!/usr/bin/env bash
# Debug script to check what results are being saved
# Usage: debug-results.sh [results-dir]

set -euo pipefail

RESULTS_DIR="${1:-test-suite/results}"

if [ ! -d "$RESULTS_DIR" ]; then
    echo "Error: Results directory not found: $RESULTS_DIR" >&2
    exit 1
fi

echo "Checking results in: $RESULTS_DIR"
echo ""

for tool in truss actionlint yamllint yaml-language-server; do
    tool_dir="$RESULTS_DIR/$tool"
    
    if [ ! -d "$tool_dir" ]; then
        echo "❌ $tool: Directory not found"
        continue
    fi
    
    file_count=$(find "$tool_dir" -name "*.json" -type f | wc -l | tr -d ' ')
    echo "📊 $tool: $file_count JSON files"
    
    if [ "$file_count" -eq 0 ]; then
        echo "   ⚠️  No result files found"
        continue
    fi
    
    # Check a few files
    valid_count=0
    error_count=0
    sample_file=""
    
    for file in "$tool_dir"/*.json; do
        [ ! -f "$file" ] && continue
        [ -z "$sample_file" ] && sample_file="$file"
        
        if python3 -m json.tool "$file" > /dev/null 2>&1; then
            valid_count=$((valid_count + 1))
            
            # Check if it has the expected structure
            if python3 -c "
import json, sys
with open('$file') as f:
    data = json.load(f)
    if 'error' in data:
        sys.exit(1)
    if 'file' not in data:
        sys.exit(1)
" 2>/dev/null; then
                : # Valid structure
            else
                error_count=$((error_count + 1))
            fi
        else
            error_count=$((error_count + 1))
        fi
    done
    
    echo "   ✓ Valid JSON: $valid_count"
    if [ "$error_count" -gt 0 ]; then
        echo "   ✗ Invalid/Error files: $error_count"
    fi
    
    # Show sample file content
    if [ -n "$sample_file" ] && [ -f "$sample_file" ]; then
        echo "   Sample file: $(basename "$sample_file")"
        echo "   Content preview:"
        head -5 "$sample_file" | sed 's/^/      /'
        echo ""
    fi
done

echo ""
echo "To check specific tool results:"
echo "  ls -lh $RESULTS_DIR/<tool>/"
echo "  cat $RESULTS_DIR/<tool>/<file>.json | jq"

