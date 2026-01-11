//! Runbook template generation.

use super::ProjectType;
use anyhow::Result;

/// Generate sample runbooks for the given project type.
/// Returns a vector of (filename, content) pairs.
pub fn generate_samples(project_type: ProjectType) -> Result<Vec<(String, String)>> {
    let mut runbooks = Vec::new();

    match project_type {
        ProjectType::NodeJs | ProjectType::React | ProjectType::NextJs => {
            runbooks.push(("deploy.yml".to_string(), NODEJS_DEPLOY.to_string()));
            runbooks.push(("dev-setup.yml".to_string(), NODEJS_DEV_SETUP.to_string()));
        }
        ProjectType::Rust => {
            runbooks.push(("build.yml".to_string(), RUST_BUILD.to_string()));
            runbooks.push(("test.yml".to_string(), RUST_TEST.to_string()));
        }
        ProjectType::Go => {
            runbooks.push(("build.yml".to_string(), GO_BUILD.to_string()));
            runbooks.push(("test.yml".to_string(), GO_TEST.to_string()));
        }
        ProjectType::Python => {
            runbooks.push(("test.yml".to_string(), PYTHON_TEST.to_string()));
            runbooks.push(("dev-setup.yml".to_string(), PYTHON_DEV_SETUP.to_string()));
        }
        ProjectType::NxMonorepo | ProjectType::Turborepo => {
            runbooks.push(("build-all.yml".to_string(), MONOREPO_BUILD_ALL.to_string()));
            runbooks.push(("deploy.yml".to_string(), NODEJS_DEPLOY.to_string()));
        }
        ProjectType::Generic => {
            runbooks.push(("example.yml".to_string(), GENERIC_EXAMPLE.to_string()));
        }
    }

    Ok(runbooks)
}

/// Generic example runbook
const GENERIC_EXAMPLE: &str = r#"name: Example Runbook
description: A sample runbook to get you started
version: 1.0.0

variables:
  environment:
    type: select
    prompt: "Select environment"
    options:
      - development
      - staging
      - production

steps:
  - name: Example step
    command: echo "Running in {{environment}} environment"
    description: This is an example step
"#;

/// Node.js deployment runbook
const NODEJS_DEPLOY: &str = r#"name: Deploy Application
description: Build and deploy the application
version: 1.0.0

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
    description: Install all dependencies

  - name: Run tests
    command: npm test
    description: Run test suite
    optional: true

  - name: Build application
    command: npm run build
    description: Build for {{environment}}
    env:
      NODE_ENV: "{{environment}}"

  - name: Deploy
    command: npm run deploy:{{environment}}
    description: Deploy to {{environment}}
    confirm: true
"#;

/// Node.js dev setup runbook
const NODEJS_DEV_SETUP: &str = r"name: Development Setup
description: Set up local development environment
version: 1.0.0

steps:
  - name: Install dependencies
    command: npm install
    description: Install all dependencies

  - name: Copy environment file
    command: cp .env.example .env
    description: Create local environment file
    optional: true

  - name: Start development server
    command: npm run dev
    description: Start development server
";

/// Rust build runbook
const RUST_BUILD: &str = r#"name: Build Project
description: Build the Rust project with different profiles
version: 1.0.0

variables:
  profile:
    type: select
    prompt: "Select build profile"
    options:
      - debug
      - release
    default: debug

steps:
  - name: Run clippy
    command: cargo clippy
    description: Run linter
    optional: true

  - name: Build
    command: cargo build --release
    description: Build in {{profile}} mode
    condition: "profile == release"

  - name: Build
    command: cargo build
    description: Build in {{profile}} mode
    condition: "profile == debug"
"#;

/// Rust test runbook
const RUST_TEST: &str = r"name: Run Tests
description: Run the test suite
version: 1.0.0

steps:
  - name: Run tests
    command: cargo test
    description: Run all tests

  - name: Run doc tests
    command: cargo test --doc
    description: Run documentation tests
    optional: true
";

/// Go build runbook
const GO_BUILD: &str = r"name: Build Project
description: Build the Go project
version: 1.0.0

steps:
  - name: Run tests
    command: go test ./...
    description: Run all tests
    optional: true

  - name: Build
    command: go build
    description: Build the application
";

/// Go test runbook
const GO_TEST: &str = r"name: Run Tests
description: Run the test suite
version: 1.0.0

steps:
  - name: Run tests
    command: go test ./...
    description: Run all tests

  - name: Run tests with coverage
    command: go test -cover ./...
    description: Run tests with coverage
    optional: true
";

/// Python test runbook
const PYTHON_TEST: &str = r"name: Run Tests
description: Run the test suite
version: 1.0.0

steps:
  - name: Run pytest
    command: pytest
    description: Run all tests

  - name: Run with coverage
    command: pytest --cov
    description: Run tests with coverage
    optional: true
";

/// Python dev setup runbook
const PYTHON_DEV_SETUP: &str = r"name: Development Setup
description: Set up local development environment
version: 1.0.0

steps:
  - name: Create virtual environment
    command: python -m venv venv
    description: Create virtual environment
    optional: true

  - name: Install dependencies
    command: pip install -r requirements.txt
    description: Install dependencies

  - name: Run migrations
    command: python manage.py migrate
    description: Run database migrations
    optional: true
";

/// Monorepo build all runbook
const MONOREPO_BUILD_ALL: &str = r"name: Build All Packages
description: Build all packages in the monorepo
version: 1.0.0

steps:
  - name: Install dependencies
    command: npm install
    description: Install all dependencies

  - name: Build all packages
    command: npm run build
    description: Build all packages

  - name: Run tests
    command: npm test
    description: Run all tests
    optional: true
";
