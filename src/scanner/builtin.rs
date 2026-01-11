//! Built-in Palrun commands scanner.
//!
//! Adds Palrun's own CLI commands to the command list so users
//! can discover and use them from within the TUI.

use std::path::Path;

use crate::core::{Command, CommandSource};

use super::Scanner;

/// Scanner for Palrun's built-in commands.
pub struct BuiltinScanner;

impl Scanner for BuiltinScanner {
    fn name(&self) -> &str {
        "builtin"
    }

    fn scan(&self, _path: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Core commands
        commands.push(
            Command::new("pal list", "pal list")
                .with_description("List all discovered commands")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal list --json", "pal list --format json")
                .with_description("List commands as JSON")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal scan", "pal scan")
                .with_description("Scan and show discovered commands")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal config", "pal config")
                .with_description("Show current configuration")
                .with_source(CommandSource::Builtin),
        );

        // Plugin commands
        #[cfg(feature = "plugins")]
        {
            commands.push(
                Command::new("pal plugin list", "pal plugin list")
                    .with_description("List installed plugins")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal plugin search", "pal plugin search ")
                    .with_description("Search for plugins in registry")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal plugin browse", "pal plugin browse")
                    .with_description("Browse available plugins")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal plugin update", "pal plugin update")
                    .with_description("Check for plugin updates")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal plugin install", "pal plugin install ")
                    .with_description("Install a plugin from registry")
                    .with_source(CommandSource::Builtin),
            );
        }

        // Environment commands
        commands.push(
            Command::new("pal env list", "pal env list")
                .with_description("List detected .env files")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal env show", "pal env show")
                .with_description("Show environment variables")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal versions", "pal versions")
                .with_description("Show runtime version requirements")
                .with_source(CommandSource::Builtin),
        );

        // Secrets commands
        commands.push(
            Command::new("pal secrets status", "pal secrets status")
                .with_description("Check secret provider status")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal secrets scan", "pal secrets scan")
                .with_description("Scan for secret references")
                .with_source(CommandSource::Builtin),
        );

        // Git hooks commands
        #[cfg(feature = "git")]
        {
            commands.push(
                Command::new("pal hooks list", "pal hooks list")
                    .with_description("List installed Git hooks")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal hooks sync", "pal hooks sync")
                    .with_description("Sync hooks from palrun.toml")
                    .with_source(CommandSource::Builtin),
            );
        }

        // AI commands
        #[cfg(feature = "ai")]
        {
            commands.push(
                Command::new("pal ai status", "pal ai status")
                    .with_description("Show AI provider status")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal ai gen", "pal ai gen ")
                    .with_description("Generate command from natural language")
                    .with_source(CommandSource::Builtin),
            );

            commands.push(
                Command::new("pal ai explain", "pal ai explain ")
                    .with_description("Explain what a command does")
                    .with_source(CommandSource::Builtin),
            );
        }

        // Shell integration
        commands.push(
            Command::new("pal init bash", "pal init bash")
                .with_description("Output bash shell integration")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal init zsh", "pal init zsh")
                .with_description("Output zsh shell integration")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal init fish", "pal init fish")
                .with_description("Output fish shell integration")
                .with_source(CommandSource::Builtin),
        );

        // CI/CD commands
        commands.push(
            Command::new("pal ci status", "pal ci status")
                .with_description("Show CI status for current branch")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal ci workflows", "pal ci workflows")
                .with_description("List available GitHub Actions workflows")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal ci runs", "pal ci runs")
                .with_description("List recent workflow runs")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal ci trigger", "pal ci trigger ")
                .with_description("Trigger a GitHub Actions workflow")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal ci open", "pal ci open")
                .with_description("Open CI page in browser")
                .with_source(CommandSource::Builtin),
        );

        // Notification commands
        commands.push(
            Command::new("pal notify slack", "pal notify slack -u ")
                .with_description("Send a message to Slack")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal notify discord", "pal notify discord -u ")
                .with_description("Send a message to Discord")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal notify test", "pal notify test -t ")
                .with_description("Test a notification endpoint")
                .with_source(CommandSource::Builtin),
        );

        // GitHub Issues commands
        commands.push(
            Command::new("pal issues list", "pal issues list")
                .with_description("List open GitHub issues")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues list all", "pal issues list --state all")
                .with_description("List all GitHub issues (open and closed)")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues view", "pal issues view ")
                .with_description("View a specific GitHub issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues create", "pal issues create --title ")
                .with_description("Create a new GitHub issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues close", "pal issues close ")
                .with_description("Close a GitHub issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues comment", "pal issues comment ")
                .with_description("Add a comment to a GitHub issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues search", "pal issues search ")
                .with_description("Search GitHub issues")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues stats", "pal issues stats")
                .with_description("Show GitHub issue statistics")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal issues open", "pal issues open")
                .with_description("Open GitHub issues in browser")
                .with_source(CommandSource::Builtin),
        );

        // Linear commands
        commands.push(
            Command::new("pal linear list", "pal linear list")
                .with_description("List your assigned Linear issues")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear view", "pal linear view ")
                .with_description("View a specific Linear issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear create", "pal linear create --title ")
                .with_description("Create a new Linear issue")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear teams", "pal linear teams")
                .with_description("List your Linear teams")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear search", "pal linear search ")
                .with_description("Search Linear issues")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear stats", "pal linear stats")
                .with_description("Show your Linear statistics")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal linear me", "pal linear me")
                .with_description("Show your Linear user info")
                .with_source(CommandSource::Builtin),
        );

        // Help
        commands.push(
            Command::new("pal --help", "pal --help")
                .with_description("Show help information")
                .with_source(CommandSource::Builtin),
        );

        commands.push(
            Command::new("pal --version", "pal --version")
                .with_description("Show version information")
                .with_source(CommandSource::Builtin),
        );

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_scanner() {
        let scanner = BuiltinScanner;
        let commands = scanner.scan(Path::new(".")).unwrap();

        // Should have several built-in commands
        assert!(!commands.is_empty());

        // Check for key commands
        assert!(commands.iter().any(|c| c.name == "pal list"));
        assert!(commands.iter().any(|c| c.name == "pal --help"));
    }
}
