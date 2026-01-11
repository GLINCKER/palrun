# Makefile for Palrun Plugin
#
# Usage:
#   make          - Build debug version
#   make release  - Build optimized release
#   make install  - Build and install plugin
#   make test     - Run tests
#   make clean    - Clean build artifacts

# Plugin configuration
PLUGIN_NAME := $(shell grep '^name' Cargo.toml | head -1 | cut -d'"' -f2 | tr '-' '_')
TARGET := wasm32-wasip1

# Paths
WASM_DEBUG := target/$(TARGET)/debug/$(PLUGIN_NAME).wasm
WASM_RELEASE := target/$(TARGET)/release/$(PLUGIN_NAME).wasm

.PHONY: all debug release install test clean check help

# Default target
all: debug

# Check for WASM target
check:
	@if ! rustup target list --installed | grep -q $(TARGET); then \
		echo "Installing $(TARGET) target..."; \
		rustup target add $(TARGET); \
	fi

# Build debug version
debug: check
	@echo "Building debug version..."
	cargo build --target $(TARGET)
	@echo "Built: $(WASM_DEBUG)"
	@ls -lh $(WASM_DEBUG) 2>/dev/null | awk '{print "Size:", $$5}'

# Build release version
release: check
	@echo "Building release version..."
	cargo build --target $(TARGET) --release
	@echo "Built: $(WASM_RELEASE)"
	@ls -lh $(WASM_RELEASE) 2>/dev/null | awk '{print "Size:", $$5}'

# Install to Palrun
install: release
	@echo "Installing plugin..."
	@if command -v pal >/dev/null 2>&1; then \
		pal plugin install $(WASM_RELEASE); \
	elif command -v palrun >/dev/null 2>&1; then \
		palrun plugin install $(WASM_RELEASE); \
	else \
		echo "Warning: pal/palrun not found. Manual install:"; \
		echo "  mkdir -p ~/.local/share/palrun/plugins/$(PLUGIN_NAME)"; \
		echo "  cp $(WASM_RELEASE) ~/.local/share/palrun/plugins/$(PLUGIN_NAME)/"; \
		echo "  cp plugin.toml ~/.local/share/palrun/plugins/$(PLUGIN_NAME)/"; \
	fi

# Run tests
test:
	cargo test

# Run clippy
lint:
	cargo clippy --target $(TARGET)

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt --check

# Clean build artifacts
clean:
	cargo clean

# Show help
help:
	@echo "Palrun Plugin Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build debug version (default)"
	@echo "  debug    - Build debug version"
	@echo "  release  - Build optimized release version"
	@echo "  install  - Build and install to Palrun"
	@echo "  test     - Run unit tests"
	@echo "  lint     - Run clippy"
	@echo "  fmt      - Format code"
	@echo "  clean    - Remove build artifacts"
	@echo "  help     - Show this help"
