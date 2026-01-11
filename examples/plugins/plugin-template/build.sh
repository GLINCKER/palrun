#!/usr/bin/env bash
#
# Build script for Palrun scanner plugin
#
# Usage:
#   ./build.sh          # Build debug version
#   ./build.sh release  # Build optimized release version
#   ./build.sh install  # Build and install to Palrun
#   ./build.sh clean    # Clean build artifacts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Plugin name from Cargo.toml
PLUGIN_NAME=$(grep '^name' Cargo.toml | head -1 | cut -d'"' -f2 | tr '-' '_')

# Target triple for WASM
TARGET="wasm32-wasip1"

# Check for Rust and the WASM target
check_requirements() {
    if ! command -v rustup &> /dev/null; then
        echo -e "${RED}Error: rustup is not installed${NC}"
        echo "Install from: https://rustup.rs"
        exit 1
    fi

    if ! rustup target list --installed | grep -q "$TARGET"; then
        echo -e "${YELLOW}Installing $TARGET target...${NC}"
        rustup target add "$TARGET"
    fi
}

# Build debug version
build_debug() {
    echo -e "${GREEN}Building debug version...${NC}"
    cargo build --target "$TARGET"

    local wasm_path="target/$TARGET/debug/${PLUGIN_NAME}.wasm"
    if [[ -f "$wasm_path" ]]; then
        local size=$(du -h "$wasm_path" | cut -f1)
        echo -e "${GREEN}Built: $wasm_path ($size)${NC}"
    fi
}

# Build release version
build_release() {
    echo -e "${GREEN}Building release version...${NC}"
    cargo build --target "$TARGET" --release

    local wasm_path="target/$TARGET/release/${PLUGIN_NAME}.wasm"
    if [[ -f "$wasm_path" ]]; then
        local size=$(du -h "$wasm_path" | cut -f1)
        echo -e "${GREEN}Built: $wasm_path ($size)${NC}"
    fi
}

# Install plugin to Palrun
install_plugin() {
    build_release

    local wasm_path="target/$TARGET/release/${PLUGIN_NAME}.wasm"

    echo -e "${GREEN}Installing plugin...${NC}"

    # Check if pal is available
    if command -v pal &> /dev/null; then
        pal plugin install "$wasm_path"
    elif command -v palrun &> /dev/null; then
        palrun plugin install "$wasm_path"
    else
        echo -e "${YELLOW}Warning: pal/palrun not found in PATH${NC}"
        echo "Copy manually to ~/.local/share/palrun/plugins/"
        echo "  cp $wasm_path ~/.local/share/palrun/plugins/"
        echo "  cp plugin.toml ~/.local/share/palrun/plugins/${PLUGIN_NAME}/"
    fi
}

# Clean build artifacts
clean() {
    echo -e "${GREEN}Cleaning build artifacts...${NC}"
    cargo clean
    echo "Done."
}

# Run tests
run_tests() {
    echo -e "${GREEN}Running tests...${NC}"
    cargo test
}

# Show help
show_help() {
    echo "Palrun Plugin Build Script"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  (none)   Build debug version"
    echo "  release  Build optimized release version"
    echo "  install  Build and install to Palrun"
    echo "  test     Run unit tests"
    echo "  clean    Remove build artifacts"
    echo "  help     Show this help"
}

# Main
check_requirements

case "${1:-}" in
    release)
        build_release
        ;;
    install)
        install_plugin
        ;;
    test)
        run_tests
        ;;
    clean)
        clean
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        build_debug
        ;;
esac
