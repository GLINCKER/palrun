//! Security module for Palrun.
//!
//! Provides comprehensive security controls including:
//! - Command input validation and injection prevention
//! - Environment variable sanitization
//! - File permission verification
//! - Path traversal prevention
//!
//! # Security Philosophy
//!
//! Palrun follows a defense-in-depth approach:
//! 1. **Input Validation**: All user input is validated before processing
//! 2. **Output Sanitization**: Environment variables are sanitized before passing to child processes
//! 3. **Least Privilege**: File permissions are checked to ensure secure defaults
//! 4. **Fail-Safe Defaults**: When in doubt, deny access

mod permissions;
mod sanitization;
mod secrets;
mod validation;

pub use permissions::{
    FilePermissions, PermissionCheck, PermissionError, PermissionLevel, SecureFileChecker,
};
pub use sanitization::{
    EnvSanitizer, SanitizationOptions, SanitizationResult, SanitizedEnv, SensitivePattern,
};
pub use secrets::{CredentialType, SecretValue, SecretsError, SecretsManager, SecretsResult};
pub use validation::{
    CommandValidation, CommandValidator, InjectionPattern, ValidationError, ValidationResult,
    ValidationSeverity,
};

use std::path::Path;

/// Security configuration for Palrun.
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Whether to enable strict command validation
    pub strict_validation: bool,

    /// Whether to sanitize environment variables
    pub sanitize_env: bool,

    /// Whether to check file permissions
    pub check_permissions: bool,

    /// Whether to allow shell metacharacters in commands
    pub allow_shell_metacharacters: bool,

    /// Maximum command length (to prevent DoS)
    pub max_command_length: usize,

    /// Maximum number of environment variables
    pub max_env_vars: usize,

    /// Custom blocked patterns
    pub custom_blocked_patterns: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            sanitize_env: true,
            check_permissions: true,
            allow_shell_metacharacters: false,
            max_command_length: 10_000,
            max_env_vars: 1_000,
            custom_blocked_patterns: Vec::new(),
        }
    }
}

impl SecurityConfig {
    /// Create a permissive configuration (for trusted environments).
    pub fn permissive() -> Self {
        Self {
            strict_validation: false,
            sanitize_env: false,
            check_permissions: false,
            allow_shell_metacharacters: true,
            max_command_length: 100_000,
            max_env_vars: 10_000,
            custom_blocked_patterns: Vec::new(),
        }
    }

    /// Create a strict configuration (for untrusted environments).
    pub fn strict() -> Self {
        Self {
            strict_validation: true,
            sanitize_env: true,
            check_permissions: true,
            allow_shell_metacharacters: false,
            max_command_length: 5_000,
            max_env_vars: 500,
            custom_blocked_patterns: Vec::new(),
        }
    }

    /// Add a custom blocked pattern.
    #[must_use]
    pub fn with_blocked_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.custom_blocked_patterns.push(pattern.into());
        self
    }
}

/// Security manager that coordinates all security checks.
#[derive(Debug)]
pub struct SecurityManager {
    config: SecurityConfig,
    validator: CommandValidator,
    sanitizer: EnvSanitizer,
    permission_checker: SecureFileChecker,
}

impl SecurityManager {
    /// Create a new security manager with the given configuration.
    pub fn new(config: SecurityConfig) -> Self {
        let mut validator = CommandValidator::new();
        for pattern in &config.custom_blocked_patterns {
            validator = validator.with_blocked_pattern(pattern.clone());
        }

        Self {
            validator,
            sanitizer: EnvSanitizer::new(),
            permission_checker: SecureFileChecker::new(),
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SecurityConfig::default())
    }

    /// Validate a command before execution.
    pub fn validate_command(&self, command: &str) -> ValidationResult {
        if !self.config.strict_validation {
            return ValidationResult::ok();
        }

        // Check length
        if command.len() > self.config.max_command_length {
            return ValidationResult::error(
                ValidationError::CommandTooLong {
                    length: command.len(),
                    max: self.config.max_command_length,
                },
                ValidationSeverity::High,
            );
        }

        self.validator.validate(command)
    }

    /// Sanitize environment variables before passing to a child process.
    pub fn sanitize_env(
        &self,
        env: &[(String, String)],
    ) -> SanitizationResult<Vec<(String, String)>> {
        if !self.config.sanitize_env {
            return SanitizationResult::ok(env.to_vec());
        }

        // Check count
        if env.len() > self.config.max_env_vars {
            return SanitizationResult::warning(
                env.iter().take(self.config.max_env_vars).cloned().collect(),
                format!(
                    "Truncated environment variables from {} to {}",
                    env.len(),
                    self.config.max_env_vars
                ),
            );
        }

        let options = SanitizationOptions::default();
        let sanitized: Vec<(String, String)> = env
            .iter()
            .filter_map(|(k, v)| {
                let sanitized = self.sanitizer.sanitize_value(v, &options);
                Some((k.clone(), sanitized))
            })
            .collect();

        SanitizationResult::ok(sanitized)
    }

    /// Check if a file has secure permissions.
    pub fn check_file_permissions(&self, path: &Path) -> Result<PermissionCheck, PermissionError> {
        if !self.config.check_permissions {
            return Ok(PermissionCheck::skipped());
        }

        self.permission_checker.check(path)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SecurityConfig {
        &self.config
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.strict_validation);
        assert!(config.sanitize_env);
        assert!(config.check_permissions);
        assert!(!config.allow_shell_metacharacters);
    }

    #[test]
    fn test_security_config_permissive() {
        let config = SecurityConfig::permissive();
        assert!(!config.strict_validation);
        assert!(!config.sanitize_env);
        assert!(!config.check_permissions);
        assert!(config.allow_shell_metacharacters);
    }

    #[test]
    fn test_security_config_strict() {
        let config = SecurityConfig::strict();
        assert!(config.strict_validation);
        assert!(config.sanitize_env);
        assert!(config.check_permissions);
        assert_eq!(config.max_command_length, 5_000);
    }

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::with_defaults();
        assert!(manager.config().strict_validation);
    }

    #[test]
    fn test_validate_safe_command() {
        let manager = SecurityManager::with_defaults();
        let result = manager.validate_command("npm run build");
        assert!(result.is_safe());
    }

    #[test]
    fn test_validate_dangerous_command() {
        let manager = SecurityManager::with_defaults();
        let result = manager.validate_command("rm -rf /");
        assert!(!result.is_safe());
    }

    #[test]
    fn test_command_length_validation() {
        let config = SecurityConfig { max_command_length: 10, ..Default::default() };
        let manager = SecurityManager::new(config);
        let result = manager.validate_command("this is a very long command");
        assert!(!result.is_safe());
    }

    #[test]
    fn test_env_sanitization() {
        let manager = SecurityManager::with_defaults();
        let env = vec![
            ("PATH".to_string(), "/usr/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
        ];
        let result = manager.sanitize_env(&env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_permissive_skips_validation() {
        let manager = SecurityManager::new(SecurityConfig::permissive());
        let result = manager.validate_command("rm -rf /");
        assert!(result.is_safe()); // Skipped due to permissive config
    }
}
