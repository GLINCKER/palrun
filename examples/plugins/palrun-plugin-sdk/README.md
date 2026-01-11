# Palrun Plugin SDK

Rust SDK for building Palrun plugins that compile to WebAssembly (WASM).

## Features

- Type-safe API for scanner plugins
- Builder pattern for creating commands
- Automatic FFI exports via macros
- Zero-cost abstractions
- Full documentation

## Quick Start

Add the SDK to your plugin's `Cargo.toml`:

```toml
[package]
name = "my-scanner"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
palrun-plugin-sdk = { path = "../palrun-plugin-sdk" }
serde_json = "1"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

Create your scanner in `src/lib.rs`:

```rust
use palrun_plugin_sdk::prelude::*;

#[derive(Default)]
struct MyScanner;

impl Scanner for MyScanner {
    fn name(&self) -> &'static str {
        "my-scanner"
    }

    fn file_patterns(&self) -> &'static [&'static str] {
        &["Myfile", "*.myext"]
    }

    fn scan(&self, context: &ScanContext) -> Vec<Command> {
        let mut commands = Vec::new();

        if let Some(content) = context.get_file("Myfile") {
            // Parse your file and discover commands
            commands.push(
                Command::new("my-command", "run-my-command")
                    .with_description("Run my custom command")
                    .with_tag("my-tool")
            );
        }

        commands
    }
}

// Export the scanner for WASM
export_scanner!(MyScanner);
```

Build for WASM:

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

## API Reference

### Command

Represents a command discovered by a scanner:

```rust
// Simple creation
let cmd = Command::new("build", "make build");

// With all options
let cmd = Command::new("deploy", "./deploy.sh")
    .with_description("Deploy to production")
    .with_working_dir("scripts")
    .with_tag("deploy")
    .with_tag("production");

// Using builder
let cmd = CommandBuilder::new()
    .name("test")
    .command("npm test")
    .description("Run tests")
    .tag("test")
    .build()
    .expect("valid command");
```

### ScanContext

Provides project information to scanners:

```rust
fn scan(&self, context: &ScanContext) -> Vec<Command> {
    // Access project info
    let project_name = &context.project_name;
    let project_path = &context.project_path;

    // Read matched files
    if let Some(content) = context.get_file("package.json") {
        // Parse and extract commands
    }

    // Check for files
    if context.has_file("Makefile") {
        // Add make commands
    }

    // Access environment (if permitted)
    if let Some(home) = context.get_env("HOME") {
        // Use environment variable
    }

    vec![]
}
```

### Scanner Trait

Implement this trait for your scanner:

```rust
impl Scanner for MyScanner {
    // Required: unique scanner name
    fn name(&self) -> &'static str {
        "my-scanner"
    }

    // Required: file patterns to match
    fn file_patterns(&self) -> &'static [&'static str] {
        &["*.config", "config.toml"]
    }

    // Required: scan implementation
    fn scan(&self, context: &ScanContext) -> Vec<Command> {
        vec![]
    }

    // Optional: description
    fn description(&self) -> Option<&'static str> {
        Some("Scans for custom config files")
    }

    // Optional: priority (higher runs first)
    fn priority(&self) -> i32 {
        0
    }
}
```

## Plugin Manifest

Create a `plugin.toml` alongside your WASM file:

```toml
[plugin]
name = "my-scanner"
version = "0.1.0"
author = "Your Name"
description = "Scans for my custom commands"
type = "scanner"
api_version = "0.1.0"
license = "MIT"
keywords = ["custom", "scanner"]

[permissions]
network = false
execute = false
environment = false

[permissions.filesystem]
read = true
write = false
paths = ["Myfile", "*.myext"]
```

## Testing

Test your scanner logic without WASM:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        let scanner = MyScanner;
        let context = ScanContext::new("/project", "test")
            .with_file("Myfile", "content here");

        let commands = scanner.scan(&context);
        assert!(!commands.is_empty());
    }
}
```

Run tests:

```bash
cargo test
```

## Building

```bash
# Debug build
cargo build --target wasm32-wasip1

# Release build (optimized, smaller)
cargo build --target wasm32-wasip1 --release
```

## Installation

```bash
# Install your plugin
pal plugin install ./target/wasm32-wasip1/release/my_scanner.wasm
```

## License

MIT
