# Environment Variables

Palrun uses environment variables for configuration, API keys, and integration settings.

## AI Provider Keys

### ANTHROPIC_API_KEY

Claude API key for AI-powered features.

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

**Used for:**
- Command generation (`palrun ai gen`)
- Command explanation (`palrun ai explain`)
- Error diagnosis (`palrun ai diagnose`)

**Get a key:** [Anthropic Console](https://console.anthropic.com/)

### OPENAI_API_KEY

OpenAI API key (alternative AI provider).

```bash
export OPENAI_API_KEY="sk-..."
```

### OLLAMA_HOST

Ollama server URL for local AI inference.

```bash
export OLLAMA_HOST="http://localhost:11434"
```

**Default:** `http://localhost:11434`

### OLLAMA_MODEL

Default Ollama model to use.

```bash
export OLLAMA_MODEL="llama3.2"
```

**Default:** `llama3.2`

## GitHub Integration

### GITHUB_TOKEN / GH_TOKEN

GitHub personal access token for GitHub features.

```bash
export GITHUB_TOKEN="ghp_..."
# or
export GH_TOKEN="ghp_..."
```

**Used for:**
- GitHub Actions integration
- Issue management
- Repository information

**Scopes needed:**
- `repo` - For private repositories
- `workflow` - For GitHub Actions
- `read:org` - For organization repositories

### GITHUB_REPOSITORY

Repository in `owner/repo` format (set automatically in GitHub Actions).

```bash
export GITHUB_REPOSITORY="GLINCKER/palrun"
```

### GITHUB_ACTOR

GitHub username (set automatically in GitHub Actions).

```bash
export GITHUB_ACTOR="username"
```

## Linear Integration

### LINEAR_API_KEY

Linear API key for issue tracking integration.

```bash
export LINEAR_API_KEY="lin_api_..."
```

**Get a key:** [Linear Settings > API](https://linear.app/settings/api)

## Shell Variables

### SHELL

User's default shell (used for shell integration).

```bash
echo $SHELL
# /bin/zsh
```

### USER / USERNAME

Current username (used for display in TUI).

```bash
echo $USER
# john
```

## Configuration Precedence

Palrun checks for configuration in this order:

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (`~/.config/palrun/config.toml`)
4. **Default values** (lowest priority)

## Secrets Management

For secure storage of API keys, Palrun supports OS keychain integration:

```bash
# Store a key in the system keychain (more secure than env vars)
palrun config set-secret claude-api-key

# Keys are retrieved automatically from keychain
# with env var fallback
```

**Supported keychains:**
- macOS: Keychain
- Linux: Secret Service (GNOME Keyring, KDE Wallet)
- Windows: Credential Manager

## CI/CD Environment Variables

When running in CI/CD environments, these variables are typically available:

### GitHub Actions

```yaml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```

### GitLab CI

```yaml
variables:
  ANTHROPIC_API_KEY: $ANTHROPIC_API_KEY
```

### CircleCI

```yaml
environment:
  ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
```

## Debug Environment Variables

### RUST_LOG

Enable debug logging for troubleshooting.

```bash
# Enable all debug logs
export RUST_LOG=debug

# Enable only palrun logs
export RUST_LOG=palrun=debug

# Enable trace logs
export RUST_LOG=palrun=trace
```

### PALRUN_DEBUG

Enable debug mode (future feature).

```bash
export PALRUN_DEBUG=1
```

## Security Best Practices

1. **Never commit API keys** to version control
2. **Use environment variables** or keychain for secrets
3. **Use `.env` files** for local development (add to `.gitignore`)
4. **Rotate keys regularly** for production environments
5. **Use minimal scopes** for API tokens

### Example .env file

```bash
# .env (add to .gitignore!)
ANTHROPIC_API_KEY=sk-ant-...
GITHUB_TOKEN=ghp_...
LINEAR_API_KEY=lin_api_...
OLLAMA_HOST=http://localhost:11434
OLLAMA_MODEL=llama3.2
```

Load with:
```bash
source .env
# or use direnv, dotenv, etc.
```

## Troubleshooting

### API Key Not Found

```
Error: ANTHROPIC_API_KEY not set
```

**Solution:**
1. Set the environment variable: `export ANTHROPIC_API_KEY="your-key"`
2. Or add to your shell profile (`~/.zshrc`, `~/.bashrc`)
3. Or use keychain: `palrun config set-secret claude-api-key`

### Wrong API Key Format

```
Error: Invalid API key format
```

**Solution:**
- Claude keys start with `sk-ant-`
- OpenAI keys start with `sk-`
- GitHub tokens start with `ghp_` or `github_pat_`

### Environment Variable Not Persisting

**Solution:**
Add exports to your shell profile:

```bash
# ~/.zshrc or ~/.bashrc
export ANTHROPIC_API_KEY="your-key"
```

Then reload:
```bash
source ~/.zshrc
```
