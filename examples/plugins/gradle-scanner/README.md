# Gradle Scanner Plugin

A Palrun plugin that scans Gradle projects for available tasks.

## Features

- Detects `build.gradle` and `build.gradle.kts` files
- Extracts task names from Gradle build files
- Supports multi-project Gradle builds
- Excludes common utility tasks (help, dependencies, etc.)

## Installation

```bash
# Build the plugin (requires Rust with wasm32-wasi target)
cargo build --target wasm32-wasi --release

# Install in Palrun
pal plugin install ./target/wasm32-wasi/release/gradle_scanner.wasm
```

## Configuration

The plugin can be configured in your `palrun.toml`:

```toml
[plugins.gradle-scanner.config]
scan_depth = 3
include_subprojects = true
exclude_patterns = ["help", "components"]
```

## Detected Commands

This plugin detects commands like:

- `gradle build` - Build the project
- `gradle test` - Run tests
- `gradle clean` - Clean build artifacts
- `gradle assemble` - Assemble the outputs
- Custom tasks defined in your build files

## Development

### Prerequisites

- Rust toolchain
- `wasm32-wasi` target: `rustup target add wasm32-wasi`

### Building

```bash
cd examples/plugins/gradle-scanner
cargo build --target wasm32-wasi --release
```

### Testing

```bash
cargo test
```

## License

MIT License - see [LICENSE](../../../LICENSE) for details.
