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

    /// Current date (YYYY-MM-DD)
    pub current_date: String,

    /// Current time (HH:MM)
    pub current_time: String,

    /// Git branch name (if in a git repo)
    pub git_branch: Option<String>,

    /// Git status summary (e.g., "3 modified, 2 untracked")
    pub git_status: Option<String>,

    /// Whether the repo has uncommitted changes
    pub git_dirty: bool,
}

impl ProjectContext {
    /// Create a new project context.
    pub fn new(project_name: impl Into<String>, current_directory: PathBuf) -> Self {
        let now = chrono::Local::now();
        Self {
            project_name: project_name.into(),
            project_type: "unknown".to_string(),
            available_commands: Vec::new(),
            current_directory,
            recent_commands: Vec::new(),
            current_date: now.format("%Y-%m-%d").to_string(),
            current_time: now.format("%H:%M").to_string(),
            git_branch: None,
            git_status: None,
            git_dirty: false,
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

        // Get git info if available
        context.populate_git_info(&cwd);

        Ok(context)
    }

    /// Populate git information from the current directory.
    fn populate_git_info(&mut self, path: &PathBuf) {
        // Try to get git branch
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !branch.is_empty() {
                    self.git_branch = Some(branch);
                }
            }
        }

        // Try to get git status
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
        {
            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = status.lines().collect();

                if !lines.is_empty() {
                    self.git_dirty = true;

                    // Count modified, untracked, etc.
                    let modified =
                        lines.iter().filter(|l| l.starts_with(" M") || l.starts_with("M ")).count();
                    let untracked = lines.iter().filter(|l| l.starts_with("??")).count();
                    let staged =
                        lines.iter().filter(|l| l.starts_with("A ") || l.starts_with("D ")).count();

                    let mut parts = Vec::new();
                    if modified > 0 {
                        parts.push(format!("{}M", modified));
                    }
                    if staged > 0 {
                        parts.push(format!("{}S", staged));
                    }
                    if untracked > 0 {
                        parts.push(format!("{}?", untracked));
                    }

                    if !parts.is_empty() {
                        self.git_status = Some(parts.join(" "));
                    }
                }
            }
        }
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

    /// Build a rich system prompt for AI chat.
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();

        // Expert framing (like Cursor/Claude Code)
        prompt.push_str(
            "You are an expert software developer assistant. You have deep knowledge of:\n",
        );
        prompt.push_str("- Terminal commands, git, and shell scripting\n");
        prompt.push_str("- The current project's tech stack and patterns\n");
        prompt.push_str("- Best practices for clean, maintainable code\n\n");

        // Response style
        prompt.push_str("Response style:\n");
        prompt.push_str("- Be direct and concise\n");
        prompt.push_str("- Use `backticks` for commands and code\n");
        prompt.push_str("- Use ```language blocks for multi-line code\n");
        prompt.push_str("- Give working solutions, not just explanations\n");
        prompt
            .push_str("- If asked 'how to X', show the command first, then explain if needed\n\n");

        // Project context
        prompt.push_str("Current project:\n");
        prompt.push_str(&format!("- Name: {}\n", self.project_name));
        prompt.push_str(&format!("- Type: {}\n", self.project_type));
        prompt.push_str(&format!("- Path: {}\n", self.current_directory.display()));

        if let Some(ref branch) = self.git_branch {
            let status = if self.git_dirty {
                format!("with changes ({})", self.git_status.as_deref().unwrap_or("modified"))
            } else {
                "clean".to_string()
            };
            prompt.push_str(&format!("- Git: {} ({})\n", branch, status));
        }

        if !self.available_commands.is_empty() {
            prompt.push_str(&format!("- Commands: {} available\n", self.available_commands.len()));
        }

        // Load project-specific rules if they exist
        if let Some(rules) = self.load_project_rules() {
            prompt.push_str("\nProject rules:\n");
            prompt.push_str(&rules);
            prompt.push('\n');
        }

        prompt
    }

    /// Load project-specific AI rules from .palrun/ai.md or PALRUN.md
    fn load_project_rules(&self) -> Option<String> {
        let palrun_ai = self.current_directory.join(".palrun/ai.md");
        let palrun_md = self.current_directory.join("PALRUN.md");

        // Try .palrun/ai.md first, then PALRUN.md
        let rules_path = if palrun_ai.exists() {
            Some(palrun_ai)
        } else if palrun_md.exists() {
            Some(palrun_md)
        } else {
            None
        };

        if let Some(path) = rules_path {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Limit to first 500 chars to avoid token overflow
                let truncated =
                    if content.len() > 500 { format!("{}...", &content[..500]) } else { content };
                return Some(truncated);
            }
        }

        None
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
        let now = chrono::Local::now();
        Self {
            project_name: "unknown".to_string(),
            project_type: "unknown".to_string(),
            available_commands: Vec::new(),
            current_directory: PathBuf::from("."),
            recent_commands: Vec::new(),
            current_date: now.format("%Y-%m-%d").to_string(),
            current_time: now.format("%H:%M").to_string(),
            git_branch: None,
            git_status: None,
            git_dirty: false,
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
        // Check that date/time are populated
        assert!(!context.current_date.is_empty());
        assert!(!context.current_time.is_empty());
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
        assert!(summary.contains('2'));
    }

    #[test]
    fn test_build_system_prompt() {
        let mut context = ProjectContext::new("my-project", PathBuf::from("/home/user/project"));
        context.project_type = "rust".to_string();
        context.available_commands = vec!["cargo build".to_string(), "cargo test".to_string()];
        context.git_branch = Some("main".to_string());
        context.git_dirty = true;
        context.git_status = Some("2M 1?".to_string());

        let prompt = context.build_system_prompt();

        // Check that key information is included
        assert!(prompt.contains("expert software developer"));
        assert!(prompt.contains("my-project"));
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("main"));
        assert!(prompt.contains("Response style"));
    }

    #[test]
    fn test_default_context() {
        let context = ProjectContext::default();
        assert_eq!(context.project_name, "unknown");
        assert_eq!(context.project_type, "unknown");
        // Date/time should be set
        assert!(!context.current_date.is_empty());
    }
}
