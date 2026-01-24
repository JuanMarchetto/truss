#!/usr/bin/env bash
# Setup test repositories from repos.json
# Usage: setup-test-repos.sh [priority-filter]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOS_JSON="$SCRIPT_DIR/../test-suite/repos.json"
REPOS_DIR="$SCRIPT_DIR/../test-suite/repos"
PRIORITY_FILTER="${1:-}"

if [ ! -f "$REPOS_JSON" ]; then
    echo "Error: repos.json not found: $REPOS_JSON" >&2
    exit 1
fi

mkdir -p "$REPOS_DIR"

# Parse repos.json and clone
python3 << EOF
import json
import subprocess
import sys
import os

with open("$REPOS_JSON", 'r') as f:
    repos = json.load(f)

priority_filter = "$PRIORITY_FILTER"
repos_dir = "$REPOS_DIR"

cloned = 0
skipped = 0
failed = 0

for repo in repos:
    name = repo['name']
    url = repo['url']
    description = repo['description']
    priority = repo.get('priority', 'medium')
    
    # Filter by priority if specified
    if priority_filter and priority != priority_filter:
        continue
    
    repo_path = os.path.join(repos_dir, name)
    
    if os.path.exists(repo_path):
        print(f"⏭️  Skipping {name} (already exists)")
        skipped += 1
        continue
    
    print(f"📦 Cloning {name}...")
    print(f"   {description}")
    
    try:
        subprocess.run(
            ['git', 'clone', '--depth', '1', url, repo_path],
            check=True,
            capture_output=True
        )
        print(f"✅ Cloned {name}")
        cloned += 1
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to clone {name}: {e}")
        failed += 1

print(f"\nSummary: {cloned} cloned, {skipped} skipped, {failed} failed")
EOF

