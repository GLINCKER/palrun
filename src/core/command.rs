//! Command data structures.
//!
//! Defines the `Command` struct that represents a runnable command
//! discovered from project configuration files.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A runnable command discovered from project configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    /// Unique identifier for this command
    pub id: String,

    /// Display name shown in the command palette
    pub name: String,

    /// The actual shell command to execute
    pub command: String,

    /// Optional description of what this command does
    pub description: Option<String>,

    /// Source of this command (package.json, Makefile, etc.)
    pub source: CommandSource,

    /// Working directory for execution (if different from project root)
    pub working_dir: Option<PathBuf>,

    /// Tags for categorization and filtering
    pub tags: Vec<String>,

    /// Whether this command requires confirmation before running
    pub confirm: bool,

    /// Environment variables to set when running
    pub env: Vec<(String, String)>,

    /// Branch patterns this command is available on (empty = all branches)
    /// Supports glob patterns like "main", "feature/*", "release/*"
    #[serde(default)]
    pub branch_patterns: Vec<String>,

    /// Workspace name (for monorepo projects)
    #[serde(default)]
    pub workspace: Option<String>,
}

impl Command {
    /// Create a new command with minimal required fields.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        let name = name.into();
        let command_str = command.into();
        let id = Self::generate_id(&name, &command_str);

        Self {
            id,
            name,
            command: command_str,
            description: None,
            source: CommandSource::Manual,
            working_dir: None,
            tags: Vec::new(),
            confirm: false,
            env: Vec::new(),
            branch_patterns: Vec::new(),
            workspace: None,
        }
    }

    /// Create a command from a package.json script.
    pub fn from_npm_script(
        script_name: &str,
        script_command: &str,
        package_manager: &str,
        working_dir: Option<PathBuf>,
    ) -> Self {
        let run_command = match package_manager {
            "yarn" => format!("yarn {script_name}"),
            "pnpm" => format!("pnpm {script_name}"),
            "bun" => format!("bun run {script_name}"),
            _ => format!("npm run {script_name}"),
        };

        let name = format!("{package_manager} run {script_name}");
        let id = Self::generate_id(&name, &run_command);

        Self {
            id,
            name,
            command: run_command,
            description: Some(script_command.to_string()),
            source: CommandSource::PackageJson(
                working_dir.clone().unwrap_or_else(|| PathBuf::from(".")),
            ),
            working_dir,
            tags: vec!["npm".to_string(), "script".to_string()],
            confirm: false,
            env: Vec::new(),
            branch_patterns: Vec::new(),
            workspace: None,
        }
    }

    /// Create a command from a Makefile target.
    pub fn from_make_target(target: &str, working_dir: Option<PathBuf>) -> Self {
        let command = format!("make {target}");
        let name = command.clone();
        let id = Self::generate_id(&name, &command);

        Self {
            id,
            name,
            command,
            description: None,
            source: CommandSource::Makefile(
                working_dir.clone().unwrap_or_else(|| PathBuf::from(".")),
            ),
            working_dir,
            tags: vec!["make".to_string()],
            confirm: false,
            env: Vec::new(),
            branch_patterns: Vec::new(),
            workspace: None,
        }
    }

    /// Create a command from an alias configuration.
    pub fn from_alias(alias: &super::config::AliasConfig) -> Self {
        let id = Self::generate_id(&alias.name, &alias.command);

        let mut tags = alias.tags.clone();
        if !tags.contains(&"alias".to_string()) {
            tags.push("alias".to_string());
        }

        Self {
            id,
            name: alias.name.clone(),
            command: alias.command.clone(),
            description: alias.description.clone(),
            source: CommandSource::Alias,
            working_dir: alias.working_dir.clone(),
            tags,
            confirm: alias.confirm,
            env: alias.env.clone(),
            branch_patterns: alias.branches.clone(),
            workspace: None,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the source.
    #[must_use]
    pub fn with_source(mut self, source: CommandSource) -> Self {
        self.source = source;
        self
    }

    /// Set the working directory.
    #[must_use]
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set multiple tags at once.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set confirmation requirement.
    #[must_use]
    pub fn with_confirm(mut self, confirm: bool) -> Self {
        self.confirm = confirm;
        self
    }

    /// Add an environment variable.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Add a branch pattern.
    #[must_use]
    pub fn with_branch_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.branch_patterns.push(pattern.into());
        self
    }

    /// Set multiple branch patterns at once.
    #[must_use]
    pub fn with_branch_patterns(mut self, patterns: Vec<String>) -> Self {
        self.branch_patterns = patterns;
        self
    }

    /// Set the workspace name (for monorepo projects).
    #[must_use]
    pub fn with_workspace(mut self, workspace: impl Into<String>) -> Self {
        self.workspace = Some(workspace.into());
        self
    }

    /// Check if this command is available on the given branch.
    ///
    /// Returns true if:
    /// - No branch patterns are specified (available on all branches)
    /// - The branch matches at least one of the patterns
    pub fn matches_branch(&self, branch: Option<&str>) -> bool {
        // If no patterns specified, available on all branches
        if self.branch_patterns.is_empty() {
            return true;
        }

        // If no branch (detached HEAD or not in git repo), only match if patterns are empty
        let branch = match branch {
            Some(b) => b,
            None => return false,
        };

        // Check if any pattern matches
        self.branch_patterns.iter().any(|pattern| Self::matches_pattern(pattern, branch))
    }

    /// Check if a branch matches a pattern (supports glob-style wildcards).
    fn matches_pattern(pattern: &str, branch: &str) -> bool {
        // Handle exact match
        if pattern == branch {
            return true;
        }

        // Handle wildcard patterns
        if pattern.contains('*') {
            // Convert glob pattern to simple matching
            let parts: Vec<&str> = pattern.split('*').collect();

            if parts.len() == 2 {
                // Pattern like "feature/*" or "*-hotfix"
                let (prefix, suffix) = (parts[0], parts[1]);
                return branch.starts_with(prefix) && branch.ends_with(suffix);
            } else if parts.len() == 1 {
                // No wildcard found (shouldn't happen but handle it)
                return pattern == branch;
            }
            // Complex patterns with multiple wildcards - do simple contains check
            return parts.iter().all(|part| part.is_empty() || branch.contains(part));
        }

        false
    }

    /// Generate a unique ID for the command.
    fn generate_id(name: &str, command: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        command.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Get the text to use for fuzzy matching.
    pub fn match_text(&self) -> String {
        let mut text = self.name.clone();
        if let Some(ref desc) = self.description {
            text.push(' ');
            text.push_str(desc);
        }
        for tag in &self.tags {
            text.push(' ');
            text.push_str(tag);
        }
        text
    }

    /// Get a short display representation.
    pub fn short_display(&self) -> &str {
        &self.name
    }

    /// Get the source type as a string (for display).
    pub fn source_type(&self) -> &'static str {
        self.source.type_name()
    }
}

impl Default for Command {
    fn default() -> Self {
        Self::new("", "")
    }
}

/// Source of a discovered command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandSource {
    /// From package.json scripts
    PackageJson(PathBuf),

    /// From nx.json or project.json
    NxProject(String),

    /// From turbo.json
    Turbo,

    /// From Makefile
    Makefile(PathBuf),

    /// From Taskfile.yml
    Taskfile(PathBuf),

    /// From docker-compose.yml
    DockerCompose(PathBuf),

    /// From Cargo.toml
    Cargo(PathBuf),

    /// From go.mod
    GoMod(PathBuf),

    /// From pyproject.toml
    Python(PathBuf),

    /// Git operations
    Git,

    /// Manually defined (config or runbook)
    Manual,

    /// From command history
    History,

    /// From user favorites
    Favorite,

    /// User-defined alias
    Alias,
}

impl CommandSource {
    /// Get the type name for display.
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::PackageJson(_) => "npm",
            Self::NxProject(_) => "nx",
            Self::Turbo => "turbo",
            Self::Makefile(_) => "make",
            Self::Taskfile(_) => "task",
            Self::DockerCompose(_) => "docker",
            Self::Cargo(_) => "cargo",
            Self::GoMod(_) => "go",
            Self::Python(_) => "python",
            Self::Git => "git",
            Self::Manual => "manual",
            Self::History => "history",
            Self::Favorite => "favorite",
            Self::Alias => "alias",
        }
    }

    /// Get the icon/emoji for this source type.
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::PackageJson(_) => "📦",
            Self::NxProject(_) => "🔷",
            Self::Turbo => "⚡",
            Self::Makefile(_) => "🔧",
            Self::Taskfile(_) => "📋",
            Self::DockerCompose(_) => "🐳",
            Self::Cargo(_) => "🦀",
            Self::GoMod(_) => "🐹",
            Self::Python(_) => "🐍",
            Self::Git => "🔀",
            Self::Manual => "📝",
            Self::History => "📜",
            Self::Favorite => "⭐",
            Self::Alias => "🔗",
        }
    }

    /// Get a short name for display in the UI.
    pub const fn short_name(&self) -> &'static str {
        self.type_name()
    }
}

impl Default for CommandSource {
    fn default() -> Self {
        Self::Manual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "npm run test");
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.command, "npm run test");
        assert!(!cmd.id.is_empty());
    }

    #[test]
    fn test_npm_script_command() {
        let cmd = Command::from_npm_script("build", "tsc", "npm", None);
        assert_eq!(cmd.name, "npm run build");
        assert_eq!(cmd.command, "npm run build");
        assert_eq!(cmd.description, Some("tsc".to_string()));
    }

    #[test]
    fn test_yarn_script_command() {
        let cmd = Command::from_npm_script("dev", "vite", "yarn", None);
        assert_eq!(cmd.name, "yarn run dev");
        assert_eq!(cmd.command, "yarn dev");
    }

    #[test]
    fn test_make_target_command() {
        let cmd = Command::from_make_target("build", None);
        assert_eq!(cmd.name, "make build");
        assert_eq!(cmd.command, "make build");
    }

    #[test]
    fn test_command_builder() {
        let cmd = Command::new("deploy", "kubectl apply")
            .with_description("Deploy to production")
            .with_tag("k8s")
            .with_confirm(true);

        assert_eq!(cmd.description, Some("Deploy to production".to_string()));
        assert!(cmd.tags.contains(&"k8s".to_string()));
        assert!(cmd.confirm);
    }

    #[test]
    fn test_match_text() {
        let cmd = Command::new("build", "npm run build")
            .with_description("Build the project")
            .with_tag("compile");

        let text = cmd.match_text();
        assert!(text.contains("build"));
        assert!(text.contains("Build the project"));
        assert!(text.contains("compile"));
    }

    #[test]
    fn test_source_type_names() {
        assert_eq!(CommandSource::PackageJson(PathBuf::new()).type_name(), "npm");
        assert_eq!(CommandSource::Makefile(PathBuf::new()).type_name(), "make");
        assert_eq!(CommandSource::Manual.type_name(), "manual");
    }

    #[test]
    fn test_branch_patterns_empty_matches_all() {
        let cmd = Command::new("test", "npm test");
        assert!(cmd.matches_branch(Some("main")));
        assert!(cmd.matches_branch(Some("feature/foo")));
        assert!(cmd.matches_branch(None));
    }

    #[test]
    fn test_branch_patterns_exact_match() {
        let cmd = Command::new("deploy", "npm run deploy")
            .with_branch_pattern("main");

        assert!(cmd.matches_branch(Some("main")));
        assert!(!cmd.matches_branch(Some("develop")));
        assert!(!cmd.matches_branch(Some("feature/foo")));
    }

    #[test]
    fn test_branch_patterns_wildcard() {
        let cmd = Command::new("test", "npm test")
            .with_branch_pattern("feature/*");

        assert!(cmd.matches_branch(Some("feature/foo")));
        assert!(cmd.matches_branch(Some("feature/bar/baz")));
        assert!(!cmd.matches_branch(Some("main")));
        assert!(!cmd.matches_branch(Some("develop")));
    }

    #[test]
    fn test_branch_patterns_multiple() {
        let cmd = Command::new("deploy", "npm run deploy")
            .with_branch_patterns(vec!["main".to_string(), "release/*".to_string()]);

        assert!(cmd.matches_branch(Some("main")));
        assert!(cmd.matches_branch(Some("release/1.0")));
        assert!(!cmd.matches_branch(Some("develop")));
        assert!(!cmd.matches_branch(Some("feature/foo")));
    }

    #[test]
    fn test_branch_patterns_suffix_wildcard() {
        let cmd = Command::new("hotfix", "npm run hotfix")
            .with_branch_pattern("*-hotfix");

        assert!(cmd.matches_branch(Some("v1.0-hotfix")));
        assert!(cmd.matches_branch(Some("urgent-hotfix")));
        assert!(!cmd.matches_branch(Some("hotfix-v1")));
    }

    #[test]
    fn test_branch_patterns_no_branch() {
        let cmd = Command::new("deploy", "npm run deploy")
            .with_branch_pattern("main");

        // With patterns but no branch (detached HEAD), should not match
        assert!(!cmd.matches_branch(None));
    }

    #[test]
    fn test_branch_patterns_special_characters() {
        // Branch names with slashes, dashes, underscores
        let cmd = Command::new("test", "npm test")
            .with_branch_patterns(vec![
                "feature/user-auth".to_string(),
                "bugfix/fix_issue_123".to_string(),
                "release/v1.0.0".to_string(),
            ]);

        assert!(cmd.matches_branch(Some("feature/user-auth")));
        assert!(cmd.matches_branch(Some("bugfix/fix_issue_123")));
        assert!(cmd.matches_branch(Some("release/v1.0.0")));
        assert!(!cmd.matches_branch(Some("feature/other")));
    }

    #[test]
    fn test_branch_patterns_complex_wildcards() {
        // Test various wildcard patterns
        let cmd = Command::new("deploy", "deploy.sh").with_branch_patterns(vec![
            "release/*".to_string(),
            "*-hotfix".to_string(),
        ]);

        assert!(cmd.matches_branch(Some("release/v1.0.0")));
        assert!(cmd.matches_branch(Some("release/2.0")));
        assert!(cmd.matches_branch(Some("urgent-hotfix")));
        assert!(cmd.matches_branch(Some("critical-hotfix")));
    }

    #[test]
    fn test_branch_patterns_empty_pattern() {
        // Empty pattern string should only match empty branch name
        let cmd = Command::new("test", "npm test").with_branch_pattern("");

        assert!(cmd.matches_branch(Some("")));
        assert!(!cmd.matches_branch(Some("main")));
    }

    #[test]
    fn test_command_from_alias() {
        use super::super::config::AliasConfig;

        let alias = AliasConfig {
            name: "deploy-dev".to_string(),
            command: "npm run build && npm run deploy:dev".to_string(),
            description: Some("Build and deploy to dev".to_string()),
            tags: vec!["deploy".to_string()],
            confirm: true,
            working_dir: Some(PathBuf::from("./packages/api")),
            env: vec![("NODE_ENV".to_string(), "development".to_string())],
            branches: vec!["main".to_string(), "develop".to_string()],
        };

        let cmd = Command::from_alias(&alias);

        assert_eq!(cmd.name, "deploy-dev");
        assert_eq!(cmd.command, "npm run build && npm run deploy:dev");
        assert_eq!(cmd.description, Some("Build and deploy to dev".to_string()));
        assert!(cmd.tags.contains(&"deploy".to_string()));
        assert!(cmd.tags.contains(&"alias".to_string())); // Auto-added tag
        assert!(cmd.confirm);
        assert_eq!(cmd.working_dir, Some(PathBuf::from("./packages/api")));
        assert_eq!(cmd.env, vec![("NODE_ENV".to_string(), "development".to_string())]);
        assert!(cmd.matches_branch(Some("main")));
        assert!(cmd.matches_branch(Some("develop")));
        assert!(!cmd.matches_branch(Some("feature/foo")));
        assert_eq!(cmd.source, CommandSource::Alias);
    }

    #[test]
    fn test_command_from_alias_minimal() {
        use super::super::config::AliasConfig;

        let alias = AliasConfig::new("test", "npm test");
        let cmd = Command::from_alias(&alias);

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.command, "npm test");
        assert!(cmd.description.is_none());
        assert!(cmd.tags.contains(&"alias".to_string()));
        assert!(!cmd.confirm);
        assert!(cmd.working_dir.is_none());
        assert!(cmd.env.is_empty());
        assert!(cmd.matches_branch(Some("any-branch"))); // No branch restriction
    }

    #[test]
    fn test_alias_source_type() {
        assert_eq!(CommandSource::Alias.type_name(), "alias");
        assert_eq!(CommandSource::Alias.icon(), "🔗");
        assert_eq!(CommandSource::Alias.short_name(), "alias");
    }
}
