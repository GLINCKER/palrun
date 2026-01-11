//! Nx monorepo scanner.
//!
//! Scans nx.json and project.json files to discover Nx targets.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Nx monorepo targets.
pub struct NxScanner;

impl Scanner for NxScanner {
    fn name(&self) -> &str {
        "nx"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for nx.json in the directory
        let nx_json_path = dir.join("nx.json");
        if !nx_json_path.exists() {
            return Ok(commands);
        }

        // Parse nx.json for workspace-level targets
        if let Ok(nx_config) = parse_nx_json(&nx_json_path) {
            // Add workspace-level targets from targetDefaults
            if let Some(target_defaults) = nx_config.target_defaults {
                for target_name in target_defaults.keys() {
                    commands.push(
                        Command::new(
                            format!("nx run-many --target={target_name}"),
                            format!("npx nx run-many --target={target_name}"),
                        )
                        .with_description(format!("Run {target_name} for all projects"))
                        .with_source(CommandSource::NxProject("workspace".to_string()))
                        .with_tags(vec!["nx".to_string(), "monorepo".to_string()]),
                    );
                }
            }
        }

        // Scan for project.json files in the workspace
        commands.extend(scan_nx_projects(dir)?);

        // Add common Nx commands
        commands.extend(get_common_nx_commands(dir));

        Ok(commands)
    }
}

/// Nx workspace configuration (nx.json).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct NxConfig {
    /// Target defaults for all projects
    target_defaults: Option<HashMap<String, serde_json::Value>>,
    /// Named inputs for caching
    #[serde(default)]
    named_inputs: HashMap<String, serde_json::Value>,
    /// Default project (optional)
    default_project: Option<String>,
}

/// Nx project configuration (project.json).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct ProjectJson {
    /// Project name
    name: Option<String>,
    /// Project targets
    #[serde(default)]
    targets: HashMap<String, Target>,
    /// Project tags
    #[serde(default)]
    tags: Vec<String>,
}

/// Nx target definition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Target {
    /// Executor to use
    executor: Option<String>,
    /// Target options
    #[serde(default)]
    options: serde_json::Value,
    /// Target configurations
    #[serde(default)]
    configurations: HashMap<String, serde_json::Value>,
}

/// Parse nx.json file.
fn parse_nx_json(path: &Path) -> anyhow::Result<NxConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: NxConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Parse project.json file.
fn parse_project_json(path: &Path) -> anyhow::Result<ProjectJson> {
    let content = std::fs::read_to_string(path)?;
    let project: ProjectJson = serde_json::from_str(&content)?;
    Ok(project)
}

/// Scan for project.json files and extract targets.
fn scan_nx_projects(dir: &Path) -> anyhow::Result<Vec<Command>> {
    let mut commands = Vec::new();

    // Common project directories in Nx workspaces
    let project_dirs = ["apps", "libs", "packages", "projects"];

    for project_dir in &project_dirs {
        let path = dir.join(project_dir);
        if path.exists() {
            commands.extend(scan_project_directory(&path)?);
        }
    }

    // Also check root-level project.json
    let root_project = dir.join("project.json");
    if root_project.exists() {
        if let Ok(project) = parse_project_json(&root_project) {
            let project_name = project.name.clone().unwrap_or_else(|| "root".to_string());
            commands.extend(project_to_commands(&project_name, &project, dir));
        }
    }

    Ok(commands)
}

/// Scan a directory for project.json files.
fn scan_project_directory(dir: &Path) -> anyhow::Result<Vec<Command>> {
    let mut commands = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let project_json = path.join("project.json");
                if project_json.exists() {
                    if let Ok(project) = parse_project_json(&project_json) {
                        let project_name = project.name.clone().unwrap_or_else(|| {
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string()
                        });
                        commands.extend(project_to_commands(&project_name, &project, &path));
                    }
                }
            }
        }
    }

    Ok(commands)
}

/// Convert a project to commands.
fn project_to_commands(
    project_name: &str,
    project: &ProjectJson,
    _project_path: &Path,
) -> Vec<Command> {
    let mut commands = Vec::new();

    for (target_name, target) in &project.targets {
        let mut tags = vec!["nx".to_string(), project_name.to_string()];
        tags.extend(project.tags.clone());

        let mut cmd = Command::new(
            format!("nx {target_name} {project_name}"),
            format!("npx nx {target_name} {project_name}"),
        )
        .with_source(CommandSource::NxProject(project_name.to_string()))
        .with_tags(tags);

        // Add description based on executor
        if let Some(executor) = &target.executor {
            cmd = cmd.with_description(format!("Nx target using {executor}"));
        }

        commands.push(cmd);

        // Add configuration variants
        for config_name in target.configurations.keys() {
            commands.push(
                Command::new(
                    format!("nx {target_name} {project_name} --configuration={config_name}"),
                    format!("npx nx {target_name} {project_name} --configuration={config_name}"),
                )
                .with_description(format!("{target_name} with {config_name} configuration"))
                .with_source(CommandSource::NxProject(project_name.to_string()))
                .with_tags(vec![
                    "nx".to_string(),
                    project_name.to_string(),
                    config_name.clone(),
                ]),
            );
        }
    }

    commands
}

/// Get common Nx commands.
fn get_common_nx_commands(dir: &Path) -> Vec<Command> {
    let nx_json = dir.join("nx.json");
    if !nx_json.exists() {
        return Vec::new();
    }

    vec![
        Command::new("nx graph", "npx nx graph")
            .with_description("Visualize the project graph")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "visualization".to_string()]),
        Command::new("nx affected --target=build", "npx nx affected --target=build")
            .with_description("Build affected projects")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "affected".to_string()]),
        Command::new("nx affected --target=test", "npx nx affected --target=test")
            .with_description("Test affected projects")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "affected".to_string()]),
        Command::new("nx affected --target=lint", "npx nx affected --target=lint")
            .with_description("Lint affected projects")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "affected".to_string()]),
        Command::new("nx run-many --target=build --all", "npx nx run-many --target=build --all")
            .with_description("Build all projects")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "all".to_string()]),
        Command::new("nx run-many --target=test --all", "npx nx run-many --target=test --all")
            .with_description("Test all projects")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "all".to_string()]),
        Command::new("nx reset", "npx nx reset")
            .with_description("Reset Nx cache")
            .with_source(CommandSource::NxProject("workspace".to_string()))
            .with_tags(vec!["nx".to_string(), "cache".to_string()]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nx_scanner_name() {
        let scanner = NxScanner;
        assert_eq!(scanner.name(), "nx");
    }

    #[test]
    fn test_parse_nx_json() {
        let json = r#"{
            "targetDefaults": {
                "build": {
                    "dependsOn": ["^build"]
                },
                "test": {
                    "dependsOn": ["build"]
                }
            },
            "namedInputs": {
                "default": ["{projectRoot}/**/*"]
            }
        }"#;

        let config: NxConfig = serde_json::from_str(json).unwrap();
        assert!(config.target_defaults.is_some());
        let defaults = config.target_defaults.unwrap();
        assert!(defaults.contains_key("build"));
        assert!(defaults.contains_key("test"));
    }

    #[test]
    fn test_parse_project_json() {
        let json = r#"{
            "name": "my-app",
            "targets": {
                "build": {
                    "executor": "@nx/webpack:webpack",
                    "options": {},
                    "configurations": {
                        "production": {}
                    }
                },
                "serve": {
                    "executor": "@nx/webpack:dev-server"
                }
            },
            "tags": ["type:app"]
        }"#;

        let project: ProjectJson = serde_json::from_str(json).unwrap();
        assert_eq!(project.name, Some("my-app".to_string()));
        assert_eq!(project.targets.len(), 2);
        assert!(project.targets.contains_key("build"));
        assert!(project.targets.contains_key("serve"));
        assert_eq!(project.tags, vec!["type:app"]);
    }

    #[test]
    fn test_project_to_commands() {
        let json = r#"{
            "name": "my-app",
            "targets": {
                "build": {
                    "executor": "@nx/webpack:webpack",
                    "configurations": {
                        "production": {}
                    }
                }
            },
            "tags": []
        }"#;

        let project: ProjectJson = serde_json::from_str(json).unwrap();
        let commands = project_to_commands("my-app", &project, Path::new("."));

        // Should have base command + production configuration
        assert_eq!(commands.len(), 2);
        assert!(commands[0].name.contains("build"));
        assert!(commands[1].name.contains("production"));
    }
}
