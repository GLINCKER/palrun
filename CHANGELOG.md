# Changelog

All notable changes to Palrun will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Project Setup Command**
  - New `palrun setup` command for intelligent project initialization
  - Automatic project type detection (Node.js, Next.js, React, Rust, Go, Python, Nx, Turborepo)
  - Generates optimized `.palrun.toml` configuration based on project type
  - Creates `.palrun/runbooks/` directory with sample workflows
  - Supports `--dry-run`, `--force`, and `--non-interactive` flags
  - Atomic file operations (write to temp, then rename)
  - Configuration validation before writing
  - 9 integration tests for all project types
  - Comprehensive setup guide documentation

- **Security Module**
  - Command validation with dangerous pattern detection (15+ patterns)
  - Shell injection prevention and path traversal detection
  - Privilege escalation warnings (sudo, setuid)
  - Fork bomb and reverse shell detection
  - Environment variable sanitization with credential redaction
  - File permission checking (world-writable, group-writable)
  - 24 security integration tests
  - SECURITY.md policy document

- **CI/CD Security Scanning**
  - cargo-audit integration for RustSec vulnerability scanning
  - cargo-deny for license and dependency security checks
  - deny.toml configuration for allowed licenses
  - Weekly Dependabot updates for Rust, npm, and GitHub Actions

- **Retry Logic & Circuit Breakers**
  - `RetryConfig` with exponential backoff and jitter
  - Presets for quick, network, and API operations
  - `CircuitBreaker` for preventing cascading failures
  - Configurable failure thresholds and reset timeouts
  - 11 unit tests for reliability

- **Dry-Run Mode**
  - Global `--dry-run` flag for all commands
  - `pal exec --dry-run` shows command details without executing
  - Shows name, command, working directory, source, and tags
  - Works with existing runbook `--dry-run` flag

- **Performance Benchmarks**
  - Criterion benchmarks for scanner performance
  - Mock data fixtures for realistic testing
  - Fuzzy search benchmarks with large command sets

- **MCP (Model Context Protocol) Integration**
  - New `pal mcp servers` command - List configured MCP servers
  - New `pal mcp tools` command - List available tools from all servers
  - New `pal mcp call` command - Call a specific MCP tool
  - New `pal mcp start` command - Start all configured MCP servers
  - New `pal mcp config` command - Show MCP configuration
  - JSON-RPC 2.0 protocol implementation
  - Multi-server management with tool routing
  - Dynamic tool discovery from external services

- **AI Agent with Tool Use**
  - New `pal ai agent <task>` command - Run AI agent with tools
  - Agentic loop with automatic tool execution
  - Ollama AgentProvider with function calling via chat API
  - MCP tool executor for external tool integration
  - Shell executor for command execution
  - Composite executor for routing tool calls
  - Context-aware prompts with project information

- **Webhooks & REST API**
  - Outgoing webhook support with event filtering
  - HMAC-SHA256 signature for webhook verification
  - Exponential backoff retry on webhook failures
  - REST API types for remote control
  - API key authentication (X-API-Key header)
  - Rate limiting (configurable per minute)
  - CORS support for web integrations

- **Issue Tracker Integrations**
  - New `pal linear list` command - View Linear issues
  - New `pal linear create` command - Create Linear issues
  - New `pal linear teams` command - List Linear teams
  - New `pal linear search` command - Search Linear issues
  - New `pal linear stats` command - Show Linear statistics
  - GitHub Issues integration with create, list, search, close
  - Issue labels and milestone support

- **npm Wrapper Package**
  - New `@glinr/palrun` npm package for easy installation
  - Automatic binary download for platform
  - Support for macOS, Linux, Windows (x64, ARM64)

- **GitHub Actions CI/CD Integration**
  - New `pal ci status` command - Show CI status for current branch
  - New `pal ci workflows` command - List available GitHub Actions workflows
  - New `pal ci runs` command - List recent workflow runs with status
  - New `pal ci trigger` command - Trigger a workflow dispatch
  - New `pal ci rerun` command - Re-run a failed workflow
  - New `pal ci cancel` command - Cancel a running workflow
  - New `pal ci open` command - Open CI page in browser
  - Auto-detection of GitHub repository from git remote
  - Comprehensive error handling for API failures

- **Notification Services**
  - New `pal notify slack` command - Send messages to Slack webhooks
  - New `pal notify discord` command - Send messages to Discord webhooks
  - New `pal notify webhook` command - Send to generic HTTP webhooks
  - New `pal notify test` command - Test notification endpoints
  - Rich message support with titles, colors, and fields
  - Environment variable support for webhook URLs

- **Built-in Command Scanner**
  - Palrun's own CLI commands now appear in the TUI command list
  - Includes plugin, ai, ci, env, secrets, and config commands
  - Commands are searchable and executable from the palette

- **Plugin Registry & Discovery**
  - New `pal plugin search <query>` command - Search for plugins in the registry
  - New `pal plugin browse` command - List all available plugins with sorting
  - New `pal plugin update [name]` command - Check and install plugin updates
  - New `pal plugin clear-cache` command - Clear the registry cache
  - Enhanced `pal plugin install <name>` - Install plugins from registry by name
  - Registry caching with 1-hour TTL for faster repeated lookups
  - SHA256 checksum verification for downloaded plugins
  - Compatibility checking against current API version
  - Popularity-based sorting (downloads + stars)

- **TUI Enhancements**
  - Directory browsing with Tab completion
  - Ghost text autocomplete (VSCode-style)
  - Slash commands (`/help`, `/history`, `/analytics`, `/quit`, etc.)
  - Pass-through mode for shell commands (cd, ls, pwd)
  - Auto-scroll when navigating command list
  - Terminal-style prompt with current directory
  - User info display in preview panel
  - Smart status bar with contextual information

### Changed

- Removed decorative emojis from search UI (terminal-native feel)
- Plugin install now checks registry if file doesn't exist locally

## [0.1.0] - 2026-01-10

### Added

- **Core Features**
  - Interactive TUI with fuzzy search powered by nucleo
  - Command execution with working directory support
  - Context-aware filtering with proximity-based scoring
  - Shell integration for bash, zsh, fish, and PowerShell

- **Project Scanners (9 total)**
  - NPM/Yarn/PNPM/Bun - package.json scripts with package manager detection
  - Cargo - Rust build commands with feature flag support
  - Go - go.mod based commands with cmd/ package detection
  - Python - pyproject.toml (Poetry, PDM) and requirements.txt
  - Makefile - target extraction with .PHONY support
  - Taskfile - Taskfile.yml task discovery
  - Docker Compose - service commands from docker-compose.yml
  - Nx - workspace targets from nx.json and project.json
  - Turborepo - pipeline tasks from turbo.json (v1 and v2 formats)

- **AI Integration (scaffold)**
  - Claude API integration structure
  - Context builder for command generation
  - Prompt templates

- **Runbook System**
  - YAML-based runbook schema
  - Step execution with variable interpolation
  - Conditional step execution

- **CLI Commands**
  - `palrun` - Launch interactive command palette
  - `palrun list` - List discovered commands (text/JSON output)
  - `palrun scan` - Preview discovered commands
  - `palrun exec` - Execute command directly
  - `palrun runbook` - Run YAML runbooks
  - `palrun init` - Shell integration setup
  - `palrun completions` - Generate shell completions
  - `palrun config` - Show configuration

- **Testing**
  - 143 unit tests covering all core modules
  - 21 CLI integration tests
  - CI/CD with GitHub Actions (Linux, macOS, Windows)

- **Distribution**
  - Cross-platform release builds (5 targets)
  - SHA256 checksums for releases
  - Automated GitHub releases

### Technical Details

- Built with Rust using ratatui for TUI, clap for CLI
- Fuzzy search powered by nucleo
- Configuration via TOML files
- Shell scripts for keyboard shortcut integration

[Unreleased]: https://github.com/GLINCKER/palrun/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/GLINCKER/palrun/releases/tag/v0.1.0
