# Getting Started with Palrun

This guide will help you get up and running with Palrun in just a few minutes.

## Prerequisites

Make sure you have [installed Palrun](installation.md) on your system.

## Step 1: Set Up Your Project

Navigate to your project directory and run:

```bash
palrun setup
```

This will:
- **Detect your project type** (Node.js, Rust, Python, etc.)
- **Create `.palrun.toml`** with optimized settings
- **Generate sample runbooks** in `.palrun/runbooks/`
- **Suggest next steps** for your project

Example output:
```
🔍 Detecting project type...

✓ Detected: Next.js

📝 Generating configuration...

✓ Created .palrun.toml
✓ Created .palrun/runbooks/
  ✓ Created deploy.yml
  ✓ Created dev-setup.yml

✨ Project initialized successfully!
```

**Options:**
```bash
palrun setup --dry-run          # Preview without creating files
palrun setup --force            # Overwrite existing files
palrun setup --non-interactive  # Use defaults without prompts
```

See the [Setup Guide](guides/setup.md) for more details.

## Step 2: Your First Command Palette

Run Palrun to see all available commands:

```bash
palrun
```

You'll see the interactive command palette with all available commands from your project:

```
+-----------------------------------------------------------------------------+
| Search: _                                                                   |
+-----------------------------------------------------------------------------+
| > npm run dev          [npm]  Start development server                     |
|   npm run build        [npm]  Build for production                         |
|   npm run test         [npm]  Run test suite                               |
|   cargo build          [cargo] Build the project                           |
|   cargo test           [cargo] Run tests                                   |
|   make all             [make]  Build everything                            |
+-----------------------------------------------------------------------------+
| 6 commands found | Use arrows to navigate, Enter to execute, Esc to quit   |
+-----------------------------------------------------------------------------+
```

**Navigation:**
- Use **arrow keys** or **Ctrl+N/P** to move up and down
- Type to **fuzzy search** commands
- Press **Enter** to execute the selected command
- Press **Escape** or **Ctrl+C** to quit

## Basic Usage Examples

### Example 1: Running a Development Server

1. Open Palrun in your project directory:
   ```bash
   palrun
   ```

2. Type "dev" to filter commands:
   ```
   Search: dev
   > npm run dev          [npm]  Start development server
   ```

3. Press **Enter** to execute

### Example 2: Running Tests

1. Open Palrun:
   ```bash
   palrun
   ```

2. Type "test":
   ```
   Search: test
   > npm run test         [npm]  Run test suite
     cargo test           [cargo] Run tests
   ```

3. Select the test command you want and press **Enter**

### Example 3: Building Your Project

1. Open Palrun and type "build"
2. Select the appropriate build command
3. Press **Enter** to execute

## Command Line Interface

Palrun offers several CLI commands for non-interactive use:

### List All Commands

View all discovered commands in your project:

```bash
palrun list
```

**Output as JSON:**
```bash
palrun list --format json
```

**Filter by source:**
```bash
palrun list --source npm
palrun list --source cargo
palrun list --source make
```

### Scan Project

Preview what commands Palrun would discover:

```bash
palrun scan
```

**Scan recursively (for monorepos):**
```bash
palrun scan --recursive
```

### Execute Directly

Run a command by name without opening the interactive palette:

```bash
palrun exec build
```

**Skip confirmation:**
```bash
palrun exec build -y
```

**Execute with fuzzy matching:**
```bash
palrun exec "npm test"
```

## Keyboard Shortcuts

Once you've set up [shell integration](installation.md#shell-integration-recommended), you can use these shortcuts:

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Open Palrun command palette |

**Within the palette:**

| Key | Action |
|-----|--------|
| `Enter` | Execute selected command |
| `Up/Down` | Navigate command list |
| `Ctrl+N/P` | Navigate (vim-style) |
| `Ctrl+U` | Clear search input |
| `Escape` | Quit |
| `Tab` | Toggle preview (if available) |
| `Ctrl+Space` | Toggle context-aware filtering |

## Understanding Command Sources

Palrun automatically detects commands from various project types:

### NPM/Yarn/PNPM/Bun Projects

Palrun reads `package.json` and discovers all scripts:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }
}
```

Commands appear as: `npm run dev`, `npm run build`, `npm run test`

Palrun automatically detects your package manager from lock files:
- `package-lock.json` → npm
- `yarn.lock` → yarn
- `pnpm-lock.yaml` → pnpm
- `bun.lockb` → bun

### Rust Projects

Palrun reads `Cargo.toml` and provides standard Cargo commands:
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo run` - Run the binary
- `cargo clippy` - Run linter
- `cargo check` - Check compilation

### Makefiles

Palrun parses `Makefile` and discovers all targets:

```makefile
all: build test

build:
    gcc -o app main.c

test:
    ./app --test
```

Commands appear as: `make all`, `make build`, `make test`

### Docker Compose

Palrun reads `docker-compose.yml` and provides common commands:
- `docker compose up` - Start services
- `docker compose down` - Stop services
- `docker compose logs` - View logs
- `docker compose ps` - List containers

### Other Project Types

Palrun also supports:
- **Go** (`go.mod`) - go build, test, run
- **Python** (`pyproject.toml`, `requirements.txt`) - pytest, pip commands
- **Task** (`Taskfile.yml`) - task commands
- **Nx** (`nx.json`) - nx build, serve, test
- **Turborepo** (`turbo.json`) - turbo run tasks

## Working with Monorepos

Palrun automatically detects monorepo structures and scans all packages:

```bash
# Scan recursively to find all packages
palrun scan --recursive
```

Commands from all packages will be available in the palette, with context-aware sorting to prioritize commands from your current directory.

## Next Steps

Now that you know the basics, explore these advanced features:

- [AI-Powered Commands](user-guide.md#ai-integration) - Generate commands from natural language
- [Runbooks](user-guide.md#runbook-system) - Create executable team documentation
- [Configuration](configuration.md) - Customize Palrun to your workflow
- [Shell Integration](installation.md#shell-integration-recommended) - Set up keyboard shortcuts

## Common Workflows

### Daily Development

1. Press `Ctrl+P` to open Palrun
2. Type "dev" and press Enter to start your dev server
3. Open a new terminal, press `Ctrl+P`
4. Type "test" and press Enter to run tests in watch mode

### Building and Deploying

1. Press `Ctrl+P`
2. Type "build" and press Enter
3. Wait for build to complete
4. Press `Ctrl+P` again
5. Type "deploy" and press Enter

### Running Tests

1. Press `Ctrl+P`
2. Type "test" to see all test commands
3. Select the specific test suite you want
4. Press Enter to execute

## Tips and Tricks

1. **Fuzzy Search**: You don't need to type the exact command name. "bld" will match "build"
2. **Context-Aware**: Commands from your current directory appear first
3. **Quick Access**: Use `Ctrl+P` from anywhere in your terminal
4. **JSON Output**: Use `palrun list --format json` to integrate with other tools
5. **Direct Execution**: Use `palrun exec <name> -y` in scripts for automation

## Getting Help

If you run into issues:

1. Check the [Troubleshooting Guide](troubleshooting.md)
2. Read the [FAQ](faq.md)
3. Search [GitHub Issues](https://github.com/GLINCKER/palrun/issues)
4. Ask in [GitHub Discussions](https://github.com/GLINCKER/palrun/discussions)

