//! Plugin manifest parsing and validation.
//!
//! A plugin manifest is a TOML file that describes a plugin's metadata,
//! permissions, and configuration.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{PluginError, PluginResult, PluginType};

/// Plugin manifest containing metadata and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata.
    pub plugin: PluginMetadata,
    /// Plugin permissions.
    #[serde(default)]
    pub permissions: PluginPermissions,
    /// Plugin-specific configuration schema.
    #[serde(default)]
    pub config: toml::Table,
}

/// Plugin metadata section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name (unique identifier).
    pub name: String,
    /// Plugin version (semver).
    pub version: String,
    /// Plugin author.
    #[serde(default)]
    pub author: Option<String>,
    /// Plugin description.
    #[serde(default)]
    pub description: Option<String>,
    /// Plugin type.
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    /// Minimum API version required.
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// Plugin homepage URL.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Plugin repository URL.
    #[serde(default)]
    pub repository: Option<String>,
    /// Plugin license.
    #[serde(default)]
    pub license: Option<String>,
    /// Keywords for search.
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_api_version() -> String {
    "0.1.0".to_string()
}

/// Plugin permissions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Filesystem access permissions.
    #[serde(default)]
    pub filesystem: FilesystemPermissions,
    /// Network access permission.
    #[serde(default)]
    pub network: bool,
    /// Command execution permission.
    #[serde(default)]
    pub execute: bool,
    /// Environment variable access.
    #[serde(default)]
    pub environment: bool,
}

/// Filesystem permission levels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilesystemPermissions {
    /// Read access to project files.
    #[serde(default)]
    pub read: bool,
    /// Write access to project files.
    #[serde(default)]
    pub write: bool,
    /// Allowed path patterns (glob).
    #[serde(default)]
    pub paths: Vec<String>,
}

impl PluginManifest {
    /// Parse a manifest from TOML string.
    pub fn from_toml(content: &str) -> PluginResult<Self> {
        toml::from_str(content).map_err(|e| PluginError::InvalidManifest(e.to_string()))
    }

    /// Parse a manifest from a file.
    pub fn from_file(path: &Path) -> PluginResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> PluginResult<String> {
        toml::to_string_pretty(self).map_err(|e| PluginError::InvalidManifest(e.to_string()))
    }

    /// Validate the manifest.
    pub fn validate(&self) -> PluginResult<()> {
        // Validate plugin name
        if self.plugin.name.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin name is required".to_string()));
        }

        if !self.plugin.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(PluginError::InvalidManifest(
                "Plugin name must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate version (basic semver check)
        if self.plugin.version.is_empty() {
            return Err(PluginError::InvalidManifest("Plugin version is required".to_string()));
        }

        let version_parts: Vec<&str> = self.plugin.version.split('.').collect();
        if version_parts.len() < 2 {
            return Err(PluginError::InvalidManifest(
                "Version must be in semver format (e.g., 1.0.0)".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if this manifest is compatible with the given API version.
    pub fn is_compatible_with(&self, host_api_version: &str) -> bool {
        // Simple compatibility check: major version must match
        let required: Vec<u32> =
            self.plugin.api_version.split('.').filter_map(|s| s.parse().ok()).collect();
        let available: Vec<u32> =
            host_api_version.split('.').filter_map(|s| s.parse().ok()).collect();

        if required.is_empty() || available.is_empty() {
            return false;
        }

        // Major version must match, and available minor must be >= required
        required[0] == available[0]
            && (available.len() < 2 || required.len() < 2 || available[1] >= required[1])
    }
}

impl PluginPermissions {
    /// Check if the plugin requires filesystem read access.
    pub fn requires_filesystem_read(&self) -> bool {
        self.filesystem.read
    }

    /// Check if the plugin requires filesystem write access.
    pub fn requires_filesystem_write(&self) -> bool {
        self.filesystem.write
    }

    /// Check if a path is allowed for read access.
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if self.filesystem.paths.is_empty() {
            return self.filesystem.read;
        }

        // Check if path matches any allowed pattern
        self.filesystem.paths.iter().any(|pattern| {
            // Simple glob matching (just * for now)
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 2];
                path.starts_with(prefix)
            } else if pattern.ends_with("/**") {
                let prefix = &pattern[..pattern.len() - 3];
                path.starts_with(prefix)
            } else {
                path == pattern
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"
[plugin]
name = "gradle-scanner"
version = "0.1.0"
author = "community"
type = "scanner"
description = "Scans Gradle build files for tasks"
api_version = "0.1.0"
keywords = ["gradle", "java", "kotlin"]

[permissions]
network = false
execute = false

[permissions.filesystem]
read = true
write = false
paths = ["build.gradle", "build.gradle.kts", "settings.gradle*"]

[config]
scan_depth = 3
include_subprojects = true
"#;

    #[test]
    fn test_parse_manifest() {
        let manifest = PluginManifest::from_toml(SAMPLE_MANIFEST).unwrap();

        assert_eq!(manifest.plugin.name, "gradle-scanner");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert_eq!(manifest.plugin.plugin_type, PluginType::Scanner);
        assert!(manifest.permissions.filesystem.read);
        assert!(!manifest.permissions.filesystem.write);
        assert!(!manifest.permissions.network);
    }

    #[test]
    fn test_validate_manifest() {
        let manifest = PluginManifest::from_toml(SAMPLE_MANIFEST).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_name() {
        let toml = r#"
[plugin]
name = ""
version = "0.1.0"
type = "scanner"
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_invalid_version() {
        let toml = r#"
[plugin]
name = "test"
version = "invalid"
type = "scanner"
"#;
        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_api_compatibility() {
        let manifest = PluginManifest::from_toml(SAMPLE_MANIFEST).unwrap();

        assert!(manifest.is_compatible_with("0.1.0"));
        assert!(manifest.is_compatible_with("0.2.0"));
        assert!(!manifest.is_compatible_with("1.0.0"));
    }

    #[test]
    fn test_path_permissions() {
        let manifest = PluginManifest::from_toml(SAMPLE_MANIFEST).unwrap();

        assert!(manifest.permissions.is_path_allowed("build.gradle"));
        assert!(manifest.permissions.is_path_allowed("build.gradle.kts"));
    }

    #[test]
    fn test_serialize_manifest() {
        let manifest = PluginManifest::from_toml(SAMPLE_MANIFEST).unwrap();
        let serialized = manifest.to_toml().unwrap();
        assert!(serialized.contains("gradle-scanner"));
    }
}
