//! Secrets management for Palrun.
//!
//! Provides secure storage and retrieval of sensitive data like API keys
//! using the operating system's native credential storage:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring, KDE Wallet, etc.)
//!
//! # Security Features
//!
//! - Secrets are stored encrypted at rest
//! - Memory is zeroed when secrets are dropped (using zeroize)
//! - Secrets are never logged
//! - Support for multiple credential types

#[cfg(feature = "secrets")]
use keyring::Entry;
#[cfg(feature = "secrets")]
use zeroize::Zeroize;

use std::fmt;
use thiserror::Error;

/// The service name used for keyring entries.
const SERVICE_NAME: &str = "palrun";

/// Result type for secrets operations.
pub type SecretsResult<T> = Result<T, SecretsError>;

/// Errors that can occur during secrets operations.
#[derive(Debug, Error)]
pub enum SecretsError {
    /// Failed to access the system keychain.
    #[error("Failed to access system keychain: {0}")]
    KeychainAccess(String),

    /// Secret not found.
    #[error("Secret not found: {0}")]
    NotFound(String),

    /// Failed to store secret.
    #[error("Failed to store secret: {0}")]
    StoreFailed(String),

    /// Failed to delete secret.
    #[error("Failed to delete secret: {0}")]
    DeleteFailed(String),

    /// Invalid secret format.
    #[error("Invalid secret format: {0}")]
    InvalidFormat(String),

    /// Feature not available.
    #[error("Secrets feature not available - compile with 'secrets' feature")]
    FeatureNotAvailable,
}

/// A secret value that is zeroed on drop.
#[cfg(feature = "secrets")]
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecretValue {
    value: String,
}

#[cfg(not(feature = "secrets"))]
#[derive(Clone)]
pub struct SecretValue {
    value: String,
}

impl SecretValue {
    /// Create a new secret value.
    pub fn new(value: impl Into<String>) -> Self {
        Self { value: value.into() }
    }

    /// Get the secret value.
    ///
    /// Note: Use sparingly and ensure the value is not logged.
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// Get the length of the secret.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Check if the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

// Prevent accidental logging of secrets
impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue([REDACTED])")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Types of credentials that can be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialType {
    /// Claude API key.
    ClaudeApiKey,
    /// OpenAI API key.
    OpenAiApiKey,
    /// Ollama API key (if using remote).
    OllamaApiKey,
    /// GitHub token.
    GitHubToken,
    /// Custom credential with user-defined name.
    Custom,
}

impl CredentialType {
    /// Get the key name for this credential type.
    fn key_name(&self) -> &'static str {
        match self {
            Self::ClaudeApiKey => "claude_api_key",
            Self::OpenAiApiKey => "openai_api_key",
            Self::OllamaApiKey => "ollama_api_key",
            Self::GitHubToken => "github_token",
            Self::Custom => "custom",
        }
    }
}

/// Manages secrets storage and retrieval.
#[derive(Debug)]
pub struct SecretsManager {
    /// Service name for keyring entries.
    service: String,
}

impl Default for SecretsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretsManager {
    /// Create a new secrets manager.
    pub fn new() -> Self {
        Self { service: SERVICE_NAME.to_string() }
    }

    /// Create a secrets manager with a custom service name.
    pub fn with_service(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }

    /// Store a secret in the system keychain.
    #[cfg(feature = "secrets")]
    pub fn store(
        &self,
        credential_type: CredentialType,
        secret: &SecretValue,
    ) -> SecretsResult<()> {
        self.store_with_key(credential_type.key_name(), secret)
    }

    /// Store a secret in the system keychain.
    #[cfg(not(feature = "secrets"))]
    pub fn store(
        &self,
        _credential_type: CredentialType,
        _secret: &SecretValue,
    ) -> SecretsResult<()> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Store a secret with a custom key.
    #[cfg(feature = "secrets")]
    pub fn store_with_key(&self, key: &str, secret: &SecretValue) -> SecretsResult<()> {
        let entry = Entry::new(&self.service, key)
            .map_err(|e| SecretsError::KeychainAccess(e.to_string()))?;

        entry.set_password(secret.expose()).map_err(|e| SecretsError::StoreFailed(e.to_string()))
    }

    /// Store a secret with a custom key.
    #[cfg(not(feature = "secrets"))]
    pub fn store_with_key(&self, _key: &str, _secret: &SecretValue) -> SecretsResult<()> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Retrieve a secret from the system keychain.
    #[cfg(feature = "secrets")]
    pub fn retrieve(&self, credential_type: CredentialType) -> SecretsResult<SecretValue> {
        self.retrieve_with_key(credential_type.key_name())
    }

    /// Retrieve a secret from the system keychain.
    #[cfg(not(feature = "secrets"))]
    pub fn retrieve(&self, _credential_type: CredentialType) -> SecretsResult<SecretValue> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Retrieve a secret with a custom key.
    #[cfg(feature = "secrets")]
    pub fn retrieve_with_key(&self, key: &str) -> SecretsResult<SecretValue> {
        let entry = Entry::new(&self.service, key)
            .map_err(|e| SecretsError::KeychainAccess(e.to_string()))?;

        let password = entry.get_password().map_err(|e| {
            if e.to_string().contains("No matching entry") || e.to_string().contains("not found") {
                SecretsError::NotFound(key.to_string())
            } else {
                SecretsError::KeychainAccess(e.to_string())
            }
        })?;

        Ok(SecretValue::new(password))
    }

    /// Retrieve a secret with a custom key.
    #[cfg(not(feature = "secrets"))]
    pub fn retrieve_with_key(&self, _key: &str) -> SecretsResult<SecretValue> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Delete a secret from the system keychain.
    #[cfg(feature = "secrets")]
    pub fn delete(&self, credential_type: CredentialType) -> SecretsResult<()> {
        self.delete_with_key(credential_type.key_name())
    }

    /// Delete a secret from the system keychain.
    #[cfg(not(feature = "secrets"))]
    pub fn delete(&self, _credential_type: CredentialType) -> SecretsResult<()> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Delete a secret with a custom key.
    #[cfg(feature = "secrets")]
    pub fn delete_with_key(&self, key: &str) -> SecretsResult<()> {
        let entry = Entry::new(&self.service, key)
            .map_err(|e| SecretsError::KeychainAccess(e.to_string()))?;

        entry.delete_credential().map_err(|e| SecretsError::DeleteFailed(e.to_string()))
    }

    /// Delete a secret with a custom key.
    #[cfg(not(feature = "secrets"))]
    pub fn delete_with_key(&self, _key: &str) -> SecretsResult<()> {
        Err(SecretsError::FeatureNotAvailable)
    }

    /// Check if a secret exists in the keychain.
    #[cfg(feature = "secrets")]
    pub fn exists(&self, credential_type: CredentialType) -> bool {
        self.exists_with_key(credential_type.key_name())
    }

    /// Check if a secret exists in the keychain.
    #[cfg(not(feature = "secrets"))]
    pub fn exists(&self, _credential_type: CredentialType) -> bool {
        false
    }

    /// Check if a secret exists with a custom key.
    #[cfg(feature = "secrets")]
    pub fn exists_with_key(&self, key: &str) -> bool {
        self.retrieve_with_key(key).is_ok()
    }

    /// Check if a secret exists with a custom key.
    #[cfg(not(feature = "secrets"))]
    pub fn exists_with_key(&self, _key: &str) -> bool {
        false
    }

    /// Get a secret from environment variable or keychain.
    ///
    /// This is the recommended way to retrieve API keys as it allows
    /// for both environment variable overrides and keychain storage.
    #[cfg(feature = "secrets")]
    pub fn get_or_env(
        &self,
        credential_type: CredentialType,
        env_var: &str,
    ) -> SecretsResult<SecretValue> {
        // First try environment variable
        if let Ok(value) = std::env::var(env_var) {
            if !value.is_empty() {
                return Ok(SecretValue::new(value));
            }
        }

        // Then try keychain
        self.retrieve(credential_type)
    }

    /// Get a secret from environment variable or keychain.
    #[cfg(not(feature = "secrets"))]
    pub fn get_or_env(
        &self,
        _credential_type: CredentialType,
        env_var: &str,
    ) -> SecretsResult<SecretValue> {
        // Only environment variable available without the feature
        std::env::var(env_var)
            .map(SecretValue::new)
            .map_err(|_| SecretsError::NotFound(env_var.to_string()))
    }
}

/// Convenience functions for common API keys.
impl SecretsManager {
    /// Get Claude API key from environment or keychain.
    pub fn get_claude_api_key(&self) -> SecretsResult<SecretValue> {
        self.get_or_env(CredentialType::ClaudeApiKey, "ANTHROPIC_API_KEY")
    }

    /// Get OpenAI API key from environment or keychain.
    pub fn get_openai_api_key(&self) -> SecretsResult<SecretValue> {
        self.get_or_env(CredentialType::OpenAiApiKey, "OPENAI_API_KEY")
    }

    /// Get GitHub token from environment or keychain.
    pub fn get_github_token(&self) -> SecretsResult<SecretValue> {
        self.get_or_env(CredentialType::GitHubToken, "GITHUB_TOKEN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_value_redacted_debug() {
        let secret = SecretValue::new("super_secret_key");
        let debug_output = format!("{secret:?}");
        assert!(!debug_output.contains("super_secret_key"));
        assert!(debug_output.contains("REDACTED"));
    }

    #[test]
    fn test_secret_value_redacted_display() {
        let secret = SecretValue::new("super_secret_key");
        let display_output = format!("{secret}");
        assert!(!display_output.contains("super_secret_key"));
        assert!(display_output.contains("REDACTED"));
    }

    #[test]
    fn test_secret_value_expose() {
        let secret = SecretValue::new("my_api_key");
        assert_eq!(secret.expose(), "my_api_key");
    }

    #[test]
    fn test_secret_value_len() {
        let secret = SecretValue::new("12345");
        assert_eq!(secret.len(), 5);
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_credential_type_key_names() {
        assert_eq!(CredentialType::ClaudeApiKey.key_name(), "claude_api_key");
        assert_eq!(CredentialType::OpenAiApiKey.key_name(), "openai_api_key");
        assert_eq!(CredentialType::GitHubToken.key_name(), "github_token");
    }

    #[test]
    fn test_secrets_manager_creation() {
        let manager = SecretsManager::new();
        assert_eq!(manager.service, "palrun");
    }

    #[test]
    fn test_secrets_manager_custom_service() {
        let manager = SecretsManager::with_service("custom_service");
        assert_eq!(manager.service, "custom_service");
    }

    // Note: Integration tests that actually interact with the keychain
    // should be run manually or in a CI environment with appropriate setup.
    // They are commented out to avoid test failures on systems without keychain access.

    // #[test]
    // #[cfg(feature = "secrets")]
    // fn test_keychain_integration() {
    //     let manager = SecretsManager::with_service("palrun_test");
    //     let secret = SecretValue::new("test_secret_value");
    //
    //     // Store
    //     manager.store_with_key("test_key", &secret).unwrap();
    //
    //     // Retrieve
    //     let retrieved = manager.retrieve_with_key("test_key").unwrap();
    //     assert_eq!(retrieved.expose(), "test_secret_value");
    //
    //     // Delete
    //     manager.delete_with_key("test_key").unwrap();
    //
    //     // Verify deleted
    //     assert!(!manager.exists_with_key("test_key"));
    // }
}
