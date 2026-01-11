# Quick Reference

Fast reference for common Palrun commands and shortcuts.

## Installation

```bash
# Cargo
cargo install palrun

# NPM
npm install -g @glinr/palrun

# From source
git clone https://github.com/GLINCKER/palrun.git
cd palrun
cargo install --path .
```

## Basic Commands

```bash
# Set up project (intelligent detection)
palrun setup
palrun setup --dry-run          # Preview
palrun setup --force            # Overwrite
palrun setup --non-interactive  # No prompts

# Launch interactive palette
palrun

# List all commands
palrun list

# List as JSON
palrun list --format json

# Filter by source
palrun list --source npm
palrun list --source cargo

# Scan project
palrun scan
palrun scan --recursive

# Execute command directly
palrun exec build
palrun exec build -y  # Skip confirmation

# Show version
palrun --version

# Enable verbose logging
palrun --verbose
```

## Shell Integration

```bash
# Bash
eval "$(palrun init bash)"

# Zsh
eval "$(palrun init zsh)"

# Fish
palrun init fish | source

# PowerShell
palrun init powershell | Invoke-Expression
```

## Shell Completions

```bash
# Bash
palrun completions bash > /etc/bash_completion.d/palrun

# Zsh
palrun completions zsh > ~/.zfunc/_palrun

# Fish
palrun completions fish > ~/.config/fish/completions/palrun.fish
```

## Configuration

```bash
# Show config path
palrun config --path

# View current config
palrun config

# Config file location
# macOS/Linux: ~/.config/palrun/config.toml
# Windows: %APPDATA%\palrun\config.toml
```

## AI Commands

```bash
# Generate command from natural language
palrun ai gen "start the dev server"
palrun ai gen "run tests" --execute

# Explain a command
palrun ai explain "npm run build"

# Diagnose error
palrun ai diagnose "npm test" "Module not found"

# Check AI status
palrun ai status
```

## Runbooks

```bash
# List runbooks
palrun runbook list

# Run a runbook
palrun runbook deploy

# Dry run (preview)
palrun runbook deploy --dry-run

# Pass variables
palrun runbook deploy --var env=production
```

## Keyboard Shortcuts

### Global (with shell integration)

| Key | Action |
|-----|--------|
| `Ctrl+P` | Open Palrun |

### In Palette

| Key | Action |
|-----|--------|
| `Enter` | Execute selected command |
| `Up/Down` | Navigate |
| `Ctrl+N/P` | Navigate (vim-style) |
| `Ctrl+U` | Clear search |
| `Escape` | Quit |
| `Ctrl+C` | Quit |
| `Tab` | Toggle preview |
| `Ctrl+Space` | Toggle context filtering |

## Configuration Examples

### Minimal Config

```toml
[theme]
highlight_color = "cyan"

[scanner]
exclude_patterns = ["node_modules", "target", ".git"]
```

### Performance Config

```toml
[scanner]
max_depth = 3
cache_enabled = true
cache_ttl = 600

[ui]
max_results = 30
```

### AI Config

```toml
[ai]
provider = "claude"
claude_model = "claude-3-5-sonnet-20241022"
timeout = 60
```

## Environment Variables

```bash
# AI Provider
export ANTHROPIC_API_KEY="your-api-key"
export PALRUN_AI_PROVIDER="claude"

# Ollama
export OLLAMA_HOST="http://localhost:11434"

# Shell
export PALRUN_SHELL="zsh"

# Scanner
export PALRUN_MAX_DEPTH="3"
```

## Supported Project Types

| Type | Config File | Commands |
|------|-------------|----------|
| NPM | package.json | npm run scripts |
| Yarn | package.json + yarn.lock | yarn scripts |
| PNPM | package.json + pnpm-lock.yaml | pnpm scripts |
| Bun | package.json + bun.lockb | bun run scripts |
| Rust | Cargo.toml | cargo build, test, run |
| Go | go.mod | go build, test, run |
| Python | pyproject.toml | pytest, pip, poetry |
| Make | Makefile | make targets |
| Task | Taskfile.yml | task commands |
| Docker | docker-compose.yml | docker compose |
| Nx | nx.json | nx commands |
| Turbo | turbo.json | turbo run |

## Common Workflows

### Daily Development

```bash
# Start dev server
Ctrl+P → type "dev" → Enter

# Run tests
Ctrl+P → type "test" → Enter

# Build
Ctrl+P → type "build" → Enter
```

### Using AI

```bash
# Generate command
palrun ai gen "install react and typescript"

# Explain before running
palrun ai explain "npm run deploy"

# Diagnose failure
palrun ai diagnose "npm test" "error message here"
```

### Runbook Workflow

```bash
# Create runbook
mkdir -p .palrun/runbooks
vim .palrun/runbooks/deploy.yml

# Run runbook
palrun runbook deploy --var env=staging

# Share with team
git add .palrun/runbooks/
git commit -m "Add deployment runbook"
```

## Troubleshooting Quick Fixes

```bash
# No commands found
palrun scan --recursive
palrun --verbose

# Binary not found
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Config issues
palrun config --path
mv ~/.config/palrun/config.toml ~/.config/palrun/config.toml.bak

# AI not working
export ANTHROPIC_API_KEY="your-key"
# or
ollama pull llama2

# Update Palrun
cargo install palrun --force
# or
npm update -g @glinr/palrun
```

## Getting Help

```bash
# Built-in help
palrun --help
palrun ai --help
palrun runbook --help

# Documentation
https://github.com/GLINCKER/palrun/tree/main/docs

# Issues
https://github.com/GLINCKER/palrun/issues

# Discussions
https://github.com/GLINCKER/palrun/discussions
```

## Links

- [Installation Guide](installation.md)
- [Getting Started](getting-started.md)
- [User Guide](user-guide.md)
- [Configuration](configuration.md)
- [Troubleshooting](troubleshooting.md)
- [FAQ](faq.md)

