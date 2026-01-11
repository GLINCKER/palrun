//! Host functions for plugins.
//!
//! This module defines the interface that plugins can use to interact
//! with the host application (Palrun).

use std::path::PathBuf;

use super::{PluginCommand, PluginPermissions, PluginResult};

/// Log level for plugin logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level (most verbose).
    Trace,
    /// Debug level.
    Debug,
    /// Info level.
    Info,
    /// Warning level.
    Warn,
    /// Error level.
    Error,
}

impl LogLevel {
    /// Convert to u32 for WASM interface.
    pub fn to_u32(self) -> u32 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }

    /// Convert from u32.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Trace),
            1 => Some(Self::Debug),
            2 => Some(Self::Info),
            3 => Some(Self::Warn),
            4 => Some(Self::Error),
            _ => None,
        }
    }
}

/// Capabilities provided by the host to plugins.
#[derive(Debug, Clone)]
pub struct HostCapabilities {
    /// Project root directory.
    pub project_root: PathBuf,
    /// Plugin permissions.
    pub permissions: PluginPermissions,
}

/// Host interface for plugins.
///
/// This trait defines the functions that the host provides to plugins.
/// Plugins can call these functions to interact with the system.
pub trait PluginHost: Send + Sync {
    /// Log a message.
    fn log(&self, level: LogLevel, message: &str);

    /// Read a file from the project.
    ///
    /// Returns the file contents or an error if the file cannot be read
    /// or the plugin doesn't have permission.
    fn read_file(&self, path: &str) -> PluginResult<String>;

    /// Check if a file exists.
    fn file_exists(&self, path: &str) -> bool;

    /// List files in a directory.
    ///
    /// Returns a list of file paths relative to the project root.
    fn list_files(&self, path: &str, pattern: Option<&str>) -> PluginResult<Vec<String>>;

    /// Register a discovered command.
    fn register_command(&mut self, command: PluginCommand);

    /// Get an environment variable.
    ///
    /// Returns None if the variable doesn't exist or permission is denied.
    fn get_env(&self, name: &str) -> Option<String>;

    /// Get the project root directory.
    fn project_root(&self) -> &std::path::Path;

    /// Get the current working directory (relative to project root).
    fn current_dir(&self) -> Option<String>;
}

/// Default implementation of PluginHost for use during plugin execution.
pub struct DefaultPluginHost {
    capabilities: HostCapabilities,
    commands: Vec<PluginCommand>,
}

impl DefaultPluginHost {
    /// Create a new plugin host.
    pub fn new(capabilities: HostCapabilities) -> Self {
        Self { capabilities, commands: Vec::new() }
    }

    /// Get the registered commands.
    pub fn take_commands(&mut self) -> Vec<PluginCommand> {
        std::mem::take(&mut self.commands)
    }
}

impl PluginHost for DefaultPluginHost {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Trace => tracing::trace!(plugin = true, "{}", message),
            LogLevel::Debug => tracing::debug!(plugin = true, "{}", message),
            LogLevel::Info => tracing::info!(plugin = true, "{}", message),
            LogLevel::Warn => tracing::warn!(plugin = true, "{}", message),
            LogLevel::Error => tracing::error!(plugin = true, "{}", message),
        }
    }

    fn read_file(&self, path: &str) -> PluginResult<String> {
        // Check permissions
        if !self.capabilities.permissions.requires_filesystem_read() {
            return Err(super::PluginError::PermissionDenied {
                plugin: "unknown".to_string(),
                permission: "filesystem.read".to_string(),
            });
        }

        if !self.capabilities.permissions.is_path_allowed(path) {
            return Err(super::PluginError::PermissionDenied {
                plugin: "unknown".to_string(),
                permission: format!("read path: {path}"),
            });
        }

        let full_path = self.capabilities.project_root.join(path);
        std::fs::read_to_string(full_path).map_err(Into::into)
    }

    fn file_exists(&self, path: &str) -> bool {
        if !self.capabilities.permissions.requires_filesystem_read() {
            return false;
        }

        let full_path = self.capabilities.project_root.join(path);
        full_path.exists()
    }

    fn list_files(&self, path: &str, pattern: Option<&str>) -> PluginResult<Vec<String>> {
        if !self.capabilities.permissions.requires_filesystem_read() {
            return Err(super::PluginError::PermissionDenied {
                plugin: "unknown".to_string(),
                permission: "filesystem.read".to_string(),
            });
        }

        let full_path = self.capabilities.project_root.join(path);
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(full_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    // Simple pattern matching
                    let matches = pattern.map_or(true, |p| {
                        if p.starts_with('*') {
                            name.ends_with(&p[1..])
                        } else if p.ends_with('*') {
                            name.starts_with(&p[..p.len() - 1])
                        } else {
                            name == p
                        }
                    });

                    if matches {
                        files.push(name.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    fn register_command(&mut self, command: PluginCommand) {
        self.commands.push(command);
    }

    fn get_env(&self, name: &str) -> Option<String> {
        if !self.capabilities.permissions.environment {
            return None;
        }

        std::env::var(name).ok()
    }

    fn project_root(&self) -> &std::path::Path {
        &self.capabilities.project_root
    }

    fn current_dir(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::FilesystemPermissions;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::from_u32(0), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_u32(2), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_u32(4), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_u32(5), None);

        assert_eq!(LogLevel::Info.to_u32(), 2);
    }

    #[test]
    fn test_host_capabilities() {
        let caps = HostCapabilities {
            project_root: PathBuf::from("/test/project"),
            permissions: PluginPermissions::default(),
        };

        assert_eq!(caps.project_root, PathBuf::from("/test/project"));
    }

    #[test]
    fn test_default_host_register_command() {
        let caps = HostCapabilities {
            project_root: PathBuf::from("/test/project"),
            permissions: PluginPermissions {
                filesystem: FilesystemPermissions { read: true, write: false, paths: vec![] },
                network: false,
                execute: false,
                environment: false,
            },
        };

        let mut host = DefaultPluginHost::new(caps);

        host.register_command(PluginCommand {
            name: "test".to_string(),
            command: "echo test".to_string(),
            description: None,
            working_dir: None,
            tags: vec![],
        });

        let commands = host.take_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "test");
    }

    #[test]
    fn test_permission_denied_without_read() {
        let caps = HostCapabilities {
            project_root: PathBuf::from("/test/project"),
            permissions: PluginPermissions::default(),
        };

        let host = DefaultPluginHost::new(caps);
        let result = host.read_file("test.txt");
        assert!(result.is_err());
    }
}
