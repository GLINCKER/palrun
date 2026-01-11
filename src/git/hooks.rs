//! Git hooks management.
//!
//! Provides functionality to detect, install, and manage Git hooks
//! that integrate with Palrun commands.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Standard Git hook names.
pub const HOOK_NAMES: &[&str] = &[
    "pre-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-auto-gc",
    "post-rewrite",
];

/// Information about an installed Git hook.
#[derive(Debug, Clone)]
pub struct HookInfo {
    /// Hook name (e.g., "pre-commit")
    pub name: String,

    /// Path to the hook file
    pub path: PathBuf,

    /// Whether the hook is managed by Palrun
    pub is_palrun: bool,

    /// Whether the hook is executable
    pub is_executable: bool,

    /// Content preview (first few lines)
    pub preview: Option<String>,
}

/// Git hooks manager.
pub struct HooksManager {
    /// Path to .git/hooks directory
    hooks_dir: PathBuf,
}

impl HooksManager {
    /// Create a hooks manager for the given repository root.
    pub fn new(repo_root: impl AsRef<Path>) -> Self {
        Self {
            hooks_dir: repo_root.as_ref().join(".git").join("hooks"),
        }
    }

    /// Create a hooks manager by discovering the Git repository.
    pub fn discover() -> Option<Self> {
        let repo = super::discover_repo()?;
        let root = repo.root()?;
        Some(Self::new(root))
    }

    /// Get the hooks directory path.
    pub fn hooks_dir(&self) -> &Path {
        &self.hooks_dir
    }

    /// Check if the hooks directory exists.
    pub fn hooks_dir_exists(&self) -> bool {
        self.hooks_dir.is_dir()
    }

    /// List all detected hooks.
    pub fn list_hooks(&self) -> Vec<HookInfo> {
        let mut hooks = Vec::new();

        for &name in HOOK_NAMES {
            let path = self.hooks_dir.join(name);
            if path.exists() {
                let is_executable = is_executable(&path);
                let content = fs::read_to_string(&path).ok();
                let is_palrun = content
                    .as_ref()
                    .map(|c| c.contains("Managed by Palrun") || c.contains("palrun") || c.contains("pal run"))
                    .unwrap_or(false);
                let preview = content.as_ref().map(|c| {
                    c.lines()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("\n")
                });

                hooks.push(HookInfo {
                    name: name.to_string(),
                    path,
                    is_palrun,
                    is_executable,
                    preview,
                });
            }
        }

        hooks
    }

    /// Check if a specific hook exists.
    pub fn hook_exists(&self, name: &str) -> bool {
        self.hooks_dir.join(name).exists()
    }

    /// Check if a hook is managed by Palrun.
    pub fn is_palrun_hook(&self, name: &str) -> bool {
        let path = self.hooks_dir.join(name);
        if let Ok(content) = fs::read_to_string(&path) {
            content.contains("# Managed by Palrun") || content.contains("pal run")
        } else {
            false
        }
    }

    /// Install a Palrun hook.
    ///
    /// If a hook already exists and is not managed by Palrun, this will
    /// return an error unless `force` is true.
    pub fn install_hook(&self, name: &str, command: &str, force: bool) -> Result<()> {
        if !HOOK_NAMES.contains(&name) {
            anyhow::bail!("Unknown hook name: {}", name);
        }

        let path = self.hooks_dir.join(name);

        // Check for existing hook
        if path.exists() && !force {
            if !self.is_palrun_hook(name) {
                anyhow::bail!(
                    "Hook '{}' already exists and is not managed by Palrun. Use --force to overwrite.",
                    name
                );
            }
        }

        // Create hooks directory if needed
        if !self.hooks_dir.exists() {
            fs::create_dir_all(&self.hooks_dir)
                .context("Failed to create hooks directory")?;
        }

        // Generate hook content
        let content = generate_hook_script(name, command);

        // Write hook file
        fs::write(&path, content).context("Failed to write hook file")?;

        // Make executable
        make_executable(&path)?;

        Ok(())
    }

    /// Uninstall a Palrun hook.
    ///
    /// Only removes hooks that are managed by Palrun unless `force` is true.
    pub fn uninstall_hook(&self, name: &str, force: bool) -> Result<()> {
        let path = self.hooks_dir.join(name);

        if !path.exists() {
            anyhow::bail!("Hook '{}' does not exist", name);
        }

        if !force && !self.is_palrun_hook(name) {
            anyhow::bail!(
                "Hook '{}' is not managed by Palrun. Use --force to remove anyway.",
                name
            );
        }

        fs::remove_file(&path).context("Failed to remove hook file")?;

        Ok(())
    }

    /// Install multiple hooks from configuration.
    pub fn install_hooks(&self, hooks: &[(String, String)], force: bool) -> Result<()> {
        for (name, command) in hooks {
            self.install_hook(name, command, force)?;
        }
        Ok(())
    }

    /// Uninstall all Palrun-managed hooks.
    pub fn uninstall_all(&self) -> Result<usize> {
        let mut count = 0;

        for &name in HOOK_NAMES {
            if self.hook_exists(name) && self.is_palrun_hook(name) {
                self.uninstall_hook(name, false)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get hook info for a specific hook.
    pub fn get_hook_info(&self, name: &str) -> Option<HookInfo> {
        let path = self.hooks_dir.join(name);
        if !path.exists() {
            return None;
        }

        let is_executable = is_executable(&path);
        let content = fs::read_to_string(&path).ok();
        let is_palrun = content
            .as_ref()
            .map(|c| c.contains("# Managed by Palrun") || c.contains("pal run"))
            .unwrap_or(false);
        let preview = content.as_ref().map(|c| {
            c.lines().take(5).collect::<Vec<_>>().join("\n")
        });

        Some(HookInfo {
            name: name.to_string(),
            path,
            is_palrun,
            is_executable,
            preview,
        })
    }
}

/// Generate a hook script that calls Palrun.
fn generate_hook_script(hook_name: &str, command: &str) -> String {
    format!(
        r#"#!/bin/sh
# Managed by Palrun - Do not edit manually
# Hook: {hook_name}
# Command: {command}

# Run the configured command
{command}

# Exit with the command's exit code
exit $?
"#,
        hook_name = hook_name,
        command = command
    )
}

/// Check if a file is executable.
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        // On Windows, check file extension
        path.extension()
            .map(|ext| ext == "exe" || ext == "cmd" || ext == "bat")
            .unwrap_or(true) // Assume shell scripts are "executable"
    }
}

/// Make a file executable.
fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_git_repo() -> (TempDir, HooksManager) {
        let temp = TempDir::new().unwrap();
        let hooks_dir = temp.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();

        let manager = HooksManager::new(temp.path());
        (temp, manager)
    }

    #[test]
    fn test_hooks_manager_creation() {
        let (temp, manager) = setup_git_repo();
        assert!(manager.hooks_dir().starts_with(temp.path()));
        assert!(manager.hooks_dir_exists());
    }

    #[test]
    fn test_list_hooks_empty() {
        let (_temp, manager) = setup_git_repo();
        let hooks = manager.list_hooks();
        assert!(hooks.is_empty());
    }

    #[test]
    fn test_install_hook() {
        let (_temp, manager) = setup_git_repo();

        manager.install_hook("pre-commit", "cargo test", false).unwrap();

        assert!(manager.hook_exists("pre-commit"));
        assert!(manager.is_palrun_hook("pre-commit"));

        let info = manager.get_hook_info("pre-commit").unwrap();
        assert!(info.is_palrun);
        assert!(info.is_executable);
    }

    #[test]
    fn test_uninstall_hook() {
        let (_temp, manager) = setup_git_repo();

        manager.install_hook("pre-commit", "cargo test", false).unwrap();
        assert!(manager.hook_exists("pre-commit"));

        manager.uninstall_hook("pre-commit", false).unwrap();
        assert!(!manager.hook_exists("pre-commit"));
    }

    #[test]
    fn test_cannot_overwrite_external_hook() {
        let (temp, manager) = setup_git_repo();

        // Create a non-Palrun hook
        let hook_path = temp.path().join(".git/hooks/pre-commit");
        fs::write(&hook_path, "#!/bin/sh\necho 'external hook'").unwrap();

        // Try to install without force
        let result = manager.install_hook("pre-commit", "cargo test", false);
        assert!(result.is_err());

        // Should work with force
        manager.install_hook("pre-commit", "cargo test", true).unwrap();
        assert!(manager.is_palrun_hook("pre-commit"));
    }

    #[test]
    fn test_hook_script_generation() {
        let script = generate_hook_script("pre-commit", "cargo test && cargo fmt --check");
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("Managed by Palrun"));
        assert!(script.contains("cargo test && cargo fmt --check"));
    }

    #[test]
    fn test_install_multiple_hooks() {
        let (_temp, manager) = setup_git_repo();

        let hooks = vec![
            ("pre-commit".to_string(), "cargo test".to_string()),
            ("pre-push".to_string(), "cargo build --release".to_string()),
        ];

        manager.install_hooks(&hooks, false).unwrap();

        assert!(manager.hook_exists("pre-commit"));
        assert!(manager.hook_exists("pre-push"));
    }

    #[test]
    fn test_uninstall_all() {
        let (_temp, manager) = setup_git_repo();

        manager.install_hook("pre-commit", "cargo test", false).unwrap();
        manager.install_hook("pre-push", "cargo build", false).unwrap();

        let count = manager.uninstall_all().unwrap();
        assert_eq!(count, 2);
        assert!(!manager.hook_exists("pre-commit"));
        assert!(!manager.hook_exists("pre-push"));
    }
}
