# Palrun Documentation

Welcome to the Palrun documentation! Palrun is a project-aware command palette for your terminal with AI-powered intelligence.

## Quick Start

**New to Palrun?** Start here:
1. [Installation Guide](installation.md) - Install Palrun on your system
2. [Getting Started](getting-started.md) - Your first steps with Palrun
3. [Shell Integration Setup](guides/shell-integration.md) - Set up `Ctrl+P` shortcut

**Ready to use?** Jump to:
- [Finding and Running Commands](guides/finding-commands.md)
- [Setting Up AI Features](guides/ai-setup.md)
- [Creating Runbooks](guides/creating-runbooks.md)

## Documentation Navigation

### 📚 How-to Guides (Task-Oriented)

Practical guides for specific tasks:

- **Setup & Installation**
  - [Installing Palrun](installation.md)
  - [Shell Integration Setup](guides/shell-integration.md)
  - [Setting Up AI (Claude/Ollama)](guides/ai-setup.md)

- **Using Commands**
  - [Finding and Running Commands](guides/finding-commands.md)
  - [Working with Monorepos](guides/monorepos.md)

- **Runbooks**
  - [Creating Runbooks](guides/creating-runbooks.md)

[→ Browse all guides](guides/README.md)

### 📖 Reference (Information-Oriented)

Technical documentation for looking up details:

- [CLI Commands Reference](reference/cli-reference.md) - All commands and options
- [Keyboard Shortcuts](reference/keyboard-shortcuts.md) - All keyboard shortcuts
- [Supported Project Types](reference/project-types.md) - All supported project types

[→ Browse all reference docs](reference/README.md)

### 🔧 Additional Resources

- [Configuration Guide](configuration.md) - Customize Palrun
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [FAQ](faq.md) - Frequently asked questions

## What is Palrun?

Palrun automatically discovers every command available in your project and presents them in a blazing-fast fuzzy-searchable interface. Stop memorizing commands - whether you're working with npm, cargo, make, docker, or any of 9+ supported project types, Palrun knows what you can run.

```
+-----------------------------------------------------------------------------+
|                              PALRUN v0.1.0                                  |
+-----------------------------------------------------------------------------+
|                                                                             |
|   Project Scan --> Command Discovery --> Fuzzy Search --> Execute          |
|                    (9+ types)             (nucleo)         (context-aware)  |
|                                                                             |
|   Cargo.toml   --> cargo build, test    --> "bui"     --> cargo build      |
|   package.json --> npm run dev, test    --> "dev"     --> npm run dev      |
|   Makefile     --> make all, clean      --> "cle"     --> make clean       |
|                                                                             |
+-----------------------------------------------------------------------------+
```

## Key Features

- **Project-Aware Discovery**: Automatically detects commands from 9+ project types
- **Fuzzy Search**: Lightning-fast fuzzy matching powered by nucleo engine
- **Context-Aware Sorting**: Commands sorted by proximity to your current directory
- **AI Integration**: Natural language command generation with Claude or Ollama
- **Runbook System**: Executable team documentation in YAML format
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **Shell Integration**: Keyboard shortcuts for instant access
- **Plugin System**: Extensible architecture for custom scanners

## Supported Project Types

| Project Type | Config Files | Commands Generated |
|-------------|--------------|-------------------|
| NPM/Yarn/PNPM/Bun | `package.json` | npm/yarn/pnpm/bun scripts |
| Rust | `Cargo.toml` | cargo build, test, run, clippy |
| Go | `go.mod` | go build, test, run |
| Python | `pyproject.toml`, `requirements.txt` | pytest, pip, poetry, pdm |
| Make | `Makefile` | make targets |
| Task | `Taskfile.yml` | task commands |
| Docker | `docker-compose.yml` | docker compose up/down/logs |
| Nx | `nx.json` | nx build, serve, test |
| Turborepo | `turbo.json` | turbo run tasks |

## Documentation Philosophy

This documentation follows the [Diátaxis framework](https://diataxis.fr/), organizing content by user needs:

- **Guides** (How-to) - Task-oriented guides for accomplishing specific goals
- **Reference** - Information-oriented technical documentation for looking up details

This structure helps you find what you need quickly, whether you're learning a new feature or looking up a specific command option.

## Getting Help

- **GitHub Issues**: [Report bugs or request features](https://github.com/GLINCKER/palrun/issues)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/GLINCKER/palrun/discussions)
- **Documentation**: You're reading it!

## Contributing

We welcome contributions! If you find errors in this documentation or want to improve it, please submit a pull request on GitHub.

## License

Palrun is released under the MIT License - free for personal and commercial use.

