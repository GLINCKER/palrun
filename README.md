# PALRUN

Project-aware command palette for your terminal with AI-powered intelligence.

[![CI](https://github.com/GLINCKER/palrun/actions/workflows/ci.yml/badge.svg)](https://github.com/GLINCKER/palrun/actions/workflows/ci.yml)
[![Release](https://github.com/GLINCKER/palrun/actions/workflows/release.yml/badge.svg)](https://github.com/GLINCKER/palrun/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/palrun.svg)](https://crates.io/crates/palrun)

## Why Palrun?

Stop memorizing commands. Palrun automatically discovers every command available in your project and presents them in a blazing-fast fuzzy-searchable interface. Whether you're working with npm, cargo, make, docker, or any of 9+ supported project types, Palrun knows what you can run.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PALRUN v0.1.0                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Project Scan ──► Command Discovery ──► Fuzzy Search ──► Execute          │
│                    (9+ types)             (nucleo)         (context-aware)  │
│                                                                             │
│   Cargo.toml   ──► cargo build, test    ──► "bui"     ──► cargo build      │
│   package.json ──► npm run dev, test    ──► "dev"     ──► npm run dev      │
│   Makefile     ──► make all, clean      ──► "cle"     ──► make clean       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Features

### Core Capabilities

- **Project-Aware Discovery**: Automatically detects commands from 9+ project types
- **Fuzzy Search**: Lightning-fast fuzzy matching powered by nucleo engine
- **Context-Aware Sorting**: Commands sorted by proximity to your current directory
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Shell Integration**: Keyboard shortcuts for instant access
- **TUI Interface**: Beautiful terminal UI with keyboard navigation
- **Plugin System**: Extensible architecture for custom scanners

### Supported Project Types

| Project Type | Config Files | Commands Generated |
|-------------|--------------|-------------------|
| NPM/Yarn/PNPM/Bun | `package.json` | npm/yarn/pnpm/bun scripts |
| Rust | `Cargo.toml` | cargo build, test, run, clippy |
| Go | `go.mod` | go build, test, run |
| Python | `pyproject.toml`, `requirements.txt` | pytest, pip, poetry, pdm |
| Make | `Makefile` | make targets |
| Task | `Taskfile.yml` | task commands |
| Docker | `docker-compose.yml` | docker compose up/down/logs |
| Nx | `nx.json` | nx build, serve, test |
| Turborepo | `turbo.json` | turbo run tasks |

## Installation

### Using Cargo

```bash
cargo install palrun
```

### From Source

```bash
git clone https://github.com/GLINCKER/palrun.git
cd palrun
cargo install --path .
```

### Homebrew (macOS/Linux)

```bash
brew tap GLINCKER/tap
brew install palrun
```

### NPM (Node.js users)

```bash
npm install -g @glinr/palrun
```

### Quick Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/GLINCKER/palrun/main/scripts/install.sh | bash
```

## Quick Start

### 1. Set Up Your Project

Initialize Palrun in your project with intelligent detection:

```bash
palrun setup
```

This will:
- Detect your project type (Node.js, Rust, Python, etc.)
- Create `.palrun.toml` with recommended settings
- Generate `.palrun/runbooks/` with sample workflows
- Suggest relevant configurations

Options:
```bash
palrun setup --dry-run          # Preview what would be created
palrun setup --force            # Overwrite existing files
palrun setup --non-interactive  # Use defaults without prompts
```

### 2. Interactive Mode

Launch the command palette:

```bash
palrun
```

Use arrow keys to navigate, type to search, and press Enter to execute.

### List Commands

Show all discovered commands:

```bash
palrun list
```

Output as JSON:

```bash
palrun list --format json
```

Filter by source type:

```bash
palrun list --source cargo
palrun list --source npm
```

### Scan Project

Preview what commands would be discovered:

```bash
palrun scan
palrun scan --recursive
```

### Execute Directly

Run a command by name:

```bash
palrun exec build
palrun exec "npm test"
```

Skip confirmation:

```bash
palrun exec build -y
```

## Shell Integration

Add to your shell configuration for keyboard shortcuts:

### Bash

```bash
eval "$(palrun init bash)"
```

### Zsh

```bash
eval "$(palrun init zsh)"
```

### Fish

```fish
palrun init fish | source
```

### PowerShell

```powershell
palrun init powershell | Invoke-Expression
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Execute selected command |
| `Up/Down` | Navigate command list |
| `Ctrl+N/P` | Navigate (vim-style) |
| `Ctrl+U` | Clear search input |
| `Escape` | Quit |
| `Tab` | Toggle preview |
| `Ctrl+Space` | Toggle context-aware filtering |

## Configuration

Configuration file location: `~/.config/palrun/config.toml`

```toml
# Theme settings
[theme]
highlight_color = "cyan"

# Shell settings
[shell]
default = "bash"

# Scanner settings
[scanner]
exclude_patterns = ["node_modules", "target", ".git"]
```

Show config path:

```bash
palrun config --path
```

## Shell Completions

Generate shell completions:

```bash
# Bash
palrun completions bash > /etc/bash_completion.d/palrun

# Zsh
palrun completions zsh > ~/.zfunc/_palrun

# Fish
palrun completions fish > ~/.config/fish/completions/palrun.fish
```

## Plugin System

Palrun supports custom scanners through a plugin architecture. Example plugins are included:

- **cargo-scanner**: Enhanced Cargo.toml scanning
- **composer-scanner**: PHP Composer support
- **gradle-scanner**: Gradle build tool support
- **maven-scanner**: Maven build tool support
- **poetry-scanner**: Python Poetry support

See `examples/plugins/` for implementation details.

## Development

### Building

```bash
cargo build
cargo build --release
```

### Testing

```bash
cargo test
cargo test --all-features
```

### Running

```bash
cargo run
cargo run -- list
cargo run -- scan
```

## Features Status

### Completed
- [x] AI-powered command suggestions (Claude, OpenAI, Ollama)
- [x] Runbook system for team workflows
- [x] Command history and analytics
- [x] Git integration (branch switching, status)
- [x] Environment management (nvm, pyenv, etc.)
- [x] Plugin system with SDK
- [x] MCP (Model Context Protocol) integration
- [x] Advanced search and filtering
- [x] Theme support (multiple built-in themes)

### Coming Soon
- [ ] Cloud sync and team collaboration
- [ ] VS Code extension
- [ ] Signed binaries for macOS/Windows
- [ ] More IDE integrations

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

- Bug reports and fixes
- New project type scanners
- Performance improvements
- Documentation updates

## License

MIT License - free for personal and commercial use.

See [LICENSE](LICENSE) for details.

## Support

- Documentation: [GitHub Wiki](https://github.com/GLINCKER/palrun/wiki)
- Issues: [GitHub Issues](https://github.com/GLINCKER/palrun/issues)
- Discussions: [GitHub Discussions](https://github.com/GLINCKER/palrun/discussions)

Built by [GLINCKER](https://glincker.com)
