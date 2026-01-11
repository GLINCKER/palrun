# Creating Runbooks

Create executable documentation for complex workflows that your team can run with a single command.

## What is a Runbook?

A runbook is a YAML file that defines a series of steps to execute. It's like a script, but with:
- Variables for customization
- Confirmations for safety
- Conditions for flexibility
- Error handling

## Creating Your First Runbook

### Step 1: Create the Runbooks Directory

```bash
mkdir -p .palrun/runbooks
```

### Step 2: Create a Runbook File

Create `.palrun/runbooks/deploy.yml`:

```yaml
name: Deploy Application
description: Deploy the application to a target environment
version: 1.0.0

steps:
  - name: Install dependencies
    command: npm install
    description: Install all dependencies
  
  - name: Run tests
    command: npm test
    description: Run test suite
  
  - name: Build application
    command: npm run build
    description: Build for production
  
  - name: Deploy
    command: npm run deploy
    description: Deploy to server
    confirm: true
```

### Step 3: Run the Runbook

```bash
palrun runbook deploy
```

## Runbook Structure

### Required Fields

```yaml
name: My Runbook          # Runbook name
steps:                    # List of steps (required)
  - name: Step 1
    command: echo "Hello"
```

### Optional Top-Level Fields

```yaml
name: My Runbook
description: What this runbook does
version: 1.0.0
author: Your Name
variables:                # Variable definitions (see below)
  env:
    type: string
steps:
  # ...
```

## Adding Variables

Variables let you customize runbook behavior.

### String Variable

```yaml
variables:
  app_name:
    type: string
    prompt: "Enter application name"
    default: "my-app"
    required: true

steps:
  - name: Deploy
    command: deploy --app={{app_name}}
```

### Boolean Variable

```yaml
variables:
  skip_tests:
    type: boolean
    prompt: "Skip tests?"
    default: false

steps:
  - name: Run tests
    command: npm test
    condition: "!skip_tests"
```

### Select Variable

```yaml
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
  - name: Deploy
    command: deploy --env={{environment}}
```

### Number Variable

```yaml
variables:
  port:
    type: number
    prompt: "Enter port number"
    default: 3000
```

## Step Options

### Basic Step

```yaml
steps:
  - name: Build
    command: npm run build
    description: Build the application
```

### Step with Confirmation

```yaml
steps:
  - name: Deploy to production
    command: npm run deploy:prod
    confirm: true  # Ask user before running
```

### Optional Step

```yaml
steps:
  - name: Run linter
    command: npm run lint
    optional: true  # Continue if this fails
```

### Conditional Step

```yaml
steps:
  - name: Deploy to production
    command: npm run deploy:prod
    condition: "environment == production"
```

### Step with Timeout

```yaml
steps:
  - name: Long running task
    command: npm run build
    timeout: 300  # 5 minutes
```

### Step with Working Directory

```yaml
steps:
  - name: Build frontend
    command: npm run build
    working_dir: packages/frontend
```

### Step with Environment Variables

```yaml
steps:
  - name: Deploy
    command: npm run deploy
    env:
      NODE_ENV: production
      API_URL: https://api.example.com
```

### Continue on Error

```yaml
steps:
  - name: Optional cleanup
    command: rm -rf temp/
    continue_on_error: true
```

## Using Variables in Commands

Use `{{variable_name}}` syntax:

```yaml
variables:
  env:
    type: string
  version:
    type: string

steps:
  - name: Deploy
    command: deploy --env={{env}} --version={{version}}
```

## Conditions

Simple boolean expressions:

```yaml
# Not operator
condition: "!skip_tests"

# Equality
condition: "environment == production"

# Inequality  
condition: "environment != development"
```

## Example Runbooks

### Simple Build and Deploy

```yaml
name: Build and Deploy
description: Build the app and deploy to production

steps:
  - name: Install
    command: npm install
  
  - name: Build
    command: npm run build
  
  - name: Deploy
    command: npm run deploy
    confirm: true
```

### Environment-Specific Deployment

```yaml
name: Deploy
description: Deploy to selected environment

variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - staging
      - production

steps:
  - name: Build
    command: npm run build
  
  - name: Deploy
    command: npm run deploy:{{environment}}
    confirm: true
```

### Full-Featured Runbook

```yaml
name: Production Deployment
description: Complete production deployment workflow
version: 1.0.0
author: DevOps Team

variables:
  skip_tests:
    type: boolean
    prompt: "Skip tests?"
    default: false
  
  version:
    type: string
    prompt: "Version to deploy"
    required: true

steps:
  - name: Install dependencies
    command: npm install
  
  - name: Run tests
    command: npm test
    condition: "!skip_tests"
    timeout: 600
  
  - name: Build
    command: npm run build
    env:
      NODE_ENV: production
  
  - name: Tag version
    command: git tag v{{version}}
    optional: true
  
  - name: Deploy
    command: npm run deploy
    confirm: true
    timeout: 300
  
  - name: Verify deployment
    command: npm run verify
    continue_on_error: true
```

## Running Runbooks

### Basic Run

```bash
palrun runbook deploy
```

### Dry Run (Preview)

```bash
palrun runbook deploy --dry-run
```

### Pass Variables

```bash
palrun runbook deploy --var environment=production --var version=1.2.3
```

## Sharing Runbooks

1. Commit `.palrun/runbooks/` to version control
2. Your team can run the same workflows
3. Document runbooks in your project README

## Best Practices

1. **Add descriptions** - Help team members understand each step
2. **Use confirmations** - For destructive operations
3. **Set timeouts** - Prevent hanging
4. **Handle errors** - Use `optional` or `continue_on_error`
5. **Version your runbooks** - Track changes
6. **Test with dry-run** - Before running for real

## Next Steps

- [Running Runbooks](running-runbooks.md)
- [Runbook Variables](runbook-variables.md)

