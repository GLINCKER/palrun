# Integration Examples

Project-specific integration examples showing how to set up Palrun for different types of projects.

## Available Examples

### Frontend Frameworks
- **[nextjs/](nextjs/)** - Next.js project setup
- **[react/](react/)** - React project setup
- **[vue/](vue/)** - Vue.js project setup

### Backend
- **[nodejs-api/](nodejs-api/)** - Node.js API project
- **[rust-api/](rust-api/)** - Rust API project
- **[go-api/](go-api/)** - Go API project

### Full Stack
- **[t3-stack/](t3-stack/)** - T3 Stack (Next.js + tRPC)
- **[mern/](mern/)** - MERN stack project

### Monorepos
- **[nx-monorepo/](nx-monorepo/)** - Nx monorepo setup
- **[turborepo/](turborepo/)** - Turborepo setup

### Other
- **[python-django/](python-django/)** - Django project
- **[rust-wasm/](rust-wasm/)** - Rust + WebAssembly project

## Quick Start

Each integration example includes:
- `.palrun.toml` - Project-specific configuration
- `.palrun/runbooks/` - Common runbooks for that project type
- `README.md` - Setup instructions and tips

### Using an Example

1. Navigate to the example directory
2. Copy the configuration and runbooks to your project
3. Customize as needed

```bash
# Example: Set up for Next.js project
cp examples/integrations/nextjs/.palrun.toml .
cp -r examples/integrations/nextjs/.palrun .
```

## Or Use `palrun init`

The easiest way is to let Palrun detect your project type and generate the configuration automatically:

```bash
cd your-project
palrun init
```

This will:
- Detect your project type
- Generate optimized `.palrun.toml`
- Create sample runbooks in `.palrun/runbooks/`
- Suggest relevant plugins

## Contributing

Have a useful integration example? Share it!

1. Create a new directory for your project type
2. Include `.palrun.toml` and sample runbooks
3. Add a README.md with setup instructions
4. Submit a pull request

## Next Steps

- [Configuration Examples](../configs/README.md)
- [Runbook Examples](../runbooks/README.md)
- [Plugin Examples](../plugins/README.md)

