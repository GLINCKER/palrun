# Changelog

All notable changes to Palrun will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
