//! Error types for Palrun plugins.

use std::fmt;

/// Error type for plugin operations.
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Failed to read a file.
    FileReadError(String),

    /// Failed to parse file content.
    ParseError(String),

    /// Invalid configuration.
    ConfigError(String),

    /// Operation not permitted.
    PermissionDenied(String),

    /// General error with message.
    Other(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileReadError(msg) => write!(f, "file read error: {msg}"),
            Self::ParseError(msg) => write!(f, "parse error: {msg}"),
            Self::ConfigError(msg) => write!(f, "config error: {msg}"),
            Self::PermissionDenied(msg) => write!(f, "permission denied: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

impl PluginError {
    /// Create a file read error.
    pub fn file_read(msg: impl Into<String>) -> Self {
        Self::FileReadError(msg.into())
    }

    /// Create a parse error.
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a config error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a permission denied error.
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a generic error.
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PluginError::file_read("not found");
        assert_eq!(err.to_string(), "file read error: not found");

        let err = PluginError::parse("invalid syntax");
        assert_eq!(err.to_string(), "parse error: invalid syntax");

        let err = PluginError::config("missing field");
        assert_eq!(err.to_string(), "config error: missing field");

        let err = PluginError::permission_denied("no read access");
        assert_eq!(err.to_string(), "permission denied: no read access");

        let err = PluginError::other("something went wrong");
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_error_is_error() {
        let err: Box<dyn std::error::Error> = Box::new(PluginError::other("test"));
        assert_eq!(err.to_string(), "test");
    }
}
