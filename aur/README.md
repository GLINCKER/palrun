# AUR Packages for Palrun

This directory contains PKGBUILD files for publishing Palrun to the Arch User Repository (AUR).

## Packages

### palrun-bin (Recommended)

Pre-built binary package - fastest installation.

```bash
yay -S palrun-bin
# or
paru -S palrun-bin
```

### palrun-git

Build from the latest git source.

```bash
yay -S palrun-git
# or
paru -S palrun-git
```

## Manual Installation

If you prefer to build manually:

```bash
# Clone the AUR package
git clone https://aur.archlinux.org/palrun-bin.git
cd palrun-bin

# Review and build
makepkg -si
```

## Publishing to AUR

To publish or update on AUR:

1. Clone/update the AUR repository:
   ```bash
   git clone ssh://aur@aur.archlinux.org/palrun-bin.git
   ```

2. Copy the PKGBUILD:
   ```bash
   cp PKGBUILD /path/to/aur/palrun-bin/
   ```

3. Update checksums:
   ```bash
   cd /path/to/aur/palrun-bin
   updpkgsums
   ```

4. Generate .SRCINFO:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

5. Commit and push:
   ```bash
   git add PKGBUILD .SRCINFO
   git commit -m "Update to version X.Y.Z"
   git push
   ```

## Post-Installation

After installation, enable shell integration:

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
