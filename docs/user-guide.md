# User Guide

Complete guide to all Palrun features and capabilities.

## Table of Contents

- [Interactive Command Palette](#interactive-command-palette)
- [Command Discovery](#command-discovery)
- [Fuzzy Search](#fuzzy-search)
- [Context-Aware Filtering](#context-aware-filtering)
- [AI Integration](#ai-integration)
- [Runbook System](#runbook-system)
- [Shell Integration](#shell-integration)
- [Plugin System](#plugin-system)

## Interactive Command Palette

The command palette is Palrun's primary interface for discovering and executing commands.

### Launching the Palette

```bash
# From any directory
palrun

# Or use the keyboard shortcut (after shell integration)
Ctrl+P
```

### Interface Overview

```
+-----------------------------------------------------------------------------+
| Search: build_                                                              |
+-----------------------------------------------------------------------------+
| > npm run build        [npm]   Build for production                        |
|   cargo build          [cargo] Build the project                           |
|   make build           [make]  Build binary                                |
|   docker compose build [docker] Build containers                           |
+-----------------------------------------------------------------------------+
| 4 commands found | Arrows: navigate | Enter: execute | Esc: quit            |
+-----------------------------------------------------------------------------+
```

**Components:**
- **Search Bar**: Type to filter commands with fuzzy matching
- **Command List**: All matching commands with icons and descriptions
- **Status Bar**: Command count and keyboard shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `Up/Down` | Move selection up/down |
| `Ctrl+N` | Move down (vim-style) |
| `Ctrl+P` | Move up (vim-style) |
| `Home` | Jump to first command |
| `End` | Jump to last command |
| `Page Up/Down` | Scroll by page |

### Actions

| Key | Action |
|-----|--------|
| `Enter` | Execute selected command |
| `Ctrl+U` | Clear search input |
| `Escape` | Quit without executing |
| `Ctrl+C` | Quit without executing |
| `Tab` | Toggle preview panel (if available) |
| `Ctrl+Space` | Toggle context-aware filtering |

### Command Execution

When you press Enter:
1. Palrun exits the TUI
2. The command executes in your current shell
3. You see the command output directly
4. Your shell prompt returns when complete

**Confirmation Prompts:**
Some commands (like destructive operations) may ask for confirmation:
```
Execute 'npm run deploy'? [y/N]
```

## Command Discovery

Palrun automatically discovers commands from your project files.

### Supported Project Types

#### NPM/Yarn/PNPM/Bun

**Detected from:** `package.json`

**Commands discovered:**
- All scripts defined in the `scripts` section
- Package manager automatically detected from lock files

**Example:**
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest",
    "lint": "eslint .",
    "deploy": "vercel --prod"
  }
}
```

**Generated commands:**
- `npm run dev`
- `npm run build`
- `npm run test`
- `npm run lint`
- `npm run deploy`

#### Rust (Cargo)

**Detected from:** `Cargo.toml`

**Commands discovered:**
- `cargo build` - Build the project
- `cargo build --release` - Build optimized
- `cargo test` - Run tests
- `cargo run` - Run the binary
- `cargo clippy` - Run linter
- `cargo check` - Check compilation
- `cargo doc` - Generate documentation

#### Go

**Detected from:** `go.mod`

**Commands discovered:**
- `go build` - Build the project
- `go test` - Run tests
- `go run .` - Run the application
- `go mod tidy` - Clean dependencies
- `go vet` - Run static analysis

#### Python

**Detected from:** `pyproject.toml`, `requirements.txt`, `setup.py`

**Commands discovered:**
- `pytest` - Run tests
- `python -m pytest` - Run tests (module)
- `pip install -r requirements.txt` - Install dependencies
- `poetry install` - Install with Poetry
- `poetry run pytest` - Run tests with Poetry
- `pdm install` - Install with PDM

#### Make

**Detected from:** `Makefile`

**Commands discovered:**
- All targets defined in the Makefile
- Excludes internal targets (starting with `.` or `_`)

**Example Makefile:**
```makefile
.PHONY: all build test clean

all: build test

build:
    gcc -o app main.c

test:
    ./app --test

clean:
    rm -f app
```

**Generated commands:**
- `make all`
- `make build`
- `make test`
- `make clean`

#### Task (Taskfile)

**Detected from:** `Taskfile.yml`

**Commands discovered:**
- All tasks defined in the Taskfile

**Example:**
```yaml
version: '3'

tasks:
  build:
    desc: Build the application
    cmds:
      - go build -o app

  test:
    desc: Run tests
    cmds:
      - go test ./...
```

**Generated commands:**
- `task build`
- `task test`

#### Docker Compose

**Detected from:** `docker-compose.yml`, `compose.yml`

**Commands discovered:**
- `docker compose up` - Start services
- `docker compose up -d` - Start in background
- `docker compose down` - Stop services
- `docker compose logs` - View logs
- `docker compose ps` - List containers
- `docker compose build` - Build images
- `docker compose restart` - Restart services

#### Nx Monorepo

**Detected from:** `nx.json`

**Commands discovered:**
- `nx build <project>` - Build a project
- `nx serve <project>` - Serve a project
- `nx test <project>` - Test a project
- `nx lint <project>` - Lint a project
- `nx affected:build` - Build affected projects
- `nx affected:test` - Test affected projects

#### Turborepo

**Detected from:** `turbo.json`

**Commands discovered:**
- `turbo run build` - Run build task
- `turbo run test` - Run test task
- `turbo run lint` - Run lint task
- All tasks defined in the pipeline

### Recursive Scanning

For monorepos and multi-project workspaces:

```bash
palrun scan --recursive
```

This scans subdirectories up to 5 levels deep, discovering commands from all packages.

### Command Metadata

Each discovered command includes:
- **Name**: The command to execute
- **Description**: What the command does
- **Source**: Where it was discovered (npm, cargo, make, etc.)
- **Icon**: Visual indicator of the source type
- **Path**: Location of the config file
- **Tags**: Searchable metadata

## Fuzzy Search

Palrun uses the nucleo fuzzy matching engine for fast, intelligent search.

### How Fuzzy Search Works

You don't need to type the exact command name. Palrun matches:
- **Subsequences**: "bld" matches "build"
- **Acronyms**: "nrd" matches "npm run dev"
- **Word boundaries**: "test" matches "run-tests"
- **Case-insensitive**: "BUILD" matches "build"

### Search Examples

| You Type | Matches |
|----------|---------|
| `dev` | npm run dev, cargo run --dev |
| `bld` | build, cargo build, npm run build |
| `test` | test, npm run test, cargo test, pytest |
| `nrt` | npm run test |
| `deploy` | npm run deploy, make deploy |

### Search Tips

1. **Start typing immediately** - No need to click or focus
2. **Use abbreviations** - Faster than typing full names
3. **Search descriptions** - Matches command descriptions too
4. **Clear with Ctrl+U** - Quick way to start a new search

## Context-Aware Filtering

Palrun prioritizes commands based on your current directory location.

### How It Works

Commands are sorted by:
1. **Proximity**: Commands from your current directory appear first
2. **Relevance**: Fuzzy match score
3. **Frequency**: Recently used commands (future feature)

### Example

In a monorepo structure:
```
my-monorepo/
├── packages/
│   ├── frontend/
│   │   └── package.json (dev, build, test)
│   └── backend/
│       └── package.json (dev, build, test)
```

When you're in `packages/frontend/`:
- `npm run dev` from frontend appears first
- `npm run dev` from backend appears lower
- Root commands appear last

### Toggle Context Filtering

Press `Ctrl+Space` to toggle context-aware filtering on/off.

## AI Integration

Palrun integrates with AI providers for intelligent command assistance.

### Supported Providers

1. **Claude (Anthropic)** - Cloud-based, requires API key
2. **Ollama** - Local LLM, runs on your machine

### Setup

**Claude:**
```bash
export ANTHROPIC_API_KEY="your-api-key"
```

**Ollama:**
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull llama2
```

Palrun automatically detects available providers and uses Claude first, falling back to Ollama.

### AI Commands

#### Generate Command from Natural Language

```bash
palrun ai gen "start the development server"
```

**Example:**
```bash
$ palrun ai gen "run tests in watch mode"
Generating command...

Generated: npm run test -- --watch

Execute? [y/N]
```

**Execute immediately:**
```bash
palrun ai gen "build for production" --execute
```

#### Explain a Command

```bash
palrun ai explain "npm run build"
```

**Example output:**
```
This command runs the 'build' script defined in package.json.
It typically compiles and bundles your application for production,
optimizing assets and generating static files in the dist/ or build/ directory.
```

#### Diagnose Errors

```bash
palrun ai diagnose "npm run build" "Module not found: Error: Can't resolve 'react'"
```

**Example output:**
```
This error indicates that the 'react' package is not installed or not found
in node_modules. This commonly happens when:

1. Dependencies haven't been installed: Run 'npm install'
2. node_modules was deleted: Run 'npm install' to restore
3. Package.json is missing the react dependency: Add it with 'npm install react'

Recommended fix: npm install
```

#### Check AI Status

```bash
palrun ai status
```

**Example output:**
```
Active AI provider: Claude (Anthropic)
```

### AI Features in Interactive Mode

AI features are also available in the interactive palette (future enhancement).

## Runbook System

Runbooks are executable team documentation written in YAML format.

### What are Runbooks?

Runbooks codify complex workflows as step-by-step scripts with:
- **Variables**: Parameterize commands
- **Conditions**: Skip steps based on criteria
- **Confirmations**: Require user approval
- **Error handling**: Continue or stop on failures

### Creating a Runbook

Create a `.palrun/runbooks/` directory in your project:

```bash
mkdir -p .palrun/runbooks
```

Create a runbook file (e.g., `deploy.yml`):

```yaml
name: Deploy to Production
description: Deploy the application to production environment
version: 1.0.0
author: DevOps Team

variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - staging
      - production
    required: true

  skip_tests:
    type: boolean
    prompt: "Skip tests?"
    default: false

steps:
  - name: Install dependencies
    command: npm install
    description: Install all dependencies

  - name: Run tests
    command: npm run test
    description: Run test suite
    condition: "!skip_tests"
    optional: true

  - name: Build application
    command: npm run build
    description: Build for production

  - name: Deploy
    command: npm run deploy -- --env={{environment}}
    description: Deploy to {{environment}}
    confirm: true
    timeout: 300
```

### Running Runbooks

**List available runbooks:**
```bash
palrun runbook list
```

**Run a runbook:**
```bash
palrun runbook deploy
```

**Dry run (preview steps):**
```bash
palrun runbook deploy --dry-run
```

**Pass variables:**
```bash
palrun runbook deploy --var environment=production --var skip_tests=false
```

### Runbook Schema

#### Top-Level Fields

- `name` (required): Runbook name
- `description` (optional): What the runbook does
- `version` (optional): Runbook version
- `author` (optional): Who created it
- `variables` (optional): Variable definitions
- `steps` (required): List of steps to execute

#### Variable Types

**String:**
```yaml
variables:
  app_name:
    type: string
    prompt: "Enter application name"
    default: "my-app"
    required: true
```

**Boolean:**
```yaml
variables:
  verbose:
    type: boolean
    prompt: "Enable verbose output?"
    default: false
```

**Number:**
```yaml
variables:
  port:
    type: number
    prompt: "Enter port number"
    default: 3000
```

**Select:**
```yaml
variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - development
      - staging
      - production
```

#### Step Fields

- `name` (required): Step name
- `command` (required): Command to execute
- `description` (optional): Step description
- `condition` (optional): Boolean expression to skip step
- `confirm` (optional): Require user confirmation
- `optional` (optional): Continue if step fails
- `continue_on_error` (optional): Don't stop on error
- `timeout` (optional): Maximum execution time in seconds
- `working_dir` (optional): Directory to run command in
- `env` (optional): Environment variables for this step

#### Variable Interpolation

Use `{{variable_name}}` in commands:

```yaml
steps:
  - name: Deploy
    command: deploy --env={{environment}} --version={{version}}
```

#### Conditions

Use simple boolean expressions:

```yaml
steps:
  - name: Run tests
    command: npm test
    condition: "!skip_tests"

  - name: Deploy to production
    command: npm run deploy:prod
    condition: "environment == production"
```

### Runbook Best Practices

1. **Add descriptions**: Help team members understand each step
2. **Use confirmations**: For destructive or critical operations
3. **Set timeouts**: Prevent hanging on long-running commands
4. **Handle errors**: Use `continue_on_error` or `optional` appropriately
5. **Version your runbooks**: Track changes over time
6. **Document variables**: Clear prompts and defaults

## Shell Integration

Shell integration enables keyboard shortcuts and seamless terminal integration.

### Setup

See [Installation Guide - Shell Integration](installation.md#shell-integration-recommended) for setup instructions.

### Features

**Keyboard Shortcut:**
- Press `Ctrl+P` from anywhere to open Palrun

**Command Execution:**
- Commands execute in your current shell
- Environment variables are preserved
- Working directory is maintained

### Shell-Specific Features

**Bash/Zsh:**
- Command history integration
- Alias expansion
- Function support

**Fish:**
- Native Fish syntax support
- Abbreviation expansion

**PowerShell:**
- Module and cmdlet support
- PowerShell-specific commands

## Plugin System

Palrun supports custom scanners through a plugin architecture.

### Available Plugins

Example plugins are included in `examples/plugins/`:

- **cargo-scanner**: Enhanced Cargo.toml scanning
- **composer-scanner**: PHP Composer support
- **gradle-scanner**: Gradle build tool support
- **maven-scanner**: Maven build tool support
- **poetry-scanner**: Python Poetry support

### Using Plugins

Plugins are automatically loaded from:
- `~/.config/palrun/plugins/`
- `.palrun/plugins/` (project-specific)

### Creating Custom Plugins

See the plugin examples for implementation details. Plugins implement the `Scanner` trait:

```rust
pub trait Scanner {
    fn name(&self) -> &str;
    fn scan(&self, path: &Path) -> Result<Vec<Command>>;
    fn can_scan(&self, path: &Path) -> bool;
}
```

## Advanced Features

### Command Aliases

Create shortcuts for frequently used commands (future feature).

### Command History

Track and search previously executed commands (future feature).

### Team Collaboration

Share runbooks and configurations with your team through version control.

### Multi-Project Workspaces

Palrun works seamlessly in monorepos and multi-project setups, discovering commands from all packages.

## Tips and Tricks

1. **Use fuzzy search**: Type abbreviations instead of full names
2. **Context matters**: Commands from your current directory appear first
3. **Keyboard shortcuts**: Learn the shortcuts for faster navigation
4. **Runbooks for workflows**: Codify complex processes
5. **AI for discovery**: Use AI to find the right command
6. **JSON output**: Integrate with other tools using `--format json`
7. **Direct execution**: Use `palrun exec` in scripts and CI/CD

## Next Steps

- [Configuration Guide](configuration.md) - Customize Palrun
- [Troubleshooting](troubleshooting.md) - Solve common issues
- [FAQ](faq.md) - Frequently asked questions

