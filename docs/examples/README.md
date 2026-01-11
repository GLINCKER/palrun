# Examples

Real-world examples of using Palrun in different project types.

## Quick Examples

### NPM Project

```bash
# Open command palette
palrun

# Or with shell integration
Ctrl+P

# Type "dev" and press Enter
# Runs: npm run dev
```

### Rust Project

```bash
cd my-rust-project
palrun

# Type "build" and press Enter
# Runs: cargo build
```

### Monorepo

```bash
cd my-monorepo
palrun scan --recursive

# Shows all commands from all packages
# Commands from current directory appear first
```

## Example Runbooks

### Simple Deployment

**.palrun/runbooks/deploy.yml:**
```yaml
name: Deploy to Production
description: Build and deploy the application

steps:
  - name: Install dependencies
    command: npm install
  
  - name: Run tests
    command: npm test
  
  - name: Build
    command: npm run build
  
  - name: Deploy
    command: npm run deploy
    confirm: true
```

**Run:**
```bash
palrun runbook deploy
```

### Environment-Specific Build

**.palrun/runbooks/build.yml:**
```yaml
name: Build Application
description: Build for a specific environment

variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - development
      - staging
      - production
    required: true

steps:
  - name: Install dependencies
    command: npm install
  
  - name: Build
    command: npm run build:{{environment}}
    env:
      NODE_ENV: "{{environment}}"
```

**Run:**
```bash
palrun runbook build
# Prompts for environment selection
```

### Docker Development Setup

**.palrun/runbooks/dev-setup.yml:**
```yaml
name: Development Setup
description: Set up local development environment

steps:
  - name: Pull latest images
    command: docker compose pull
  
  - name: Build containers
    command: docker compose build
  
  - name: Start database
    command: docker compose up -d db
  
  - name: Run migrations
    command: npm run db:migrate
  
  - name: Seed database
    command: npm run db:seed
    optional: true
  
  - name: Start all services
    command: docker compose up
```

**Run:**
```bash
palrun runbook dev-setup
```

## AI Examples

### Generate Commands

```bash
# Start development server
palrun ai gen "start the dev server"
# Output: npm run dev

# Run tests with coverage
palrun ai gen "run tests with coverage report"
# Output: npm test -- --coverage

# Build for production
palrun ai gen "build the app for production deployment"
# Output: npm run build -- --mode production
```

### Explain Commands

```bash
# Explain what a command does
palrun ai explain "npm run build"
# Output: This command builds your application for production...

# Explain complex command
palrun ai explain "cargo test --release --features full"
# Output: This runs Rust tests in release mode with all features enabled...
```

### Diagnose Errors

```bash
# Diagnose build failure
palrun ai diagnose "npm run build" "Module not found: 'react'"
# Output: The error indicates that React is not installed...

# Diagnose test failure
palrun ai diagnose "npm test" "TypeError: Cannot read property 'map' of undefined"
# Output: This error suggests that you're trying to call .map() on undefined...
```

## Configuration Examples

### Basic Configuration

**~/.config/palrun/config.toml:**
```toml
[scanner]
max_depth = 5
exclude_patterns = ["node_modules", "target", ".git"]

[ui]
theme = "dark"

[ai]
provider = "auto"
timeout = 30
```

### Custom Theme

```toml
[ui]
theme = "custom"

[ui.colors]
primary = "#00ff00"
secondary = "#0000ff"
background = "#1a1a1a"
text = "#ffffff"
```

### AI Configuration

```toml
[ai]
provider = "claude"  # or "ollama" or "auto"
claude_model = "claude-3-5-sonnet-20241022"
ollama_model = "llama2"
timeout = 60
```

## Project-Specific Examples

### Next.js Project

```bash
palrun ai gen "start next dev server"
# Output: npm run dev

palrun ai gen "build next app for production"
# Output: npm run build

palrun ai gen "start production server"
# Output: npm start
```

### Rust + WASM Project

```bash
palrun ai gen "build for wasm target"
# Output: cargo build --target wasm32-unknown-unknown

palrun ai gen "run clippy with all features"
# Output: cargo clippy --all-features
```

### Python Project

```bash
palrun ai gen "run pytest with verbose output"
# Output: pytest -v

palrun ai gen "install dependencies from requirements"
# Output: pip install -r requirements.txt
```

## Next Steps

- [How-to Guides](../guides/README.md)
- [Reference Documentation](../reference/README.md)
- [Creating Runbooks](../guides/creating-runbooks.md)

