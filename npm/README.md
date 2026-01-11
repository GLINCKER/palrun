# @glinr/palrun

> AI command palette for your terminal - discover and run project commands instantly

This is the npm wrapper package for [Palrun](https://github.com/GLINCKER/palrun). It downloads the pre-built binary for your platform during installation.

## Installation

```bash
npm install -g @glinr/palrun
```

Or with yarn:

```bash
yarn global add @glinr/palrun
```

Or with pnpm:

```bash
pnpm add -g @glinr/palrun
```

## Usage

After installation, you can run:

```bash
palrun
# or
pal
```

## Features

- Fuzzy search across all your project commands
- Auto-discovers npm scripts, Cargo targets, Make tasks, and more
- AI-powered command generation (Claude + Ollama)
- Runbooks for team workflows
- Works with monorepos (npm, pnpm, yarn, Nx, Turbo)

## Alternative Installation Methods

If npm installation fails, you can install Palrun using:

### Cargo (Rust)

```bash
cargo install palrun
```

### From Source

```bash
git clone https://github.com/GLINCKER/palrun.git
cd palrun
cargo build --release
```

### Direct Download

Download pre-built binaries from the [GitHub Releases](https://github.com/GLINCKER/palrun/releases) page.

## Supported Platforms

- macOS (x64, ARM64)
- Linux (x64, ARM64)
- Windows (x64)

## License

MIT - see [LICENSE](https://github.com/GLINCKER/palrun/blob/main/LICENSE)

## Links

- [GitHub Repository](https://github.com/GLINCKER/palrun)
- [Documentation](https://glinr.com/palrun)
- [Issues](https://github.com/GLINCKER/palrun/issues)
