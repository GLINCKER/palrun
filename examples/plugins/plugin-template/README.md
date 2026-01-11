# My Scanner - Palrun Plugin Template

A template for creating Palrun scanner plugins.

## Getting Started

1. **Copy this template** to a new directory:
   ```bash
   cp -r plugin-template my-scanner
   cd my-scanner
   ```

2. **Update `Cargo.toml`**:
   - Change `name` to your plugin name
   - Update `author` and `description`

3. **Update `plugin.toml`**:
   - Set the same `name` as in Cargo.toml
   - Update `description` and `author`
   - Configure `permissions` as needed
   - Set `file_patterns` for your scanner

4. **Implement your scanner** in `src/lib.rs`:
   - Update `file_patterns()` to match your files
   - Implement `scan()` to parse files and return commands

5. **Build and test**:
   ```bash
   make test      # Run unit tests
   make release   # Build WASM
   make install   # Install to Palrun
   ```

## Project Structure

```
my-scanner/
├── Cargo.toml      # Rust package configuration
├── plugin.toml     # Palrun plugin manifest
├── Makefile        # Build commands
├── build.sh        # Alternative build script
├── README.md       # Documentation
└── src/
    └── lib.rs      # Scanner implementation
```

## Building

### Prerequisites

- Rust toolchain (rustup)
- WASM target: `rustup target add wasm32-wasip1`

### Commands

```bash
# Build debug version
make

# Build optimized release
make release

# Run tests
make test

# Install to Palrun
make install

# Clean build artifacts
make clean
```

Or using the shell script:

```bash
./build.sh          # Debug build
./build.sh release  # Release build
./build.sh install  # Build and install
./build.sh test     # Run tests
```

## Development

### Adding Dependencies

Add dependencies in `Cargo.toml`:

```toml
[dependencies]
toml = "0.8"        # For parsing TOML
regex = "1"         # For pattern matching
```

Note: Keep dependencies minimal for smaller WASM size.

### Testing

Write tests in `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        let scanner = MyScanner;
        let context = ScanContext::new("/project", "test")
            .with_file("Myfile", "build: make build");

        let commands = scanner.scan(&context);
        assert!(!commands.is_empty());
    }
}
```

Run tests:
```bash
cargo test
```

### Debugging

Enable debug logging in your scanner:
```rust
// The host will capture and log this
eprintln!("Debug: processing file {}", path);
```

## Publishing

1. Update version in `Cargo.toml` and `plugin.toml`
2. Build release: `make release`
3. Create release package:
   ```bash
   mkdir -p release
   cp target/wasm32-wasip1/release/my_scanner.wasm release/
   cp plugin.toml release/
   cp README.md release/
   ```
4. Distribute via GitHub releases or your preferred method

## License

MIT
