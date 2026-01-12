//! Command registry for managing IDE targets.
//!
//! The registry maintains a list of all Palrun commands and
//! can install them to any detected IDE.

use std::path::Path;

use once_cell::sync::Lazy;

use super::target::{CommandArg, CommandCategory, CommandTarget, PalrunCommand};

/// All available Palrun commands.
pub static PALRUN_COMMANDS: Lazy<Vec<PalrunCommand>> = Lazy::new(|| {
    vec![
        // Project commands
        PalrunCommand::new(
            "new-project",
            "Initialize new Palrun project with PROJECT.md and STATE.md",
            "palrun project new",
            CommandCategory::Project,
        ),
        PalrunCommand::new(
            "analyze",
            "Analyze codebase and generate CODEBASE.md",
            "palrun analyze",
            CommandCategory::Project,
        ),
        // Planning commands
        PalrunCommand::new(
            "create-roadmap",
            "Create ROADMAP.md from PROJECT.md",
            "palrun roadmap create",
            CommandCategory::Planning,
        ),
        PalrunCommand::new(
            "plan-phase",
            "Create PLAN.md for a specific phase",
            "palrun plan phase",
            CommandCategory::Planning,
        )
        .with_arg(CommandArg::required("phase", "Phase number to plan")),
        // Execution commands
        PalrunCommand::new(
            "execute",
            "Execute the current plan",
            "palrun execute",
            CommandCategory::Execution,
        )
        .with_arg(CommandArg::optional("task", "Specific task number to execute")),
        PalrunCommand::new(
            "run",
            "Run a specific command from the palette",
            "palrun run",
            CommandCategory::Execution,
        )
        .with_arg(CommandArg::required("command", "Command to run")),
        // Status commands
        PalrunCommand::new(
            "status",
            "Show current project status",
            "palrun status",
            CommandCategory::Status,
        ),
        PalrunCommand::new(
            "verify",
            "Run verification steps for current task",
            "palrun verify",
            CommandCategory::Status,
        ),
        // Utility commands
        PalrunCommand::new(
            "ai-ask",
            "Ask AI a question about the codebase",
            "palrun ai ask",
            CommandCategory::Utility,
        )
        .with_arg(CommandArg::required("question", "Question to ask")),
        PalrunCommand::new(
            "ai-generate",
            "Generate a command from natural language",
            "palrun ai generate",
            CommandCategory::Utility,
        )
        .with_arg(CommandArg::required("prompt", "What you want to do")),
    ]
});

/// Registry for managing command targets.
pub struct SlashCommandRegistry {
    targets: Vec<Box<dyn CommandTarget>>,
}

impl SlashCommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self { targets: Vec::new() }
    }

    /// Register a command target.
    pub fn register(&mut self, target: Box<dyn CommandTarget>) {
        self.targets.push(target);
    }

    /// Get all registered targets.
    pub fn targets(&self) -> &[Box<dyn CommandTarget>] {
        &self.targets
    }

    /// Detect which IDEs are installed.
    pub fn detect_installed(&self) -> Vec<&dyn CommandTarget> {
        self.targets.iter().filter(|t| t.detect()).map(|t| t.as_ref()).collect()
    }

    /// Get a target by name.
    pub fn get(&self, name: &str) -> Option<&dyn CommandTarget> {
        self.targets.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    /// Install commands to all detected IDEs.
    pub fn install_all(&self) -> anyhow::Result<Vec<String>> {
        let mut installed = Vec::new();
        for target in self.detect_installed() {
            self.install_target(target)?;
            installed.push(target.name().to_string());
        }
        Ok(installed)
    }

    /// Install commands to a specific target.
    pub fn install_to(&self, name: &str) -> anyhow::Result<()> {
        let target = self.get(name).ok_or_else(|| anyhow::anyhow!("Unknown target: {}", name))?;

        if !target.detect() {
            anyhow::bail!("{} is not installed on this system", target.display_name());
        }

        self.install_target(target)
    }

    /// Install commands to a target.
    fn install_target(&self, target: &dyn CommandTarget) -> anyhow::Result<()> {
        let path = target.install_path()?;
        std::fs::create_dir_all(&path)?;

        for cmd in PALRUN_COMMANDS.iter() {
            let content = target.generate(cmd)?;
            let filename = target.filename(cmd);
            std::fs::write(path.join(&filename), content)?;
        }

        Ok(())
    }

    /// List installed commands for a target.
    pub fn list_installed(&self, name: &str) -> anyhow::Result<Vec<String>> {
        let target = self.get(name).ok_or_else(|| anyhow::anyhow!("Unknown target: {}", name))?;

        let path = target.install_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut commands = Vec::new();
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("palrun-") {
                    commands.push(name.to_string());
                }
            }
        }

        Ok(commands)
    }

    /// Sync commands (reinstall all).
    pub fn sync(&self) -> anyhow::Result<Vec<String>> {
        self.install_all()
    }

    /// Uninstall commands from a target.
    pub fn uninstall(&self, name: &str) -> anyhow::Result<()> {
        let target = self.get(name).ok_or_else(|| anyhow::anyhow!("Unknown target: {}", name))?;

        let path = target.install_path()?;
        if path.exists() {
            // Only remove palrun command files
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("palrun-") {
                        std::fs::remove_file(entry.path())?;
                    }
                }
            }

            // Remove directory if empty
            if is_dir_empty(&path)? {
                std::fs::remove_dir(&path)?;
            }
        }

        Ok(())
    }
}

impl Default for SlashCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn is_dir_empty(path: &Path) -> anyhow::Result<bool> {
    Ok(std::fs::read_dir(path)?.next().is_none())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    struct MockTarget {
        installed: bool,
    }

    impl MockTarget {
        fn new(installed: bool) -> Self {
            Self { installed }
        }
    }

    impl CommandTarget for MockTarget {
        fn name(&self) -> &str {
            "mock"
        }

        fn detect(&self) -> bool {
            self.installed
        }

        fn install_path(&self) -> anyhow::Result<PathBuf> {
            Ok(PathBuf::from("/tmp/mock-commands"))
        }

        fn generate(&self, cmd: &PalrunCommand) -> anyhow::Result<String> {
            Ok(format!(
                "# {}\n\n{}\n\n```bash\n{}\n```",
                cmd.name, cmd.description, cmd.palrun_command
            ))
        }
    }

    #[test]
    fn test_palrun_commands_defined() {
        assert!(!PALRUN_COMMANDS.is_empty());
        assert!(PALRUN_COMMANDS.iter().any(|c| c.name == "new-project"));
        assert!(PALRUN_COMMANDS.iter().any(|c| c.name == "analyze"));
    }

    #[test]
    fn test_registry_new() {
        let registry = SlashCommandRegistry::new();
        assert!(registry.targets().is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = SlashCommandRegistry::new();
        registry.register(Box::new(MockTarget::new(true)));
        assert_eq!(registry.targets().len(), 1);
    }

    #[test]
    fn test_registry_detect_installed() {
        let mut registry = SlashCommandRegistry::new();
        registry.register(Box::new(MockTarget::new(true)));
        registry.register(Box::new(MockTarget::new(false)));

        // Note: both have same name "mock", so only one detected
        let installed = registry.detect_installed();
        assert_eq!(installed.len(), 1);
    }

    #[test]
    fn test_registry_get() {
        let mut registry = SlashCommandRegistry::new();
        registry.register(Box::new(MockTarget::new(true)));

        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
