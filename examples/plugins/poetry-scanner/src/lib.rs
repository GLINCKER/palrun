//! Poetry Scanner Plugin for Palrun
//!
//! Scans Python Poetry projects (pyproject.toml) and extracts available
//! scripts and commands.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Common Poetry commands.
const POETRY_COMMANDS: &[(&str, &str)] = &[
    ("install", "Install dependencies from poetry.lock"),
    ("update", "Update dependencies to latest versions"),
    ("add", "Add a new dependency"),
    ("remove", "Remove a dependency"),
    ("show", "Show installed packages"),
    ("build", "Build the package"),
    ("publish", "Publish the package to PyPI"),
    ("run", "Run a command in the virtual environment"),
    ("shell", "Spawn a shell within the virtual environment"),
    ("check", "Check pyproject.toml validity"),
    ("lock", "Lock dependencies without installing"),
    ("export", "Export locked dependencies to requirements.txt"),
];

/// Common Python/pytest commands.
const PYTHON_COMMANDS: &[(&str, &str)] = &[
    ("pytest", "Run tests with pytest"),
    ("pytest -v", "Run tests with verbose output"),
    ("pytest --cov", "Run tests with coverage"),
    ("mypy .", "Type check with mypy"),
    ("black .", "Format code with black"),
    ("ruff check .", "Lint with ruff"),
    ("flake8", "Lint with flake8"),
    ("isort .", "Sort imports with isort"),
];

/// Parse a pyproject.toml file and extract available commands.
pub fn parse_pyproject_toml(content: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();

    // Check if it's a Poetry project
    let is_poetry = content.contains("[tool.poetry]");

    if is_poetry {
        // Add Poetry commands
        for (cmd, desc) in POETRY_COMMANDS {
            commands.push(PluginCommand {
                name: format!("poetry {}", cmd),
                command: format!("poetry {}", cmd),
                description: Some(desc.to_string()),
                working_dir: None,
                tags: vec!["poetry".to_string(), "python".to_string()],
            });
        }

        // Add poetry run commands for common tools
        for (cmd, desc) in PYTHON_COMMANDS {
            commands.push(PluginCommand {
                name: format!("poetry run {}", cmd),
                command: format!("poetry run {}", cmd),
                description: Some(desc.to_string()),
                working_dir: None,
                tags: vec!["poetry".to_string(), "python".to_string(), "dev".to_string()],
            });
        }
    }

    // Parse custom scripts from [tool.poetry.scripts]
    for (name, _) in extract_scripts(content) {
        commands.push(PluginCommand {
            name: format!("poetry run {}", name),
            command: format!("poetry run {}", name),
            description: Some(format!("Run '{}' script", name)),
            working_dir: None,
            tags: vec!["poetry".to_string(), "script".to_string()],
        });
    }

    // Parse [project.scripts] (PEP 621)
    for (name, _) in extract_pep621_scripts(content) {
        let cmd = if is_poetry {
            format!("poetry run {}", name)
        } else {
            name.clone()
        };
        commands.push(PluginCommand {
            name: cmd.clone(),
            command: cmd,
            description: Some(format!("Run '{}' entry point", name)),
            working_dir: None,
            tags: vec!["python".to_string(), "entrypoint".to_string()],
        });
    }

    commands
}

/// Extract scripts from [tool.poetry.scripts] section.
fn extract_scripts(content: &str) -> Vec<(String, String)> {
    let mut scripts = Vec::new();
    let mut in_scripts = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[tool.poetry.scripts]" {
            in_scripts = true;
            continue;
        }

        if in_scripts {
            if line.starts_with('[') {
                break;
            }

            if let Some((name, value)) = line.split_once('=') {
                let name = name.trim().trim_matches('"');
                let value = value.trim().trim_matches('"');
                scripts.push((name.to_string(), value.to_string()));
            }
        }
    }

    scripts
}

/// Extract scripts from [project.scripts] section (PEP 621).
fn extract_pep621_scripts(content: &str) -> Vec<(String, String)> {
    let mut scripts = Vec::new();
    let mut in_scripts = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[project.scripts]" {
            in_scripts = true;
            continue;
        }

        if in_scripts {
            if line.starts_with('[') {
                break;
            }

            if let Some((name, value)) = line.split_once('=') {
                let name = name.trim().trim_matches('"');
                let value = value.trim().trim_matches('"');
                scripts.push((name.to_string(), value.to_string()));
            }
        }
    }

    scripts
}

/// Main entry point for the scanner plugin.
#[no_mangle]
pub extern "C" fn scan(project_path_ptr: *const u8, project_path_len: usize) -> *mut u8 {
    let project_path = unsafe {
        let slice = std::slice::from_raw_parts(project_path_ptr, project_path_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let commands = scan_project(project_path);
    let json = serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string());
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

/// Scan a project directory for Poetry commands.
pub fn scan_project(project_path: &str) -> Vec<PluginCommand> {
    let pyproject_path = std::path::Path::new(project_path).join("pyproject.toml");

    if pyproject_path.exists() {
        // In a real plugin, read through host API
        parse_pyproject_toml("[tool.poetry]")
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_poetry_commands() {
        let content = "[tool.poetry]\nname = \"myproject\"";
        let commands = parse_pyproject_toml(content);

        assert!(commands.iter().any(|c| c.name == "poetry install"));
        assert!(commands.iter().any(|c| c.name == "poetry build"));
        assert!(commands.iter().any(|c| c.name == "poetry run pytest"));
    }

    #[test]
    fn test_extract_scripts() {
        let content = r#"
[tool.poetry.scripts]
mycli = "mypackage.cli:main"
serve = "mypackage.server:run"
"#;

        let scripts = extract_scripts(content);
        assert_eq!(scripts.len(), 2);
        assert!(scripts.iter().any(|(n, _)| n == "mycli"));
        assert!(scripts.iter().any(|(n, _)| n == "serve"));
    }

    #[test]
    fn test_script_commands() {
        let content = r#"
[tool.poetry]
name = "test"

[tool.poetry.scripts]
mycli = "pkg:main"
"#;

        let commands = parse_pyproject_toml(content);
        assert!(commands.iter().any(|c| c.name == "poetry run mycli"));
    }

    #[test]
    fn test_extract_pep621_scripts() {
        let content = r#"
[project.scripts]
myapp = "mypackage:main"
"#;

        let scripts = extract_pep621_scripts(content);
        assert_eq!(scripts.len(), 1);
        assert!(scripts.iter().any(|(n, _)| n == "myapp"));
    }

    #[test]
    fn test_non_poetry_project() {
        let content = "[project]\nname = \"test\"";
        let commands = parse_pyproject_toml(content);

        // Should not have Poetry commands without [tool.poetry]
        assert!(!commands.iter().any(|c| c.name == "poetry install"));
    }
}
