//! Git commands scanner.
//!
//! Adds common git operations to the command palette when in a git repository.

use std::path::Path;

use crate::core::{Command, CommandSource};

use super::Scanner;

/// Scanner for git commands.
pub struct GitScanner;

impl Scanner for GitScanner {
    fn name(&self) -> &str {
        "git"
    }

    fn scan(&self, path: &Path) -> anyhow::Result<Vec<Command>> {
        // Check if we're in a git repository
        #[cfg(feature = "git")]
        {
            use crate::git::GitRepository;

            if GitRepository::discover(path).is_none() {
                return Ok(Vec::new());
            }
        }

        #[cfg(not(feature = "git"))]
        {
            // Fallback: check for .git directory
            let git_dir = path.join(".git");
            if !git_dir.exists() {
                // Try to find .git in parent directories
                let mut current = path.to_path_buf();
                loop {
                    if current.join(".git").exists() {
                        break;
                    }
                    if !current.pop() {
                        return Ok(Vec::new());
                    }
                }
            }
        }

        // Build list of git commands
        let commands = vec![
            // Status & Info
            git_command("git status", "git status", "Show the working tree status"),
            git_command("git log", "git log --oneline -20", "Show recent commit history"),
            git_command("git diff", "git diff", "Show unstaged changes"),
            git_command("git diff staged", "git diff --staged", "Show staged changes"),

            // Basic Operations
            git_command("git pull", "git pull", "Fetch and integrate with remote"),
            git_command("git push", "git push", "Push commits to remote"),
            git_command("git fetch", "git fetch --all", "Download objects from remote"),

            // Staging
            git_command("git add all", "git add -A", "Stage all changes"),
            git_command("git add interactive", "git add -p", "Interactively stage changes"),
            git_command("git reset", "git reset", "Unstage all staged changes"),

            // Stash
            git_command("git stash", "git stash", "Stash current changes"),
            git_command("git stash pop", "git stash pop", "Apply and remove latest stash"),
            git_command("git stash list", "git stash list", "List all stashes"),
            git_command("git stash drop", "git stash drop", "Remove latest stash"),

            // Branches
            git_command("git branch list", "git branch -a", "List all branches"),
            git_command("git branch current", "git branch --show-current", "Show current branch name"),

            // Commit (basic - for now without interactive input)
            git_command("git commit", "git commit", "Create a commit (opens editor)"),
            git_command("git commit amend", "git commit --amend", "Amend the last commit"),

            // Cleanup
            git_command("git clean", "git clean -fd", "Remove untracked files and directories"),
            git_command("git gc", "git gc", "Cleanup and optimize repository"),

            // Remote
            git_command("git remote", "git remote -v", "Show remote repositories"),
        ];

        Ok(commands)
    }
}

/// Create a git command with the given name, command, and description.
fn git_command(name: &str, command: &str, description: &str) -> Command {
    Command::new(name, command)
        .with_description(description)
        .with_source(CommandSource::Git)
        .with_tag("git")
        .with_tag("vcs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_scanner_name() {
        let scanner = GitScanner;
        assert_eq!(scanner.name(), "git");
    }

    #[test]
    fn test_git_command_helper() {
        let cmd = git_command("git status", "git status", "Show status");
        assert_eq!(cmd.name, "git status");
        assert_eq!(cmd.command, "git status");
        assert!(cmd.tags.contains(&"git".to_string()));
    }

    #[test]
    fn test_scan_in_git_repo() {
        // This test runs in the palrun repo which is a git repo
        let scanner = GitScanner;
        let commands = scanner.scan(Path::new(".")).unwrap();

        // Should find git commands since we're in a git repo
        assert!(!commands.is_empty());

        // Check for expected commands
        let names: Vec<_> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"git status"));
        assert!(names.contains(&"git pull"));
        assert!(names.contains(&"git push"));
    }
}
