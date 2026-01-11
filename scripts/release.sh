#!/bin/bash
# Release script for Palrun
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.0

set -e

VERSION="$1"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

# Validate version format (semver)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$'; then
    echo "Error: Invalid version format. Use semantic versioning (e.g., 0.1.0 or 0.1.0-beta)"
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
    # Fallback to sed
    sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
    rm -f Cargo.toml.bak
fi

# Update Cargo.lock
cargo update -p palrun

# Run tests
echo "Running tests..."
cargo test

# Build release
echo "Building release..."
cargo build --release

# Commit and tag
echo "Creating git commit and tag..."
git add Cargo.toml Cargo.lock
git commit -m "chore: release v${VERSION}"
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo ""
echo "Release v${VERSION} prepared!"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git show"
echo "  2. Push to remote: git push && git push --tags"
echo "  3. The GitHub Actions workflow will create the release"
echo ""
echo "To publish to crates.io after the GitHub release:"
echo "  cargo publish"
