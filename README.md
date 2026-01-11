# Palrun

AI command palette for your terminal. Palrun automatically discovers your project's available commands and presents them in a fuzzy-searchable interface.

## Features

- **Project-Aware**: Automatically detects commands from 9+ project types
- **Fuzzy Search**: Fast fuzzy matching powered by nucleo
- **Context-Aware**: Commands sorted by proximity to your current directory
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Shell Integration**: Keyboard shortcuts for quick access

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

### From Source

```bash
git clone https://github.com/yourusername/palrun.git
cd palrun
cargo install --path .
```

### Using Cargo

```bash
cargo install palrun
```

## Usage

### Interactive Mode (Default)

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

## License

MIT
