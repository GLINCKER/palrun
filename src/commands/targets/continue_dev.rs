//! Continue.dev command target.
//!
//! Generates slash commands for Continue.dev in its configuration format.

use std::path::PathBuf;

use serde_json::json;

use super::super::target::{CommandTarget, PalrunCommand};

/// Continue.dev command target.
///
/// Continue is an open-source AI code assistant that works with any LLM.
/// Commands are defined in `~/.continue/config.json` under the `slashCommands` array.
/// We generate individual command files that can be imported.
pub struct ContinueDevTarget;

impl CommandTarget for ContinueDevTarget {
    fn name(&self) -> &str {
        "continue"
    }

    fn display_name(&self) -> &str {
        "Continue.dev"
    }

    fn detect(&self) -> bool {
        // Check for ~/.continue directory
        dirs::home_dir().map(|h| h.join(".continue").exists()).unwrap_or(false)
    }

    fn install_path(&self) -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory found"))?;
        Ok(home.join(".continue/commands/palrun"))
    }

    fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
        // Continue.dev uses a specific format for slash commands
        // The command runs a shell command and streams the output
        let args: Vec<_> = cmd
            .args
            .iter()
            .map(|a| {
                json!({
                    "name": a.name,
                    "description": a.description,
                    "required": a.required
                })
            })
            .collect();

        let command = json!({
            "name": cmd.name,
            "description": cmd.description,
            "command": {
                "type": "shell",
                "command": cmd.palrun_command
            },
            "params": args,
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
    fn test_continue_target_name() {
        let target = ContinueDevTarget;
        assert_eq!(target.name(), "continue");
        assert_eq!(target.display_name(), "Continue.dev");
    }

    #[test]
    fn test_continue_target_extension() {
        let target = ContinueDevTarget;
        assert_eq!(target.file_extension(), "json");
    }

    #[test]
    fn test_continue_target_generate() {
        let target = ContinueDevTarget;
        let cmd = PalrunCommand {
            name: "analyze".to_string(),
            description: "Analyze project".to_string(),
            palrun_command: "palrun analyze".to_string(),
            category: CommandCategory::Project,
            args: Vec::new(),
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("\"name\": \"analyze\""));
        assert!(content.contains("\"description\": \"Analyze project\""));
        assert!(content.contains("\"type\": \"shell\""));
        assert!(content.contains("\"command\": \"palrun analyze\""));
        assert!(content.contains("\"source\": \"palrun\""));
    }
}
