#!/usr/bin/env bash
# Wrapper script for yamllint validation
# yamllint is a YAML linter that checks syntax and style

set -euo pipefail

if [ $# -eq 0 ]; then
    echo "Usage: $0 <yaml-file>" >&2
    exit 1
fi

YAML_FILE="$1"

if [ ! -f "$YAML_FILE" ]; then
    echo "Error: File not found: $YAML_FILE" >&2
    exit 1
fi

# Check if yamllint is available
if command -v yamllint &> /dev/null; then
    # Create a temporary config that disables style checks
    # This ensures we're only checking syntax for fair performance comparison
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
    # Run yamllint with the minimal config (only syntax errors will be reported)
    yamllint -c "$TMP_CONFIG" "$YAML_FILE" > /dev/null 2>&1
    EXIT_CODE=$?
    rm -f "$TMP_CONFIG"
    exit $EXIT_CODE
fi

# Fallback to Python's yaml module if yamllint is not available
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml" 2>/dev/null; then
        python3 -c "
import yaml
import sys
try:
    with open('$YAML_FILE', 'r') as f:
        yaml.safe_load(f)
    sys.exit(0)
except yaml.YAMLError:
    sys.exit(1)
except Exception:
    sys.exit(1)
" 2>/dev/null
        exit $?
    fi
fi

echo "Error: yamllint not found. Please install it:" >&2
echo "  pip install yamllint" >&2
echo "  or: brew install yamllint" >&2
echo "  or: apt-get install yamllint" >&2
exit 1


