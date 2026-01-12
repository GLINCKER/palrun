//! Command target abstraction for IDE-agnostic command generation.
//!
//! Defines the `CommandTarget` trait that allows Palrun to generate
//! slash commands for any AI IDE (Claude Code, Cursor, Windsurf, etc.).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Trait for IDE-specific command targets.
///
/// Each supported IDE implements this trait to define how
/// Palrun commands should be generated and installed.
pub trait CommandTarget: Send + Sync {
    /// Name of the target IDE (e.g., "claude", "cursor").
    fn name(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str {
        self.name()
    }

    /// Check if this IDE is installed on the system.
    fn detect(&self) -> bool;

    /// Where to install commands for this IDE.
    fn install_path(&self) -> anyhow::Result<PathBuf>;

    /// Generate command file content in IDE-specific format.
    fn generate(&self, command: &PalrunCommand) -> anyhow::Result<String>;

    /// File extension for command files.
    fn file_extension(&self) -> &str {
        "md"
    }

    /// Generate filename for a command.
    fn filename(&self, command: &PalrunCommand) -> String {
        format!("palrun-{}.{}", command.name, self.file_extension())
    }
}

/// A Palrun command that can be exposed to IDEs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalrunCommand {
    /// Command name (used in slash command, e.g., "new-project").
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// The actual palrun CLI command to run.
    pub palrun_command: String,

    /// Command category.
    pub category: CommandCategory,

    /// Optional arguments the command accepts.
    pub args: Vec<CommandArg>,
}

impl PalrunCommand {
    /// Create a new command.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        palrun_command: impl Into<String>,
        category: CommandCategory,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            palrun_command: palrun_command.into(),
            category,
            args: Vec::new(),
        }
    }

    /// Add an argument.
    pub fn with_arg(mut self, arg: CommandArg) -> Self {
        self.args.push(arg);
        self
    }
}

/// Command category for organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandCategory {
    /// Project commands (new, analyze).
    Project,
    /// Planning commands (plan, roadmap).
    Planning,
    /// Execution commands (execute, run).
    Execution,
    /// Status commands (status, verify).
    Status,
    /// Utility commands.
    Utility,
}

impl CommandCategory {
    /// Get display name for the category.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Project => "Project",
            Self::Planning => "Planning",
            Self::Execution => "Execution",
            Self::Status => "Status",
            Self::Utility => "Utility",
        }
    }
}

/// A command argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    /// Argument name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Whether the argument is required.
    pub required: bool,

    /// Default value if any.
    pub default: Option<String>,
}

impl CommandArg {
    /// Create a new required argument.
    pub fn required(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { name: name.into(), description: description.into(), required: true, default: None }
    }

    /// Create a new optional argument.
    pub fn optional(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { name: name.into(), description: description.into(), required: false, default: None }
    }

    /// Set a default value.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTarget;

    impl CommandTarget for MockTarget {
        fn name(&self) -> &str {
            "mock"
        }

        fn detect(&self) -> bool {
            true
        }

        fn install_path(&self) -> anyhow::Result<PathBuf> {
            Ok(PathBuf::from("/tmp/mock"))
        }

        fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
            Ok(format!("# {}\n{}", cmd.name, cmd.description))
        }
    }

    #[test]
    fn test_mock_target() {
        let target = MockTarget;
        assert_eq!(target.name(), "mock");
        assert!(target.detect());
        assert_eq!(target.file_extension(), "md");
    }

    #[test]
    fn test_mock_target_generate() {
        let target = MockTarget;
        let cmd =
            PalrunCommand::new("test", "Test command", "palrun test", CommandCategory::Utility);
        let content = target.generate(&cmd).unwrap();
        assert!(content.contains("# test"));
        assert!(content.contains("Test command"));
    }

    #[test]
    fn test_palrun_command() {
        let cmd = PalrunCommand::new(
            "analyze",
            "Analyze the codebase",
            "palrun analyze",
            CommandCategory::Project,
        )
        .with_arg(CommandArg::optional("verbose", "Enable verbose output"));

        assert_eq!(cmd.name, "analyze");
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_command_category() {
        assert_eq!(CommandCategory::Project.display_name(), "Project");
        assert_eq!(CommandCategory::Planning.display_name(), "Planning");
    }

    #[test]
    fn test_command_arg() {
        let arg = CommandArg::required("name", "The name").with_default("default");

        assert!(arg.required);
        assert_eq!(arg.default, Some("default".to_string()));
    }
}
