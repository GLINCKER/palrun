#!/bin/bash
# Git hooks installation script for Palrun
#
# This script installs the git hooks from .githooks/ to .git/hooks/
# Run this once after cloning the repository.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Installing Palrun git hooks...${NC}"
echo ""

# Ensure .git/hooks directory exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo -e "${RED}Error: .git/hooks directory not found${NC}"
    echo "Are you in a git repository?"
    exit 1
fi

# List of hooks to install
HOOKS=("pre-commit" "pre-push" "commit-msg")

for hook in "${HOOKS[@]}"; do
    src="$SCRIPT_DIR/$hook"
    dst="$GIT_HOOKS_DIR/$hook"

    if [ -f "$src" ]; then
        # Backup existing hook if it exists and is not a symlink to ours
        if [ -f "$dst" ] && [ ! -L "$dst" ]; then
            echo -e "${YELLOW}Backing up existing $hook hook to $hook.backup${NC}"
            mv "$dst" "$dst.backup"
        fi

        # Create symlink
        ln -sf "$src" "$dst"
        chmod +x "$src"
        echo -e "${GREEN}✓${NC} Installed $hook"
    else
        echo -e "${YELLOW}⚠${NC} $hook hook not found in .githooks/"
    fi
done

echo ""
echo -e "${GREEN}Git hooks installed successfully!${NC}"
echo ""
echo "Installed hooks:"
echo "  pre-commit  - Format check, clippy, build verification"
echo "  pre-push    - Full test suite, security audit, license check"
echo "  commit-msg  - Conventional commit format validation"
echo ""
echo "To skip hooks temporarily, use:"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""

# Optional: Check for required tools
echo -e "${BLUE}Checking for optional tools...${NC}"

check_tool() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} $1 not found - install with: $2"
        return 1
    fi
}

check_tool "cargo-audit" "cargo install cargo-audit"
check_tool "cargo-deny" "cargo install cargo-deny"

echo ""
echo -e "${GREEN}Setup complete!${NC}"
