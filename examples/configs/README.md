# Configuration Examples

Example Palrun configuration files for different use cases.

## File Format

Palrun uses **TOML format only** for configuration:
- **Project-level**: `.palrun.toml` in your project root
- **Global**: `~/.config/palrun/config.toml`

> **Note**: Only TOML format is supported. JSON and TypeScript config files are not supported.

## Configuration Precedence

Settings are loaded in this order (later overrides earlier):
1. Global config (`~/.config/palrun/config.toml`)
2. Project config (`.palrun.toml`)
3. Environment variables
4. CLI flags

## Available Examples

### Basic Configurations

- **[basic.toml](basic.toml)** - Minimal configuration with common settings
- **[full.toml](full.toml)** - Complete configuration with all available options
- **[minimal.toml](minimal.toml)** - Bare minimum configuration

### Project-Specific

- **[nodejs.toml](nodejs.toml)** - Optimized for Node.js/NPM projects
- **[rust.toml](rust.toml)** - Optimized for Rust/Cargo projects
- **[go.toml](go.toml)** - Optimized for Go projects
- **[python.toml](python.toml)** - Optimized for Python projects
- **[monorepo.toml](monorepo.toml)** - Optimized for monorepo projects (Nx, Turbo)

### Feature-Specific

- **[ai-enabled.toml](ai-enabled.toml)** - Configuration with AI features enabled
- **[team.toml](team.toml)** - Team-friendly configuration with shared settings
- **[ci-cd.toml](ci-cd.toml)** - Configuration for CI/CD environments

## Quick Start

### 1. Generate Config for Your Project

```bash
palrun init
```

This automatically detects your project type and creates an optimized `.palrun.toml`.

### 2. Or Copy a Template

```bash
# For a Node.js project
cp examples/configs/nodejs.toml .palrun.toml

# For a Rust project
cp examples/configs/rust.toml .palrun.toml

# For a monorepo
cp examples/configs/monorepo.toml .palrun.toml
```

### 3. Customize

Edit `.palrun.toml` to match your preferences.

## Configuration Sections

### [general]
General application settings:
- `show_hidden` - Show hidden commands
- `confirm_dangerous` - Confirm before dangerous operations
- `max_history` - Maximum history entries
- `shell` - Default shell for command execution

### [ui]
UI/TUI appearance:
- `theme` - Color theme (default, dark, light, custom)
- `show_preview` - Show command preview panel
- `show_icons` - Show source icons
- `max_display` - Maximum commands to display
- `mouse` - Enable mouse support

### [scanner]
Command scanning behavior:
- `enabled` - List of enabled scanners
- `ignore_dirs` - Directories to ignore
- `max_depth` - Maximum scan depth for monorepos
- `recursive` - Enable recursive scanning

### [ai]
AI integration settings (requires `ai` feature):
- `enabled` - Enable AI features
- `provider` - AI provider (claude, ollama, openai)
- `model` - Model to use
- `ollama.base_url` - Ollama server URL
- `ollama.model` - Ollama model name

### [keys]
Keyboard shortcuts:
- `quit` - Key to quit
- `select` - Key to select
- `up` - Key to move up
- `down` - Key to move down
- `clear` - Key to clear input
- `ai_mode` - Key to toggle AI mode

## Environment Variables

Override config with environment variables:

```bash
# AI provider
export PALRUN_AI_PROVIDER=claude
export ANTHROPIC_API_KEY=sk-ant-...

# Ollama
export PALRUN_OLLAMA_URL=http://localhost:11434
export PALRUN_OLLAMA_MODEL=codellama

# Scanner settings
export PALRUN_MAX_DEPTH=10
export PALRUN_RECURSIVE=true
```

## Validation

Test your configuration:

```bash
palrun config
```

This shows the loaded configuration and highlights any issues.

## Tips

1. **Start simple** - Use `palrun init` or copy `basic.toml`
2. **Project-specific** - Use `.palrun.toml` for project settings
3. **Global defaults** - Use `~/.config/palrun/config.toml` for personal preferences
4. **Version control** - Commit `.palrun.toml` to share with your team
5. **Ignore sensitive data** - Never commit API keys in config files

## Next Steps

- [Runbook Examples](../runbooks/README.md)
- [Integration Examples](../integrations/README.md)
- [Configuration Reference](../../docs/reference/config-reference.md)

