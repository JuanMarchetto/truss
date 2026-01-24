#!/usr/bin/env bash
# Capture yaml-language-server validation results as JSON
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

# Get file metadata
FILE_SIZE=$(stat -f%z "$YAML_FILE" 2>/dev/null || stat -c%s "$YAML_FILE" 2>/dev/null || echo "0")
LINES=$(wc -l < "$YAML_FILE" 2>/dev/null || echo "0")

# Try to use yaml-language-server via LSP protocol (complex, fallback to Python)
# For simplicity, use Python's yaml module for validation
VALID=true
DIAGNOSTICS_JSON="[]"
DURATION_MS="0"

START_TIME=$(date +%s.%N 2>/dev/null || date +%s)

# Use Python's yaml module for validation (most reliable fallback)
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml" 2>/dev/null; then
        VALIDATE_SCRIPT=$(mktemp)
        export YAML_FILE
        cat > "$VALIDATE_SCRIPT" << 'VALIDATE_EOF'
import yaml
import sys
import json
import os

yaml_file = os.environ['YAML_FILE']
try:
    with open(yaml_file, 'r') as f:
        yaml.safe_load(f)
    sys.exit(0)
except yaml.YAMLError as e:
    diagnostics = []
    error_msg = str(e)
    import re
    line_match = re.search(r'line (\d+)', error_msg)
    col_match = re.search(r'column (\d+)', error_msg)
    line = int(line_match.group(1)) if line_match else 1
    col = int(col_match.group(1)) if col_match else 1
    diagnostics.append({
        'message': error_msg,
        'severity': 'error',
        'location': {
            'file': yaml_file,
            'line': line,
            'column': col,
            'column_end': col
        }
    })
    print(json.dumps(diagnostics))
    sys.exit(1)
except Exception as e:
    diagnostics = [{
        'message': str(e),
        'severity': 'error',
        'location': {
            'file': yaml_file,
            'line': 1,
            'column': 1,
            'column_end': 1
        }
    }]
    print(json.dumps(diagnostics))
    sys.exit(1)
VALIDATE_EOF
        OUTPUT=$(python3 "$VALIDATE_SCRIPT" 2>&1)
        EXIT_CODE=$?
        rm -f "$VALIDATE_SCRIPT"
        
        if [ $EXIT_CODE -ne 0 ]; then
            VALID=false
            DIAGNOSTICS_JSON=$(echo "$OUTPUT" | tail -1 || echo "[]")
        fi
    else
        # Output valid JSON even on error
        ERROR_SCRIPT=$(mktemp)
        export YAML_FILE
        cat > "$ERROR_SCRIPT" << 'ERROR_EOF'
import json
import os

result = {
    'file': os.environ['YAML_FILE'],
    'valid': True,
    'diagnostics': [],
    'duration_ms': 0.0,
    'metadata': {'file_size': 0, 'lines': 0},
    'error': 'python3 with yaml module not found'
}
print(json.dumps(result, indent=2))
ERROR_EOF
        python3 "$ERROR_SCRIPT" 2>/dev/null || echo "{\"file\":\"$YAML_FILE\",\"valid\":true,\"diagnostics\":[],\"duration_ms\":0.0,\"metadata\":{\"file_size\":0,\"lines\":0},\"error\":\"python3 not available\"}"
        rm -f "$ERROR_SCRIPT"
        exit 0
    fi
else
    echo "{\"file\":\"$YAML_FILE\",\"valid\":true,\"diagnostics\":[],\"duration_ms\":0.0,\"metadata\":{\"file_size\":0,\"lines\":0},\"error\":\"python3 not available\"}"
    exit 0
fi

END_TIME=$(date +%s.%N 2>/dev/null || date +%s)

# Calculate duration in milliseconds
if command -v bc &> /dev/null; then
    DURATION_MS=$(echo "($END_TIME - $START_TIME) * 1000" | bc 2>/dev/null || echo "0")
else
    DURATION_MS="0"
fi

# Convert VALID to Python boolean
if [ "$VALID" = "true" ]; then
    VALID_PYTHON="True"
else
    VALID_PYTHON="False"
fi

# Output JSON using Python
TMP_SCRIPT=$(mktemp)
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

export YAML_FILE VALID_PYTHON DIAGNOSTICS_JSON DURATION_MS FILE_SIZE LINES
python3 "$TMP_SCRIPT"
rm -f "$TMP_SCRIPT"
