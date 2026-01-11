//! Scan context provided to scanner plugins.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context provided to scanner plugins during scanning.
///
/// Contains information about the project being scanned and
/// the files that matched the scanner's file patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanContext {
    /// Absolute path to the project root directory.
    pub project_path: String,

    /// Name of the project (typically derived from directory name).
    pub project_name: String,

    /// Files that matched the scanner's file patterns.
    ///
    /// Keys are file paths relative to project root,
    /// values are the file contents.
    #[serde(default)]
    pub matched_files: HashMap<String, String>,

    /// Environment variables available to the plugin.
    /// Only populated if the plugin has environment permission.
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

impl ScanContext {
    /// Create a new scan context.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Absolute path to the project root
    /// * `project_name` - Name of the project
    pub fn new(project_path: impl Into<String>, project_name: impl Into<String>) -> Self {
        Self {
            project_path: project_path.into(),
            project_name: project_name.into(),
            matched_files: HashMap::new(),
            environment: HashMap::new(),
        }
    }

    /// Get the content of a matched file.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to the file
    ///
    /// # Returns
    ///
    /// File contents if the file was matched, None otherwise.
    pub fn get_file(&self, path: &str) -> Option<&str> {
        self.matched_files.get(path).map(String::as_str)
    }

    /// Check if a file exists in the matched files.
    pub fn has_file(&self, path: &str) -> bool {
        self.matched_files.contains_key(path)
    }

    /// Get an environment variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Environment variable name
    ///
    /// # Returns
    ///
    /// Variable value if available, None otherwise.
    pub fn get_env(&self, name: &str) -> Option<&str> {
        self.environment.get(name).map(String::as_str)
    }

    /// Get all matched file paths.
    pub fn file_paths(&self) -> impl Iterator<Item = &str> {
        self.matched_files.keys().map(String::as_str)
    }

    /// Add a matched file to the context.
    ///
    /// This is primarily used for testing.
    pub fn with_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.matched_files.insert(path.into(), content.into());
        self
    }

    /// Add an environment variable to the context.
    ///
    /// This is primarily used for testing.
    pub fn with_env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(name.into(), value.into());
        self
    }
}

impl Default for ScanContext {
    fn default() -> Self {
        Self::new(".", "unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = ScanContext::new("/path/to/project", "my-project");
        assert_eq!(ctx.project_path, "/path/to/project");
        assert_eq!(ctx.project_name, "my-project");
        assert!(ctx.matched_files.is_empty());
    }

    #[test]
    fn test_context_with_file() {
        let ctx = ScanContext::new("/project", "test")
            .with_file("Cargo.toml", "[package]\nname = \"test\"")
            .with_file("src/main.rs", "fn main() {}");

        assert!(ctx.has_file("Cargo.toml"));
        assert!(ctx.has_file("src/main.rs"));
        assert!(!ctx.has_file("nonexistent"));

        assert_eq!(
            ctx.get_file("Cargo.toml"),
            Some("[package]\nname = \"test\"")
        );
    }

    #[test]
    fn test_context_file_paths() {
        let ctx = ScanContext::new("/project", "test")
            .with_file("a.txt", "a")
            .with_file("b.txt", "b");

        let paths: Vec<_> = ctx.file_paths().collect();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_context_environment() {
        let ctx = ScanContext::new("/project", "test")
            .with_env("PATH", "/usr/bin")
            .with_env("HOME", "/home/user");

        assert_eq!(ctx.get_env("PATH"), Some("/usr/bin"));
        assert_eq!(ctx.get_env("HOME"), Some("/home/user"));
        assert_eq!(ctx.get_env("NONEXISTENT"), None);
    }

    #[test]
    fn test_context_serialization() {
        let ctx =
            ScanContext::new("/project", "test").with_file("Makefile", "all:\n\techo hello");

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: ScanContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.project_path, ctx.project_path);
        assert_eq!(deserialized.project_name, ctx.project_name);
        assert_eq!(deserialized.get_file("Makefile"), ctx.get_file("Makefile"));
    }
}
