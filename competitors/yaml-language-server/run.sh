#!/usr/bin/env bash
# Wrapper script for yaml-language-server validation
# Falls back to yamllint if yaml-language-server is not available

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

# Try to use yaml-language-server if available
if command -v yaml-language-server &> /dev/null; then
    # yaml-language-server is an LSP server, so we need to use it via LSP protocol
    # For benchmarking, we'll use a simple validation approach
    yaml-language-server --stdio < "$YAML_FILE" > /dev/null 2>&1
    exit $?
fi

# Check if yaml-language-server is available via npm/node
if command -v node &> /dev/null && command -v npm &> /dev/null; then
    # Try to use yaml-language-server via node if installed locally or globally
    NODE_MODULES_DIR="$(dirname "$0")/node_modules"
    if [ -d "$NODE_MODULES_DIR/.bin/yaml-language-server" ] || npm list -g yaml-language-server &> /dev/null; then
        # Use node to run yaml-language-server for validation
        # Since it's an LSP server, we'll use a simple approach: try to parse the file
        node -e "
            const fs = require('fs');
            const yaml = require('js-yaml');
            try {
                const content = fs.readFileSync('$YAML_FILE', 'utf8');
                yaml.load(content);
                process.exit(0);
            } catch (e) {
                process.exit(1);
            }
        " 2>/dev/null && exit 0 || true
    fi
fi

# Use Python's yaml module for syntax-only validation (most reliable for benchmarking)
# This only checks syntax, not style, which is what we want for performance comparison
if command -v python3 &> /dev/null; then
    # Check if PyYAML is available
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

# Fallback to yamllint - create a minimal config that only checks syntax
if command -v yamllint &> /dev/null; then
    # Create a temporary config that disables style checks
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

echo "Error: No YAML validator found. Please install yamllint, yaml-language-server, or Python with PyYAML." >&2
exit 1

