# Generating Commands with AI

Use natural language to generate commands for your project.

## Prerequisites

Make sure you have AI set up first:
- [Setting Up AI (Claude/Ollama)](ai-setup.md)

## Basic Usage

```bash
palrun ai gen "<what you want to do>"
```

The AI will:
1. Analyze your project structure
2. Understand available commands
3. Generate the most appropriate command
4. Show you the command before executing

## Examples

### Development Commands

```bash
palrun ai gen "start the development server"
# Output: npm run dev

palrun ai gen "start dev server in watch mode"
# Output: npm run dev -- --watch

palrun ai gen "run the app in development mode"
# Output: cargo run
```

### Build Commands

```bash
palrun ai gen "build for production"
# Output: npm run build

palrun ai gen "build with optimizations"
# Output: cargo build --release

palrun ai gen "create production build"
# Output: npm run build -- --mode production
```

### Test Commands

```bash
palrun ai gen "run all tests"
# Output: npm test

palrun ai gen "run tests in watch mode"
# Output: npm test -- --watch

palrun ai gen "run tests with coverage"
# Output: npm test -- --coverage

palrun ai gen "test a specific file"
# Output: npm test -- src/components/Button.test.ts
```

### Docker Commands

```bash
palrun ai gen "start all docker containers"
# Output: docker compose up

palrun ai gen "start docker in background"
# Output: docker compose up -d

palrun ai gen "view docker logs"
# Output: docker compose logs -f
```

### Linting and Formatting

```bash
palrun ai gen "lint the code"
# Output: npm run lint

palrun ai gen "fix linting errors automatically"
# Output: npm run lint -- --fix

palrun ai gen "format all files"
# Output: npm run format
```

## Execute Immediately

Add `--execute` or `-x` to run the command without confirmation:

```bash
palrun ai gen "run tests" --execute
# Runs: npm test (immediately)
```

**Use with caution!** Only use `--execute` when you trust the generated command.

## How It Works

The AI considers:

1. **Project type** - NPM, Cargo, Make, etc.
2. **Available commands** - From package.json, Cargo.toml, etc.
3. **Common patterns** - Standard command conventions
4. **Your intent** - What you're trying to accomplish

## Writing Good Prompts

### Be Specific

**Bad:**
```bash
palrun ai gen "build"
```

**Good:**
```bash
palrun ai gen "build the frontend for production"
```

### Include Context

**Bad:**
```bash
palrun ai gen "test"
```

**Good:**
```bash
palrun ai gen "run unit tests with coverage report"
```

### Mention Tools

**Bad:**
```bash
palrun ai gen "start server"
```

**Good:**
```bash
palrun ai gen "start the vite dev server"
```

## Multi-Step Workflows

For complex workflows, use runbooks instead:

```bash
# Instead of:
palrun ai gen "install deps, run tests, build, deploy"

# Create a runbook:
palrun runbook deploy
```

See [Creating Runbooks](creating-runbooks.md).

## Troubleshooting

### "No AI provider available"

Make sure you've set up Claude or Ollama:
- [Setting Up AI](ai-setup.md)

### Wrong command generated

Try being more specific:

```bash
# Instead of:
palrun ai gen "build"

# Try:
palrun ai gen "build the React frontend app for production deployment"
```

### Command doesn't exist

The AI might suggest a command that doesn't exist in your project. Check available commands:

```bash
palrun list
```

### Timeout errors

For Ollama, try a smaller model:

```bash
ollama pull llama2:7b
```

Or increase timeout in `~/.config/palrun/config.toml`:

```toml
[ai]
timeout = 60
```

## Privacy

**Claude:**
- Sends your prompt and project structure to Anthropic
- No file contents are sent
- See [Anthropic's privacy policy](https://www.anthropic.com/privacy)

**Ollama:**
- Runs completely locally
- No data leaves your machine

## Next Steps

- [Explaining Commands](ai-explain-commands.md)
- [Diagnosing Errors](ai-diagnose-errors.md)
- [AI Setup Guide](ai-setup.md)

