# Homebrew Tap for Palrun

This is the official Homebrew tap for [Palrun](https://github.com/GLINCKER/palrun) - an AI command palette for your terminal.

> **Repository Setup:** This directory should be published as a separate repository named `homebrew-tap` at `github.com/GLINCKER/homebrew-tap`

## Installation

```bash
# Add the tap (the homebrew- prefix is handled automatically)
brew tap GLINCKER/tap

# Install palrun
brew install palrun
```

Or install directly without adding the tap:

```bash
brew install GLINCKER/tap/palrun
```

> **Note:** The `homebrew-` prefix in the repository name (`homebrew-tap`) is required by Homebrew but omitted in commands.

## Usage

After installation, you can use either `palrun` or `pal`:

```bash
# Open the command palette
pal

# Scan for commands
pal scan

# List available commands
pal list
```

## Shell Integration

Add shell integration for the best experience:

**Bash** (~/.bashrc):
```bash
eval "$(palrun init bash)"
```

**Zsh** (~/.zshrc):
```bash
eval "$(palrun init zsh)"
```

**Fish** (~/.config/fish/config.fish):
```fish
palrun init fish | source
```

## Updating

```bash
brew upgrade palrun
```

## Uninstalling

```bash
brew uninstall palrun
brew untap GLINCKER/tap
```

## Issues

If you encounter any issues, please report them at:
https://github.com/GLINCKER/palrun/issues
