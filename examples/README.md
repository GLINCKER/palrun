# Palrun Examples

This directory contains example configurations, runbooks, and templates to help you get started with Palrun.

## Directory Structure

```
examples/
├── configs/           # Configuration file examples
├── runbooks/          # Sample runbook templates
├── plugins/           # Plugin examples (existing)
└── integrations/      # Project-specific integration examples
```

## Quick Start

### 1. Initialize Your Project

Run `palrun init` in your project directory to automatically generate a personalized configuration:

```bash
cd your-project
palrun init
```

This will:
- Detect your project type (NPM, Cargo, Go, Python, etc.)
- Create `.palrun.toml` with recommended settings
- Generate `.palrun/runbooks/` directory with sample runbooks
- Suggest relevant plugins based on your project

### 2. Browse Examples

Explore the examples in this directory to learn about advanced features:

- **[configs/](configs/)** - Configuration file templates for different use cases
- **[runbooks/](runbooks/)** - Ready-to-use runbook templates
- **[integrations/](integrations/)** - Project-specific setup examples

### 3. Customize

Copy and modify examples to fit your workflow:

```bash
# Copy a runbook template
cp examples/runbooks/deploy.yml .palrun/runbooks/

# Copy a config template
cp examples/configs/monorepo.toml .palrun.toml
```

## Configuration Examples

See [configs/README.md](configs/README.md) for:
- Basic configuration
- Monorepo configuration
- AI-enabled configuration
- Team configuration
- CI/CD configuration

## Runbook Examples

See [runbooks/README.md](runbooks/README.md) for:
- Deployment runbooks
- Testing runbooks
- Build runbooks
- Database migration runbooks
- Docker workflow runbooks

## Integration Examples

See [integrations/README.md](integrations/README.md) for:
- Next.js projects
- Rust projects
- Go projects
- Python projects
- Monorepo projects

## Plugin Examples

See [plugins/README.md](plugins/README.md) for:
- Custom scanner plugins
- Integration plugins
- Example implementations

## Contributing

Have a useful configuration or runbook? Share it with the community!

1. Add your example to the appropriate directory
2. Include clear comments and documentation
3. Submit a pull request

## License

All examples are provided under the MIT License and are free to use and modify.

