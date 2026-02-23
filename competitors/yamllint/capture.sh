#!/usr/bin/env bash
# Capture yamllint validation results as JSON
# Usage: capture.sh <yaml-file>

set -euo pipefail

if [ $# -eq 0 ]; then
    echo "{\"error\": \"Usage: $0 <yaml-file>\"}" >&2
    exit 1
fi

YAML_FILE="$1"

if [ ! -f "$YAML_FILE" ]; then
    echo "{\"error\": \"File not found: $YAML_FILE\"}" >&2
    exit 1
fi

# Check if yamllint is available
YAMLLINT_CMD=""
if command -v yamllint &> /dev/null; then
    YAMLLINT_CMD="yamllint"
fi

if [ -z "$YAMLLINT_CMD" ]; then
    # Output valid JSON even on error (use env var to avoid path injection)
    YAML_FILE="$YAML_FILE" python3 -c "
import json, os
result = {
    'file': os.environ['YAML_FILE'],
    'valid': True,
    'diagnostics': [],
    'duration_ms': 0.0,
    'metadata': {'file_size': 0, 'lines': 0},
    'error': 'yamllint not found'
}
print(json.dumps([result], indent=2))
"
    exit 0
fi

# Get file metadata
FILE_SIZE=$(stat -f%z "$YAML_FILE" 2>/dev/null || stat -c%s "$YAML_FILE" 2>/dev/null || echo "0")
LINES=$(wc -l < "$YAML_FILE" 2>/dev/null || echo "0")

# Create minimal config for syntax-only checking
TMP_CONFIG=$(mktemp)
cat > "$TMP_CONFIG" << 'EOF'
extends: default
rules:
  line-length: disable
  document-start: disable
  truthy: disable
  comments: disable
  indentation: disable
  trailing-spaces: disable
  key-ordering: disable
  brackets: disable
  new-line-at-end-of-file: disable
EOF

# Run yamllint and capture output with timing
START_TIME=$(date +%s.%N 2>/dev/null || date +%s)
OUTPUT=$("$YAMLLINT_CMD" -c "$TMP_CONFIG" -f parsable "$YAML_FILE" 2>&1 || true)
EXIT_CODE=$?
END_TIME=$(date +%s.%N 2>/dev/null || date +%s)

# Clean up temp config
rm -f "$TMP_CONFIG"

# Calculate duration in milliseconds
if command -v bc &> /dev/null; then
    DURATION_MS=$(echo "($END_TIME - $START_TIME) * 1000" | bc 2>/dev/null || echo "0")
else
    DURATION_MS="0"
fi

# Parse yamllint output - format: file:line:col: [error|warning] message (rule)
VALID=true
DIAGNOSTICS_JSON="[]"

if [ $EXIT_CODE -ne 0 ] && [ -n "$OUTPUT" ]; then
    VALID=false
    # Use Python to parse yamllint parsable format
    DIAGNOSTICS_JSON=$(python3 -c "
import sys
import re
import json

output = sys.stdin.read()
diagnostics = []

for line in output.strip().split('\n'):
    if not line:
        continue
    # Match: file:line:col: [error|warning] message (rule)
    match = re.match(r'^([^:]+):(\d+):(\d+):\s+\[(error|warning)\]\s+(.+?)(?:\s+\((.+)\))?$', line)
    if match:
        file_path, line_num, col_num, severity, message, rule = match.groups()
        diagnostics.append({
            'message': message.strip(),
            'severity': severity,
            'location': {
                'file': file_path,
                'line': int(line_num),
                'column': int(col_num),
                'column_end': int(col_num)
            },
            'rule': rule if rule else None
        })

print(json.dumps(diagnostics))
" <<< "$OUTPUT" 2>/dev/null || echo "[]")
fi

# Convert VALID to Python boolean
if [ "$VALID" = "true" ]; then
    VALID_PYTHON="True"
else
    VALID_PYTHON="False"
fi

# Output JSON using Python
TMP_SCRIPT=$(mktemp)
export YAML_FILE VALID_PYTHON DIAGNOSTICS_JSON DURATION_MS FILE_SIZE LINES
cat > "$TMP_SCRIPT" << 'PYTHON_EOF'
import json
import sys
import os

result = {
    'file': os.environ['YAML_FILE'],
    'valid': os.environ['VALID_PYTHON'] == 'True',
    'diagnostics': json.loads(os.environ['DIAGNOSTICS_JSON']),
    'duration_ms': float(os.environ['DURATION_MS']),
    'metadata': {
        'file_size': int(os.environ['FILE_SIZE']),
        'lines': int(os.environ['LINES'])
    }
}

print(json.dumps([result], indent=2))
PYTHON_EOF

python3 "$TMP_SCRIPT"
rm -f "$TMP_SCRIPT"

