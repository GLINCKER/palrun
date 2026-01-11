//! Environment variable sanitization module.

use std::collections::HashSet;

/// Patterns that indicate sensitive environment variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensitivePattern {
    ApiKey,
    Token,
    Password,
    Secret,
    PrivateKey,
    Database,
    Custom(String),
}

impl SensitivePattern {
    pub fn patterns(&self) -> Vec<&str> {
        match self {
            Self::ApiKey => vec!["api_key", "apikey", "api-key"],
            Self::Token => vec!["token", "auth_token", "access_token", "bearer"],
            Self::Password => vec!["password", "passwd", "pwd"],
            Self::Secret => vec!["secret", "private"],
            Self::PrivateKey => vec!["private_key", "privatekey", "priv_key"],
            Self::Database => vec!["database_url", "db_password", "db_user"],
            Self::Custom(p) => vec![p.as_str()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SanitizationResult<T> {
    pub value: T,
    pub success: bool,
    pub warnings: Vec<String>,
}

impl<T> SanitizationResult<T> {
    pub fn ok(value: T) -> Self {
        Self { value, success: true, warnings: Vec::new() }
    }

    pub fn warning(value: T, warning: String) -> Self {
        Self { value, success: true, warnings: vec![warning] }
    }

    pub fn is_ok(&self) -> bool {
        self.success
    }
}

#[derive(Debug, Clone)]
pub struct SanitizationOptions {
    pub max_value_length: usize,
    pub redact_sensitive: bool,
    pub sensitive_patterns: Vec<String>,
    pub allowlist: HashSet<String>,
    pub blocklist: HashSet<String>,
}

impl Default for SanitizationOptions {
    fn default() -> Self {
        Self {
            max_value_length: 10_000,
            redact_sensitive: true,
            sensitive_patterns: Vec::new(),
            allowlist: HashSet::new(),
            blocklist: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedEnv {
    pub name: String,
    pub value: String,
    pub redacted: bool,
    pub truncated: bool,
}

#[derive(Debug, Default)]
pub struct EnvSanitizer {
    sensitive_names: Vec<String>,
}

impl EnvSanitizer {
    pub fn new() -> Self {
        Self {
            sensitive_names: vec![
                "password".into(),
                "passwd".into(),
                "pwd".into(),
                "secret".into(),
                "token".into(),
                "api_key".into(),
                "apikey".into(),
                "private".into(),
                "credential".into(),
                "auth".into(),
            ],
        }
    }

    pub fn add_sensitive_pattern(&mut self, pattern: impl Into<String>) {
        self.sensitive_names.push(pattern.into());
    }

    pub fn is_sensitive(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.sensitive_names.iter().any(|p| name_lower.contains(p))
    }

    pub fn sanitize(&self, name: &str, value: &str, options: &SanitizationOptions) -> SanitizedEnv {
        let mut sanitized_value = value.to_string();
        let mut redacted = false;
        let mut truncated = false;

        if options.blocklist.contains(name) {
            return SanitizedEnv {
                name: name.to_string(),
                value: "[BLOCKED]".to_string(),
                redacted: true,
                truncated: false,
            };
        }

        if options.redact_sensitive && !options.allowlist.contains(name) && self.is_sensitive(name)
        {
            sanitized_value = "[REDACTED]".to_string();
            redacted = true;
        }

        if sanitized_value.len() > options.max_value_length {
            sanitized_value.truncate(options.max_value_length);
            sanitized_value.push_str("...[TRUNCATED]");
            truncated = true;
        }

        SanitizedEnv { name: name.to_string(), value: sanitized_value, redacted, truncated }
    }

    pub fn sanitize_value(&self, value: &str, options: &SanitizationOptions) -> String {
        let mut result = value.to_string();
        if result.len() > options.max_value_length {
            result.truncate(options.max_value_length);
            result.push_str("...[TRUNCATED]");
        }
        result
    }

    pub fn sanitize_all(
        &self,
        env: &[(String, String)],
        options: &SanitizationOptions,
    ) -> Vec<SanitizedEnv> {
        env.iter().map(|(name, value)| self.sanitize(name, value, options)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_detection() {
        let sanitizer = EnvSanitizer::new();
        assert!(sanitizer.is_sensitive("DATABASE_PASSWORD"));
        assert!(sanitizer.is_sensitive("API_TOKEN"));
        assert!(!sanitizer.is_sensitive("PATH"));
    }

    #[test]
    fn test_redaction() {
        let sanitizer = EnvSanitizer::new();
        let options = SanitizationOptions::default();
        let result = sanitizer.sanitize("API_TOKEN", "secret", &options);
        assert!(result.redacted);
        assert_eq!(result.value, "[REDACTED]");
    }
}
