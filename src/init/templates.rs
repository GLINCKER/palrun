//! Configuration template generation.

use super::ProjectType;
use anyhow::Result;

/// Generate a configuration file for the given project type.
pub fn generate_config(project_type: ProjectType) -> Result<String> {
    let template = match project_type {
        ProjectType::NodeJs => NODEJS_TEMPLATE,
        ProjectType::NextJs => NEXTJS_TEMPLATE,
        ProjectType::React => REACT_TEMPLATE,
        ProjectType::Rust => RUST_TEMPLATE,
        ProjectType::Go => GO_TEMPLATE,
        ProjectType::Python => PYTHON_TEMPLATE,
        ProjectType::NxMonorepo => NX_TEMPLATE,
        ProjectType::Turborepo => TURBO_TEMPLATE,
        ProjectType::Generic => GENERIC_TEMPLATE,
    };

    Ok(template.to_string())
}

/// Generic/default template
const GENERIC_TEMPLATE: &str = r#"# Palrun Configuration
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "npm",
    "cargo",
    "make",
    "docker",
    "go",
    "python",
    "nx",
    "turbo",
    "taskfile",
]

ignore_dirs = [
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
]

max_depth = 5
recursive = true

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Node.js template
const NODEJS_TEMPLATE: &str = r#"# Palrun Configuration for Node.js Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "npm",
    "docker",
    "make",
]

ignore_dirs = [
    "node_modules",
    ".git",
    "dist",
    "build",
    "coverage",
]

max_depth = 5
recursive = false

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Next.js template
const NEXTJS_TEMPLATE: &str = r#"# Palrun Configuration for Next.js Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "npm",
    "docker",
]

ignore_dirs = [
    "node_modules",
    ".git",
    ".next",
    "out",
    "dist",
    "build",
    "coverage",
]

max_depth = 5
recursive = false

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// React template
const REACT_TEMPLATE: &str = NODEJS_TEMPLATE;

/// Rust template
const RUST_TEMPLATE: &str = r#"# Palrun Configuration for Rust Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "cargo",
    "make",
    "docker",
    "taskfile",
]

ignore_dirs = [
    "target",
    ".git",
    "node_modules",
]

max_depth = 5
recursive = true

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Go template
const GO_TEMPLATE: &str = r#"# Palrun Configuration for Go Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "go",
    "make",
    "docker",
]

ignore_dirs = [
    ".git",
    "vendor",
    "bin",
]

max_depth = 5
recursive = false

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Python template
const PYTHON_TEMPLATE: &str = r#"# Palrun Configuration for Python Project
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 1000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 50
mouse = true

[scanner]
enabled = [
    "python",
    "make",
    "docker",
]

ignore_dirs = [
    ".git",
    "__pycache__",
    ".venv",
    "venv",
    ".pytest_cache",
    "dist",
    "build",
]

max_depth = 5
recursive = false

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Nx monorepo template
const NX_TEMPLATE: &str = r#"# Palrun Configuration for Nx Monorepo
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 2000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 100
mouse = true

[scanner]
enabled = [
    "npm",
    "nx",
    "docker",
    "make",
]

ignore_dirs = [
    "node_modules",
    ".git",
    "dist",
    "build",
    ".nx",
    "coverage",
]

max_depth = 10
recursive = true

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;

/// Turborepo template
const TURBO_TEMPLATE: &str = r#"# Palrun Configuration for Turborepo
# Auto-generated configuration file

[general]
confirm_dangerous = true
max_history = 2000

[ui]
theme = "default"
show_preview = true
show_icons = true
max_display = 100
mouse = true

[scanner]
enabled = [
    "npm",
    "turbo",
    "docker",
    "make",
]

ignore_dirs = [
    "node_modules",
    ".git",
    "dist",
    "build",
    ".turbo",
    "coverage",
]

max_depth = 10
recursive = true

[keys]
quit = "q"
select = "enter"
up = "up"
down = "down"
clear = "ctrl+u"
"#;
