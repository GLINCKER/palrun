# Finding and Running Commands

Learn how to discover and execute commands in your projects with Palrun.

## Opening the Command Palette

**With shell integration:**
```bash
Ctrl+P
```

**Without shell integration:**
```bash
palrun
```

## The Command Palette Interface

```
+-----------------------------------------------------------------------------+
| Search: _                                                                   |
+-----------------------------------------------------------------------------+
| > npm run dev          [npm]   Start development server                    |
|   npm run build        [npm]   Build for production                        |
|   npm run test         [npm]   Run test suite                              |
|   cargo build          [cargo] Build the project                           |
|   cargo test           [cargo] Run tests                                   |
+-----------------------------------------------------------------------------+
| 5 commands found | Arrows: navigate | Enter: execute | Esc: quit            |
+-----------------------------------------------------------------------------+
```

**Parts:**
- **Search bar**: Type to filter commands
- **Command list**: All matching commands with icons
- **Status bar**: Shortcuts and command count

## Navigating Commands

| Key | Action |
|-----|--------|
| `Up/Down` | Move selection |
| `Ctrl+N` | Move down (vim-style) |
| `Ctrl+P` | Move up (vim-style) |
| `Enter` | Execute selected command |
| `Escape` | Quit without executing |
| `Ctrl+C` | Quit without executing |

## Searching Commands

Just start typing - no need to click or focus the search bar.

**Examples:**

Type `dev` to find:
- npm run dev
- npm run dev:server
- cargo run --dev

Type `test` to find:
- npm run test
- cargo test
- pytest
- go test

Type `build` to find:
- npm run build
- cargo build
- make build
- go build

## Understanding Command Sources

Each command shows its source with an icon:

- `[npm]` - From package.json scripts
- `[cargo]` - Rust Cargo commands
- `[make]` - Makefile targets
- `[docker]` - Docker Compose commands
- `[go]` - Go commands
- `[python]` - Python commands
- `[task]` - Taskfile commands
- `[nx]` - Nx monorepo commands
- `[turbo]` - Turborepo commands

## What Commands Are Discovered?

Palrun automatically finds commands from:

### NPM/Yarn/PNPM/Bun Projects
**File:** `package.json`

All scripts in the `scripts` section:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }
}
```

### Rust Projects
**File:** `Cargo.toml`

Standard Cargo commands:
- cargo build
- cargo test
- cargo run
- cargo clippy
- cargo check

### Makefiles
**File:** `Makefile`

All targets (except internal ones starting with `.` or `_`)

### Docker Projects
**File:** `docker-compose.yml`

Common Docker Compose commands:
- docker compose up
- docker compose down
- docker compose logs
- docker compose ps

### Other Project Types

- **Go** (`go.mod`) - go build, test, run
- **Python** (`pyproject.toml`) - pytest, pip commands
- **Task** (`Taskfile.yml`) - task commands
- **Nx** (`nx.json`) - nx commands
- **Turborepo** (`turbo.json`) - turbo commands

## Executing Commands

1. Navigate to the command you want
2. Press `Enter`
3. Palrun exits and the command runs
4. You see the output directly in your terminal

**Example:**
```bash
# You press Ctrl+P
# Type "dev"
# Press Enter
# Palrun runs: npm run dev
```

## Commands with Confirmation

Some commands ask for confirmation before running:

```
Execute 'npm run deploy'? [y/N]
```

Type `y` and press Enter to proceed, or `n` to cancel.

## Viewing All Commands

To see all discovered commands without the interactive palette:

```bash
palrun list
```

**Filter by source:**
```bash
palrun list --source npm
palrun list --source cargo
palrun list --source make
```

**JSON output:**
```bash
palrun list --format json
```

## Scanning Your Project

To see what Palrun would discover:

```bash
palrun scan
```

**For monorepos:**
```bash
palrun scan --recursive
```

## Context-Aware Sorting

Commands from your current directory appear first.

**Example in a monorepo:**
```
my-monorepo/
├── packages/
│   ├── frontend/  <- You are here
│   └── backend/
```

When you open Palrun from `packages/frontend/`:
1. Frontend commands appear first
2. Backend commands appear lower
3. Root commands appear last

**Toggle context filtering:**
Press `Ctrl+Space` in the palette.

## Tips

1. **Start typing immediately** - Search is auto-focused
2. **Use abbreviations** - "bld" matches "build"
3. **Clear search** - Press `Ctrl+U`
4. **Quick access** - Use `Ctrl+P` from anywhere
5. **Check what's available** - Run `palrun scan` in new projects

## Next Steps

- [Using Fuzzy Search](fuzzy-search.md)
- [Direct Command Execution](direct-execution.md)
- [Working with Monorepos](monorepos.md)

