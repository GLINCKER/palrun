#!/bin/sh
# Palrun Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/GLINCKER/palrun/main/scripts/install.sh | sh
#
# This script downloads and installs the latest version of Palrun.
# It detects your operating system and architecture automatically.

set -e

# Configuration
REPO="GLINCKER/palrun"
BINARY_NAME="palrun"
INSTALL_DIR="${PALRUN_INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
    exit 1
}

# Detect operating system
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux" ;;
        Darwin*)    echo "darwin" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)          error "Unsupported operating system: $(uname -s)" ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# Get the latest release version from GitHub
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Download a file
download() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Main installation function
main() {
    echo ""
    echo "  ____       _                   "
    echo " |  _ \ __ _| |_ __ _   _ _ __   "
    echo " | |_) / _\` | | '__| | | | '_ \  "
    echo " |  __/ (_| | | |  | |_| | | | | "
    echo " |_|   \__,_|_|_|   \__,_|_| |_| "
    echo ""
    echo "  AI command palette for your terminal"
    echo ""

    # Detect platform
    OS=$(detect_os)
    ARCH=$(detect_arch)

    info "Detected platform: ${OS}-${ARCH}"

    # Get version
    VERSION="${PALRUN_VERSION:-$(get_latest_version)}"
    if [ -z "$VERSION" ]; then
        error "Could not determine the latest version. Please specify PALRUN_VERSION."
    fi

    info "Installing Palrun ${VERSION}..."

    # Construct download URL
    case "$OS" in
        linux)
            ARCHIVE_NAME="palrun-${VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
            ;;
        darwin)
            ARCHIVE_NAME="palrun-${VERSION}-${ARCH}-apple-darwin.tar.gz"
            ;;
        windows)
            ARCHIVE_NAME="palrun-${VERSION}-${ARCH}-pc-windows-msvc.zip"
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"

    info "Downloading from: ${DOWNLOAD_URL}"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download archive
    ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"
    download "$DOWNLOAD_URL" "$ARCHIVE_PATH" || error "Failed to download release"

    # Extract archive
    info "Extracting archive..."
    cd "$TMP_DIR"

    case "$ARCHIVE_NAME" in
        *.tar.gz)
            tar xzf "$ARCHIVE_PATH"
            ;;
        *.zip)
            unzip -q "$ARCHIVE_PATH"
            ;;
    esac

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Find and install binary
    BINARY_PATH=$(find "$TMP_DIR" -name "$BINARY_NAME" -o -name "${BINARY_NAME}.exe" | head -1)

    if [ -z "$BINARY_PATH" ]; then
        error "Could not find binary in archive"
    fi

    # Install binary
    cp "$BINARY_PATH" "$INSTALL_DIR/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    # Also create 'pal' symlink
    if [ "$OS" != "windows" ]; then
        ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/pal"
    fi

    success "Palrun installed to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if install directory is in PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            success "Installation complete!"
            ;;
        *)
            warn "Add ${INSTALL_DIR} to your PATH to use palrun:"
            echo ""
            echo "  For bash/zsh, add to your ~/.bashrc or ~/.zshrc:"
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "  For fish, add to your ~/.config/fish/config.fish:"
            echo "    fish_add_path ${INSTALL_DIR}"
            echo ""
            ;;
    esac

    # Show version
    echo ""
    info "Installed version:"
    "${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null || true

    # Offer shell integration setup
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    info "Shell Integration (Recommended)"
    echo ""
    printf "  For the best experience, add shell integration to your config:\n"
    echo ""
    printf "  ${GREEN}Bash${NC} (~/.bashrc):\n"
    printf "    eval \"\$(palrun init bash)\"\n"
    echo ""
    printf "  ${GREEN}Zsh${NC} (~/.zshrc):\n"
    printf "    eval \"\$(palrun init zsh)\"\n"
    echo ""
    printf "  ${GREEN}Fish${NC} (~/.config/fish/config.fish):\n"
    printf "    palrun init fish | source\n"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    success "Installation complete! Run 'palrun' or 'pal' to get started!"
    echo ""
}

# Uninstall function
uninstall() {
    echo ""
    info "Uninstalling Palrun..."

    INSTALL_DIR="${PALRUN_INSTALL_DIR:-$HOME/.local/bin}"

    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        rm -f "${INSTALL_DIR}/pal"
        success "Palrun has been uninstalled from ${INSTALL_DIR}"
    else
        warn "Palrun not found in ${INSTALL_DIR}"
    fi

    echo ""
    info "To complete uninstallation:"
    echo "  1. Remove shell integration from your shell config"
    echo "  2. Optionally remove config: rm -rf ~/.palrun"
    echo ""
}

# Show help
show_help() {
    echo "Palrun Installation Script"
    echo ""
    echo "Usage:"
    echo "  curl -fsSL https://raw.githubusercontent.com/GLINCKER/palrun/main/scripts/install.sh | sh"
    echo ""
    echo "Options:"
    echo "  --help, -h       Show this help message"
    echo "  --uninstall, -u  Uninstall Palrun"
    echo ""
    echo "Environment Variables:"
    echo "  PALRUN_VERSION      Install a specific version (e.g., v0.1.0)"
    echo "  PALRUN_INSTALL_DIR  Custom install directory (default: ~/.local/bin)"
    echo ""
}

# Parse arguments
case "${1:-}" in
    --uninstall|-u)
        uninstall
        ;;
    --help|-h)
        show_help
        ;;
    *)
        main "$@"
        ;;
esac
