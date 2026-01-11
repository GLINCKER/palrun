//! Python project scanner.
//!
//! Scans pyproject.toml, setup.py, and requirements.txt to discover
//! Python project commands.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Python projects.
pub struct PythonScanner;

impl Scanner for PythonScanner {
    fn name(&self) -> &str {
        "python"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        let pyproject_path = dir.join("pyproject.toml");
        let setup_py_path = dir.join("setup.py");
        let requirements_path = dir.join("requirements.txt");

        // Check for pyproject.toml first (modern Python projects)
        if pyproject_path.exists() {
            let config = parse_pyproject_toml(&pyproject_path)?;
            let source = CommandSource::Python(pyproject_path.clone());
            let tool_type = detect_tool_type(&config);

            // Get project name for context
            let project_name = config
                .project
                .as_ref()
                .and_then(|p| p.name.clone())
                .or_else(|| {
                    config
                        .tool
                        .as_ref()
                        .and_then(|t| t.poetry.as_ref())
                        .and_then(|p| p.name.clone())
                })
                .unwrap_or_else(|| "project".to_string());

            // Add tool-specific commands
            match tool_type {
                ToolType::Poetry => {
                    commands.extend(generate_poetry_commands(&config, &source, &project_name));
                }
                ToolType::Pdm => {
                    commands.extend(generate_pdm_commands(&config, &source, &project_name));
                }
                ToolType::Hatch => {
                    commands.extend(generate_hatch_commands(&config, &source, &project_name));
                }
                ToolType::Generic => {
                    commands.extend(generate_generic_commands(&source, &project_name));
                }
            }

            // Add pytest commands if configured
            if has_pytest_config(&config) || pyproject_path.exists() {
                commands.push(
                    Command::new("python -m pytest", "python -m pytest")
                        .with_description(format!("Run tests for {project_name}"))
                        .with_source(source.clone())
                        .with_tags(vec!["python".to_string(), "test".to_string()]),
                );

                commands.push(
                    Command::new("python -m pytest -v", "python -m pytest -v")
                        .with_description("Run tests with verbose output")
                        .with_source(source.clone())
                        .with_tags(vec!["python".to_string(), "test".to_string()]),
                );

                commands.push(
                    Command::new("python -m pytest --cov", "python -m pytest --cov")
                        .with_description("Run tests with coverage")
                        .with_source(source.clone())
                        .with_tags(vec![
                            "python".to_string(),
                            "test".to_string(),
                            "coverage".to_string(),
                        ]),
                );
            }

            return Ok(commands);
        }

        // Check for setup.py (legacy projects)
        if setup_py_path.exists() {
            let source = CommandSource::Python(setup_py_path.clone());

            commands.push(
                Command::new("python setup.py install", "python setup.py install")
                    .with_description("Install package")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "setup.py".to_string()]),
            );

            commands.push(
                Command::new("python setup.py develop", "python setup.py develop")
                    .with_description("Install package in development mode")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "setup.py".to_string()]),
            );

            commands.push(
                Command::new("python setup.py build", "python setup.py build")
                    .with_description("Build package")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "setup.py".to_string()]),
            );

            commands.push(
                Command::new("python setup.py test", "python setup.py test")
                    .with_description("Run tests")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "test".to_string()]),
            );

            commands.push(
                Command::new("python setup.py sdist", "python setup.py sdist")
                    .with_description("Create source distribution")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "dist".to_string()]),
            );

            commands.push(
                Command::new("python setup.py bdist_wheel", "python setup.py bdist_wheel")
                    .with_description("Create wheel distribution")
                    .with_source(source.clone())
                    .with_tags(vec!["python".to_string(), "dist".to_string()]),
            );

            return Ok(commands);
        }

        // Check for requirements.txt (simple projects)
        if requirements_path.exists() {
            let source = CommandSource::Python(requirements_path.clone());

            commands.push(
                Command::new(
                    "python -m pip install -r requirements.txt",
                    "python -m pip install -r requirements.txt",
                )
                .with_description("Install dependencies from requirements.txt")
                .with_source(source.clone())
                .with_tags(vec!["python".to_string(), "pip".to_string()]),
            );

            // Check for dev requirements
            let dev_requirements = dir.join("requirements-dev.txt");
            if dev_requirements.exists() {
                commands.push(
                    Command::new(
                        "python -m pip install -r requirements-dev.txt",
                        "python -m pip install -r requirements-dev.txt",
                    )
                    .with_description("Install dev dependencies")
                    .with_source(source.clone())
                    .with_tags(vec![
                        "python".to_string(),
                        "pip".to_string(),
                        "dev".to_string(),
                    ]),
                );
            }

            // Also check for test requirements
            let test_requirements = dir.join("requirements-test.txt");
            if test_requirements.exists() {
                commands.push(
                    Command::new(
                        "python -m pip install -r requirements-test.txt",
                        "python -m pip install -r requirements-test.txt",
                    )
                    .with_description("Install test dependencies")
                    .with_source(source.clone())
                    .with_tags(vec![
                        "python".to_string(),
                        "pip".to_string(),
                        "test".to_string(),
                    ]),
                );
            }

            return Ok(commands);
        }

        Ok(commands)
    }
}

/// Detected Python tool type.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ToolType {
    Poetry,
    Pdm,
    Hatch,
    Generic,
}

/// pyproject.toml configuration.
#[derive(Debug, Deserialize, Default)]
struct PyProjectConfig {
    /// PEP 621 project metadata
    project: Option<ProjectMetadata>,
    /// Tool-specific configurations
    tool: Option<ToolConfig>,
}

/// PEP 621 project metadata.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ProjectMetadata {
    /// Project name
    name: Option<String>,
    /// Project version
    version: Option<String>,
    /// Project scripts (entry points)
    scripts: Option<HashMap<String, String>>,
}

/// Tool-specific configurations.
#[derive(Debug, Deserialize)]
struct ToolConfig {
    /// Poetry configuration
    poetry: Option<PoetryConfig>,
    /// PDM configuration
    pdm: Option<PdmConfig>,
    /// Hatch configuration
    hatch: Option<HatchConfig>,
    /// pytest configuration
    pytest: Option<PytestConfig>,
}

/// Poetry configuration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PoetryConfig {
    /// Package name
    name: Option<String>,
    /// Poetry scripts
    scripts: Option<HashMap<String, String>>,
}

/// PDM configuration.
#[derive(Debug, Deserialize)]
struct PdmConfig {
    /// PDM scripts
    scripts: Option<HashMap<String, PdmScript>>,
}

/// PDM script can be a simple string or a complex object.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum PdmScript {
    /// Simple command string
    Simple(String),
    /// Complex script with cmd and other options
    Complex {
        cmd: Option<String>,
        shell: Option<String>,
        call: Option<String>,
        composite: Option<Vec<String>>,
        help: Option<String>,
    },
}

impl PdmScript {
    /// Get the command to run.
    fn get_command(&self) -> Option<String> {
        match self {
            Self::Simple(cmd) => Some(cmd.clone()),
            Self::Complex { cmd, shell, call, composite, .. } => {
                if let Some(c) = cmd {
                    Some(c.clone())
                } else if let Some(s) = shell {
                    Some(s.clone())
                } else if let Some(c) = call {
                    Some(c.clone())
                } else if let Some(comp) = composite {
                    Some(comp.join(" && "))
                } else {
                    None
                }
            }
        }
    }

    /// Get the help text.
    fn get_help(&self) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Complex { help, .. } => help.as_deref(),
        }
    }
}

/// Hatch configuration.
#[derive(Debug, Deserialize)]
struct HatchConfig {
    /// Hatch environments with scripts
    envs: Option<HashMap<String, HatchEnv>>,
}

/// Hatch environment configuration.
#[derive(Debug, Deserialize)]
struct HatchEnv {
    /// Scripts in this environment
    scripts: Option<HashMap<String, HatchScript>>,
}

/// Hatch script can be a string or array of strings.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HatchScript {
    /// Simple command
    Simple(String),
    /// Multiple commands
    Multiple(Vec<String>),
}

impl HatchScript {
    /// Get the command string.
    fn get_command(&self) -> String {
        match self {
            Self::Simple(cmd) => cmd.clone(),
            Self::Multiple(cmds) => cmds.join(" && "),
        }
    }
}

/// pytest configuration (just to detect its presence).
#[derive(Debug, Deserialize)]
struct PytestConfig {
    /// pytest.ini_options or just presence
    #[serde(flatten)]
    _options: HashMap<String, toml::Value>,
}

/// Parse pyproject.toml file.
fn parse_pyproject_toml(path: &Path) -> anyhow::Result<PyProjectConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: PyProjectConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Detect which Python tool is being used.
fn detect_tool_type(config: &PyProjectConfig) -> ToolType {
    if let Some(tool) = &config.tool {
        if tool.poetry.is_some() {
            return ToolType::Poetry;
        }
        if tool.pdm.is_some() {
            return ToolType::Pdm;
        }
        if tool.hatch.is_some() {
            return ToolType::Hatch;
        }
    }
    ToolType::Generic
}

/// Check if pytest is configured.
fn has_pytest_config(config: &PyProjectConfig) -> bool {
    config.tool.as_ref().is_some_and(|t| t.pytest.is_some())
}

/// Generate Poetry-specific commands.
fn generate_poetry_commands(
    config: &PyProjectConfig,
    source: &CommandSource,
    project_name: &str,
) -> Vec<Command> {
    let mut commands = Vec::new();

    // Basic Poetry commands
    commands.push(
        Command::new("poetry install", "poetry install")
            .with_description(format!("Install dependencies for {project_name}"))
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "poetry".to_string()]),
    );

    commands.push(
        Command::new("poetry update", "poetry update")
            .with_description("Update dependencies")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "poetry".to_string()]),
    );

    commands.push(
        Command::new("poetry build", "poetry build")
            .with_description("Build package")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "poetry".to_string(), "build".to_string()]),
    );

    commands.push(
        Command::new("poetry publish", "poetry publish")
            .with_description("Publish package to PyPI")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "poetry".to_string()]),
    );

    commands.push(
        Command::new("poetry shell", "poetry shell")
            .with_description("Activate virtual environment")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "poetry".to_string()]),
    );

    // Add Poetry scripts
    if let Some(tool) = &config.tool {
        if let Some(poetry) = &tool.poetry {
            if let Some(scripts) = &poetry.scripts {
                for (name, _entry_point) in scripts {
                    commands.push(
                        Command::new(format!("poetry run {name}"), format!("poetry run {name}"))
                            .with_description(format!("Run {name} script"))
                            .with_source(source.clone())
                            .with_tags(vec![
                                "python".to_string(),
                                "poetry".to_string(),
                                "script".to_string(),
                            ]),
                    );
                }
            }
        }
    }

    commands
}

/// Generate PDM-specific commands.
fn generate_pdm_commands(
    config: &PyProjectConfig,
    source: &CommandSource,
    project_name: &str,
) -> Vec<Command> {
    let mut commands = Vec::new();

    // Basic PDM commands
    commands.push(
        Command::new("pdm install", "pdm install")
            .with_description(format!("Install dependencies for {project_name}"))
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pdm".to_string()]),
    );

    commands.push(
        Command::new("pdm update", "pdm update")
            .with_description("Update dependencies")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pdm".to_string()]),
    );

    commands.push(
        Command::new("pdm build", "pdm build")
            .with_description("Build package")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pdm".to_string(), "build".to_string()]),
    );

    commands.push(
        Command::new("pdm publish", "pdm publish")
            .with_description("Publish package to PyPI")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pdm".to_string()]),
    );

    // Add PDM scripts
    if let Some(tool) = &config.tool {
        if let Some(pdm) = &tool.pdm {
            if let Some(scripts) = &pdm.scripts {
                for (name, script) in scripts {
                    let description = script
                        .get_help()
                        .map(String::from)
                        .or_else(|| script.get_command())
                        .unwrap_or_else(|| format!("Run {name} script"));

                    commands.push(
                        Command::new(format!("pdm run {name}"), format!("pdm run {name}"))
                            .with_description(description)
                            .with_source(source.clone())
                            .with_tags(vec![
                                "python".to_string(),
                                "pdm".to_string(),
                                "script".to_string(),
                            ]),
                    );
                }
            }
        }
    }

    commands
}

/// Generate Hatch-specific commands.
fn generate_hatch_commands(
    config: &PyProjectConfig,
    source: &CommandSource,
    project_name: &str,
) -> Vec<Command> {
    let mut commands = Vec::new();

    // Basic Hatch commands
    commands.push(
        Command::new("hatch env create", "hatch env create")
            .with_description(format!("Create environment for {project_name}"))
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "hatch".to_string()]),
    );

    commands.push(
        Command::new("hatch build", "hatch build")
            .with_description("Build package")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "hatch".to_string(), "build".to_string()]),
    );

    commands.push(
        Command::new("hatch publish", "hatch publish")
            .with_description("Publish package to PyPI")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "hatch".to_string()]),
    );

    commands.push(
        Command::new("hatch shell", "hatch shell")
            .with_description("Activate shell in default environment")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "hatch".to_string()]),
    );

    commands.push(
        Command::new("hatch test", "hatch test")
            .with_description("Run tests")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "hatch".to_string(), "test".to_string()]),
    );

    // Add Hatch environment scripts
    if let Some(tool) = &config.tool {
        if let Some(hatch) = &tool.hatch {
            if let Some(envs) = &hatch.envs {
                for (env_name, env) in envs {
                    if let Some(scripts) = &env.scripts {
                        for (script_name, script) in scripts {
                            let cmd = if env_name == "default" {
                                format!("hatch run {script_name}")
                            } else {
                                format!("hatch run {env_name}:{script_name}")
                            };

                            commands.push(
                                Command::new(&cmd, &cmd)
                                    .with_description(script.get_command())
                                    .with_source(source.clone())
                                    .with_tags(vec![
                                        "python".to_string(),
                                        "hatch".to_string(),
                                        "script".to_string(),
                                    ]),
                            );
                        }
                    }
                }
            }
        }
    }

    commands
}

/// Generate generic Python commands (no specific tool detected).
fn generate_generic_commands(source: &CommandSource, project_name: &str) -> Vec<Command> {
    let mut commands = Vec::new();

    commands.push(
        Command::new("pip install -e .", "pip install -e .")
            .with_description(format!("Install {project_name} in editable mode"))
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pip".to_string()]),
    );

    commands.push(
        Command::new("pip install .", "pip install .")
            .with_description(format!("Install {project_name}"))
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "pip".to_string()]),
    );

    commands.push(
        Command::new("python -m build", "python -m build")
            .with_description("Build package")
            .with_source(source.clone())
            .with_tags(vec!["python".to_string(), "build".to_string()]),
    );

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_scanner_name() {
        let scanner = PythonScanner;
        assert_eq!(scanner.name(), "python");
    }

    #[test]
    fn test_parse_simple_pyproject() {
        let toml = r#"
[project]
name = "my-project"
version = "0.1.0"
"#;

        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(config.project.is_some());
        let project = config.project.unwrap();
        assert_eq!(project.name, Some("my-project".to_string()));
        assert_eq!(project.version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_parse_poetry_pyproject() {
        let toml = r#"
[tool.poetry]
name = "poetry-project"

[tool.poetry.scripts]
serve = "myapp.main:run_server"
worker = "myapp.worker:start"
"#;

        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(config.tool.is_some());
        let tool = config.tool.unwrap();
        assert!(tool.poetry.is_some());
        let poetry = tool.poetry.unwrap();
        assert_eq!(poetry.name, Some("poetry-project".to_string()));

        let scripts = poetry.scripts.unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts.contains_key("serve"));
        assert!(scripts.contains_key("worker"));
    }

    #[test]
    fn test_parse_pdm_pyproject() {
        let toml = r#"
[project]
name = "pdm-project"

[tool.pdm.scripts]
start = "python app.py"
test = { cmd = "pytest tests/", help = "Run test suite" }
lint = { composite = ["ruff check .", "mypy ."] }
"#;

        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(config.tool.is_some());
        let tool = config.tool.unwrap();
        assert!(tool.pdm.is_some());
        let pdm = tool.pdm.unwrap();

        let scripts = pdm.scripts.unwrap();
        assert_eq!(scripts.len(), 3);

        // Test simple script
        if let Some(PdmScript::Simple(cmd)) = scripts.get("start") {
            assert_eq!(cmd, "python app.py");
        } else {
            panic!("Expected simple script for 'start'");
        }

        // Test complex script with cmd
        if let Some(script) = scripts.get("test") {
            assert_eq!(script.get_command(), Some("pytest tests/".to_string()));
            assert_eq!(script.get_help(), Some("Run test suite"));
        } else {
            panic!("Expected script 'test'");
        }

        // Test composite script
        if let Some(script) = scripts.get("lint") {
            assert_eq!(script.get_command(), Some("ruff check . && mypy .".to_string()));
        } else {
            panic!("Expected script 'lint'");
        }
    }

    #[test]
    fn test_parse_hatch_pyproject() {
        let toml = r#"
[project]
name = "hatch-project"

[tool.hatch.envs.default.scripts]
test = "pytest"
cov = "pytest --cov"

[tool.hatch.envs.lint.scripts]
all = ["ruff check .", "mypy ."]
"#;

        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(config.tool.is_some());
        let tool = config.tool.unwrap();
        assert!(tool.hatch.is_some());
        let hatch = tool.hatch.unwrap();

        let envs = hatch.envs.unwrap();
        assert!(envs.contains_key("default"));
        assert!(envs.contains_key("lint"));

        let default_env = envs.get("default").unwrap();
        let default_scripts = default_env.scripts.as_ref().unwrap();
        assert!(default_scripts.contains_key("test"));
        assert!(default_scripts.contains_key("cov"));
    }

    #[test]
    fn test_detect_tool_type_poetry() {
        let toml = r#"
[tool.poetry]
name = "test"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert_eq!(detect_tool_type(&config), ToolType::Poetry);
    }

    #[test]
    fn test_detect_tool_type_pdm() {
        let toml = r#"
[tool.pdm]
version = "1.0"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert_eq!(detect_tool_type(&config), ToolType::Pdm);
    }

    #[test]
    fn test_detect_tool_type_hatch() {
        let toml = r#"
[tool.hatch]
version = "1.0"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert_eq!(detect_tool_type(&config), ToolType::Hatch);
    }

    #[test]
    fn test_detect_tool_type_generic() {
        let toml = r#"
[project]
name = "generic-project"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert_eq!(detect_tool_type(&config), ToolType::Generic);
    }

    #[test]
    fn test_has_pytest_config() {
        let toml = r#"
[tool.pytest]
ini_options = { addopts = "-v" }
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(has_pytest_config(&config));
    }

    #[test]
    fn test_no_pytest_config() {
        let toml = r#"
[project]
name = "no-pytest"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        assert!(!has_pytest_config(&config));
    }

    #[test]
    fn test_pdm_script_simple() {
        let script = PdmScript::Simple("python app.py".to_string());
        assert_eq!(script.get_command(), Some("python app.py".to_string()));
        assert_eq!(script.get_help(), None);
    }

    #[test]
    fn test_pdm_script_complex() {
        let script = PdmScript::Complex {
            cmd: Some("pytest".to_string()),
            shell: None,
            call: None,
            composite: None,
            help: Some("Run tests".to_string()),
        };
        assert_eq!(script.get_command(), Some("pytest".to_string()));
        assert_eq!(script.get_help(), Some("Run tests"));
    }

    #[test]
    fn test_hatch_script_simple() {
        let script = HatchScript::Simple("pytest".to_string());
        assert_eq!(script.get_command(), "pytest");
    }

    #[test]
    fn test_hatch_script_multiple() {
        let script = HatchScript::Multiple(vec!["ruff check .".to_string(), "mypy .".to_string()]);
        assert_eq!(script.get_command(), "ruff check . && mypy .");
    }

    #[test]
    fn test_generate_poetry_commands() {
        let toml = r#"
[tool.poetry]
name = "test-poetry"

[tool.poetry.scripts]
serve = "app:main"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        let source = CommandSource::Python(std::path::PathBuf::from("pyproject.toml"));
        let commands = generate_poetry_commands(&config, &source, "test-poetry");

        // Should have basic commands plus one script
        assert!(commands.iter().any(|c| c.name == "poetry install"));
        assert!(commands.iter().any(|c| c.name == "poetry build"));
        assert!(commands.iter().any(|c| c.name == "poetry run serve"));
    }

    #[test]
    fn test_generate_pdm_commands() {
        let toml = r#"
[tool.pdm.scripts]
test = "pytest"
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        let source = CommandSource::Python(std::path::PathBuf::from("pyproject.toml"));
        let commands = generate_pdm_commands(&config, &source, "test-pdm");

        assert!(commands.iter().any(|c| c.name == "pdm install"));
        assert!(commands.iter().any(|c| c.name == "pdm run test"));
    }

    #[test]
    fn test_generate_hatch_commands() {
        let toml = r#"
[tool.hatch.envs.default.scripts]
test = "pytest"

[tool.hatch.envs.lint.scripts]
check = "ruff check ."
"#;
        let config: PyProjectConfig = toml::from_str(toml).unwrap();
        let source = CommandSource::Python(std::path::PathBuf::from("pyproject.toml"));
        let commands = generate_hatch_commands(&config, &source, "test-hatch");

        assert!(commands.iter().any(|c| c.name == "hatch build"));
        assert!(commands.iter().any(|c| c.name == "hatch run test"));
        assert!(commands.iter().any(|c| c.name == "hatch run lint:check"));
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let scanner = PythonScanner;
        let result = scanner.scan(std::path::Path::new("/nonexistent/path"));
        // Should return Ok with empty vec for non-Python projects
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
