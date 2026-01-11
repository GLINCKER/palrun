//! Context-aware filtering for commands.
//!
//! This module provides proximity-based scoring and filtering for commands
//! based on the current working directory and command locations.

use std::path::{Path, PathBuf};

use super::{Command, CommandSource};

/// Context information for filtering and scoring commands.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Project root directory (where scan started)
    pub project_root: PathBuf,

    /// Active filter (if any)
    pub filter: ContextFilter,
}

/// Filter options for command context.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ContextFilter {
    /// Show all commands
    #[default]
    All,

    /// Show only commands from current directory
    CurrentDir,

    /// Show only commands from project root
    ProjectRoot,

    /// Show only commands from a specific workspace/package
    Workspace(String),

    /// Show only commands from a specific source type
    SourceType(String),
}

impl CommandContext {
    /// Create a new command context.
    pub fn new(cwd: &Path, project_root: &Path) -> Self {
        Self {
            cwd: cwd.to_path_buf(),
            project_root: project_root.to_path_buf(),
            filter: ContextFilter::default(),
        }
    }

    /// Create context for the current directory.
    pub fn current() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self::new(&cwd, &cwd))
    }

    /// Set the context filter.
    pub fn with_filter(mut self, filter: ContextFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Calculate proximity score for a command.
    ///
    /// Returns a score from 0-100 where:
    /// - 100 = command is in current directory
    /// - 80 = command is in project root
    /// - 60-79 = command is in a nearby subdirectory
    /// - 40-59 = command is in a sibling directory
    /// - 20-39 = command is elsewhere in project
    /// - 0 = command has no working directory
    pub fn proximity_score(&self, command: &Command) -> u32 {
        let Some(cmd_dir) = self.get_command_dir(command) else {
            return 0;
        };

        // Canonicalize paths for comparison
        let cmd_dir = cmd_dir.canonicalize().unwrap_or(cmd_dir);
        let cwd = self.cwd.canonicalize().unwrap_or_else(|_| self.cwd.clone());
        let root = self.project_root.canonicalize().unwrap_or_else(|_| self.project_root.clone());

        // Exact match with cwd
        if cmd_dir == cwd {
            return 100;
        }

        // In project root
        if cmd_dir == root {
            return 80;
        }

        // Check if command is in a subdirectory of cwd
        if cmd_dir.starts_with(&cwd) {
            let depth = cmd_dir.strip_prefix(&cwd).map(|p| p.components().count()).unwrap_or(0);
            return 70_u32.saturating_sub(depth as u32 * 5).max(60);
        }

        // Check if cwd is in a subdirectory of command dir
        if cwd.starts_with(&cmd_dir) {
            let depth = cwd.strip_prefix(&cmd_dir).map(|p| p.components().count()).unwrap_or(0);
            return 60_u32.saturating_sub(depth as u32 * 5).max(40);
        }

        // Check if both are in the project
        if cmd_dir.starts_with(&root) && cwd.starts_with(&root) {
            // Calculate how many directories apart they are
            let cmd_depth =
                cmd_dir.strip_prefix(&root).map(|p| p.components().count()).unwrap_or(0);
            let cwd_depth = cwd.strip_prefix(&root).map(|p| p.components().count()).unwrap_or(0);
            let distance = cmd_depth.abs_diff(cwd_depth);
            return 40_u32.saturating_sub(distance as u32 * 5).max(20);
        }

        // Outside project
        10
    }

    /// Get the relative path display for a command.
    ///
    /// Returns a short path indicator like:
    /// - "." for current directory
    /// - "/" for project root
    /// - "packages/foo" for subdirectory
    /// - "../sibling" for sibling directory
    pub fn relative_path_display(&self, command: &Command) -> String {
        let Some(cmd_dir) = self.get_command_dir(command) else {
            return String::new();
        };

        // Try to get relative path from cwd
        if let Ok(rel) = cmd_dir.strip_prefix(&self.cwd) {
            if rel.as_os_str().is_empty() {
                return ".".to_string();
            }
            return rel.display().to_string();
        }

        // Try to get relative path from project root
        if let Ok(rel) = cmd_dir.strip_prefix(&self.project_root) {
            if rel.as_os_str().is_empty() {
                return "/".to_string();
            }
            return format!("/{}", rel.display());
        }

        // Fall back to full path
        cmd_dir.display().to_string()
    }

    /// Get workspace name from command path.
    pub fn workspace_name(&self, command: &Command) -> Option<String> {
        let cmd_dir = command.working_dir.clone().or_else(|| self.get_source_path(command))?;

        // Try to get the first component after project root
        let rel = cmd_dir.strip_prefix(&self.project_root).ok()?;
        let first_component = rel.components().next()?;

        Some(first_component.as_os_str().to_string_lossy().to_string())
    }

    /// Check if a command passes the current filter.
    pub fn matches_filter(&self, command: &Command) -> bool {
        match &self.filter {
            ContextFilter::All => true,
            ContextFilter::CurrentDir => {
                let cmd_dir = command.working_dir.clone().or_else(|| self.get_source_path(command));
                cmd_dir.map(|p| p == self.cwd || p.starts_with(&self.cwd)).unwrap_or(true)
            }
            ContextFilter::ProjectRoot => {
                let cmd_dir = command.working_dir.clone().or_else(|| self.get_source_path(command));
                cmd_dir.map(|p| p == self.project_root).unwrap_or(true)
            }
            ContextFilter::Workspace(name) => self.workspace_name(command).as_ref() == Some(name),
            ContextFilter::SourceType(source_type) => command.source.type_name() == source_type,
        }
    }

    /// Get the working directory for a command.
    ///
    /// Returns the command's working_dir if set, otherwise tries to infer
    /// from the command source.
    fn get_command_dir(&self, command: &Command) -> Option<PathBuf> {
        command.working_dir.clone().or_else(|| self.get_source_path(command))
    }

    /// Get the source path for a command (from its source).
    fn get_source_path(&self, command: &Command) -> Option<PathBuf> {
        match &command.source {
            CommandSource::PackageJson(p)
            | CommandSource::Makefile(p)
            | CommandSource::Taskfile(p)
            | CommandSource::DockerCompose(p)
            | CommandSource::Cargo(p)
            | CommandSource::GoMod(p)
            | CommandSource::Python(p) => Some(p.clone()),
            CommandSource::NxProject(_) | CommandSource::Turbo => Some(self.project_root.clone()),
            CommandSource::Git
            | CommandSource::Manual
            | CommandSource::History
            | CommandSource::Favorite
            | CommandSource::Alias
            | CommandSource::Builtin
            | CommandSource::Mcp { .. } => None,
        }
    }

    /// Sort commands by proximity score.
    pub fn sort_by_proximity<'a>(&self, commands: &mut [&'a Command]) {
        commands.sort_by(|a, b| {
            let score_a = self.proximity_score(a);
            let score_b = self.proximity_score(b);
            score_b.cmp(&score_a) // Higher score first
        });
    }

    /// Filter and sort commands by context.
    pub fn filter_and_sort<'a>(&self, commands: &'a [Command]) -> Vec<&'a Command> {
        let mut filtered: Vec<&Command> =
            commands.iter().filter(|c| self.matches_filter(c)).collect();
        self.sort_by_proximity(&mut filtered);
        filtered
    }
}

/// Location indicator for displaying command context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationIndicator {
    /// Short display string (e.g., ".", "/", "pkg/foo")
    pub display: String,

    /// Whether this is the current directory
    pub is_current: bool,

    /// Whether this is the project root
    pub is_root: bool,

    /// Proximity score (0-100)
    pub score: u32,
}

impl LocationIndicator {
    /// Create a location indicator for a command.
    pub fn for_command(context: &CommandContext, command: &Command) -> Self {
        let display = context.relative_path_display(command);
        let score = context.proximity_score(command);

        Self { is_current: display == ".", is_root: display == "/", display, score }
    }

    /// Get a colored/formatted display string based on proximity.
    pub fn formatted_display(&self) -> String {
        if self.display.is_empty() {
            return String::new();
        }

        match self.score {
            80..=100 => format!("[{}]", self.display),
            50..=79 => format!("[{}]", self.display),
            _ => format!("[{}]", self.display),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> CommandContext {
        CommandContext::new(Path::new("/project/packages/app"), Path::new("/project"))
    }

    fn create_command_with_dir(name: &str, dir: &str) -> Command {
        Command::new(name, name).with_working_dir(PathBuf::from(dir))
    }

    #[test]
    fn test_context_creation() {
        let ctx = create_test_context();
        assert_eq!(ctx.cwd, PathBuf::from("/project/packages/app"));
        assert_eq!(ctx.project_root, PathBuf::from("/project"));
    }

    #[test]
    fn test_proximity_score_current_dir() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app");
        assert_eq!(ctx.proximity_score(&cmd), 100);
    }

    #[test]
    fn test_proximity_score_project_root() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project");
        assert_eq!(ctx.proximity_score(&cmd), 80);
    }

    #[test]
    fn test_proximity_score_subdirectory() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app/src");
        // Subdirectory of cwd should score 65-70
        let score = ctx.proximity_score(&cmd);
        assert!((60..=70).contains(&score), "Score was {score}");
    }

    #[test]
    fn test_proximity_score_sibling() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/lib");
        // Sibling directory should score 30-50
        let score = ctx.proximity_score(&cmd);
        assert!((20..=50).contains(&score), "Score was {score}");
    }

    #[test]
    fn test_relative_path_display_current() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app");
        assert_eq!(ctx.relative_path_display(&cmd), ".");
    }

    #[test]
    fn test_relative_path_display_subdirectory() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app/src");
        assert_eq!(ctx.relative_path_display(&cmd), "src");
    }

    #[test]
    fn test_relative_path_display_project_root() {
        let ctx = CommandContext::new(Path::new("/project"), Path::new("/project"));
        let cmd = create_command_with_dir("test", "/project");
        assert_eq!(ctx.relative_path_display(&cmd), ".");
    }

    #[test]
    fn test_workspace_name() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app");
        assert_eq!(ctx.workspace_name(&cmd), Some("packages".to_string()));
    }

    #[test]
    fn test_filter_all() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/anywhere");
        assert!(ctx.matches_filter(&cmd));
    }

    #[test]
    fn test_filter_current_dir() {
        let ctx = create_test_context().with_filter(ContextFilter::CurrentDir);

        let cmd_current = create_command_with_dir("test", "/project/packages/app");
        let cmd_other = create_command_with_dir("test", "/project/packages/lib");

        assert!(ctx.matches_filter(&cmd_current));
        assert!(!ctx.matches_filter(&cmd_other));
    }

    #[test]
    fn test_filter_source_type() {
        let ctx = create_test_context().with_filter(ContextFilter::SourceType("npm".to_string()));

        let cmd_npm = Command::new("npm test", "npm test")
            .with_source(CommandSource::PackageJson(PathBuf::from("/project")));
        let cmd_make = Command::new("make build", "make build")
            .with_source(CommandSource::Makefile(PathBuf::from("/project")));

        assert!(ctx.matches_filter(&cmd_npm));
        assert!(!ctx.matches_filter(&cmd_make));
    }

    #[test]
    fn test_sort_by_proximity() {
        let ctx = create_test_context();

        let cmd_current = create_command_with_dir("current", "/project/packages/app");
        let cmd_root = create_command_with_dir("root", "/project");
        let cmd_sibling = create_command_with_dir("sibling", "/project/packages/lib");

        let mut commands = vec![&cmd_sibling, &cmd_root, &cmd_current];
        ctx.sort_by_proximity(&mut commands);

        // Should be sorted: current (100), root (80), sibling (~35)
        assert_eq!(commands[0].name, "current");
        assert_eq!(commands[1].name, "root");
        assert_eq!(commands[2].name, "sibling");
    }

    #[test]
    fn test_location_indicator() {
        let ctx = create_test_context();
        let cmd = create_command_with_dir("test", "/project/packages/app");

        let indicator = LocationIndicator::for_command(&ctx, &cmd);
        assert!(indicator.is_current);
        assert_eq!(indicator.display, ".");
        assert_eq!(indicator.score, 100);
    }

    #[test]
    fn test_filter_and_sort() {
        let ctx = create_test_context();

        let commands = vec![
            create_command_with_dir("sibling", "/project/packages/lib"),
            create_command_with_dir("current", "/project/packages/app"),
            create_command_with_dir("root", "/project"),
        ];

        let sorted = ctx.filter_and_sort(&commands);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "current");
        assert_eq!(sorted[1].name, "root");
    }
}
