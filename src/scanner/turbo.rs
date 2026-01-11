//! Turbo monorepo scanner.
//!
//! Scans turbo.json to discover Turborepo pipelines.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Turborepo pipelines.
pub struct TurboScanner;

impl Scanner for TurboScanner {
    fn name(&self) -> &str {
        "turbo"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for turbo.json in the directory
        let turbo_json_path = dir.join("turbo.json");
        if !turbo_json_path.exists() {
            return Ok(commands);
        }

        // Parse turbo.json
        let config = parse_turbo_json(&turbo_json_path)?;

        // Extract pipeline tasks
        if let Some(pipeline) = config.pipeline {
            for task_name in pipeline.keys() {
                // Skip internal tasks (prefixed with #)
                if task_name.starts_with('#') {
                    continue;
                }

                // Handle scoped tasks (project#task)
                if task_name.contains('#') {
                    let parts: Vec<&str> = task_name.split('#').collect();
                    if parts.len() == 2 {
                        let project = parts[0];
                        let task = parts[1];
                        commands.push(
                            Command::new(
                                format!("turbo run {task} --filter={project}"),
                                format!("npx turbo run {task} --filter={project}"),
                            )
                            .with_description(format!("Run {task} for {project}"))
                            .with_source(CommandSource::Turbo)
                            .with_tags(vec!["turbo".to_string(), project.to_string()]),
                        );
                    }
                } else {
                    // Workspace-level task
                    commands.push(
                        Command::new(
                            format!("turbo run {task_name}"),
                            format!("npx turbo run {task_name}"),
                        )
                        .with_description(format!("Run {task_name} for all packages"))
                        .with_source(CommandSource::Turbo)
                        .with_tags(vec!["turbo".to_string(), "monorepo".to_string()]),
                    );
                }
            }
        }

        // Handle tasks in newer turbo.json format
        if let Some(tasks) = config.tasks {
            for task_name in tasks.keys() {
                if task_name.starts_with('#') {
                    continue;
                }

                if !task_name.contains('#') {
                    commands.push(
                        Command::new(
                            format!("turbo run {task_name}"),
                            format!("npx turbo run {task_name}"),
                        )
                        .with_description(format!("Run {task_name} for all packages"))
                        .with_source(CommandSource::Turbo)
                        .with_tags(vec!["turbo".to_string(), "monorepo".to_string()]),
                    );
                }
            }
        }

        // Add common Turbo commands
        commands.extend(get_common_turbo_commands());

        Ok(commands)
    }
}

/// Turborepo configuration (turbo.json).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct TurboConfig {
    /// Schema version
    #[serde(rename = "$schema")]
    schema: Option<String>,
    /// Pipeline definition (legacy)
    pipeline: Option<HashMap<String, PipelineTask>>,
    /// Tasks definition (new format)
    tasks: Option<HashMap<String, PipelineTask>>,
    /// Global dependencies
    global_dependencies: Option<Vec<String>>,
    /// Global env
    global_env: Option<Vec<String>>,
}

/// Pipeline task definition.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct PipelineTask {
    /// Task dependencies
    depends_on: Option<Vec<String>>,
    /// Output directories to cache
    outputs: Option<Vec<String>>,
    /// Environment variables
    env: Option<Vec<String>>,
    /// Cache behavior
    cache: Option<bool>,
    /// Persistent task
    persistent: Option<bool>,
}

/// Parse turbo.json file.
fn parse_turbo_json(path: &Path) -> anyhow::Result<TurboConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: TurboConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Get common Turbo commands.
fn get_common_turbo_commands() -> Vec<Command> {
    vec![
        Command::new("turbo run build", "npx turbo run build")
            .with_description("Build all packages")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "build".to_string()]),
        Command::new("turbo run test", "npx turbo run test")
            .with_description("Test all packages")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "test".to_string()]),
        Command::new("turbo run lint", "npx turbo run lint")
            .with_description("Lint all packages")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "lint".to_string()]),
        Command::new("turbo run dev", "npx turbo run dev")
            .with_description("Start development servers")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "dev".to_string()]),
        Command::new("turbo prune --scope=<package>", "npx turbo prune --scope=")
            .with_description("Prune workspace for deployment")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "deploy".to_string()]),
        Command::new("turbo daemon stop", "npx turbo daemon stop")
            .with_description("Stop Turbo daemon")
            .with_source(CommandSource::Turbo)
            .with_tags(vec!["turbo".to_string(), "daemon".to_string()]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turbo_scanner_name() {
        let scanner = TurboScanner;
        assert_eq!(scanner.name(), "turbo");
    }

    #[test]
    fn test_parse_turbo_json_legacy() {
        let json = r#"{
            "$schema": "https://turbo.build/schema.json",
            "pipeline": {
                "build": {
                    "dependsOn": ["^build"],
                    "outputs": ["dist/**"]
                },
                "test": {
                    "dependsOn": ["build"],
                    "outputs": []
                },
                "lint": {
                    "outputs": []
                }
            },
            "globalDependencies": ["package.json"]
        }"#;

        let config: TurboConfig = serde_json::from_str(json).unwrap();
        assert!(config.pipeline.is_some());
        let pipeline = config.pipeline.unwrap();
        assert_eq!(pipeline.len(), 3);
        assert!(pipeline.contains_key("build"));
        assert!(pipeline.contains_key("test"));
        assert!(pipeline.contains_key("lint"));
    }

    #[test]
    fn test_parse_turbo_json_new_format() {
        let json = r#"{
            "$schema": "https://turbo.build/schema.json",
            "tasks": {
                "build": {
                    "dependsOn": ["^build"],
                    "outputs": ["dist/**"]
                },
                "dev": {
                    "cache": false,
                    "persistent": true
                }
            }
        }"#;

        let config: TurboConfig = serde_json::from_str(json).unwrap();
        assert!(config.tasks.is_some());
        let tasks = config.tasks.unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains_key("build"));
        assert!(tasks.contains_key("dev"));
    }

    #[test]
    fn test_parse_pipeline_task() {
        let json = r#"{
            "dependsOn": ["^build"],
            "outputs": ["dist/**", ".next/**"],
            "env": ["NODE_ENV"],
            "cache": true,
            "persistent": false
        }"#;

        let task: PipelineTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.depends_on, Some(vec!["^build".to_string()]));
        assert_eq!(task.outputs, Some(vec!["dist/**".to_string(), ".next/**".to_string()]));
        assert_eq!(task.env, Some(vec!["NODE_ENV".to_string()]));
        assert_eq!(task.cache, Some(true));
        assert_eq!(task.persistent, Some(false));
    }

    #[test]
    fn test_common_turbo_commands() {
        let commands = get_common_turbo_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name.contains("build")));
        assert!(commands.iter().any(|c| c.name.contains("test")));
    }
}
