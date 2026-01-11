# Contributing to Palrun

Thank you for your interest in contributing to Palrun! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear and descriptive title
- Detailed steps to reproduce the issue
- Expected vs actual behavior
- Your environment (OS, Rust version, Palrun version)
- Relevant logs or error messages

### Suggesting Features

Feature requests are welcome! Please:

- Use a clear and descriptive title
- Provide a detailed description of the proposed feature
- Explain why this feature would be useful
- Include examples of how it would work

### Pull Requests

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run formatting (`cargo fmt`)
6. Run clippy (`cargo clippy`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- Cargo
- Git

### Building

```bash
git clone https://github.com/GLINCKER/palrun.git
cd palrun
cargo build
```

### Running Tests

```bash
cargo test
cargo test --all-features
```

### Code Style

We use `rustfmt` and `clippy` to maintain code quality:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Project Structure

```
palrun/
├── src/
│   ├── scanner/     # Project type scanners
│   ├── tui/         # Terminal UI components
│   ├── core/        # Core functionality
│   ├── plugin/      # Plugin system
│   └── ai/          # AI integration (future)
├── examples/        # Example plugins
├── tests/           # Integration tests
└── shell/           # Shell integration scripts
```

## Adding a New Scanner

To add support for a new project type:

1. Create a new file in `src/scanner/`
2. Implement the `Scanner` trait
3. Add tests for your scanner
4. Update documentation
5. Add example project in `tests/fixtures/`

Example:

```rust
use crate::core::command::Command;
use anyhow::Result;

pub struct MyScanner;

impl MyScanner {
    pub fn scan(&self, path: &Path) -> Result<Vec<Command>> {
        // Implementation
    }
}
```

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### Test Coverage

We aim for high test coverage. Please add tests for new features.

## Documentation

- Update README.md for user-facing changes
- Add inline documentation for public APIs
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/)

## Commit Messages

Follow conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

Example: `feat: add support for Poetry scanner`

## Release Process

Releases are automated through GitHub Actions when a tag is pushed:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit changes
4. Create and push tag: `git tag v0.1.0 && git push origin v0.1.0`

## Questions?

Feel free to open an issue for questions or join our discussions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

