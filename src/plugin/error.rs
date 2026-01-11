//! Plugin system error types.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Errors that can occur during plugin operations.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin file not found.
    #[error("Plugin not found: {0}")]
    NotFound(PathBuf),

    /// Plugin already installed.
    #[error("Plugin '{0}' is already installed")]
    AlreadyInstalled(String),

    /// Invalid plugin manifest.
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),

    /// Plugin loading failed.
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    /// Plugin execution failed.
    #[error("Plugin execution failed: {0}")]
    ExecutionError(String),

    /// Plugin version incompatible.
    #[error("Plugin '{name}' requires API version {required}, but host provides {available}")]
    IncompatibleVersion { name: String, required: String, available: String },

    /// Permission denied.
    #[error("Plugin '{plugin}' requires permission '{permission}' which is not granted")]
    PermissionDenied { plugin: String, permission: String },

    /// Plugin timed out.
    #[error("Plugin '{0}' timed out after {1} seconds")]
    Timeout(String, u64),

    /// Plugin disabled.
    #[error("Plugin '{0}' is disabled")]
    Disabled(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// WASM error.
    #[error("WASM error: {0}")]
    Wasm(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error (for plugin downloads).
    #[error("Network error: {0}")]
    Network(String),

    /// Validation error (e.g., checksum mismatch).
    #[error("Validation error: {0}")]
    Validation(String),
}
