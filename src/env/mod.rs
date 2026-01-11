//! Environment management module.
//!
//! Provides functionality for detecting, parsing, and switching between
//! .env files, viewing environment variables, and managing runtime versions.

pub mod secrets;
pub mod version;

pub use secrets::{ProviderStatus, ResolvedSecret, SecretProvider, SecretReference, SecretsManager};
pub use version::{RuntimeType, RuntimeVersion, VersionManager};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result};

/// Known .env file patterns to scan for.
pub const ENV_FILE_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.development",
    ".env.development.local",
    ".env.test",
    ".env.test.local",
    ".env.staging",
    ".env.staging.local",
    ".env.production",
    ".env.production.local",
    ".env.example",
    ".env.sample",
    ".env.template",
];

/// Sensitive variable name patterns that should be masked.
const SENSITIVE_PATTERNS: &[&str] = &[
    "KEY",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "TOKEN",
    "CREDENTIAL",
    "AUTH",
    "PRIVATE",
    "API_KEY",
    "ACCESS_KEY",
    "CLIENT_SECRET",
];

/// Information about a detected .env file.
#[derive(Debug, Clone)]
pub struct EnvFile {
    /// Name of the environment (extracted from filename)
    pub name: String,

    /// Full path to the .env file
    pub path: PathBuf,

    /// Number of variables defined in the file
    pub variable_count: usize,

    /// Whether this is a template/example file
    pub is_template: bool,

    /// Whether this file is currently active (loaded)
    pub is_active: bool,
}

impl EnvFile {
    /// Get the environment type from the filename.
    pub fn env_type(&self) -> &str {
        if self.name.contains("production") || self.name.contains("prod") {
            "production"
        } else if self.name.contains("staging") {
            "staging"
        } else if self.name.contains("test") {
            "test"
        } else if self.name.contains("development") || self.name.contains("dev") {
            "development"
        } else if self.is_template {
            "template"
        } else {
            "default"
        }
    }

    /// Get an icon for the environment type.
    pub fn icon(&self) -> &str {
        match self.env_type() {
            "production" => "🔴",
            "staging" => "🟡",
            "test" => "🧪",
            "development" => "🟢",
            "template" => "📋",
            _ => "📄",
        }
    }
}

/// An environment variable with metadata.
#[derive(Debug, Clone)]
pub struct EnvVariable {
    /// Variable name
    pub name: String,

    /// Variable value
    pub value: String,

    /// Source of the variable (.env file, system, shell)
    pub source: EnvSource,

    /// Whether this is a sensitive variable
    pub is_sensitive: bool,
}

impl EnvVariable {
    /// Get the masked value for display.
    pub fn masked_value(&self) -> String {
        if self.is_sensitive {
            if self.value.len() <= 4 {
                "****".to_string()
            } else {
                format!("{}****", &self.value[..2])
            }
        } else {
            self.value.clone()
        }
    }
}

/// Source of an environment variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvSource {
    /// From a .env file
    DotEnv(PathBuf),
    /// From the system environment
    System,
    /// From shell configuration
    Shell,
    /// Unknown source
    Unknown,
}

impl EnvSource {
    /// Get a display string for the source.
    pub fn display(&self) -> String {
        match self {
            EnvSource::DotEnv(path) => {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(".env")
                    .to_string()
            }
            EnvSource::System => "system".to_string(),
            EnvSource::Shell => "shell".to_string(),
            EnvSource::Unknown => "unknown".to_string(),
        }
    }
}

/// Environment manager for detecting and managing .env files.
pub struct EnvManager {
    /// Project root directory
    root: PathBuf,

    /// Detected .env files
    env_files: Vec<EnvFile>,

    /// Currently loaded environment variables from .env
    loaded_vars: HashMap<String, String>,

    /// Path to the currently active .env file
    active_file: Option<PathBuf>,
}

impl EnvManager {
    /// Create a new environment manager for the given project root.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            env_files: Vec::new(),
            loaded_vars: HashMap::new(),
            active_file: None,
        }
    }

    /// Scan for .env files in the project root.
    pub fn scan(&mut self) -> Result<&[EnvFile]> {
        self.env_files.clear();

        for pattern in ENV_FILE_PATTERNS {
            let path = self.root.join(pattern);
            if path.exists() && path.is_file() {
                if let Ok(env_file) = self.parse_env_file(&path) {
                    self.env_files.push(env_file);
                }
            }
        }

        // Sort by name
        self.env_files.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(&self.env_files)
    }

    /// Parse a .env file and return metadata.
    fn parse_env_file(&self, path: &Path) -> Result<EnvFile> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".env")
            .to_string();

        let is_template = name.contains("example")
            || name.contains("sample")
            || name.contains("template");

        // Count non-empty, non-comment lines
        let variable_count = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .count();

        let is_active = self.active_file.as_ref() == Some(&path.to_path_buf());

        Ok(EnvFile {
            name,
            path: path.to_path_buf(),
            variable_count,
            is_template,
            is_active,
        })
    }

    /// Get all detected .env files.
    pub fn get_env_files(&self) -> &[EnvFile] {
        &self.env_files
    }

    /// Load a specific .env file.
    pub fn load_env_file(&mut self, path: &Path) -> Result<usize> {
        self.loaded_vars.clear();

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = Self::parse_value(value.trim());
                self.loaded_vars.insert(key, value);
            }
        }

        self.active_file = Some(path.to_path_buf());

        // Update the is_active flag for env_files
        for env_file in &mut self.env_files {
            env_file.is_active = env_file.path == path;
        }

        Ok(self.loaded_vars.len())
    }

    /// Parse an environment variable value, handling quotes.
    fn parse_value(value: &str) -> String {
        let value = value.trim();

        // Handle quoted strings
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value[1..value.len() - 1].to_string()
        } else {
            // Remove inline comments
            value.split('#').next().unwrap_or(value).trim().to_string()
        }
    }

    /// Apply loaded environment variables to the current process.
    pub fn apply_to_process(&self) {
        for (key, value) in &self.loaded_vars {
            env::set_var(key, value);
        }
    }

    /// Get the currently active .env file path.
    pub fn active_file(&self) -> Option<&Path> {
        self.active_file.as_deref()
    }

    /// Get the name of the active environment.
    pub fn active_env_name(&self) -> Option<&str> {
        self.env_files
            .iter()
            .find(|f| f.is_active)
            .map(|f| f.name.as_str())
    }

    /// Get all environment variables with their sources.
    pub fn get_all_variables(&self) -> Vec<EnvVariable> {
        let mut variables = Vec::new();

        // Add loaded .env variables
        for (name, value) in &self.loaded_vars {
            variables.push(EnvVariable {
                name: name.clone(),
                value: value.clone(),
                source: self
                    .active_file
                    .as_ref()
                    .map(|p| EnvSource::DotEnv(p.clone()))
                    .unwrap_or(EnvSource::Unknown),
                is_sensitive: Self::is_sensitive_var(name),
            });
        }

        // Add system environment variables (that aren't in .env)
        for (name, value) in env::vars() {
            if !self.loaded_vars.contains_key(&name) {
                variables.push(EnvVariable {
                    name: name.clone(),
                    value,
                    source: EnvSource::System,
                    is_sensitive: Self::is_sensitive_var(&name),
                });
            }
        }

        // Sort by name
        variables.sort_by(|a, b| a.name.cmp(&b.name));

        variables
    }

    /// Check if a variable name is sensitive.
    fn is_sensitive_var(name: &str) -> bool {
        let upper = name.to_uppercase();
        SENSITIVE_PATTERNS.iter().any(|pattern| upper.contains(pattern))
    }

    /// Get variables from a specific .env file without loading it.
    pub fn preview_env_file(&self, path: &Path) -> Result<Vec<EnvVariable>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let mut variables = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let name = key.trim().to_string();
                let value = Self::parse_value(value.trim());
                variables.push(EnvVariable {
                    name: name.clone(),
                    value,
                    source: EnvSource::DotEnv(path.to_path_buf()),
                    is_sensitive: Self::is_sensitive_var(&name),
                });
            }
        }

        Ok(variables)
    }

    /// Compare two .env files and return differences.
    pub fn compare_env_files(&self, file1: &Path, file2: &Path) -> Result<EnvDiff> {
        let vars1 = self.preview_env_file(file1)?;
        let vars2 = self.preview_env_file(file2)?;

        let map1: HashMap<_, _> = vars1.iter().map(|v| (&v.name, &v.value)).collect();
        let map2: HashMap<_, _> = vars2.iter().map(|v| (&v.name, &v.value)).collect();

        let mut only_in_first = Vec::new();
        let mut only_in_second = Vec::new();
        let mut different = Vec::new();

        for var in &vars1 {
            if let Some(value2) = map2.get(&var.name) {
                if *value2 != &var.value {
                    different.push((var.name.clone(), var.value.clone(), (*value2).clone()));
                }
            } else {
                only_in_first.push(var.name.clone());
            }
        }

        for var in &vars2 {
            if !map1.contains_key(&var.name) {
                only_in_second.push(var.name.clone());
            }
        }

        Ok(EnvDiff {
            only_in_first,
            only_in_second,
            different,
        })
    }
}

/// Difference between two .env files.
#[derive(Debug)]
pub struct EnvDiff {
    /// Variables only in the first file
    pub only_in_first: Vec<String>,

    /// Variables only in the second file
    pub only_in_second: Vec<String>,

    /// Variables with different values (name, value1, value2)
    pub different: Vec<(String, String, String)>,
}

impl EnvDiff {
    /// Check if there are any differences.
    pub fn has_differences(&self) -> bool {
        !self.only_in_first.is_empty()
            || !self.only_in_second.is_empty()
            || !self.different.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_env_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_env_manager_scan() {
        let temp = TempDir::new().unwrap();

        create_test_env_file(
            temp.path(),
            ".env",
            "DB_HOST=localhost\nDB_PORT=5432\n",
        );
        create_test_env_file(
            temp.path(),
            ".env.development",
            "NODE_ENV=development\nDEBUG=true\n",
        );

        let mut manager = EnvManager::new(temp.path());
        manager.scan().unwrap();

        assert_eq!(manager.get_env_files().len(), 2);
    }

    #[test]
    fn test_env_file_parsing() {
        let temp = TempDir::new().unwrap();

        create_test_env_file(
            temp.path(),
            ".env",
            r#"
# Database config
DB_HOST=localhost
DB_PORT=5432
DB_PASSWORD="secret123"
API_KEY='my-api-key'
EMPTY=
# Comment line
DEBUG=true  # inline comment
"#,
        );

        let mut manager = EnvManager::new(temp.path());
        manager.scan().unwrap();

        let env_file = &manager.get_env_files()[0];
        assert_eq!(env_file.variable_count, 6); // DB_HOST, DB_PORT, DB_PASSWORD, API_KEY, EMPTY, DEBUG
        let path = env_file.path.clone();

        manager.load_env_file(&path).unwrap();

        let vars = manager.get_all_variables();
        let db_host = vars.iter().find(|v| v.name == "DB_HOST").unwrap();
        assert_eq!(db_host.value, "localhost");

        let db_password = vars.iter().find(|v| v.name == "DB_PASSWORD").unwrap();
        assert_eq!(db_password.value, "secret123");
        assert!(db_password.is_sensitive);

        let api_key = vars.iter().find(|v| v.name == "API_KEY").unwrap();
        assert_eq!(api_key.value, "my-api-key");
        assert!(api_key.is_sensitive);

        let debug = vars.iter().find(|v| v.name == "DEBUG").unwrap();
        assert_eq!(debug.value, "true");
    }

    #[test]
    fn test_sensitive_detection() {
        assert!(EnvManager::is_sensitive_var("API_KEY"));
        assert!(EnvManager::is_sensitive_var("DATABASE_PASSWORD"));
        assert!(EnvManager::is_sensitive_var("SECRET_TOKEN"));
        assert!(EnvManager::is_sensitive_var("AWS_ACCESS_KEY_ID"));
        assert!(!EnvManager::is_sensitive_var("NODE_ENV"));
        assert!(!EnvManager::is_sensitive_var("PORT"));
    }

    #[test]
    fn test_env_file_types() {
        let production = EnvFile {
            name: ".env.production".to_string(),
            path: PathBuf::from(".env.production"),
            variable_count: 5,
            is_template: false,
            is_active: false,
        };
        assert_eq!(production.env_type(), "production");
        assert_eq!(production.icon(), "🔴");

        let development = EnvFile {
            name: ".env.development".to_string(),
            path: PathBuf::from(".env.development"),
            variable_count: 5,
            is_template: false,
            is_active: false,
        };
        assert_eq!(development.env_type(), "development");
        assert_eq!(development.icon(), "🟢");

        let example = EnvFile {
            name: ".env.example".to_string(),
            path: PathBuf::from(".env.example"),
            variable_count: 5,
            is_template: true,
            is_active: false,
        };
        assert_eq!(example.env_type(), "template");
        assert_eq!(example.icon(), "📋");
    }

    #[test]
    fn test_compare_env_files() {
        let temp = TempDir::new().unwrap();

        let file1 = create_test_env_file(
            temp.path(),
            ".env.development",
            "NODE_ENV=development\nDB_HOST=localhost\nDEBUG=true\n",
        );
        let file2 = create_test_env_file(
            temp.path(),
            ".env.production",
            "NODE_ENV=production\nDB_HOST=prod.db.com\nLOG_LEVEL=error\n",
        );

        let manager = EnvManager::new(temp.path());
        let diff = manager.compare_env_files(&file1, &file2).unwrap();

        assert!(diff.has_differences());
        assert_eq!(diff.only_in_first, vec!["DEBUG"]);
        assert_eq!(diff.only_in_second, vec!["LOG_LEVEL"]);
        assert_eq!(diff.different.len(), 2); // NODE_ENV and DB_HOST differ
    }

    #[test]
    fn test_masked_value() {
        let sensitive = EnvVariable {
            name: "API_KEY".to_string(),
            value: "sk-abc123xyz789".to_string(),
            source: EnvSource::System,
            is_sensitive: true,
        };
        assert_eq!(sensitive.masked_value(), "sk****");

        let short_sensitive = EnvVariable {
            name: "KEY".to_string(),
            value: "abc".to_string(),
            source: EnvSource::System,
            is_sensitive: true,
        };
        assert_eq!(short_sensitive.masked_value(), "****");

        let non_sensitive = EnvVariable {
            name: "PORT".to_string(),
            value: "3000".to_string(),
            source: EnvSource::System,
            is_sensitive: false,
        };
        assert_eq!(non_sensitive.masked_value(), "3000");
    }
}
