# Configuration Guide

Customize Palrun to match your workflow and preferences.

## Configuration File

Palrun uses a TOML configuration file located at:

**macOS/Linux:**
```
~/.config/palrun/config.toml
```

**Windows:**
```
%APPDATA%\palrun\config.toml
```

### Finding Your Config Path

```bash
palrun config --path
```

### Viewing Current Configuration

```bash
palrun config
```

## Configuration Schema

### Complete Example

```toml
# Palrun Configuration File

[theme]
highlight_color = "cyan"
selected_color = "green"
border_color = "blue"
text_color = "white"
dim_color = "gray"

[ui]
show_icons = true
show_descriptions = true
max_results = 50
preview_enabled = false

[shell]
default = "bash"
preserve_env = true

[scanner]
exclude_patterns = ["node_modules", "target", ".git", "dist", "build"]
max_depth = 5
follow_symlinks = false
scan_hidden = false

[search]
case_sensitive = false
smart_case = true
min_score = 0.3

[ai]
provider = "auto"  # auto, claude, ollama, none
claude_model = "claude-3-5-sonnet-20241022"
ollama_model = "llama2"
timeout = 30

[execution]
confirm_destructive = true
timeout = 300
shell_args = []

[keybindings]
quit = ["Esc", "Ctrl+C"]
execute = ["Enter"]
clear_input = ["Ctrl+U"]
toggle_preview = ["Tab"]
toggle_context = ["Ctrl+Space"]
```

## Configuration Sections

### Theme

Customize the visual appearance of the TUI.

```toml
[theme]
highlight_color = "cyan"      # Color for highlighted text
selected_color = "green"      # Color for selected item
border_color = "blue"         # Color for borders
text_color = "white"          # Default text color
dim_color = "gray"            # Color for dimmed text
```

**Available colors:**
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `gray`, `dark_gray`, `light_red`, `light_green`, `light_yellow`
- `light_blue`, `light_magenta`, `light_cyan`
- Hex colors: `"#FF5733"`
- RGB: `"rgb(255, 87, 51)"`

### UI Settings

Control the user interface behavior.

```toml
[ui]
show_icons = true              # Show source type icons
show_descriptions = true       # Show command descriptions
max_results = 50               # Maximum commands to display
preview_enabled = false        # Enable preview panel
compact_mode = false           # Use compact display
```

### Shell Configuration

Configure shell behavior and defaults.

```toml
[shell]
default = "bash"               # Default shell (bash, zsh, fish, powershell)
preserve_env = true            # Preserve environment variables
interactive = true             # Run in interactive mode
```

**Supported shells:**
- `bash`
- `zsh`
- `fish`
- `powershell` / `pwsh`

### Scanner Settings

Control how Palrun scans for commands.

```toml
[scanner]
exclude_patterns = [           # Directories to exclude
  "node_modules",
  "target",
  ".git",
  "dist",
  "build",
  ".next",
  ".nuxt"
]
max_depth = 5                  # Maximum recursion depth
follow_symlinks = false        # Follow symbolic links
scan_hidden = false            # Scan hidden directories
cache_enabled = true           # Cache scan results
cache_ttl = 300                # Cache time-to-live (seconds)
```

### Search Configuration

Fine-tune fuzzy search behavior.

```toml
[search]
case_sensitive = false         # Case-sensitive search
smart_case = true              # Case-sensitive if query has uppercase
min_score = 0.3                # Minimum fuzzy match score (0.0-1.0)
max_results = 100              # Maximum search results
```

**Smart case behavior:**
- `test` matches `Test`, `TEST`, `test`
- `Test` only matches `Test`

### AI Settings

Configure AI provider preferences.

```toml
[ai]
provider = "auto"              # auto, claude, ollama, none
claude_model = "claude-3-5-sonnet-20241022"
ollama_model = "llama2"
timeout = 30                   # AI request timeout (seconds)
max_tokens = 1000              # Maximum response tokens
temperature = 0.7              # Response creativity (0.0-1.0)
```

**Provider options:**
- `auto` - Try Claude, fallback to Ollama
- `claude` - Use Claude only
- `ollama` - Use Ollama only
- `none` - Disable AI features

**Claude models:**
- `claude-3-5-sonnet-20241022` (recommended)
- `claude-3-opus-20240229`
- `claude-3-sonnet-20240229`
- `claude-3-haiku-20240307`

**Ollama models:**
- `llama2`
- `codellama`
- `mistral`
- `mixtral`
- Any model you've pulled with `ollama pull`

### Execution Settings

Control command execution behavior.

```toml
[execution]
confirm_destructive = true     # Confirm dangerous commands
timeout = 300                  # Command timeout (seconds)
shell_args = []                # Additional shell arguments
working_dir = "."              # Default working directory
```

### Keybindings

Customize keyboard shortcuts (future feature).

```toml
[keybindings]
quit = ["Esc", "Ctrl+C"]
execute = ["Enter"]
clear_input = ["Ctrl+U"]
toggle_preview = ["Tab"]
toggle_context = ["Ctrl+Space"]
```

## Project-Specific Configuration

Override global settings for specific projects by creating `.palrun/config.toml` in your project root:

```toml
# .palrun/config.toml

[scanner]
exclude_patterns = ["node_modules", "dist"]

[ai]
provider = "ollama"  # Use local AI for this project
```

Project configuration merges with global configuration, with project settings taking precedence.

## Environment Variables

Override configuration with environment variables:

```bash
# AI Provider
export PALRUN_AI_PROVIDER="claude"
export ANTHROPIC_API_KEY="your-api-key"

# Ollama
export OLLAMA_HOST="http://localhost:11434"

# Shell
export PALRUN_SHELL="zsh"

# Scanner
export PALRUN_MAX_DEPTH="3"
```

## Configuration Precedence

Settings are applied in this order (later overrides earlier):

1. Default values (built-in)
2. Global config (`~/.config/palrun/config.toml`)
3. Project config (`.palrun/config.toml`)
4. Environment variables
5. Command-line flags

## Common Configurations

### Minimal Configuration

```toml
[theme]
highlight_color = "cyan"

[scanner]
exclude_patterns = ["node_modules", "target", ".git"]
```

### Performance-Optimized

```toml
[scanner]
max_depth = 3
cache_enabled = true
cache_ttl = 600

[search]
max_results = 50

[ui]
max_results = 30
```

### AI-Focused

```toml
[ai]
provider = "claude"
claude_model = "claude-3-5-sonnet-20241022"
timeout = 60
max_tokens = 2000
```

### Monorepo Configuration

```toml
[scanner]
max_depth = 10
exclude_patterns = [
  "node_modules",
  "dist",
  "build",
  ".next",
  "target"
]
follow_symlinks = true

[ui]
max_results = 100
```

## Troubleshooting Configuration

### Config Not Loading

1. Check file location: `palrun config --path`
2. Verify TOML syntax: Use a TOML validator
3. Check file permissions: Ensure readable

### Invalid Configuration

Palrun validates configuration on startup. Check for:
- Syntax errors in TOML
- Invalid color names
- Out-of-range values
- Unknown configuration keys

### Reset to Defaults

Delete or rename your config file:

```bash
# Backup current config
mv ~/.config/palrun/config.toml ~/.config/palrun/config.toml.bak

# Palrun will use defaults
palrun
```

## Next Steps

- [User Guide](user-guide.md) - Learn all features
- [Troubleshooting](troubleshooting.md) - Solve common issues
- [FAQ](faq.md) - Frequently asked questions

