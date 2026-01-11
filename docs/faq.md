# Frequently Asked Questions

Common questions about Palrun and their answers.

## General Questions

### What is Palrun?

Palrun is a project-aware command palette for your terminal. It automatically discovers commands from your project files (package.json, Cargo.toml, Makefile, etc.) and presents them in a fast, fuzzy-searchable interface. Think of it as Spotlight or Command Palette, but for your terminal commands.

### Why should I use Palrun?

- **Stop memorizing commands**: No need to remember `npm run dev` vs `yarn dev` vs `pnpm dev`
- **Faster workflow**: Press `Ctrl+P`, type a few letters, hit Enter
- **Project-aware**: Automatically adapts to any project type
- **AI-powered**: Generate commands from natural language
- **Team collaboration**: Share runbooks for complex workflows

### How is Palrun different from other command runners?

- **Automatic discovery**: No configuration needed - Palrun finds commands automatically
- **Multi-language**: Works with npm, cargo, make, docker, and 9+ project types
- **Fuzzy search**: Fast, intelligent matching powered by nucleo
- **AI integration**: Natural language command generation
- **Runbook system**: Executable team documentation

### Is Palrun free?

Yes! Palrun is open source under the MIT License, free for personal and commercial use.

## Installation & Setup

### Which installation method should I use?

- **Have Rust?** Use `cargo install palrun` (recommended)
- **Prefer npm?** Use `npm install -g @glinr/palrun`
- **Building from source?** Clone and `cargo install --path .`

All methods install the same binary.

### Do I need to install Rust to use Palrun?

No, if you install via npm. The npm package downloads a pre-built binary for your platform.

### How do I update Palrun?

**Cargo:**
```bash
cargo install palrun --force
```

**NPM:**
```bash
npm update -g @glinr/palrun
```

### Can I use Palrun without shell integration?

Yes! Just run `palrun` directly. Shell integration only adds the `Ctrl+P` keyboard shortcut.

## Usage Questions

### How does Palrun find commands?

Palrun scans your project directory for configuration files:
- `package.json` → npm/yarn/pnpm/bun scripts
- `Cargo.toml` → cargo commands
- `Makefile` → make targets
- `docker-compose.yml` → docker compose commands
- And 5+ more project types

### Can Palrun work with monorepos?

Yes! Use `palrun scan --recursive` to discover commands from all packages. Commands are sorted by proximity to your current directory.

### Does Palrun modify my project files?

No. Palrun only reads your project files, never modifies them.

### Can I use Palrun in CI/CD?

Yes! Use non-interactive commands:
```bash
palrun exec build -y
palrun list --format json
```

### How do I exclude certain directories from scanning?

Configure exclusions in `~/.config/palrun/config.toml`:
```toml
[scanner]
exclude_patterns = ["node_modules", "target", ".git", "dist"]
```

## AI Features

### Do I need an API key to use AI features?

For Claude, yes. For Ollama (local), no.

**Claude:**
```bash
export ANTHROPIC_API_KEY="your-api-key"
```

**Ollama (free, local):**
```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama2
```

### Which AI provider should I use?

- **Claude**: More accurate, requires API key, costs money
- **Ollama**: Free, runs locally, requires more setup

Palrun tries Claude first, falls back to Ollama automatically.

### How much does Claude cost?

Claude charges per token. Typical Palrun AI commands cost $0.001-0.01 per request. See [Anthropic pricing](https://www.anthropic.com/pricing) for details.

### Can I use AI features offline?

Yes, with Ollama. Install Ollama and pull a model:
```bash
ollama pull llama2
```

### Are my commands sent to AI providers?

Only when you explicitly use AI features (`palrun ai gen`, etc.). Regular command discovery and execution is entirely local.

## Runbooks

### What are runbooks?

Runbooks are executable team documentation in YAML format. They codify complex workflows as step-by-step scripts with variables, conditions, and confirmations.

### Where do I put runbooks?

Create a `.palrun/runbooks/` directory in your project:
```bash
mkdir -p .palrun/runbooks
```

Add YAML files with your runbooks.

### Can I share runbooks with my team?

Yes! Commit `.palrun/runbooks/` to version control. Your team can run the same workflows.

### Do runbooks require AI?

No. Runbooks are independent of AI features.

## Configuration

### Where is the config file?

**macOS/Linux:** `~/.config/palrun/config.toml`
**Windows:** `%APPDATA%\palrun\config.toml`

Find it with: `palrun config --path`

### Can I have project-specific configuration?

Yes! Create `.palrun/config.toml` in your project root. It merges with global config.

### How do I reset configuration to defaults?

Delete or rename your config file:
```bash
mv ~/.config/palrun/config.toml ~/.config/palrun/config.toml.bak
```

## Performance

### Is Palrun fast?

Yes! Palrun uses:
- Rust for native performance
- Nucleo for fast fuzzy matching
- Caching to avoid repeated scans

Typical startup time: <100ms

### Can I make Palrun faster?

Yes, enable caching and reduce scan depth:
```toml
[scanner]
cache_enabled = true
max_depth = 3
exclude_patterns = ["node_modules", "target"]
```

### Does Palrun slow down my terminal?

No. Shell integration is lightweight and only activates when you press `Ctrl+P`.

## Compatibility

### Which operating systems are supported?

- macOS (Intel and Apple Silicon)
- Linux (x64 and ARM64)
- Windows (x64)

### Which shells are supported?

- Bash
- Zsh
- Fish
- PowerShell

### Which project types are supported?

- NPM/Yarn/PNPM/Bun (package.json)
- Rust (Cargo.toml)
- Go (go.mod)
- Python (pyproject.toml, requirements.txt)
- Make (Makefile)
- Task (Taskfile.yml)
- Docker (docker-compose.yml)
- Nx (nx.json)
- Turborepo (turbo.json)

### Can I add support for other project types?

Yes! Palrun has a plugin system. See `examples/plugins/` for examples.

## Troubleshooting

### Palrun shows "No commands found"

1. Verify you're in a project directory
2. Check that project files exist (package.json, etc.)
3. Try `palrun scan` to see what's detected
4. Check exclusion patterns in config

### Commands don't execute

1. Test the command manually
2. Check shell configuration
3. Verify file permissions
4. Enable verbose logging: `palrun --verbose`

### TUI looks broken

1. Check terminal compatibility: `echo $TERM`
2. Use a modern terminal (iTerm2, Alacritty, Windows Terminal)
3. Try a different theme in config

See [Troubleshooting Guide](troubleshooting.md) for more solutions.

## Contributing

### How can I contribute?

- Report bugs on [GitHub Issues](https://github.com/GLINCKER/palrun/issues)
- Suggest features in [Discussions](https://github.com/GLINCKER/palrun/discussions)
- Submit pull requests
- Improve documentation
- Create plugins for new project types

### Where is the source code?

https://github.com/GLINCKER/palrun

### How do I report a bug?

1. Search existing issues
2. Create a new issue with:
   - Palrun version
   - Operating system
   - Steps to reproduce
   - Expected vs actual behavior

## Privacy & Security

### Does Palrun collect data?

No. Palrun runs entirely locally and collects no telemetry or analytics.

### Is it safe to use Palrun with private projects?

Yes. Palrun only reads local files and doesn't send data anywhere (except when you explicitly use AI features).

### What data is sent to AI providers?

When using AI features:
- Your prompt
- Project context (file names, available commands)
- No file contents or sensitive data

## Future Features

### What features are planned?

- Command history and analytics
- Custom command aliases
- Enhanced preview panel
- Team collaboration features
- More AI capabilities
- Additional project type support

### Can I request a feature?

Yes! Open a discussion on [GitHub Discussions](https://github.com/GLINCKER/palrun/discussions).

### When will feature X be released?

Check the [GitHub Issues](https://github.com/GLINCKER/palrun/issues) and [Roadmap](https://github.com/GLINCKER/palrun/projects) for planned features and timelines.

## Getting Help

### Where can I get help?

1. Read the [documentation](README.md)
2. Check [Troubleshooting Guide](troubleshooting.md)
3. Search [GitHub Issues](https://github.com/GLINCKER/palrun/issues)
4. Ask in [GitHub Discussions](https://github.com/GLINCKER/palrun/discussions)

### How do I contact the developers?

- GitHub Issues for bugs
- GitHub Discussions for questions
- Email: hello@glinr.com

## Next Steps

- [Installation Guide](installation.md) - Get started
- [User Guide](user-guide.md) - Learn all features
- [Configuration](configuration.md) - Customize Palrun
- [Troubleshooting](troubleshooting.md) - Solve issues

