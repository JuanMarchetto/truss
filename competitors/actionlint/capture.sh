#!/usr/bin/env bash
# Capture actionlint validation results as JSON
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

# Find actionlint
ACTIONLINT_CMD=""
if command -v actionlint &> /dev/null; then
    ACTIONLINT_CMD="actionlint"
elif [ -n "${GOPATH:-}" ] && [ -f "${GOPATH}/bin/actionlint" ]; then
    ACTIONLINT_CMD="${GOPATH}/bin/actionlint"
elif [ -n "${GOBIN:-}" ] && [ -f "${GOBIN}/actionlint" ]; then
    ACTIONLINT_CMD="${GOBIN}/actionlint"
fi

if [ -z "$ACTIONLINT_CMD" ]; then
    # Output valid JSON even on error
    python3 -c "
import json
result = {
    'file': '$YAML_FILE',
    'valid': True,
    'diagnostics': [],
    'duration_ms': 0.0,
    'metadata': {'file_size': 0, 'lines': 0},
    'error': 'actionlint not found'
}
print(json.dumps(result, indent=2))
"
    exit 0
fi

# Get file metadata
FILE_SIZE=$(stat -f%z "$YAML_FILE" 2>/dev/null || stat -c%s "$YAML_FILE" 2>/dev/null || echo "0")
LINES=$(wc -l < "$YAML_FILE" 2>/dev/null || echo "0")

# Run actionlint and capture output with timing
START_TIME=$(date +%s.%N 2>/dev/null || date +%s)
OUTPUT=$("$ACTIONLINT_CMD" -no-color "$YAML_FILE" 2>&1 || true)
EXIT_CODE=$?
END_TIME=$(date +%s.%N 2>/dev/null || date +%s)

# Calculate duration in milliseconds
if command -v bc &> /dev/null; then
    DURATION_MS=$(echo "($END_TIME - $START_TIME) * 1000" | bc 2>/dev/null || echo "0")
else
    DURATION_MS="0"
fi

# Parse actionlint output - format: file:line:col: message
VALID=true
DIAGNOSTICS_JSON="[]"

if [ $EXIT_CODE -ne 0 ] && [ -n "$OUTPUT" ]; then
    VALID=false
    # Use Python to parse and convert to JSON (more reliable than jq for complex parsing)
    DIAGNOSTICS_JSON=$(python3 -c "
import sys
import re
import json

output = sys.stdin.read()
diagnostics = []

for line in output.strip().split('\n'):
    if not line:
        continue
    # Match: file:line:col: message or file:line:col:col2: message
    match = re.match(r'^([^:]+):(\d+):(\d+)(?::(\d+))?: (.+)$', line)
    if match:
        file_path, line_num, col_start, col_end, message = match.groups()
        diagnostics.append({
            'message': message.strip(),
            'severity': 'error',
            'location': {
                'file': file_path,
                'line': int(line_num),
                'column': int(col_start),
                'column_end': int(col_end) if col_end else int(col_start)
            }
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

# Output JSON using Python for better compatibility
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

print(json.dumps(result, indent=2))
PYTHON_EOF

python3 "$TMP_SCRIPT"
rm -f "$TMP_SCRIPT"
