# Keyboard Shortcuts Reference

Complete reference for all keyboard shortcuts in Palrun.

## Global Shortcuts

Available when shell integration is enabled.

| Shortcut | Action | Shell Support |
|----------|--------|---------------|
| `Ctrl+P` | Open Palrun command palette | Bash, Zsh, Fish, PowerShell |

## Interactive Palette Shortcuts

Available when the command palette is open.

### Navigation

| Shortcut | Action |
|----------|--------|
| `Up` | Move selection up |
| `Down` | Move selection down |
| `Ctrl+P` | Move selection up (vim-style) |
| `Ctrl+N` | Move selection down (vim-style) |
| `Home` | Jump to first command |
| `End` | Jump to last command |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |

### Actions

| Shortcut | Action |
|----------|--------|
| `Enter` | Execute selected command |
| `Escape` | Quit without executing |
| `Ctrl+C` | Quit without executing |

### Search

| Shortcut | Action |
|----------|--------|
| `Ctrl+U` | Clear search input |
| `Backspace` | Delete last character |
| Any character | Add to search |

### View Controls

| Shortcut | Action |
|----------|--------|
| `Tab` | Toggle preview panel (if available) |
| `Ctrl+Space` | Toggle context-aware filtering |

## Command Line Shortcuts

Standard terminal shortcuts work in Palrun:

| Shortcut | Action |
|----------|--------|
| `Ctrl+L` | Clear screen (before opening Palrun) |
| `Ctrl+D` | Exit terminal |
| `Ctrl+Z` | Suspend process |

## Customizing Shortcuts

Keyboard shortcuts are currently not customizable. This is a planned feature for a future release.

## Shortcut Conflicts

If `Ctrl+P` conflicts with another tool:

1. Disable the conflicting tool's shortcut
2. Use `palrun` command directly instead
3. Wait for customizable shortcuts in a future release

## Platform-Specific Notes

### macOS

All shortcuts work as documented. `Ctrl` refers to the Control key, not Command (⌘).

### Linux

All shortcuts work as documented.

### Windows

- PowerShell: All shortcuts work
- CMD: Limited support (use PowerShell instead)
- WSL: Full support

## Next Steps

- [CLI Commands Reference](cli-reference.md)
- [Configuration Reference](config-reference.md)

