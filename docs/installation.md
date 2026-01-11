# Installation Guide

This guide covers all the ways to install Palrun on your system.

## System Requirements

- **Operating System**: macOS, Linux, or Windows
- **Architecture**: x64 or ARM64
- **Rust** (if building from source): 1.75 or later
- **Node.js** (if installing via npm): 14.0 or later

## Installation Methods

### Method 1: Using Cargo (Recommended)

The easiest way to install Palrun if you have Rust installed:

```bash
cargo install palrun
```

This will download, compile, and install the latest version of Palrun.

**Verify installation:**

```bash
palrun --version
```

### Method 2: Using NPM

Install Palrun globally using npm:

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

The npm package downloads the pre-built binary for your platform during installation.

**Verify installation:**

```bash
palrun --version
# or use the short alias
pal --version
```

### Method 3: From Source

Clone the repository and build from source:

```bash
# Clone the repository
git clone https://github.com/GLINCKER/palrun.git
cd palrun

# Build and install
cargo install --path .
```

**For development:**

```bash
# Build in debug mode
cargo build

# Run directly
cargo run

# Run with arguments
cargo run -- list
```

### Method 4: Homebrew (macOS/Linux)

Install using Homebrew:

```bash
# Add the tap
brew tap GLINCKER/palrun

# Install palrun
brew install palrun
```

**Update:**
```bash
brew upgrade palrun
```

### Method 5: Download Pre-built Binaries

Pre-built binaries are available from the [GitHub releases page](https://github.com/GLINCKER/palrun/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| macOS | Intel (x64) | `palrun-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon (ARM64) | `palrun-aarch64-apple-darwin.tar.gz` |
| Linux | x64 | `palrun-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `palrun-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x64 | `palrun-x86_64-pc-windows-msvc.zip` |

**Installation:**
```bash
# Download and extract (example for macOS Apple Silicon)
curl -LO https://github.com/GLINCKER/palrun/releases/latest/download/palrun-aarch64-apple-darwin.tar.gz
tar -xzf palrun-aarch64-apple-darwin.tar.gz
sudo mv palrun pal /usr/local/bin/

# Verify checksums (recommended)
curl -LO https://github.com/GLINCKER/palrun/releases/latest/download/checksums-sha256.txt
sha256sum -c checksums-sha256.txt
```

### Method 6: Arch Linux (AUR)

For Arch Linux users:

```bash
# Using yay
yay -S palrun-bin

# Or from git
yay -S palrun-git
```

## Platform-Specific Notes

### macOS

**Intel Macs:**
All installation methods work without additional configuration.

**Apple Silicon (M1/M2/M3):**
All installation methods work natively on ARM64.

**Security Note:**
If you download a pre-built binary, you may need to allow it in System Preferences:
```bash
xattr -d com.apple.quarantine /path/to/palrun
```

### Linux

**Dependencies:**
Most Linux distributions have all required dependencies. If you encounter issues, ensure you have:

```bash
# Debian/Ubuntu
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc

# Arch
sudo pacman -S base-devel
```

**Installation Location:**
Cargo installs binaries to `~/.cargo/bin/`. Ensure this is in your PATH:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Windows

**PowerShell Execution Policy:**
You may need to adjust your execution policy to run the shell integration:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**PATH Configuration:**
Cargo installs to `%USERPROFILE%\.cargo\bin`. This should be added to your PATH automatically.

## Post-Installation Setup

### Shell Integration (Recommended)

Enable keyboard shortcuts for quick access to Palrun:

**Bash:**
```bash
echo 'eval "$(palrun init bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
echo 'eval "$(palrun init zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Fish:**
```fish
echo 'palrun init fish | source' >> ~/.config/fish/config.fish
source ~/.config/fish/config.fish
```

**PowerShell:**
```powershell
Add-Content $PROFILE 'palrun init powershell | Invoke-Expression'
. $PROFILE
```

After shell integration, press `Ctrl+P` to open Palrun from anywhere in your terminal.

### Shell Completions (Optional)

Generate shell completions for better command-line experience:

**Bash:**
```bash
palrun completions bash | sudo tee /etc/bash_completion.d/palrun
```

**Zsh:**
```bash
palrun completions zsh > ~/.zfunc/_palrun
# Add to ~/.zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

**Fish:**
```bash
palrun completions fish > ~/.config/fish/completions/palrun.fish
```

## Updating Palrun

### Cargo Installation

```bash
cargo install palrun --force
```

### NPM Installation

```bash
npm update -g @glinr/palrun
```

## Uninstalling Palrun

### Cargo Installation

```bash
cargo uninstall palrun
```

### NPM Installation

```bash
npm uninstall -g @glinr/palrun
```

Don't forget to remove the shell integration from your shell configuration file.

## Next Steps

Now that Palrun is installed, check out the [Getting Started Guide](getting-started.md) to learn how to use it!

