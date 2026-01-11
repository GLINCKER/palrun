#!/bin/bash
# Release script for Palrun
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0-beta.1

set -e

VERSION="$1"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0-beta.1"
    exit 1
fi

# Validate version format (semver with optional prerelease)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'; then
    echo "Error: Invalid version format. Use semantic versioning (e.g., 0.1.0 or 0.1.0-beta.1)"
    exit 1
fi

echo "Preparing release v${VERSION}..."

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "Warning: You have uncommitted changes."
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Update version in Cargo.toml
echo "Updating Cargo.toml version to ${VERSION}..."
if command -v cargo-set-version >/dev/null 2>&1; then
    cargo set-version "$VERSION"
else
    # Fallback to sed (macOS compatible)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
    fi
fi

# Update npm package version
if [ -f "npm/package.json" ]; then
    echo "Updating npm/package.json version to ${VERSION}..."
    cd npm
    npm version "$VERSION" --no-git-tag-version --allow-same-version 2>/dev/null || true
    cd ..
fi

# Update Cargo.lock
cargo update -p palrun

# Run tests
echo "Running tests..."
cargo test --lib

# Build release
echo "Building release..."
cargo build --release

# Generate changelog
echo "Generating changelog..."
if command -v git-cliff >/dev/null 2>&1; then
    git cliff --unreleased --tag "v${VERSION}" --prepend CHANGELOG.md 2>/dev/null || true
fi

# Commit and tag
echo "Creating git commit and tag..."
git add Cargo.toml Cargo.lock CHANGELOG.md
[ -f "npm/package.json" ] && git add npm/package.json
git commit -m "chore(release): v${VERSION}"
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo ""
echo "Release v${VERSION} prepared!"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git show"
echo "  2. Push to remote: git push && git push --tags"
echo "  3. GitHub Actions will:"
echo "     - Build binaries for all platforms"
echo "     - Create GitHub Release"
echo "     - Publish to npm (with provenance)"
echo "     - Publish to crates.io"
