#!/usr/bin/env bash
# Wrapper script for actionlint validation
# actionlint is a static checker for GitHub Actions workflow files

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

# Check if actionlint is available
if command -v actionlint &> /dev/null; then
    # actionlint validates GitHub Actions workflows
    # Use -no-color to avoid terminal escape sequences in output
    # Redirect stderr to stdout and then to /dev/null to suppress output
    actionlint -no-color "$YAML_FILE" > /dev/null 2>&1
    exit $?
fi

# Check if actionlint is available via go install
if command -v go &> /dev/null; then
    # Try to find actionlint in GOPATH/bin or GOBIN
    if [ -n "${GOPATH:-}" ] && [ -f "${GOPATH}/bin/actionlint" ]; then
        "${GOPATH}/bin/actionlint" -no-color "$YAML_FILE" > /dev/null 2>&1
        exit $?
    fi
    if [ -n "${GOBIN:-}" ] && [ -f "${GOBIN}/actionlint" ]; then
        "${GOBIN}/actionlint" -no-color "$YAML_FILE" > /dev/null 2>&1
        exit $?
    fi
fi

echo "Error: actionlint not found. Please install it:" >&2
echo "  brew install actionlint" >&2
echo "  or: go install github.com/rhymond/actionlint@latest" >&2
echo "  or: https://github.com/rhymond/actionlint" >&2
exit 1

