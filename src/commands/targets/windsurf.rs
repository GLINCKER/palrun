//! Windsurf command target.
//!
//! Generates slash commands for Windsurf (Codeium's AI IDE) in JSON format.

use std::path::PathBuf;

use serde_json::json;

use super::super::target::{CommandTarget, PalrunCommand};

/// Windsurf IDE command target.
///
/// Windsurf is Codeium's AI-powered IDE, based on VSCode.
/// Commands are installed to `.windsurf/commands/` in the project directory
/// or `~/.windsurf/commands/` for global commands.
pub struct WindsurfTarget;

impl CommandTarget for WindsurfTarget {
    fn name(&self) -> &str {
        "windsurf"
    }

    fn display_name(&self) -> &str {
        "Windsurf"
    }

    fn detect(&self) -> bool {
        // Check for global .windsurf directory
        let global = dirs::home_dir().map(|h| h.join(".windsurf").exists()).unwrap_or(false);

        // Check for project-level .windsurf directory
        let local =
            std::env::current_dir().map(|cwd| cwd.join(".windsurf").exists()).unwrap_or(false);

        global || local
    }

    fn install_path(&self) -> anyhow::Result<PathBuf> {
        // Prefer project-level if .windsurf exists, otherwise use global
        if let Ok(cwd) = std::env::current_dir() {
            let project_windsurf = cwd.join(".windsurf");
            if project_windsurf.exists() {
                return Ok(project_windsurf.join("commands/palrun"));
            }
        }

        // Fall back to global
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory found"))?;
        Ok(home.join(".windsurf/commands/palrun"))
    }

    fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
        let args: Vec<_> = cmd
            .args
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "description": a.description,
                    "required": a.required,
                    "default": a.default
                })
            })
            .collect();

        let command = json!({
            "name": format!("palrun:{}", cmd.name),
            "description": cmd.description,
            "command": cmd.palrun_command,
            "category": cmd.category.display_name(),
            "arguments": args,
            "source": "palrun"
        });

        Ok(serde_json::to_string_pretty(&command)?)
    }

    fn file_extension(&self) -> &str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::CommandCategory;

    use super::*;

    #[test]
    fn test_windsurf_target_name() {
        let target = WindsurfTarget;
        assert_eq!(target.name(), "windsurf");
        assert_eq!(target.display_name(), "Windsurf");
    }

    #[test]
    fn test_windsurf_target_extension() {
        let target = WindsurfTarget;
        assert_eq!(target.file_extension(), "json");
    }

    #[test]
    fn test_windsurf_target_generate() {
        let target = WindsurfTarget;
        let cmd = PalrunCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
            palrun_command: "palrun test".to_string(),
            category: CommandCategory::Utility,
            args: Vec::new(),
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("\"name\": \"palrun:test\""));
        assert!(content.contains("\"description\": \"Test command\""));
        assert!(content.contains("\"command\": \"palrun test\""));
        assert!(content.contains("\"source\": \"palrun\""));
    }
}
