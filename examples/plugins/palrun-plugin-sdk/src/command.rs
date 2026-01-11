//! Command types for scanner plugins.

use serde::{Deserialize, Serialize};

/// A command discovered by a scanner plugin.
///
/// Commands represent executable actions that can be run in a project.
/// They are displayed in the Palrun command palette.
///
/// # Example
///
/// ```rust
/// use palrun_plugin_sdk::Command;
///
/// let cmd = Command::new("build", "cargo build")
///     .with_description("Build the project")
///     .with_tag("cargo")
///     .with_tag("build");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Command {
    /// Display name shown in the command palette.
    pub name: String,

    /// The actual command to execute.
    pub command: String,

    /// Optional description explaining what the command does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Working directory relative to project root.
    /// If None, uses the project root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Tags for categorization and filtering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Command {
    /// Create a new command with a name and command string.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the command
    /// * `command` - The shell command to execute
    ///
    /// # Example
    ///
    /// ```rust
    /// use palrun_plugin_sdk::Command;
    ///
    /// let cmd = Command::new("test", "cargo test");
    /// assert_eq!(cmd.name, "test");
    /// assert_eq!(cmd.command, "cargo test");
    /// ```
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: None,
            working_dir: None,
            tags: Vec::new(),
        }
    }

    /// Add a description to the command.
    ///
    /// # Example
    ///
    /// ```rust
    /// use palrun_plugin_sdk::Command;
    ///
    /// let cmd = Command::new("build", "make build")
    ///     .with_description("Compile the project");
    /// ```
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the working directory for the command.
    ///
    /// The path should be relative to the project root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use palrun_plugin_sdk::Command;
    ///
    /// let cmd = Command::new("test", "npm test")
    ///     .with_working_dir("frontend");
    /// ```
    #[must_use]
    pub fn with_working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    /// Add a tag to the command.
    ///
    /// Tags help users filter and find commands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use palrun_plugin_sdk::Command;
    ///
    /// let cmd = Command::new("lint", "eslint .")
    ///     .with_tag("lint")
    ///     .with_tag("javascript");
    /// ```
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the command.
    ///
    /// # Example
    ///
    /// ```rust
    /// use palrun_plugin_sdk::Command;
    ///
    /// let cmd = Command::new("build", "./gradlew build")
    ///     .with_tags(["gradle", "java", "build"]);
    /// ```
    #[must_use]
    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }
}

/// Builder for creating commands with a fluent API.
///
/// This is an alternative to using `Command::new()` with chained methods.
///
/// # Example
///
/// ```rust
/// use palrun_plugin_sdk::CommandBuilder;
///
/// let cmd = CommandBuilder::new()
///     .name("deploy")
///     .command("./deploy.sh")
///     .description("Deploy to production")
///     .tag("deploy")
///     .tag("production")
///     .build()
///     .expect("command should be valid");
/// ```
#[derive(Debug, Default)]
pub struct CommandBuilder {
    name: Option<String>,
    command: Option<String>,
    description: Option<String>,
    working_dir: Option<String>,
    tags: Vec<String>,
}

impl CommandBuilder {
    /// Create a new command builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the command string.
    #[must_use]
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the working directory.
    #[must_use]
    pub fn working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags.
    #[must_use]
    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Build the command.
    ///
    /// Returns `None` if name or command are not set.
    pub fn build(self) -> Option<Command> {
        Some(Command {
            name: self.name?,
            command: self.command?,
            description: self.description,
            working_dir: self.working_dir,
            tags: self.tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_new() {
        let cmd = Command::new("test", "cargo test");
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.command, "cargo test");
        assert!(cmd.description.is_none());
        assert!(cmd.working_dir.is_none());
        assert!(cmd.tags.is_empty());
    }

    #[test]
    fn test_command_builder_chain() {
        let cmd = Command::new("build", "make")
            .with_description("Build project")
            .with_working_dir("src")
            .with_tag("build")
            .with_tag("make");

        assert_eq!(cmd.name, "build");
        assert_eq!(cmd.description, Some("Build project".to_string()));
        assert_eq!(cmd.working_dir, Some("src".to_string()));
        assert_eq!(cmd.tags, vec!["build", "make"]);
    }

    #[test]
    fn test_command_with_tags() {
        let cmd = Command::new("lint", "npm run lint").with_tags(["lint", "npm", "js"]);

        assert_eq!(cmd.tags.len(), 3);
        assert!(cmd.tags.contains(&"lint".to_string()));
    }

    #[test]
    fn test_command_builder() {
        let cmd = CommandBuilder::new()
            .name("deploy")
            .command("./deploy.sh")
            .description("Deploy app")
            .tag("deploy")
            .build();

        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.name, "deploy");
        assert_eq!(cmd.command, "./deploy.sh");
    }

    #[test]
    fn test_command_builder_missing_fields() {
        let cmd = CommandBuilder::new().name("test").build();
        assert!(cmd.is_none());

        let cmd = CommandBuilder::new().command("test").build();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_command_serialization() {
        let cmd = Command::new("build", "cargo build")
            .with_description("Build")
            .with_tag("cargo");

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"name\":\"build\""));
        assert!(json.contains("\"command\":\"cargo build\""));

        let deserialized: Command = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cmd);
    }

    #[test]
    fn test_command_serialization_skips_empty() {
        let cmd = Command::new("test", "npm test");
        let json = serde_json::to_string(&cmd).unwrap();

        // Empty fields should be skipped
        assert!(!json.contains("description"));
        assert!(!json.contains("working_dir"));
        assert!(!json.contains("tags"));
    }
}
