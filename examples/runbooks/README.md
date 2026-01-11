# Runbook Examples

Sample runbook templates for common workflows.

## What are Runbooks?

Runbooks are YAML files that define multi-step workflows. They're like executable documentation that your team can run with a single command.

## Available Examples

### Deployment
- **[deploy-simple.yml](deploy-simple.yml)** - Basic deployment workflow
- **[deploy-env.yml](deploy-env.yml)** - Environment-specific deployment
- **[deploy-docker.yml](deploy-docker.yml)** - Docker-based deployment

### Testing
- **[test-full.yml](test-full.yml)** - Complete test suite
- **[test-ci.yml](test-ci.yml)** - CI-optimized testing

### Build
- **[build-prod.yml](build-prod.yml)** - Production build workflow
- **[build-multi.yml](build-multi.yml)** - Multi-target build

### Database
- **[db-migrate.yml](db-migrate.yml)** - Database migration workflow
- **[db-seed.yml](db-seed.yml)** - Database seeding

### Development
- **[dev-setup.yml](dev-setup.yml)** - Development environment setup
- **[dev-reset.yml](dev-reset.yml)** - Reset development environment

## Quick Start

### 1. Create Runbooks Directory

```bash
mkdir -p .palrun/runbooks
```

### 2. Copy a Template

```bash
cp examples/runbooks/deploy-simple.yml .palrun/runbooks/deploy.yml
```

### 3. Customize

Edit the runbook to match your project:

```yaml
name: Deploy My App
description: Deploy to production

steps:
  - name: Build
    command: npm run build
  
  - name: Deploy
    command: npm run deploy
    confirm: true
```

### 4. Run

```bash
palrun runbook deploy
```

## Runbook Structure

```yaml
name: Runbook Name
description: What this runbook does
version: 1.0.0
author: Your Name

variables:
  env:
    type: select
    prompt: "Select environment"
    options:
      - development
      - staging
      - production

steps:
  - name: Step Name
    command: command to run
    description: What this step does
    confirm: true  # Ask before running
    optional: true  # Continue if fails
    condition: "env == production"  # Conditional execution
```

## Variable Types

### String
```yaml
variables:
  app_name:
    type: string
    prompt: "Enter app name"
    default: "my-app"
    required: true
```

### Boolean
```yaml
variables:
  skip_tests:
    type: boolean
    prompt: "Skip tests?"
    default: false
```

### Select
```yaml
variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - dev
      - staging
      - prod
```

### Number
```yaml
variables:
  port:
    type: number
    prompt: "Port number"
    default: 3000
```

## Step Options

### Basic Step
```yaml
- name: Build
  command: npm run build
```

### With Confirmation
```yaml
- name: Deploy
  command: npm run deploy
  confirm: true
```

### Optional (Continue on Error)
```yaml
- name: Cleanup
  command: rm -rf temp/
  optional: true
```

### Conditional
```yaml
- name: Deploy to prod
  command: deploy --prod
  condition: "env == production"
```

### With Working Directory
```yaml
- name: Build frontend
  command: npm run build
  working_dir: packages/frontend
```

### With Environment Variables
```yaml
- name: Deploy
  command: npm run deploy
  env:
    NODE_ENV: production
    API_URL: https://api.example.com
```

## Best Practices

1. **Add descriptions** - Help team members understand each step
2. **Use confirmations** - For destructive operations
3. **Handle errors** - Use `optional` or `continue_on_error`
4. **Use variables** - Make runbooks reusable
5. **Version control** - Commit runbooks to share with team
6. **Test with dry-run** - `palrun runbook name --dry-run`

## Next Steps

- [Configuration Examples](../configs/README.md)
- [Integration Examples](../integrations/README.md)
- [Creating Runbooks Guide](../../docs/guides/creating-runbooks.md)

