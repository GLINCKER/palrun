# Setting Up AI Features

Configure Claude or Ollama to use Palrun's AI-powered command generation and assistance.

## Choose Your AI Provider

Palrun supports two AI providers:

| Provider | Type | Cost | Setup Difficulty |
|----------|------|------|------------------|
| **Claude** | Cloud | Paid (API usage) | Easy |
| **Ollama** | Local | Free | Medium |

Palrun tries Claude first, then falls back to Ollama if available.

## Option 1: Claude (Recommended)

Claude provides the most accurate command generation and explanations.

### Step 1: Get an API Key

1. Go to [console.anthropic.com](https://console.anthropic.com/)
2. Sign up or log in
3. Navigate to API Keys
4. Create a new API key
5. Copy the key (starts with `sk-ant-`)

### Step 2: Set the API Key

**Bash/Zsh** - Add to `~/.bashrc` or `~/.zshrc`:
```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

**Fish** - Add to `~/.config/fish/config.fish`:
```fish
set -x ANTHROPIC_API_KEY "sk-ant-your-key-here"
```

**PowerShell** - Add to your profile:
```powershell
$env:ANTHROPIC_API_KEY = "sk-ant-your-key-here"
```

### Step 3: Reload Your Shell

```bash
source ~/.bashrc  # or ~/.zshrc
```

Or open a new terminal.

### Step 4: Test It

```bash
palrun ai status
```

Should show: `Active AI provider: Claude (Anthropic)`

### Pricing

Claude charges per API request:
- Typical command generation: $0.001-0.01 per request
- See [Anthropic pricing](https://www.anthropic.com/pricing) for details

## Option 2: Ollama (Free, Local)

Ollama runs AI models locally on your machine - completely free and private.

### Step 1: Install Ollama

**macOS/Linux:**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**Windows:**
Download from [ollama.com/download](https://ollama.com/download)

### Step 2: Pull a Model

```bash
# Recommended: Llama 2 (4GB)
ollama pull llama2

# Alternative: Smaller model (2GB)
ollama pull llama2:7b

# For code: CodeLlama
ollama pull codellama
```

### Step 3: Verify Ollama is Running

```bash
ollama list
```

Should show your downloaded models.

### Step 4: Test It

```bash
palrun ai status
```

Should show: `Active AI provider: Ollama`

### System Requirements

- **RAM**: 8GB minimum (16GB recommended)
- **Disk**: 4-10GB per model
- **CPU**: Modern processor (Apple Silicon works great)

## Using AI Features

Once set up, you can use these commands:

### Generate Commands

```bash
palrun ai gen "start the development server"
palrun ai gen "run tests in watch mode"
palrun ai gen "build for production"
```

### Explain Commands

```bash
palrun ai explain "npm run build"
palrun ai explain "cargo test --release"
```

### Diagnose Errors

```bash
palrun ai diagnose "npm test" "Module not found: react"
```

## Configuration

Customize AI behavior in `~/.config/palrun/config.toml`:

```toml
[ai]
provider = "auto"  # auto, claude, ollama, none
claude_model = "claude-3-5-sonnet-20241022"
ollama_model = "llama2"
timeout = 30
```

**Provider options:**
- `auto` - Try Claude, fallback to Ollama (default)
- `claude` - Use Claude only
- `ollama` - Use Ollama only
- `none` - Disable AI features

## Troubleshooting

### "No AI provider available"

**For Claude:**
```bash
# Check if key is set
echo $ANTHROPIC_API_KEY

# Should show your key
```

**For Ollama:**
```bash
# Check if Ollama is running
ollama list

# If not running, start it
ollama serve
```

### AI requests timeout

**Increase timeout:**
```toml
[ai]
timeout = 60  # Increase from 30
```

**For Ollama - use smaller model:**
```bash
ollama pull llama2:7b
```

### Wrong or irrelevant commands

Be more specific in your prompts:

**Bad:**
```bash
palrun ai gen "build"
```

**Good:**
```bash
palrun ai gen "build the frontend React app for production"
```

## Privacy & Security

**Claude:**
- Sends your prompt and project context to Anthropic's API
- No file contents are sent
- See [Anthropic's privacy policy](https://www.anthropic.com/privacy)

**Ollama:**
- Runs completely locally
- No data leaves your machine
- Fully private

## Next Steps

- [Generating Commands with AI](ai-generate-commands.md)
- [Explaining Commands](ai-explain-commands.md)
- [Diagnosing Errors](ai-diagnose-errors.md)

