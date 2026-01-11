//! Project context for AI requests.
//!
//! Builds context information about the current project for AI providers.

use std::path::PathBuf;

/// Project context for AI requests.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// Name of the project
    pub project_name: String,

    /// Type of project (node, rust, python, etc.)
    pub project_type: String,

    /// List of available commands (summarized)
    pub available_commands: Vec<String>,

    /// Current working directory
    pub current_directory: PathBuf,

    /// Recent commands run in this project
    pub recent_commands: Vec<String>,
}

impl ProjectContext {
    /// Create a new project context.
    pub fn new(project_name: impl Into<String>, current_directory: PathBuf) -> Self {
        Self {
            project_name: project_name.into(),
            project_type: "unknown".to_string(),
            available_commands: Vec::new(),
            current_directory,
            recent_commands: Vec::new(),
        }
    }

    /// Build context from the current directory.
    pub fn from_current_dir() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        let project_name =
            cwd.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

        let mut context = Self::new(project_name, cwd.clone());

        // Detect project type
        context.project_type = detect_project_type(&cwd);

        Ok(context)
    }

    /// Set the available commands.
    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        // Limit to avoid token overflow
        self.available_commands = commands.into_iter().take(50).collect();
        self
    }

    /// Set the recent commands.
    pub fn with_recent(mut self, commands: Vec<String>) -> Self {
        self.recent_commands = commands.into_iter().take(10).collect();
        self
    }

    /// Summarize context as a string (for debugging or logging).
    pub fn summarize(&self) -> String {
        format!(
            "Project: {} ({})\nCommands: {}\nRecent: {}",
            self.project_name,
            self.project_type,
            self.available_commands.len(),
            self.recent_commands.len()
        )
    }
}

/// Detect the project type from files in the directory.
fn detect_project_type(path: &PathBuf) -> String {
    if path.join("package.json").exists() {
        if path.join("nx.json").exists() {
            return "nx".to_string();
        }
        if path.join("turbo.json").exists() {
            return "turbo".to_string();
        }
        return "node".to_string();
    }

    if path.join("Cargo.toml").exists() {
        return "rust".to_string();
    }

    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return "python".to_string();
    }

    if path.join("go.mod").exists() {
        return "go".to_string();
    }

    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        return "java".to_string();
    }

    if path.join("Gemfile").exists() {
        return "ruby".to_string();
    }

    if path.join("composer.json").exists() {
        return "php".to_string();
    }

    "unknown".to_string()
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            project_name: "unknown".to_string(),
            project_type: "unknown".to_string(),
            available_commands: Vec::new(),
            current_directory: PathBuf::from("."),
            recent_commands: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = ProjectContext::new("test", PathBuf::from("/tmp/test"));
        assert_eq!(context.project_name, "test");
        assert_eq!(context.project_type, "unknown");
    }

    #[test]
    fn test_context_with_commands() {
        let context = ProjectContext::new("test", PathBuf::from("."))
            .with_commands(vec!["npm run build".to_string(), "npm run test".to_string()]);

        assert_eq!(context.available_commands.len(), 2);
    }

    #[test]
    fn test_summarize() {
        let context = ProjectContext::new("test", PathBuf::from("."))
            .with_commands(vec!["cmd1".to_string(), "cmd2".to_string()]);

        let summary = context.summarize();
        assert!(summary.contains("test"));
        assert!(summary.contains("2"));
    }
}
