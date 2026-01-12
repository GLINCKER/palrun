<div align="center">

# PALRUN

**Stop memorizing commands. Start shipping.**

A blazing-fast command palette for your terminal with multi-provider AI intelligence.

[![Crates.io](https://img.shields.io/crates/v/palrun?style=for-the-badge&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/palrun)
[![Downloads](https://img.shields.io/crates/d/palrun?style=for-the-badge&logo=rust&logoColor=white&color=orange)](https://crates.io/crates/palrun)
[![CI](https://img.shields.io/github/actions/workflow/status/GLINCKER/palrun/ci.yml?style=for-the-badge&logo=github&label=CI)](https://github.com/GLINCKER/palrun/actions)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)

<br>

```bash
brew install GLINCKER/palrun/palrun
```

**Works on Mac, Windows, and Linux.**

<br>

```
┌────────────────────────────────────────────────────────────────────┐
│  PALRUN                                          [rust] main ✓    │
├────────────────────────────────────────────────────────────────────┤
│  > build                                                          │
├────────────────────────────────────────────────────────────────────┤
│  → cargo build              Build the project                     │
│    cargo build --release    Build optimized binary                │
│    npm run build            Bundle frontend                       │
│    make build               Run makefile target                   │
├────────────────────────────────────────────────────────────────────┤
│  ↑↓ navigate  ⏎ execute  tab preview  esc quit                    │
└────────────────────────────────────────────────────────────────────┘
```

<br>

*"Finally stopped grepping through package.json to find scripts."*

*"The AI diagnostics saved me 2 hours debugging a cryptic npm error."*

*"Just type 3 letters and hit enter. That's it."*

<br>

[Why Palrun](#why-palrun) · [Install](#install) · [How It Works](#how-it-works) · [AI Features](#ai-features) · [Commands](#commands)

</div>

---

## Why Palrun

Every project has commands scattered everywhere. npm scripts in package.json. Cargo commands in Cargo.toml. Make targets. Docker compose. Task runners. You end up:

- Scrolling through 50 npm scripts to find the right one
- Forgetting that obscure cargo command you used last week
- Grepping through config files looking for targets
- Context-switching to docs constantly

Palrun fixes this. It scans your project, finds every command, and gives you a fuzzy-searchable interface. Type 2-3 characters, hit enter, done.

The AI features are optional but powerful — generate commands from natural language, explain what complex commands do, diagnose errors without leaving your terminal.

---

## Install

```bash
# Homebrew (macOS/Linux) - Recommended
brew install GLINCKER/palrun/palrun

# Cargo
cargo install palrun

# NPM
npm install -g @glincker/palrun

# Download binary
# https://github.com/GLINCKER/palrun/releases
```

Then just run:

```bash
palrun
```

---

## How It Works

### 1. Auto-Discovery

Palrun scans your project and finds commands from:

| Source | Files | What It Finds |
|--------|-------|---------------|
| **Node.js** | `package.json` | npm/yarn/pnpm/bun scripts |
| **Rust** | `Cargo.toml` | cargo build, test, run, clippy |
| **Go** | `go.mod` | go build, test, run |
| **Python** | `pyproject.toml` | pytest, poetry, pdm commands |
| **Make** | `Makefile` | All make targets |
| **Docker** | `docker-compose.yml` | compose up/down/logs |
| **Task** | `Taskfile.yml` | task commands |
| **Monorepos** | `nx.json`, `turbo.json` | nx/turbo commands |

### 2. Fuzzy Search

Type a few characters, palrun finds the match:

- `bui` → `cargo build`
- `td` → `npm run test:debug`
- `dcu` → `docker compose up`

Powered by [nucleo](https://github.com/helix-editor/nucleo) — the same engine behind Helix editor.

### 3. Context-Aware

Commands are ranked by proximity to your current directory. Working in `src/api/`? API-related commands appear first.

---

## AI Features

Palrun supports multiple AI providers with automatic fallback:

| Provider | API Key Env Var | Best For |
|----------|-----------------|----------|
| **Claude** | `ANTHROPIC_API_KEY` | Complex reasoning |
| **OpenAI** | `OPENAI_API_KEY` | Fast, general purpose |
| **Azure OpenAI** | `AZURE_OPENAI_API_KEY` | Enterprise deployments |
| **Grok** | `XAI_API_KEY` | Alternative option |
| **Ollama** | None (local) | Offline, privacy |

### Generate Commands

```bash
palrun ai "run tests with coverage"
# → cargo test --all-features -- --nocapture
```

### Explain Commands

```bash
palrun ai explain "git rebase -i HEAD~5"
# Explains what interactive rebase does
```

### Diagnose Errors

```bash
palrun ai diagnose "npm ERR! peer dep missing: react@18"
# Suggests: npm install react@18 --save-peer
```

### Configuration

Set keys via environment variables or config file:

```bash
# Environment (recommended)
export ANTHROPIC_API_KEY="sk-ant-..."

# Or in ~/.config/palrun/palrun.toml
[ai.claude]
api_key = "sk-ant-..."
```

---

## Commands

### Interactive Mode

```bash
palrun              # Launch TUI
palrun list         # List all commands
palrun list --json  # JSON output for scripting
```

### Direct Execution

```bash
palrun exec build        # Run by name
palrun exec "npm test"   # Run specific command
palrun exec build -y     # Skip confirmation
```

### Project Setup

```bash
palrun setup              # Initialize for your project
palrun setup --dry-run    # Preview changes
```

### IDE Integration

Generate slash commands for AI coding tools:

```bash
palrun slash generate claude   # For Claude Code
palrun slash generate cursor   # For Cursor
palrun slash generate aider    # For Aider
```

---

## Shell Integration

Add keyboard shortcuts to your shell:

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

---

## Configuration

Create `~/.config/palrun/palrun.toml`:

```toml
[general]
confirm_dangerous = true

[ui]
theme = "default"
show_preview = true
show_icons = true

[ai]
provider = "claude"
fallback_enabled = true

[ai.claude]
model = "claude-sonnet-4-20250514"
```

For API keys, use environment variables or the system config file — never commit secrets to your repo.

---

## Why Not Just Use...

| Alternative | Palrun Advantage |
|-------------|------------------|
| `cat package.json \| jq` | One command, fuzzy search, instant |
| fzf + custom scripts | Zero setup, auto-discovers everything |
| IDE command palette | Works in terminal, any project type |
| Memorizing commands | You have better things to remember |

**For AI tools:** Pre-computed command index saves ~1500 tokens per query. AI doesn't need to scan your project.

---

## Development

```bash
# Build
cargo build --release

# Test (527 tests)
cargo test --all-features

# Run locally
cargo run -- list
```

### Git Hooks

Local quality gates (auto-installed):

```bash
./.githooks/install.sh
# pre-commit: format, clippy, build
# pre-push: tests, security audit
# commit-msg: conventional commits
```

---

## Roadmap

- [x] Multi-provider AI (Claude, OpenAI, Azure, Grok, Ollama)
- [x] Agentic workflow system
- [x] IDE slash command generation
- [x] Hierarchical config with secrets management
- [ ] MCP server mode for AI agents
- [ ] Chat history and session persistence
- [ ] Streaming AI responses
- [ ] VS Code extension

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
git clone https://github.com/GLINCKER/palrun.git
cd palrun
cargo test
cargo run
```

---

## License

MIT License — free for personal and commercial use.

---

<div align="center">

**Your terminal has hundreds of commands. Palrun finds the right one instantly.**

[GitHub](https://github.com/GLINCKER/palrun) · [Issues](https://github.com/GLINCKER/palrun/issues) · [Discussions](https://github.com/GLINCKER/palrun/discussions)

Built by [GLINCKER](https://glincker.com)

</div>
