//! Aider command target.
//!
//! Generates slash commands for Aider in Markdown format.

use std::path::PathBuf;

use super::super::target::{CommandTarget, PalrunCommand};

/// Aider command target.
///
/// Aider is an AI pair programming tool that works in the terminal.
/// It uses markdown files for custom commands, loaded from `.aider/commands/`.
pub struct AiderTarget;

impl CommandTarget for AiderTarget {
    fn name(&self) -> &str {
        "aider"
    }

    fn display_name(&self) -> &str {
        "Aider"
    }

    fn detect(&self) -> bool {
        // Check for .aider directory in current project
        let local = std::env::current_dir().map(|cwd| cwd.join(".aider").exists()).unwrap_or(false);

        // Check for global .aider directory
        let global = dirs::home_dir().map(|h| h.join(".aider").exists()).unwrap_or(false);

        local || global
    }

    fn install_path(&self) -> anyhow::Result<PathBuf> {
        // Prefer project-level if .aider exists
        if let Ok(cwd) = std::env::current_dir() {
            let project_aider = cwd.join(".aider");
            if project_aider.exists() {
                return Ok(project_aider.join("commands/palrun"));
            }
        }

        // Fall back to global
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory found"))?;
        Ok(home.join(".aider/commands/palrun"))
    }

    fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
        let mut content = String::new();

        // Command header
        content.push_str(&format!("# /palrun-{}\n\n", cmd.name));

        // Description
        content.push_str(&format!("{}\n\n", cmd.description));

        // Arguments section
        if !cmd.args.is_empty() {
            content.push_str("## Arguments\n\n");
            for arg in &cmd.args {
                let req = if arg.required { "required" } else { "optional" };
                if let Some(default) = &arg.default {
                    content.push_str(&format!(
                        "- `{}` ({}) - {} [default: {}]\n",
                        arg.name, req, arg.description, default
                    ));
                } else {
                    content
                        .push_str(&format!("- `{}` ({}) - {}\n", arg.name, req, arg.description));
                }
            }
            content.push('\n');
        }

        // Usage section
        content.push_str("## Usage\n\n");
        content.push_str("Run this command in your terminal:\n\n");
        content.push_str("```bash\n");
        content.push_str(&cmd.palrun_command);
        content.push_str("\n```\n\n");

        // Category tag
        content.push_str(&format!("---\n*Category: {}*\n", cmd.category.display_name()));

        Ok(content)
    }

    fn file_extension(&self) -> &str {
        "md"
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{CommandArg, CommandCategory};

    use super::*;

    #[test]
    fn test_aider_target_name() {
        let target = AiderTarget;
        assert_eq!(target.name(), "aider");
        assert_eq!(target.display_name(), "Aider");
    }

    #[test]
    fn test_aider_target_extension() {
        let target = AiderTarget;
        assert_eq!(target.file_extension(), "md");
    }

    #[test]
    fn test_aider_target_generate() {
        let target = AiderTarget;
        let cmd = PalrunCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
            palrun_command: "palrun test".to_string(),
            category: CommandCategory::Utility,
            args: Vec::new(),
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("# /palrun-test"));
        assert!(content.contains("Test command"));
        assert!(content.contains("```bash"));
        assert!(content.contains("palrun test"));
        assert!(content.contains("Category: Utility"));
    }

    #[test]
    fn test_aider_target_generate_with_args() {
        let target = AiderTarget;
        let cmd = PalrunCommand {
            name: "build".to_string(),
            description: "Build the project".to_string(),
            palrun_command: "palrun build".to_string(),
            category: CommandCategory::Execution,
            args: vec![
                CommandArg::optional("target", "Build target"),
                CommandArg::required("config", "Config file"),
            ],
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("## Arguments"));
        assert!(content.contains("`target` (optional)"));
        assert!(content.contains("`config` (required)"));
    }
}
