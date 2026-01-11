# Troubleshooting Guide

Solutions to common issues and problems with Palrun.

## Installation Issues

### Cargo Install Fails

**Problem:** `cargo install palrun` fails with compilation errors.

**Solutions:**

1. **Update Rust:**
   ```bash
   rustup update stable
   ```

2. **Check Rust version:**
   ```bash
   rustc --version  # Should be 1.75 or later
   ```

3. **Clear cargo cache:**
   ```bash
   cargo clean
   rm -rf ~/.cargo/registry/cache
   cargo install palrun
   ```

4. **Install with verbose output:**
   ```bash
   cargo install palrun -v
   ```

### NPM Install Fails

**Problem:** `npm install -g @glinr/palrun` fails to download binary.

**Solutions:**

1. **Check Node version:**
   ```bash
   node --version  # Should be 14.0 or later
   ```

2. **Clear npm cache:**
   ```bash
   npm cache clean --force
   npm install -g @glinr/palrun
   ```

3. **Check network/proxy:**
   ```bash
   npm config get proxy
   npm config get https-proxy
   ```

4. **Install with verbose logging:**
   ```bash
   npm install -g @glinr/palrun --verbose
   ```

### Binary Not Found After Installation

**Problem:** `palrun: command not found` after installation.

**Solutions:**

1. **Check PATH (Cargo):**
   ```bash
   echo $PATH | grep .cargo/bin
   # If not present, add to shell config:
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. **Check PATH (NPM):**
   ```bash
   npm config get prefix
   # Ensure this directory is in your PATH
   ```

3. **Verify installation:**
   ```bash
   which palrun
   ls -la ~/.cargo/bin/palrun  # Cargo
   ls -la $(npm config get prefix)/bin/palrun  # NPM
   ```

## Runtime Issues

### No Commands Found

**Problem:** Palrun shows "No commands found" in a project.

**Solutions:**

1. **Verify project files exist:**
   ```bash
   ls package.json Cargo.toml Makefile  # etc.
   ```

2. **Check current directory:**
   ```bash
   pwd
   # Make sure you're in the project root
   ```

3. **Try explicit scan:**
   ```bash
   palrun scan
   palrun scan --recursive
   ```

4. **Check scanner exclusions:**
   ```bash
   palrun config
   # Look for exclude_patterns
   ```

5. **Enable verbose logging:**
   ```bash
   palrun --verbose
   ```

### Commands Not Executing

**Problem:** Commands appear in the list but don't execute.

**Solutions:**

1. **Check command syntax:**
   ```bash
   palrun list --format json
   # Verify the command field is correct
   ```

2. **Test command manually:**
   ```bash
   # Copy the command from palrun list and run it
   npm run build
   ```

3. **Check shell configuration:**
   ```bash
   echo $SHELL
   palrun config
   # Verify shell setting matches your shell
   ```

4. **Check permissions:**
   ```bash
   ls -la package.json
   # Ensure files are readable
   ```

### TUI Display Issues

**Problem:** TUI looks broken or has rendering issues.

**Solutions:**

1. **Check terminal compatibility:**
   ```bash
   echo $TERM
   # Should be xterm-256color or similar
   ```

2. **Update terminal:**
   - Use a modern terminal emulator
   - iTerm2, Alacritty, Windows Terminal recommended

3. **Try different theme:**
   ```toml
   # ~/.config/palrun/config.toml
   [theme]
   highlight_color = "white"
   ```

4. **Resize terminal:**
   - Ensure terminal is at least 80x24 characters

### Slow Performance

**Problem:** Palrun is slow to start or search.

**Solutions:**

1. **Enable caching:**
   ```toml
   [scanner]
   cache_enabled = true
   cache_ttl = 600
   ```

2. **Reduce scan depth:**
   ```toml
   [scanner]
   max_depth = 3
   ```

3. **Add exclusions:**
   ```toml
   [scanner]
   exclude_patterns = [
     "node_modules",
     "target",
     ".git",
     "dist",
     "build"
   ]
   ```

4. **Limit results:**
   ```toml
   [ui]
   max_results = 30
   ```

## AI Integration Issues

### AI Provider Not Available

**Problem:** `palrun ai status` shows "No AI provider available".

**Solutions:**

1. **For Claude - Set API key:**
   ```bash
   export ANTHROPIC_API_KEY="your-api-key"
   # Add to ~/.bashrc or ~/.zshrc for persistence
   ```

2. **For Ollama - Install and start:**
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Pull a model
   ollama pull llama2
   
   # Verify it's running
   ollama list
   ```

3. **Check provider setting:**
   ```toml
   [ai]
   provider = "auto"  # or "claude" or "ollama"
   ```

### AI Requests Timeout

**Problem:** AI commands timeout or take too long.

**Solutions:**

1. **Increase timeout:**
   ```toml
   [ai]
   timeout = 60  # Increase from default 30
   ```

2. **Use faster model:**
   ```toml
   [ai]
   claude_model = "claude-3-haiku-20240307"  # Faster than Sonnet
   ```

3. **Check network:**
   ```bash
   curl -I https://api.anthropic.com
   ```

4. **For Ollama - Use smaller model:**
   ```bash
   ollama pull llama2:7b  # Smaller, faster
   ```

### AI Generates Wrong Commands

**Problem:** AI generates incorrect or irrelevant commands.

**Solutions:**

1. **Be more specific:**
   ```bash
   # Instead of: "build"
   palrun ai gen "build the frontend for production"
   ```

2. **Provide context:**
   ```bash
   # Run from the correct directory
   cd packages/frontend
   palrun ai gen "start dev server"
   ```

3. **Use explain first:**
   ```bash
   palrun ai explain "npm run build"
   # Understand what commands do before generating
   ```

## Shell Integration Issues

### Keyboard Shortcut Not Working

**Problem:** `Ctrl+P` doesn't open Palrun.

**Solutions:**

1. **Verify shell integration:**
   ```bash
   # Check if init command is in your shell config
   cat ~/.bashrc | grep palrun
   cat ~/.zshrc | grep palrun
   ```

2. **Re-run init:**
   ```bash
   # Bash
   eval "$(palrun init bash)"
   
   # Zsh
   eval "$(palrun init zsh)"
   ```

3. **Reload shell:**
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   # Or open a new terminal
   ```

4. **Check for conflicts:**
   ```bash
   # Some tools may bind Ctrl+P
   # Try the command directly:
   palrun
   ```

### Shell Integration Breaks Terminal

**Problem:** Terminal behaves strangely after shell integration.

**Solutions:**

1. **Remove integration temporarily:**
   ```bash
   # Comment out in ~/.bashrc or ~/.zshrc:
   # eval "$(palrun init bash)"
   ```

2. **Check shell compatibility:**
   ```bash
   echo $SHELL
   # Ensure using supported shell
   ```

3. **Update shell:**
   ```bash
   # Bash
   bash --version  # Should be 4.0+
   
   # Zsh
   zsh --version  # Should be 5.0+
   ```

## Configuration Issues

### Config Not Loading

**Problem:** Configuration changes don't take effect.

**Solutions:**

1. **Check config location:**
   ```bash
   palrun config --path
   ```

2. **Verify TOML syntax:**
   ```bash
   # Use a TOML validator or check for errors
   cat ~/.config/palrun/config.toml
   ```

3. **Check file permissions:**
   ```bash
   ls -la ~/.config/palrun/config.toml
   chmod 644 ~/.config/palrun/config.toml
   ```

4. **Test with minimal config:**
   ```toml
   [theme]
   highlight_color = "cyan"
   ```

### Invalid Configuration Error

**Problem:** Palrun shows "Invalid configuration" error.

**Solutions:**

1. **Check TOML syntax:**
   - Ensure proper quoting
   - Check for typos in keys
   - Verify array/table syntax

2. **Validate values:**
   - Color names must be valid
   - Numbers must be in valid ranges
   - Booleans must be true/false

3. **Reset to defaults:**
   ```bash
   mv ~/.config/palrun/config.toml ~/.config/palrun/config.toml.bak
   palrun  # Will use defaults
   ```

## Platform-Specific Issues

### macOS: Permission Denied

**Problem:** "Permission denied" when running Palrun.

**Solutions:**

1. **Check quarantine attribute:**
   ```bash
   xattr -l $(which palrun)
   # If quarantined:
   xattr -d com.apple.quarantine $(which palrun)
   ```

2. **Check executable permission:**
   ```bash
   chmod +x $(which palrun)
   ```

### Linux: Missing Dependencies

**Problem:** Error about missing shared libraries.

**Solutions:**

1. **Install build essentials:**
   ```bash
   # Debian/Ubuntu
   sudo apt-get install build-essential
   
   # Fedora
   sudo dnf install gcc
   
   # Arch
   sudo pacman -S base-devel
   ```

### Windows: PowerShell Execution Policy

**Problem:** Cannot run shell integration script.

**Solutions:**

1. **Set execution policy:**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

2. **Verify policy:**
   ```powershell
   Get-ExecutionPolicy -List
   ```

## Getting More Help

If you're still experiencing issues:

1. **Enable verbose logging:**
   ```bash
   palrun --verbose
   ```

2. **Check GitHub Issues:**
   - Search existing issues: https://github.com/GLINCKER/palrun/issues
   - Create a new issue with:
     - Palrun version (`palrun --version`)
     - Operating system and version
     - Shell and version
     - Steps to reproduce
     - Error messages or logs

3. **Ask in Discussions:**
   - https://github.com/GLINCKER/palrun/discussions

4. **Include diagnostic info:**
   ```bash
   palrun --version
   echo $SHELL
   uname -a
   palrun config
   ```

## Next Steps

- [FAQ](faq.md) - Frequently asked questions
- [Configuration](configuration.md) - Customize Palrun
- [User Guide](user-guide.md) - Learn all features

