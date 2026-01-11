# Working with Monorepos

Use Palrun effectively in monorepo projects with multiple packages.

## What is a Monorepo?

A monorepo is a repository containing multiple projects or packages. Common structures:

```
my-monorepo/
├── packages/
│   ├── frontend/
│   │   └── package.json
│   ├── backend/
│   │   └── package.json
│   └── shared/
│       └── package.json
└── package.json
```

## Scanning Monorepos

### Recursive Scan

To discover commands from all packages:

```bash
palrun scan --recursive
```

This scans subdirectories up to 5 levels deep.

**Output:**
```
Discovered 15 commands in "."

NPM (packages/frontend):
  - npm run dev
  - npm run build
  - npm run test

NPM (packages/backend):
  - npm run dev
  - npm run build
  - npm run test

NPM (root):
  - npm run dev:all
  - npm run build:all
```

### Configure Scan Depth

In `~/.config/palrun/config.toml`:

```toml
[scanner]
max_depth = 10  # Increase for deeper monorepos
```

## Context-Aware Filtering

Palrun prioritizes commands based on your current directory.

### Example

In this structure:
```
my-monorepo/
├── packages/
│   ├── frontend/  <- You are here
│   └── backend/
```

When you run `palrun` from `packages/frontend/`:

1. **Frontend commands appear first**
   - npm run dev (frontend)
   - npm run build (frontend)

2. **Backend commands appear lower**
   - npm run dev (backend)
   - npm run build (backend)

3. **Root commands appear last**
   - npm run dev:all

### Toggle Context Filtering

Press `Ctrl+Space` in the palette to toggle context-aware sorting on/off.

## Common Monorepo Patterns

### NPM Workspaces

**Root package.json:**
```json
{
  "workspaces": [
    "packages/*"
  ],
  "scripts": {
    "dev:all": "npm run dev --workspaces",
    "build:all": "npm run build --workspaces"
  }
}
```

Palrun discovers:
- All workspace package scripts
- Root-level scripts

### Yarn Workspaces

Same as NPM workspaces. Palrun detects `yarn.lock` and uses `yarn` commands.

### PNPM Workspaces

**pnpm-workspace.yaml:**
```yaml
packages:
  - 'packages/*'
```

Palrun detects `pnpm-lock.yaml` and uses `pnpm` commands.

### Nx Monorepo

**nx.json:**
```json
{
  "projects": {
    "frontend": "packages/frontend",
    "backend": "packages/backend"
  }
}
```

Palrun discovers Nx commands:
- `nx build frontend`
- `nx serve frontend`
- `nx test frontend`
- `nx build backend`
- etc.

### Turborepo

**turbo.json:**
```json
{
  "pipeline": {
    "build": {},
    "test": {},
    "dev": {}
  }
}
```

Palrun discovers:
- `turbo run build`
- `turbo run test`
- `turbo run dev`

## Excluding Directories

Exclude certain packages from scanning:

```toml
[scanner]
exclude_patterns = [
  "node_modules",
  "target",
  ".git",
  "packages/legacy",  # Exclude specific package
  "packages/archived"
]
```

## Running Commands in Specific Packages

### From Package Directory

```bash
cd packages/frontend
palrun  # Shows frontend commands first
```

### From Root Directory

1. Open Palrun: `palrun`
2. Type the package name: "frontend"
3. Commands from that package appear
4. Select and execute

## Monorepo Runbooks

Create runbooks for monorepo workflows:

**.palrun/runbooks/build-all.yml:**
```yaml
name: Build All Packages
description: Build all packages in order

steps:
  - name: Build shared
    command: npm run build
    working_dir: packages/shared
  
  - name: Build frontend
    command: npm run build
    working_dir: packages/frontend
  
  - name: Build backend
    command: npm run build
    working_dir: packages/backend
```

Run with:
```bash
palrun runbook build-all
```

## Tips for Monorepos

1. **Use recursive scan** - `palrun scan --recursive`
2. **Navigate to package** - Run from package directory for focused commands
3. **Use context filtering** - Press `Ctrl+Space` to toggle
4. **Create runbooks** - For cross-package workflows
5. **Configure exclusions** - Skip archived or legacy packages
6. **Increase scan depth** - If packages are deeply nested

## Example Workflows

### Development Workflow

```bash
# Start frontend dev server
cd packages/frontend
palrun  # Type "dev", press Enter

# In another terminal, start backend
cd packages/backend
palrun  # Type "dev", press Enter
```

### Build Workflow

```bash
# From root
palrun  # Type "build:all", press Enter
```

### Testing Specific Package

```bash
cd packages/frontend
palrun  # Type "test", press Enter
```

## Next Steps

- [Creating Runbooks](creating-runbooks.md)
- [Configuring Scanners](configuring-scanners.md)
- [Project Types Reference](../reference/project-types.md)

