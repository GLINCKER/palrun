# CLI Commands Reference

Complete reference for all Palrun command-line commands and options.

## Global Options

Available for all commands:

```bash
palrun [OPTIONS] [COMMAND]
```

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose logging |
| `-h, --help` | Show help information |
| `-V, --version` | Show version number |

## Commands

### `palrun` (no command)

Launch the interactive command palette.

```bash
palrun
```

**Options:**
- `--non-interactive` - List commands and exit (same as `palrun list`)

**Examples:**
```bash
palrun                    # Open interactive palette
palrun --non-interactive  # List commands
palrun --verbose          # Open with debug logging
```

---

### `palrun list`

List all discovered commands.

```bash
palrun list [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `-f, --format <FORMAT>` | Output format: `text` or `json` | `text` |
| `-s, --source <SOURCE>` | Filter by source type | (none) |

**Examples:**
```bash
palrun list                    # List all commands
palrun list --format json      # Output as JSON
palrun list --source npm       # Only npm scripts
palrun list --source cargo     # Only cargo commands
```

**Output (text):**
```
npm run dev - Start development server
npm run build - Build for production
cargo build - Build the project
Total: 3 commands
```

**Output (json):**
```json
[
  {
    "name": "npm run dev",
    "command": "npm run dev",
    "description": "Start development server",
    "source": "npm"
  }
]
```

---

### `palrun exec`

Execute a command directly by name.

```bash
palrun exec <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Command name or pattern to execute

**Options:**
- `-y, --yes` - Skip confirmation prompt

**Examples:**
```bash
palrun exec build           # Execute build command
palrun exec "npm test"      # Execute specific command
palrun exec deploy -y       # Execute without confirmation
```

---

### `palrun scan`

Scan the project and show discovered commands.

```bash
palrun scan [PATH] [OPTIONS]
```

**Arguments:**
- `[PATH]` - Directory to scan (default: current directory)

**Options:**
- `-r, --recursive` - Scan subdirectories recursively

**Examples:**
```bash
palrun scan                 # Scan current directory
palrun scan --recursive     # Scan recursively
palrun scan packages/app    # Scan specific directory
```

**Output:**
```
Discovered 5 commands in "."

NPM:
  - npm run dev
  - npm run build
  - npm run test

CARGO:
  - cargo build
  - cargo test
```

---

### `palrun runbook`

Run a runbook.

```bash
palrun runbook <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Runbook name (without .yml extension)

**Options:**
- `--dry-run` - Show steps without executing
- `--var <KEY=VALUE>` - Set variable value (can be used multiple times)

**Examples:**
```bash
palrun runbook deploy                           # Run deploy runbook
palrun runbook deploy --dry-run                 # Preview steps
palrun runbook deploy --var env=production      # Set variable
palrun runbook deploy --var env=prod --var skip_tests=true
```

---

### `palrun init`

Generate shell integration script.

```bash
palrun init <SHELL>
```

**Arguments:**
- `<SHELL>` - Shell type: `bash`, `zsh`, `fish`, or `powershell`

**Examples:**
```bash
eval "$(palrun init bash)"              # Bash
eval "$(palrun init zsh)"               # Zsh
palrun init fish | source               # Fish
palrun init powershell | Invoke-Expression  # PowerShell
```

---

### `palrun completions`

Generate shell completion scripts.

```bash
palrun completions <SHELL>
```

**Arguments:**
- `<SHELL>` - Shell type: `bash`, `zsh`, `fish`, or `powershell`

**Examples:**
```bash
palrun completions bash > /etc/bash_completion.d/palrun
palrun completions zsh > ~/.zfunc/_palrun
palrun completions fish > ~/.config/fish/completions/palrun.fish
```

---

### `palrun config`

Show configuration.

```bash
palrun config [OPTIONS]
```

**Options:**
- `--path` - Show config file path only

**Examples:**
```bash
palrun config         # Show current configuration
palrun config --path  # Show config file location
```

---

### `palrun ai` (requires `ai` feature)

AI-powered command assistance.

```bash
palrun ai <OPERATION>
```

#### `palrun ai gen`

Generate a command from natural language.

```bash
palrun ai gen <PROMPT> [OPTIONS]
```

**Arguments:**
- `<PROMPT>` - Natural language description

**Options:**
- `-x, --execute` - Execute the generated command immediately

**Examples:**
```bash
palrun ai gen "start the dev server"
palrun ai gen "run tests in watch mode"
palrun ai gen "build for production" --execute
```

#### `palrun ai explain`

Explain what a command does.

```bash
palrun ai explain <COMMAND>
```

**Arguments:**
- `<COMMAND>` - The command to explain

**Examples:**
```bash
palrun ai explain "npm run build"
palrun ai explain "cargo test --release"
```

#### `palrun ai diagnose`

Diagnose why a command failed.

```bash
palrun ai diagnose <COMMAND> <ERROR>
```

**Arguments:**
- `<COMMAND>` - The command that failed
- `<ERROR>` - The error message

**Examples:**
```bash
palrun ai diagnose "npm test" "Module not found: react"
palrun ai diagnose "cargo build" "error: could not compile"
```

#### `palrun ai status`

Show active AI provider.

```bash
palrun ai status
```

**Output:**
```
Active AI provider: Claude (Anthropic)
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Command not found |
| 130 | Interrupted (Ctrl+C) |

## Environment Variables

See [Environment Variables Reference](environment-variables.md) for all environment variables.

## Next Steps

- [Keyboard Shortcuts](keyboard-shortcuts.md)
- [Configuration Reference](config-reference.md)
- [Runbook Schema](runbook-schema.md)

