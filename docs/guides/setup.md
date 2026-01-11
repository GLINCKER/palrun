# Project Setup Guide

The `palrun setup` command intelligently detects your project type and generates optimized configuration files.

## Quick Start

```bash
# Set up in current directory
palrun setup

# Set up in specific directory
palrun setup /path/to/project

# Preview without creating files
palrun setup --dry-run

# Force overwrite existing files
palrun setup --force

# Non-interactive mode (use defaults)
palrun setup --non-interactive
```

## What It Does

When you run `palrun setup`, it:

1. **Detects Project Type** - Analyzes your project files to determine the type
2. **Generates Configuration** - Creates `.palrun.toml` with optimized settings
3. **Creates Runbooks Directory** - Sets up `.palrun/runbooks/` with sample workflows
4. **Validates Configuration** - Ensures generated config is valid before writing
5. **Provides Next Steps** - Suggests relevant commands and integrations

## Supported Project Types

### Node.js / NPM
**Detection**: `package.json` exists
**Generated Config**:
- Scanners: `npm`, `docker`
- Ignore: `node_modules`, `.git`, `dist`, `build`, `coverage`
- Max depth: 5
- Recursive: false

**Sample Runbooks**:
- `deploy.yml` - Build and deploy workflow
- `dev-setup.yml` - Development environment setup

### Next.js
**Detection**: `next.config.js`, `next.config.mjs`, or `next.config.ts` exists
**Generated Config**:
- Scanners: `npm`, `docker`
- Ignore: `node_modules`, `.git`, `.next`, `out`, `dist`, `build`, `coverage`
- Max depth: 5
- Recursive: false

**Sample Runbooks**:
- `deploy.yml` - Build and deploy workflow
- `dev-setup.yml` - Development environment setup

### React
**Detection**: `package.json` contains `"react"`
**Generated Config**: Same as Node.js
**Sample Runbooks**: Same as Node.js

### Rust / Cargo
**Detection**: `Cargo.toml` exists
**Generated Config**:
- Scanners: `cargo`, `make`, `docker`, `taskfile`
- Ignore: `target`, `.git`, `node_modules`
- Max depth: 5
- Recursive: true (for workspaces)

**Sample Runbooks**:
- `build.yml` - Build with different profiles
- `test.yml` - Run test suite

### Go
**Detection**: `go.mod` exists
**Generated Config**:
- Scanners: `go`, `make`, `docker`
- Ignore: `.git`, `vendor`, `bin`
- Max depth: 5
- Recursive: false

**Sample Runbooks**:
- `build.yml` - Build application
- `test.yml` - Run tests with coverage

### Python
**Detection**: `pyproject.toml`, `setup.py`, or `requirements.txt` exists
**Generated Config**:
- Scanners: `python`, `make`, `docker`
- Ignore: `.git`, `__pycache__`, `.venv`, `venv`, `.pytest_cache`, `dist`, `build`
- Max depth: 5
- Recursive: false

**Sample Runbooks**:
- `test.yml` - Run pytest
- `dev-setup.yml` - Set up virtual environment

### Nx Monorepo
**Detection**: `nx.json` exists
**Generated Config**:
- Scanners: `npm`, `nx`, `docker`, `make`
- Ignore: `node_modules`, `.git`, `dist`, `build`, `.nx`, `coverage`
- Max depth: 10 (for deep monorepo structures)
- Recursive: true

**Sample Runbooks**:
- `build-all.yml` - Build all packages
- `deploy.yml` - Deploy workflow

### Turborepo
**Detection**: `turbo.json` exists
**Generated Config**:
- Scanners: `npm`, `turbo`, `docker`, `make`
- Ignore: `node_modules`, `.git`, `dist`, `build`, `.turbo`, `coverage`
- Max depth: 10 (for deep monorepo structures)
- Recursive: true

**Sample Runbooks**:
- `build-all.yml` - Build all packages
- `deploy.yml` - Deploy workflow

### Generic
**Detection**: Fallback when no specific type is detected
**Generated Config**:
- Scanners: All available (`npm`, `cargo`, `make`, `docker`, `go`, `python`, `nx`, `turbo`, `taskfile`)
- Ignore: `node_modules`, `.git`, `target`, `dist`, `build`
- Max depth: 5
- Recursive: true

**Sample Runbooks**:
- `example.yml` - Example runbook template

## Configuration File Format

All configurations use **TOML format only** (`.palrun.toml`).

Example generated config:
```toml
# Palrun Configuration for Rust Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "cargo",
    "make",
    "docker",
    "taskfile",
]

ignore_dirs = [
    "target",
    ".git",
    "node_modules",
]

max_depth = 5
recursive = true

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
```

## Customization

After running `palrun setup`, you can customize the generated files:

### Modify `.palrun.toml`
- Add/remove scanners from `enabled` list
- Adjust `ignore_dirs` for your project
- Change `max_depth` and `recursive` settings
- Customize keyboard shortcuts in `[keys]` section

### Edit Runbooks
- Modify sample runbooks in `.palrun/runbooks/`
- Add new runbooks for your workflows
- See [Runbook Guide](runbooks.md) for syntax

### Add More Scanners
Enable additional scanners based on your needs:
```toml
[scanner]
enabled = [
    "npm",
    "cargo",
    "make",
    "docker",
    "git",      # Add Git commands
    "taskfile", # Add Taskfile support
]
```

## Examples

### Set Up a Next.js Project
```bash
cd my-nextjs-app
palrun setup

# Output:
# 🔍 Detecting project type...
# ✓ Detected: Next.js
# 📝 Generating configuration...
# ✓ Created .palrun.toml
# ✓ Created .palrun/runbooks/
#   ✓ Created deploy.yml
#   ✓ Created dev-setup.yml
```

### Set Up a Rust Workspace
```bash
cd my-rust-workspace
palrun setup

# Output:
# 🔍 Detecting project type...
# ✓ Detected: Rust/Cargo
# 📝 Generating configuration...
# ✓ Created .palrun.toml
# ✓ Created .palrun/runbooks/
#   ✓ Created build.yml
#   ✓ Created test.yml
```

### Preview Before Creating
```bash
palrun setup --dry-run

# Shows what would be created without actually creating files
```

### Force Overwrite Existing Config
```bash
palrun setup --force

# Overwrites .palrun.toml without prompting
```

## Troubleshooting

### "Config already exists"
If `.palrun.toml` already exists, you'll be prompted to overwrite:
```
.palrun.toml already exists. Overwrite? [y/N]
```

Use `--force` to skip the prompt:
```bash
palrun setup --force
```

### Wrong Project Type Detected
If the wrong type is detected, you can:
1. Manually edit `.palrun.toml` after generation
2. Use a different template from `examples/configs/`
3. Report an issue with your project structure

### Generated Config Invalid
If you see "Generated config is invalid", this is a bug. Please:
1. Report the issue with your project type
2. Use a manual config from `examples/configs/`

## Next Steps

After running `palrun setup`:

1. **Review Configuration**
   ```bash
   cat .palrun.toml
   ```

2. **Test Command Discovery**
   ```bash
   palrun list
   ```

3. **Try a Runbook**
   ```bash
   palrun runbook deploy --dry-run
   ```

4. **Set Up Shell Integration**
   ```bash
   eval "$(palrun init bash)"
   ```

5. **Start Using Palrun**
   ```bash
   palrun
   ```

## See Also

- [Configuration Reference](../reference/configuration.md)
- [Runbook Guide](runbooks.md)
- [CLI Reference](../reference/cli.md)
- [Examples](../../examples/README.md)

