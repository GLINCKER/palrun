#!/bin/bash
# Version bump script for Palrun
# Analyzes conventional commits to suggest version bumps
# Usage: ./scripts/version-bump.sh [major|minor|patch|<version>]
#
# Without arguments: suggests version based on commits since last tag
# With argument: bumps to specified version or level

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$ROOT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
get_current_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Parse semver into components
parse_version() {
    local version="$1"
    # Remove any prerelease suffix for parsing
    local base="${version%%-*}"
    local prerelease=""
    if [[ "$version" == *"-"* ]]; then
        prerelease="${version#*-}"
    fi

    IFS='.' read -r major minor patch <<< "$base"
    echo "$major $minor $patch $prerelease"
}

# Analyze commits since last tag to suggest bump type
analyze_commits() {
    local last_tag
    last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

    local commits_range
    if [ -n "$last_tag" ]; then
        commits_range="${last_tag}..HEAD"
    else
        commits_range="HEAD"
    fi

    local has_breaking=false
    local has_feat=false
    local has_fix=false

    # Analyze commit messages
    while IFS= read -r commit; do
        if echo "$commit" | grep -qiE '^[a-z]+(\([^)]+\))?!:|BREAKING CHANGE:'; then
            has_breaking=true
        elif echo "$commit" | grep -qiE '^feat(\([^)]+\))?:'; then
            has_feat=true
        elif echo "$commit" | grep -qiE '^fix(\([^)]+\))?:'; then
            has_fix=true
        fi
    done < <(git log --pretty=format:"%s" $commits_range 2>/dev/null)

    if $has_breaking; then
        echo "major"
    elif $has_feat; then
        echo "minor"
    elif $has_fix; then
        echo "patch"
    else
        echo "patch"
    fi
}

# Calculate new version based on bump type
calculate_new_version() {
    local current="$1"
    local bump_type="$2"

    read -r major minor patch prerelease <<< "$(parse_version "$current")"

    case "$bump_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            # Assume it's a full version string
            echo "$bump_type"
            ;;
    esac
}

# Update version in Cargo.toml
update_cargo_version() {
    local new_version="$1"

    if command -v cargo-set-version >/dev/null 2>&1; then
        cargo set-version "$new_version"
    else
        # Use sed with macOS compatibility
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^version = \".*\"/version = \"${new_version}\"/" Cargo.toml
        else
            sed -i "s/^version = \".*\"/version = \"${new_version}\"/" Cargo.toml
        fi
    fi
}

# Update version in npm/package.json
update_npm_version() {
    local new_version="$1"

    if [ -f "npm/package.json" ]; then
        cd npm
        npm version "$new_version" --no-git-tag-version --allow-same-version 2>/dev/null || true
        cd ..
    fi
}

# Main logic
main() {
    local current_version
    current_version=$(get_current_version)

    echo -e "${BLUE}Current version: ${GREEN}${current_version}${NC}"
    echo ""

    local bump_type="$1"
    local new_version

    if [ -z "$bump_type" ]; then
        # Auto-detect from commits
        bump_type=$(analyze_commits)
        echo -e "${YELLOW}Suggested bump type based on commits: ${GREEN}${bump_type}${NC}"

        new_version=$(calculate_new_version "$current_version" "$bump_type")
        echo -e "${YELLOW}Suggested new version: ${GREEN}${new_version}${NC}"
        echo ""

        read -p "Accept this version? [Y/n/custom] " -r response
        case "$response" in
            [nN]*)
                echo "Aborted."
                exit 0
                ;;
            ""|[yY]*)
                # Accept suggested version
                ;;
            *)
                # Custom version entered
                new_version="$response"
                ;;
        esac
    elif [[ "$bump_type" =~ ^(major|minor|patch)$ ]]; then
        new_version=$(calculate_new_version "$current_version" "$bump_type")
    else
        # Assume full version string
        new_version="$bump_type"
    fi

    # Validate version format
    if ! echo "$new_version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
        echo -e "${RED}Error: Invalid version format '${new_version}'${NC}"
        echo "Use semantic versioning (e.g., 0.1.0 or 0.1.0-beta.1)"
        exit 1
    fi

    echo ""
    echo -e "${BLUE}Updating versions to: ${GREEN}${new_version}${NC}"

    # Update Cargo.toml
    echo -e "  ${YELLOW}Updating Cargo.toml...${NC}"
    update_cargo_version "$new_version"

    # Update npm/package.json
    if [ -f "npm/package.json" ]; then
        echo -e "  ${YELLOW}Updating npm/package.json...${NC}"
        update_npm_version "$new_version"
    fi

    # Update Cargo.lock
    echo -e "  ${YELLOW}Updating Cargo.lock...${NC}"
    cargo update -p palrun 2>/dev/null || cargo generate-lockfile

    echo ""
    echo -e "${GREEN}Version updated to ${new_version}!${NC}"
    echo ""
    echo "Files modified:"
    echo "  - Cargo.toml"
    echo "  - Cargo.lock"
    [ -f "npm/package.json" ] && echo "  - npm/package.json"
    echo ""
    echo "Next steps:"
    echo "  1. Review changes: git diff"
    echo "  2. Commit: git add -A && git commit -m 'chore: bump version to ${new_version}'"
    echo "  3. Tag: git tag -a v${new_version} -m 'Release v${new_version}'"
    echo "  4. Push: git push && git push --tags"
}

main "$@"
