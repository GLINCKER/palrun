//! Secrets management module.
//!
//! Provides integration with secret managers like 1Password, HashiCorp Vault,
//! and custom providers to securely inject secrets into environment variables.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

use anyhow::{Context, Result};

/// Secret provider types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretProvider {
    /// 1Password CLI (op command)
    OnePassword,
    /// HashiCorp Vault
    Vault,
    /// Custom command-based provider
    Custom(String),
}

impl SecretProvider {
    /// Get the display name for the provider.
    pub fn name(&self) -> &str {
        match self {
            SecretProvider::OnePassword => "1Password",
            SecretProvider::Vault => "HashiCorp Vault",
            SecretProvider::Custom(_) => "Custom",
        }
    }

    /// Get an icon for the provider.
    pub fn icon(&self) -> &str {
        match self {
            SecretProvider::OnePassword => "🔐",
            SecretProvider::Vault => "🗄️",
            SecretProvider::Custom(_) => "🔧",
        }
    }
}

/// A secret reference found in an environment file.
#[derive(Debug, Clone)]
pub struct SecretReference {
    /// The environment variable name
    pub variable: String,

    /// The secret reference (e.g., "op://vault/item/field")
    pub reference: String,

    /// The provider for this secret
    pub provider: SecretProvider,

    /// Source file where the reference was found
    pub source: PathBuf,
}

impl SecretReference {
    /// Parse the provider from a secret reference string.
    pub fn parse(variable: &str, reference: &str, source: &Path) -> Option<Self> {
        let reference = reference.trim();

        // 1Password: op://vault/item/field
        if reference.starts_with("op://") {
            return Some(Self {
                variable: variable.to_string(),
                reference: reference.to_string(),
                provider: SecretProvider::OnePassword,
                source: source.to_path_buf(),
            });
        }

        // Vault: vault://path/to/secret#field
        if reference.starts_with("vault://") {
            return Some(Self {
                variable: variable.to_string(),
                reference: reference.to_string(),
                provider: SecretProvider::Vault,
                source: source.to_path_buf(),
            });
        }

        // Custom: ${secret:key} or similar patterns could be added
        None
    }
}

/// Result of resolving a secret.
#[derive(Debug, Clone)]
pub struct ResolvedSecret {
    /// The environment variable name
    pub variable: String,

    /// The resolved secret value
    pub value: String,

    /// The provider that resolved this secret
    pub provider: SecretProvider,
}

/// Status of a secret provider.
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    /// The provider
    pub provider: SecretProvider,

    /// Whether the provider CLI is installed
    pub installed: bool,

    /// Whether the provider is authenticated/configured
    pub authenticated: bool,

    /// Version of the CLI (if available)
    pub version: Option<String>,

    /// Error message if not available
    pub error: Option<String>,
}

/// Secrets manager for detecting and resolving secret references.
pub struct SecretsManager {
    /// Project root directory
    root: PathBuf,

    /// Detected secret references
    references: Vec<SecretReference>,

    /// Provider statuses
    providers: HashMap<String, ProviderStatus>,
}

impl SecretsManager {
    /// Create a new secrets manager for the given project root.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            references: Vec::new(),
            providers: HashMap::new(),
        }
    }

    /// Check available secret providers.
    pub fn check_providers(&mut self) -> &HashMap<String, ProviderStatus> {
        self.providers.clear();

        // Check 1Password
        self.providers.insert(
            "1password".to_string(),
            Self::check_onepassword(),
        );

        // Check Vault
        self.providers.insert(
            "vault".to_string(),
            Self::check_vault(),
        );

        &self.providers
    }

    /// Check if 1Password CLI is available.
    fn check_onepassword() -> ProviderStatus {
        let output = Command::new("op").args(["--version"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();

                // Check if signed in by trying to list vaults
                let auth_check = Command::new("op")
                    .args(["vault", "list", "--format=json"])
                    .output();

                let authenticated = auth_check
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                ProviderStatus {
                    provider: SecretProvider::OnePassword,
                    installed: true,
                    authenticated,
                    version: Some(version),
                    error: if !authenticated {
                        Some("Not signed in. Run 'op signin' first.".to_string())
                    } else {
                        None
                    },
                }
            }
            Ok(_) => ProviderStatus {
                provider: SecretProvider::OnePassword,
                installed: false,
                authenticated: false,
                version: None,
                error: Some("1Password CLI not found. Install from https://1password.com/downloads/command-line/".to_string()),
            },
            Err(_) => ProviderStatus {
                provider: SecretProvider::OnePassword,
                installed: false,
                authenticated: false,
                version: None,
                error: Some("1Password CLI (op) not found in PATH".to_string()),
            },
        }
    }

    /// Check if HashiCorp Vault CLI is available.
    fn check_vault() -> ProviderStatus {
        let output = Command::new("vault").args(["version"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();

                // Check if authenticated by trying to get token status
                let auth_check = Command::new("vault")
                    .args(["token", "lookup"])
                    .output();

                let authenticated = auth_check
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                ProviderStatus {
                    provider: SecretProvider::Vault,
                    installed: true,
                    authenticated,
                    version: Some(version),
                    error: if !authenticated {
                        Some("Not authenticated. Run 'vault login' first.".to_string())
                    } else {
                        None
                    },
                }
            }
            Ok(_) => ProviderStatus {
                provider: SecretProvider::Vault,
                installed: false,
                authenticated: false,
                version: None,
                error: Some("Vault CLI not found. Install from https://www.vaultproject.io/downloads".to_string()),
            },
            Err(_) => ProviderStatus {
                provider: SecretProvider::Vault,
                installed: false,
                authenticated: false,
                version: None,
                error: Some("Vault CLI not found in PATH".to_string()),
            },
        }
    }

    /// Get provider status.
    pub fn get_provider_status(&self, provider: &str) -> Option<&ProviderStatus> {
        self.providers.get(provider)
    }

    /// Scan .env files for secret references.
    pub fn scan_references(&mut self) -> Result<&[SecretReference]> {
        self.references.clear();

        // Scan common .env files
        let env_patterns = [
            ".env",
            ".env.local",
            ".env.development",
            ".env.production",
            ".env.staging",
            ".env.test",
        ];

        for pattern in env_patterns {
            let path = self.root.join(pattern);
            if path.exists() {
                self.scan_file(&path)?;
            }
        }

        Ok(&self.references)
    }

    /// Scan a single file for secret references.
    fn scan_file(&mut self, path: &Path) -> Result<()> {
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
                let key = key.trim();
                let value = value.trim();

                // Check if value is a secret reference
                if let Some(reference) = SecretReference::parse(key, value, path) {
                    self.references.push(reference);
                }
            }
        }

        Ok(())
    }

    /// Get all detected secret references.
    pub fn get_references(&self) -> &[SecretReference] {
        &self.references
    }

    /// Get references for a specific provider.
    pub fn get_references_for_provider(&self, provider: &SecretProvider) -> Vec<&SecretReference> {
        self.references
            .iter()
            .filter(|r| &r.provider == provider)
            .collect()
    }

    /// Resolve a 1Password secret reference.
    pub fn resolve_onepassword(&self, reference: &str) -> Result<String> {
        // op read "op://vault/item/field"
        let output = Command::new("op")
            .args(["read", reference])
            .output()
            .context("Failed to execute 1Password CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("1Password error: {}", stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Resolve a Vault secret reference.
    pub fn resolve_vault(&self, reference: &str) -> Result<String> {
        // Parse vault://path/to/secret#field
        let path = reference
            .strip_prefix("vault://")
            .ok_or_else(|| anyhow::anyhow!("Invalid Vault reference"))?;

        let (secret_path, field) = if let Some((p, f)) = path.rsplit_once('#') {
            (p, Some(f))
        } else {
            (path, None)
        };

        // vault kv get -field=<field> <path>
        let mut args = vec!["kv", "get"];
        if let Some(f) = field {
            args.push("-field");
            args.push(f);
        }
        args.push(secret_path);

        let output = Command::new("vault")
            .args(&args)
            .output()
            .context("Failed to execute Vault CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Vault error: {}", stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Resolve a single secret reference.
    pub fn resolve_reference(&self, reference: &SecretReference) -> Result<ResolvedSecret> {
        let value = match &reference.provider {
            SecretProvider::OnePassword => self.resolve_onepassword(&reference.reference)?,
            SecretProvider::Vault => self.resolve_vault(&reference.reference)?,
            SecretProvider::Custom(cmd) => self.resolve_custom(cmd, &reference.reference)?,
        };

        Ok(ResolvedSecret {
            variable: reference.variable.clone(),
            value,
            provider: reference.provider.clone(),
        })
    }

    /// Resolve a custom secret using a command.
    fn resolve_custom(&self, command_template: &str, reference: &str) -> Result<String> {
        // Replace {reference} with the actual reference
        let command = command_template.replace("{reference}", reference);

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &command])
                .output()
        } else {
            Command::new("sh")
                .args(["-c", &command])
                .output()
        }
        .context("Failed to execute custom secret command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Custom provider error: {}", stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Resolve all secret references and return resolved secrets.
    pub fn resolve_all(&self) -> Vec<Result<ResolvedSecret>> {
        self.references
            .iter()
            .map(|r| self.resolve_reference(r))
            .collect()
    }

    /// Inject resolved secrets into environment variables.
    pub fn inject_secrets(&self, secrets: &[ResolvedSecret]) {
        for secret in secrets {
            std::env::set_var(&secret.variable, &secret.value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_parse_onepassword_reference() {
        let path = PathBuf::from(".env");
        let reference = SecretReference::parse(
            "API_KEY",
            "op://Private/API Keys/credential",
            &path,
        );

        assert!(reference.is_some());
        let ref_val = reference.unwrap();
        assert_eq!(ref_val.variable, "API_KEY");
        assert_eq!(ref_val.reference, "op://Private/API Keys/credential");
        assert_eq!(ref_val.provider, SecretProvider::OnePassword);
    }

    #[test]
    fn test_parse_vault_reference() {
        let path = PathBuf::from(".env");
        let reference = SecretReference::parse(
            "DB_PASSWORD",
            "vault://secret/data/database#password",
            &path,
        );

        assert!(reference.is_some());
        let ref_val = reference.unwrap();
        assert_eq!(ref_val.variable, "DB_PASSWORD");
        assert_eq!(ref_val.reference, "vault://secret/data/database#password");
        assert_eq!(ref_val.provider, SecretProvider::Vault);
    }

    #[test]
    fn test_parse_regular_value() {
        let path = PathBuf::from(".env");
        let reference = SecretReference::parse("PORT", "3000", &path);
        assert!(reference.is_none());
    }

    #[test]
    fn test_scan_references() {
        let temp = TempDir::new().unwrap();

        create_test_file(
            temp.path(),
            ".env",
            r#"
# Database config
DB_HOST=localhost
DB_PASSWORD=op://vault/database/password
API_KEY=vault://secret/api#key
PORT=3000
"#,
        );

        let mut manager = SecretsManager::new(temp.path());
        manager.scan_references().unwrap();

        let refs = manager.get_references();
        assert_eq!(refs.len(), 2);

        // Check 1Password reference
        let op_ref = refs.iter().find(|r| r.variable == "DB_PASSWORD").unwrap();
        assert_eq!(op_ref.provider, SecretProvider::OnePassword);

        // Check Vault reference
        let vault_ref = refs.iter().find(|r| r.variable == "API_KEY").unwrap();
        assert_eq!(vault_ref.provider, SecretProvider::Vault);
    }

    #[test]
    fn test_get_references_for_provider() {
        let temp = TempDir::new().unwrap();

        create_test_file(
            temp.path(),
            ".env",
            r#"
SECRET1=op://vault/item1/field
SECRET2=op://vault/item2/field
SECRET3=vault://path/secret#field
"#,
        );

        let mut manager = SecretsManager::new(temp.path());
        manager.scan_references().unwrap();

        let op_refs = manager.get_references_for_provider(&SecretProvider::OnePassword);
        assert_eq!(op_refs.len(), 2);

        let vault_refs = manager.get_references_for_provider(&SecretProvider::Vault);
        assert_eq!(vault_refs.len(), 1);
    }

    #[test]
    fn test_provider_name_and_icon() {
        assert_eq!(SecretProvider::OnePassword.name(), "1Password");
        assert_eq!(SecretProvider::OnePassword.icon(), "🔐");

        assert_eq!(SecretProvider::Vault.name(), "HashiCorp Vault");
        assert_eq!(SecretProvider::Vault.icon(), "🗄️");

        let custom = SecretProvider::Custom("my-tool".to_string());
        assert_eq!(custom.name(), "Custom");
        assert_eq!(custom.icon(), "🔧");
    }
}
