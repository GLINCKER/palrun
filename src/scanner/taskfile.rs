//! Taskfile scanner.
//!
//! Scans Taskfile.yml/Taskfile.yaml to discover task runner tasks.
//! See: https://taskfile.dev

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Taskfile tasks.
pub struct TaskfileScanner;

impl Scanner for TaskfileScanner {
    fn name(&self) -> &str {
        "taskfile"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for Taskfile in various forms
        let taskfile_path = find_taskfile(dir);
        let taskfile_path = match taskfile_path {
            Some(p) => p,
            None => return Ok(commands),
        };

        // Parse the taskfile
        let content = std::fs::read_to_string(&taskfile_path)?;
        let taskfile: Taskfile = serde_yaml::from_str(&content)?;

        // Extract tasks
        if let Some(tasks) = taskfile.tasks {
            for (task_name, task) in tasks {
                // Skip internal tasks (starting with _)
                if task_name.starts_with('_') {
                    continue;
                }

                let mut cmd =
                    Command::new(format!("task {task_name}"), format!("task {task_name}"))
                        .with_source(CommandSource::Manual)
                        .with_tags(vec!["task".to_string(), "taskfile".to_string()]);

                // Add description if available
                if let Some(desc) = task.desc {
                    cmd = cmd.with_description(desc);
                } else if let Some(summary) = task.summary {
                    cmd = cmd.with_description(summary);
                }

                commands.push(cmd);
            }
        }

        // Add common task commands
        commands.push(
            Command::new("task --list", "task --list")
                .with_description("List all available tasks")
                .with_source(CommandSource::Manual)
                .with_tags(vec!["task".to_string(), "taskfile".to_string()]),
        );

        commands.push(
            Command::new("task --list-all", "task --list-all")
                .with_description("List all tasks including internal ones")
                .with_source(CommandSource::Manual)
                .with_tags(vec!["task".to_string(), "taskfile".to_string()]),
        );

        Ok(commands)
    }
}

/// Find the Taskfile in a directory.
fn find_taskfile(dir: &Path) -> Option<std::path::PathBuf> {
    let candidates = [
        "Taskfile.yml",
        "Taskfile.yaml",
        "taskfile.yml",
        "taskfile.yaml",
        "Taskfile.dist.yml",
        "Taskfile.dist.yaml",
    ];

    for candidate in &candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Taskfile configuration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Taskfile {
    /// Taskfile version
    version: Option<String>,
    /// Tasks defined in this file
    tasks: Option<HashMap<String, Task>>,
    /// Includes for other taskfiles
    includes: Option<HashMap<String, serde_yaml::Value>>,
}

/// A single task definition.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Task {
    /// Task description (shown in --list)
    desc: Option<String>,
    /// Longer task summary
    summary: Option<String>,
    /// Commands to run
    cmds: Option<Vec<serde_yaml::Value>>,
    /// Task dependencies
    deps: Option<Vec<serde_yaml::Value>>,
    /// Environment variables
    env: Option<HashMap<String, String>>,
    /// Whether task should run always (ignore up-to-date check)
    #[serde(default)]
    run: Option<String>,
    /// Sources for up-to-date checking
    sources: Option<Vec<String>>,
    /// Generates for up-to-date checking
    generates: Option<Vec<String>>,
    /// Whether to run in silent mode
    #[serde(default)]
    silent: bool,
    /// Internal task (not shown in --list)
    #[serde(default)]
    internal: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taskfile_scanner_name() {
        let scanner = TaskfileScanner;
        assert_eq!(scanner.name(), "taskfile");
    }

    #[test]
    fn test_parse_simple_taskfile() {
        let yaml = r#"
version: '3'

tasks:
  build:
    desc: Build the project
    cmds:
      - go build ./...

  test:
    desc: Run tests
    cmds:
      - go test ./...

  lint:
    desc: Run linters
    cmds:
      - golangci-lint run
"#;

        let taskfile: Taskfile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(taskfile.version, Some("3".to_string()));
        assert!(taskfile.tasks.is_some());
        let tasks = taskfile.tasks.unwrap();
        assert_eq!(tasks.len(), 3);
        assert!(tasks.contains_key("build"));
        assert!(tasks.contains_key("test"));
        assert!(tasks.contains_key("lint"));
    }

    #[test]
    fn test_parse_task_with_deps() {
        let yaml = r#"
version: '3'

tasks:
  build:
    desc: Build the project
    deps: [generate]
    cmds:
      - go build ./...

  generate:
    desc: Generate code
    cmds:
      - go generate ./...
"#;

        let taskfile: Taskfile = serde_yaml::from_str(yaml).unwrap();
        let tasks = taskfile.tasks.unwrap();
        let build = &tasks["build"];
        assert!(build.deps.is_some());
    }

    #[test]
    fn test_parse_task_with_env() {
        let yaml = r#"
version: '3'

tasks:
  test:
    desc: Run tests
    env:
      GO_ENV: test
      VERBOSE: "true"
    cmds:
      - go test ./...
"#;

        let taskfile: Taskfile = serde_yaml::from_str(yaml).unwrap();
        let tasks = taskfile.tasks.unwrap();
        let test = &tasks["test"];
        assert!(test.env.is_some());
        let env = test.env.as_ref().unwrap();
        assert_eq!(env.get("GO_ENV"), Some(&"test".to_string()));
    }

    #[test]
    fn test_internal_task_detection() {
        let yaml = r#"
version: '3'

tasks:
  _internal:
    internal: true
    cmds:
      - echo "internal"

  public:
    desc: Public task
    cmds:
      - echo "public"
"#;

        let taskfile: Taskfile = serde_yaml::from_str(yaml).unwrap();
        let tasks = taskfile.tasks.unwrap();
        assert!(tasks["_internal"].internal);
        assert!(!tasks["public"].internal);
    }
}
