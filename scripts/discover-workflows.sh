#!/usr/bin/env bash
# Discover all .github/workflows/*.yml files in a repository
# Usage: discover-workflows.sh <repo-path> [output-json-file]

set -euo pipefail

REPO_PATH="${1:-}"
OUTPUT_FILE="${2:-}"

if [ -z "$REPO_PATH" ]; then
    echo "Usage: $0 <repo-path> [output-json-file]" >&2
    exit 1
fi

if [ ! -d "$REPO_PATH" ]; then
    echo "Error: Repository path not found: $REPO_PATH" >&2
    exit 1
fi

# Find all workflow files and write to temp file to avoid shell escaping issues
TMP_FILE=$(mktemp)
find "$REPO_PATH" -type f \( -name "*.yml" -o -name "*.yaml" \) -path "*/.github/workflows/*" 2>/dev/null > "$TMP_FILE" || true

if [ ! -s "$TMP_FILE" ]; then
    echo "[]" | python3 -m json.tool
    rm -f "$TMP_FILE"
    exit 0
fi

# Build JSON inventory using Python
python3 << EOF
import json
import os
import sys
from pathlib import Path

repo_path = "$REPO_PATH"
tmp_file = "$TMP_FILE"

# Read workflow files from temp file
workflow_files = []
if os.path.exists(tmp_file):
    with open(tmp_file, 'r') as f:
        workflow_files = [line.strip() for line in f if line.strip()]

inventory = []

for file_path in workflow_files:
    if not file_path or not os.path.exists(file_path):
        continue
    
    try:
        # Get relative path from repo root
        rel_path = os.path.relpath(file_path, repo_path)
        
        # Get file stats
        stat = os.stat(file_path)
        file_size = stat.st_size
        
        # Count lines
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            lines = len(f.readlines())
        
        # Basic complexity metrics (count jobs, steps)
        jobs = 0
        steps = 0
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                # Simple pattern matching for jobs and steps
                jobs = content.count('  [a-zA-Z_][a-zA-Z0-9_-]*:') + content.count('\n  [a-zA-Z_][a-zA-Z0-9_-]*:')
                steps = content.count('- name:') + content.count('- uses:') + content.count('- run:')
        except:
            pass
        
        inventory.append({
            "path": rel_path,
            "absolute_path": file_path,
            "file_size": file_size,
            "lines": lines,
            "complexity": {
                "estimated_jobs": max(0, jobs - 1),  # Subtract 1 for root level
                "estimated_steps": steps
            }
        })
    except Exception as e:
        # Skip files that can't be processed
        continue

# Output JSON
output = json.dumps(inventory, indent=2)
output_file = "$OUTPUT_FILE"
if output_file:
    with open(output_file, 'w') as f:
        f.write(output)
else:
    print(output)

# Clean up temp file
if os.path.exists(tmp_file):
    os.remove(tmp_file)
EOF

# Clean up temp file if Python didn't
rm -f "$TMP_FILE"

