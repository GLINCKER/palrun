# Shell Integration Setup

Set up keyboard shortcuts to launch Palrun instantly from anywhere in your terminal.

## What is Shell Integration?

Shell integration adds a keyboard shortcut (`Ctrl+P`) that opens Palrun from anywhere in your terminal, without typing `palrun` each time.

## Setup by Shell

### Bash

Add this line to your `~/.bashrc`:

```bash
eval "$(palrun init bash)"
```

**Apply changes:**
```bash
source ~/.bashrc
```

Or open a new terminal window.

### Zsh

Add this line to your `~/.zshrc`:

```bash
eval "$(palrun init zsh)"
```

**Apply changes:**
```bash
source ~/.zshrc
```

Or open a new terminal window.

### Fish

Add this line to your `~/.config/fish/config.fish`:

```fish
palrun init fish | source
```

**Apply changes:**
```fish
source ~/.config/fish/config.fish
```

Or open a new terminal window.

### PowerShell

Add this line to your PowerShell profile:

```powershell
palrun init powershell | Invoke-Expression
```

**Find your profile location:**
```powershell
echo $PROFILE
```

**Apply changes:**
```powershell
. $PROFILE
```

Or open a new PowerShell window.

## Testing the Integration

After setup, press `Ctrl+P` in your terminal. Palrun should open immediately.

If it doesn't work:
1. Make sure you reloaded your shell configuration
2. Check that Palrun is in your PATH: `which palrun`
3. Try running `palrun` directly first

## Using the Keyboard Shortcut

Once set up:

1. Press `Ctrl+P` from anywhere in your terminal
2. Palrun opens with all available commands
3. Type to search, press Enter to execute
4. The command runs in your current directory

## Removing Shell Integration

To remove the integration, delete or comment out the line you added to your shell configuration file:

**Bash/Zsh:**
```bash
# eval "$(palrun init bash)"
```

**Fish:**
```fish
# palrun init fish | source
```

**PowerShell:**
```powershell
# palrun init powershell | Invoke-Expression
```

Then reload your shell.

## Troubleshooting

### Ctrl+P doesn't work

**Check if integration is loaded:**
```bash
# Bash/Zsh
type palrun-widget

# Fish
functions palrun-widget
```

If you get "not found", the integration didn't load. Check your shell config file.

### Conflicts with other tools

Some tools also use `Ctrl+P`. If you have conflicts, you can:

1. Use `palrun` directly instead of the shortcut
2. Customize the keybinding (future feature)
3. Disable the conflicting tool's shortcut

### Integration breaks terminal

If your terminal behaves strangely after adding integration:

1. Remove the integration line from your config
2. Reload your shell
3. Report the issue on [GitHub](https://github.com/GLINCKER/palrun/issues)

## Next Steps

- [Finding and Running Commands](finding-commands.md)
- [Using Fuzzy Search](fuzzy-search.md)
- [Setting Up AI](ai-setup.md)

