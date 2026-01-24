#!/usr/bin/env bash
# Manage test repository lifecycle
# Usage: manage-repos.sh <command> [args...]
# Commands: clone, update, list, clean

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOS_DIR="${REPOS_DIR:-$SCRIPT_DIR/../test-suite/repos}"

command="${1:-}"

case "$command" in
    clone)
        REPO_URL="${2:-}"
        if [ -z "$REPO_URL" ]; then
            echo "Usage: $0 clone <repo-url-or-owner/repo>" >&2
            exit 1
        fi
        
        # Convert owner/repo to URL if needed
        if [[ ! "$REPO_URL" =~ ^https?:// ]]; then
            REPO_URL="https://github.com/$REPO_URL.git"
        fi
        
        REPO_NAME=$(basename "$REPO_URL" .git)
        REPO_PATH="$REPOS_DIR/$REPO_NAME"
        
        if [ -d "$REPO_PATH" ]; then
            echo "Repository already exists: $REPO_PATH" >&2
            echo "Use 'update' command to update it" >&2
            exit 1
        fi
        
        mkdir -p "$REPOS_DIR"
        echo "Cloning $REPO_URL to $REPO_PATH..."
        git clone --depth 1 "$REPO_URL" "$REPO_PATH" || {
            echo "Error: Failed to clone repository" >&2
            exit 1
        }
        echo "Cloned successfully"
        ;;
    
    update)
        REPO_NAME="${2:-}"
        if [ -z "$REPO_NAME" ]; then
            echo "Usage: $0 update <repo-name>" >&2
            exit 1
        fi
        
        REPO_PATH="$REPOS_DIR/$REPO_NAME"
        if [ ! -d "$REPO_PATH" ]; then
            echo "Error: Repository not found: $REPO_PATH" >&2
            exit 1
        fi
        
        echo "Updating $REPO_NAME..."
        (cd "$REPO_PATH" && git pull --ff-only) || {
            echo "Warning: Failed to update repository (may need manual intervention)" >&2
        }
        ;;
    
    list)
        if [ ! -d "$REPOS_DIR" ]; then
            echo "No repositories directory found"
            exit 0
        fi
        
        echo "Repositories in $REPOS_DIR:"
        for repo in "$REPOS_DIR"/*; do
            if [ -d "$repo" ] && [ -d "$repo/.git" ]; then
                REPO_NAME=$(basename "$repo")
                WORKFLOW_COUNT=$(find "$repo" -type f \( -name "*.yml" -o -name "*.yaml" \) -path "*/.github/workflows/*" 2>/dev/null | wc -l | tr -d ' ')
                echo "  - $REPO_NAME ($WORKFLOW_COUNT workflow files)"
            fi
        done
        ;;
    
    clean)
        DAYS="${2:-30}"
        if [ ! -d "$REPOS_DIR" ]; then
            echo "No repositories directory found"
            exit 0
        fi
        
        echo "Cleaning repositories older than $DAYS days..."
        find "$REPOS_DIR" -type d -name ".git" -prune -o -type d -mtime +$DAYS -print | while read dir; do
            if [ -d "$dir/.git" ]; then
                REPO_NAME=$(basename "$dir")
                echo "Removing $REPO_NAME..."
                rm -rf "$dir"
            fi
        done
        ;;
    
    *)
        echo "Usage: $0 <command> [args...]" >&2
        echo "" >&2
        echo "Commands:" >&2
        echo "  clone <repo-url-or-owner/repo>  Clone a repository" >&2
        echo "  update <repo-name>               Update a cloned repository" >&2
        echo "  list                            List all cloned repositories" >&2
        echo "  clean [days]                    Remove repositories older than N days (default: 30)" >&2
        exit 1
        ;;
esac

