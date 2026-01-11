# Supported Project Types Reference

Complete reference for all project types that Palrun can detect and scan.

## Overview

Palrun automatically detects project types by scanning for specific configuration files. When found, it extracts available commands and presents them in the command palette.

## NPM/Yarn/PNPM/Bun

**Detection Files:**
- `package.json` (required)
- `package-lock.json` (npm)
- `yarn.lock` (yarn)
- `pnpm-lock.yaml` (pnpm)
- `bun.lockb` (bun)

**Commands Discovered:**
All scripts defined in `package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest"
  }
}
```

**Generated Commands:**
- `npm run dev` (or `yarn dev`, `pnpm dev`, `bun run dev`)
- `npm run build`
- `npm run test`

**Package Manager Detection:**
Palrun automatically detects the package manager from lock files and uses the correct command prefix.

---

## Rust (Cargo)

**Detection Files:**
- `Cargo.toml`

**Commands Discovered:**
Standard Cargo commands:

| Command | Description |
|---------|-------------|
| `cargo build` | Build the project |
| `cargo build --release` | Build with optimizations |
| `cargo test` | Run tests |
| `cargo run` | Run the binary |
| `cargo clippy` | Run linter |
| `cargo check` | Check compilation |
| `cargo doc` | Generate documentation |
| `cargo clean` | Clean build artifacts |

---

## Go

**Detection Files:**
- `go.mod`

**Commands Discovered:**

| Command | Description |
|---------|-------------|
| `go build` | Build the project |
| `go test` | Run tests |
| `go run .` | Run the application |
| `go mod tidy` | Clean dependencies |
| `go vet` | Run static analysis |
| `go fmt` | Format code |

---

## Python

**Detection Files:**
- `pyproject.toml`
- `requirements.txt`
- `setup.py`

**Commands Discovered:**

| Command | Description |
|---------|-------------|
| `pytest` | Run tests |
| `python -m pytest` | Run tests (module) |
| `pip install -r requirements.txt` | Install dependencies |
| `poetry install` | Install with Poetry |
| `poetry run pytest` | Run tests with Poetry |
| `pdm install` | Install with PDM |

---

## Make

**Detection Files:**
- `Makefile`
- `makefile`

**Commands Discovered:**
All targets defined in the Makefile (except internal targets starting with `.` or `_`).

**Example Makefile:**
```makefile
.PHONY: all build test clean

all: build test

build:
    gcc -o app main.c

test:
    ./app --test

clean:
    rm -f app
```

**Generated Commands:**
- `make all`
- `make build`
- `make test`
- `make clean`

---

## Task (Taskfile)

**Detection Files:**
- `Taskfile.yml`
- `Taskfile.yaml`

**Commands Discovered:**
All tasks defined in the Taskfile.

**Example Taskfile:**
```yaml
version: '3'

tasks:
  build:
    desc: Build the application
    cmds:
      - go build -o app

  test:
    desc: Run tests
    cmds:
      - go test ./...
```

**Generated Commands:**
- `task build`
- `task test`

---

## Docker Compose

**Detection Files:**
- `docker-compose.yml`
- `docker-compose.yaml`
- `compose.yml`
- `compose.yaml`

**Commands Discovered:**

| Command | Description |
|---------|-------------|
| `docker compose up` | Start services |
| `docker compose up -d` | Start in background |
| `docker compose down` | Stop services |
| `docker compose logs` | View logs |
| `docker compose ps` | List containers |
| `docker compose build` | Build images |
| `docker compose restart` | Restart services |
| `docker compose exec` | Execute in container |

---

## Nx Monorepo

**Detection Files:**
- `nx.json`

**Commands Discovered:**

| Command | Description |
|---------|-------------|
| `nx build <project>` | Build a project |
| `nx serve <project>` | Serve a project |
| `nx test <project>` | Test a project |
| `nx lint <project>` | Lint a project |
| `nx affected:build` | Build affected projects |
| `nx affected:test` | Test affected projects |
| `nx run-many` | Run command for multiple projects |

---

## Turborepo

**Detection Files:**
- `turbo.json`

**Commands Discovered:**
All tasks defined in the pipeline.

**Example turbo.json:**
```json
{
  "pipeline": {
    "build": {},
    "test": {},
    "lint": {}
  }
}
```

**Generated Commands:**
- `turbo run build`
- `turbo run test`
- `turbo run lint`

---

## Detection Priority

When multiple project types are detected in the same directory, Palrun discovers commands from all of them. For example, a project with both `package.json` and `Cargo.toml` will show both npm scripts and cargo commands.

## Exclusions

Palrun excludes certain directories from scanning by default:
- `node_modules/`
- `target/`
- `.git/`
- `dist/`
- `build/`

Configure exclusions in `~/.config/palrun/config.toml`:

```toml
[scanner]
exclude_patterns = ["node_modules", "target", ".git"]
```

## Recursive Scanning

For monorepos, use recursive scanning:

```bash
palrun scan --recursive
```

This scans subdirectories up to 5 levels deep (configurable).

## Adding Custom Project Types

Palrun supports a plugin system for adding custom project type scanners. See the plugin examples in the repository for implementation details.

## Next Steps

- [CLI Commands Reference](cli-reference.md)
- [Configuration Reference](config-reference.md)

