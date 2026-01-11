# Project Setup Feature

The `palrun setup` command provides intelligent project initialization with automatic detection and configuration generation.

## Overview

Instead of manually creating configuration files, `palrun setup` analyzes your project and generates optimized settings automatically.

## Key Features

### 🔍 Intelligent Detection
Automatically detects project type by analyzing:
- `package.json` → Node.js/NPM
- `next.config.js` → Next.js
- `Cargo.toml` → Rust
- `go.mod` → Go
- `pyproject.toml` → Python
- `nx.json` → Nx Monorepo
- `turbo.json` → Turborepo

### 📝 Optimized Configuration
Generates `.palrun.toml` with:
- **Recommended scanners** for your project type
- **Ignore directories** specific to your stack
- **Scan depth** optimized for your structure
- **Recursive settings** for monorepos

### 📚 Sample Runbooks
Creates `.palrun/runbooks/` with workflows like:
- `deploy.yml` - Build and deployment
- `dev-setup.yml` - Development environment
- `build.yml` - Build configurations
- `test.yml` - Test execution

### ✅ Safety Features
- **Atomic writes** - Write to temp file, then rename
- **Validation** - Parse config before writing
- **Confirmation** - Prompt before overwriting
- **Dry-run mode** - Preview without changes

## Usage

### Basic Setup
```bash
palrun setup
```

### Preview Changes
```bash
palrun setup --dry-run
```

### Force Overwrite
```bash
palrun setup --force
```

### Non-Interactive
```bash
palrun setup --non-interactive
```

## Supported Project Types

| Type | Detection | Scanners | Runbooks |
|------|-----------|----------|----------|
| **Node.js** | `package.json` | npm, docker | deploy, dev-setup |
| **Next.js** | `next.config.*` | npm, docker | deploy, dev-setup |
| **React** | `"react"` in package.json | npm, docker | deploy, dev-setup |
| **Rust** | `Cargo.toml` | cargo, make, docker, taskfile | build, test |
| **Go** | `go.mod` | go, make, docker | build, test |
| **Python** | `pyproject.toml`, `requirements.txt` | python, make, docker | test, dev-setup |
| **Nx** | `nx.json` | npm, nx, docker, make | build-all, deploy |
| **Turborepo** | `turbo.json` | npm, turbo, docker, make | build-all, deploy |
| **Generic** | Fallback | All scanners | example |

## Configuration Examples

### Next.js Project
```toml
[scanner]
enabled = ["npm", "docker"]
ignore_dirs = ["node_modules", ".git", ".next", "out", "dist", "build", "coverage"]
max_depth = 5
recursive = false
```

### Rust Workspace
```toml
[scanner]
enabled = ["cargo", "make", "docker", "taskfile"]
ignore_dirs = ["target", ".git", "node_modules"]
max_depth = 5
recursive = true
```

### Nx Monorepo
```toml
[scanner]
enabled = ["npm", "nx", "docker", "make"]
ignore_dirs = ["node_modules", ".git", "dist", "build", ".nx", "coverage"]
max_depth = 10
recursive = true
```

## Implementation Details

### Architecture
```
src/init/
├── mod.rs        # Public API and setup logic
├── detector.rs   # Project type detection
├── templates.rs  # Config templates (embedded)
└── runbooks.rs   # Runbook templates (embedded)
```

### Detection Priority
1. Framework-specific (Next.js, Nx, Turborepo)
2. Language-specific (Rust, Go, Python)
3. Generic (Node.js)
4. Fallback (Generic)

### Template System
- Templates embedded as const strings
- Compile-time validation
- Simple variable substitution
- Type-safe generation

### Safety Mechanisms
1. **Atomic writes**: Write to `.palrun.toml.tmp`, then rename
2. **Validation**: Parse TOML before writing
3. **Confirmation**: Prompt if file exists
4. **Dry-run**: Preview without side effects

## Testing

### Integration Tests
- 9 project type detection tests
- Config generation tests
- Runbook creation tests
- Dry-run mode tests

### Test Coverage
```bash
cargo test --test integration_setup
```

## See Also

- [Setup Guide](../guides/setup.md) - Detailed usage guide
- [Configuration Reference](../reference/configuration.md) - Config options
- [Runbook Guide](../guides/creating-runbooks.md) - Runbook syntax
- [Examples](../../examples/README.md) - Example configurations

