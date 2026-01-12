//! Claude Code command target.
//!
//! Generates slash commands for Claude Code in Markdown + YAML frontmatter format.

use std::path::PathBuf;

use super::super::target::{CommandTarget, PalrunCommand};

/// Claude Code command target.
///
/// Claude Code uses Markdown files with YAML frontmatter for slash commands.
/// Commands are installed to `~/.claude/commands/palrun/`.
pub struct ClaudeCodeTarget;

impl CommandTarget for ClaudeCodeTarget {
    fn name(&self) -> &str {
        "claude"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn detect(&self) -> bool {
        dirs::home_dir().map(|h| h.join(".claude").exists()).unwrap_or(false)
    }

    fn install_path(&self) -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory found"))?;
        Ok(home.join(".claude/commands/palrun"))
    }

    fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
        let mut content = String::new();

        // YAML frontmatter
        content.push_str("---\n");
        content.push_str(&format!("name: palrun:{}\n", cmd.name));
        content.push_str(&format!("description: {}\n", cmd.description));
        content.push_str("---\n\n");

        // Command title
        content.push_str(&format!("# {}\n\n", cmd.name));

        // Description
        content.push_str(&format!("{}\n\n", cmd.description));

        // Arguments if any
        if !cmd.args.is_empty() {
            content.push_str("## Arguments\n\n");
            for arg in &cmd.args {
                let req = if arg.required { "(required)" } else { "(optional)" };
                content.push_str(&format!("- `{}` {} - {}\n", arg.name, req, arg.description));
            }
            content.push('\n');
        }

        // Command to run
        content.push_str("## Command\n\n");
        content.push_str("```bash\n");
        content.push_str(&cmd.palrun_command);
        content.push_str("\n```\n");

        Ok(content)
    }

    fn file_extension(&self) -> &str {
        "md"
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::CommandCategory;

    use super::*;

    #[test]
    fn test_claude_target_name() {
        let target = ClaudeCodeTarget;
        assert_eq!(target.name(), "claude");
        assert_eq!(target.display_name(), "Claude Code");
    }

    #[test]
    fn test_claude_target_extension() {
        let target = ClaudeCodeTarget;
        assert_eq!(target.file_extension(), "md");
    }

    #[test]
    fn test_claude_target_generate() {
        let target = ClaudeCodeTarget;
        let cmd = PalrunCommand {
            name: "test".to_string(),
            description: "Test command".to_string(),
            palrun_command: "palrun test".to_string(),
            category: CommandCategory::Utility,
            args: Vec::new(),
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("name: palrun:test"));
        assert!(content.contains("description: Test command"));
        assert!(content.contains("```bash"));
        assert!(content.contains("palrun test"));
    }

    #[test]
    fn test_claude_target_generate_with_args() {
        let target = ClaudeCodeTarget;
        let cmd = PalrunCommand {
            name: "analyze".to_string(),
            description: "Analyze codebase".to_string(),
            palrun_command: "palrun analyze".to_string(),
            category: CommandCategory::Project,
            args: vec![crate::commands::CommandArg::optional("verbose", "Enable verbose output")],
        };

        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("## Arguments"));
        assert!(content.contains("`verbose`"));
    }
}
