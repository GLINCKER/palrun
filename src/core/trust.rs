//! Directory trust management for Palrun.
//!
//! Implements a trust system similar to Claude Code where users must
//! explicitly trust directories before Palrun can execute commands.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Trust store for managing trusted directories.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustStore {
    /// Set of trusted directory paths (canonical paths)
    #[serde(default)]
    pub trusted_directories: HashSet<PathBuf>,

    /// Whether to skip trust check for home directory subpaths
    #[serde(default)]
    pub trust_home_subdirs: bool,
}

impl TrustStore {
    /// Load the trust store from the default location.
    ///
    /// Location: `~/.config/palrun/trust.json`
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::store_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let store: Self = serde_json::from_str(&content)?;
        Ok(store)
    }

    /// Save the trust store to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::store_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Get the path to the trust store file.
    fn store_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("palrun").join("trust.json"))
    }

    /// Check if a directory is trusted.
    ///
    /// A directory is trusted if:
    /// 1. It's in the trusted_directories set, OR
    /// 2. It's a parent of a trusted directory, OR
    /// 3. trust_home_subdirs is true and it's under the home directory
    pub fn is_trusted(&self, path: &Path) -> bool {
        // Canonicalize the path
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        // Check if exactly in trusted set
        if self.trusted_directories.contains(&canonical) {
            return true;
        }

        // Check if any trusted directory is a subdirectory of this path
        // (i.e., if we've trusted a child, trust the parent too)
        for trusted in &self.trusted_directories {
            if trusted.starts_with(&canonical) {
                return true;
            }
        }

        // Check home directory option
        if self.trust_home_subdirs {
            if let Some(home) = dirs::home_dir() {
                if canonical.starts_with(&home) {
                    return true;
                }
            }
        }

        false
    }

    /// Add a directory to the trusted set.
    pub fn trust_directory(&mut self, path: &Path) -> anyhow::Result<()> {
        let canonical = path.canonicalize()?;
        self.trusted_directories.insert(canonical);
        self.save()
    }

    /// Remove a directory from the trusted set.
    #[allow(dead_code)]
    pub fn untrust_directory(&mut self, path: &Path) -> anyhow::Result<()> {
        let canonical = path.canonicalize()?;
        self.trusted_directories.remove(&canonical);
        self.save()
    }

    /// Enable trusting all home subdirectories.
    #[allow(dead_code)]
    pub fn trust_all_home_subdirs(&mut self) -> anyhow::Result<()> {
        self.trust_home_subdirs = true;
        self.save()
    }
}

/// Trust confirmation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustDecision {
    /// User trusts the directory
    Trust,
    /// User declined to trust (exit)
    Decline,
}

/// Information about what trusting a directory means.
pub fn trust_warning_message(path: &Path) -> Vec<String> {
    vec![
        "Do you trust the files in this folder?".to_string(),
        String::new(),
        format!("  {}", path.display()),
        String::new(),
        "Palrun may read files and execute commands in this".to_string(),
        "directory. Only trust folders with code you trust.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_trust_store_default() {
        let store = TrustStore::default();
        assert!(store.trusted_directories.is_empty());
        assert!(!store.trust_home_subdirs);
    }

    #[test]
    fn test_trust_directory() {
        let temp = tempdir().unwrap();
        // Use canonical path to handle symlinks (e.g., /tmp -> /private/tmp on macOS)
        let path = temp.path().canonicalize().unwrap();

        let mut store = TrustStore::default();
        // Don't save to disk in test - use canonical path
        store.trusted_directories.insert(path.clone());

        assert!(store.is_trusted(&path));
    }

    #[test]
    fn test_untrusted_directory() {
        let temp = tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap();

        let store = TrustStore::default();
        assert!(!store.is_trusted(&path));
    }

    #[test]
    fn test_child_trusts_parent() {
        let temp = tempdir().unwrap();
        let parent = temp.path().canonicalize().unwrap();
        let child = parent.join("subdir");
        fs::create_dir(&child).unwrap();
        let child = child.canonicalize().unwrap();

        let mut store = TrustStore::default();
        store.trusted_directories.insert(child.clone());

        // Parent should be trusted if child is trusted
        assert!(store.is_trusted(&parent));
        assert!(store.is_trusted(&child));
    }

    #[test]
    fn test_serialization() {
        let mut store = TrustStore::default();
        store.trusted_directories.insert(PathBuf::from("/tmp/test"));
        store.trust_home_subdirs = true;

        let json = serde_json::to_string(&store).unwrap();
        let loaded: TrustStore = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.trusted_directories.len(), 1);
        assert!(loaded.trust_home_subdirs);
    }
}
