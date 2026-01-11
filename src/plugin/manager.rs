//! Plugin manager for installing, loading, and managing plugins.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    PluginError, PluginManifest, PluginResult, PluginRuntime, PluginType, PLUGIN_API_VERSION,
};

/// State of an installed plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginState {
    /// Plugin is enabled and active.
    Enabled,
    /// Plugin is disabled.
    Disabled,
    /// Plugin failed to load.
    Error,
}

impl Default for PluginState {
    fn default() -> Self {
        Self::Enabled
    }
}

/// An installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// Plugin manifest.
    pub manifest: PluginManifest,
    /// Path to the plugin WASM file.
    pub wasm_path: PathBuf,
    /// Plugin state.
    pub state: PluginState,
    /// Installation timestamp.
    pub installed_at: u64,
    /// Last error message (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// Plugin registry file format.
#[derive(Debug, Default, Serialize, Deserialize)]
struct PluginRegistry {
    plugins: HashMap<String, InstalledPlugin>,
}

/// Manages plugin installation, loading, and lifecycle.
pub struct PluginManager {
    /// Directory where plugins are stored.
    plugins_dir: PathBuf,
    /// Installed plugins.
    plugins: HashMap<String, InstalledPlugin>,
    /// Plugin runtimes (lazily initialized).
    #[allow(dead_code)]
    runtimes: HashMap<String, PluginRuntime>,
}

impl PluginManager {
    /// Create a new plugin manager.
    pub fn new(plugins_dir: PathBuf) -> PluginResult<Self> {
        // Ensure plugins directory exists
        std::fs::create_dir_all(&plugins_dir)?;

        let mut manager = Self { plugins_dir, plugins: HashMap::new(), runtimes: HashMap::new() };

        // Load plugin registry
        manager.load_registry()?;

        Ok(manager)
    }

    /// Get the plugins directory.
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Get the path to the registry file.
    fn registry_path(&self) -> PathBuf {
        self.plugins_dir.join("registry.json")
    }

    /// Load the plugin registry from disk.
    fn load_registry(&mut self) -> PluginResult<()> {
        let registry_path = self.registry_path();

        if registry_path.exists() {
            let content = std::fs::read_to_string(&registry_path)?;
            let registry: PluginRegistry =
                serde_json::from_str(&content).map_err(|e| PluginError::Config(e.to_string()))?;
            self.plugins = registry.plugins;
        }

        Ok(())
    }

    /// Save the plugin registry to disk.
    fn save_registry(&self) -> PluginResult<()> {
        let registry = PluginRegistry { plugins: self.plugins.clone() };

        let content = serde_json::to_string_pretty(&registry)
            .map_err(|e| PluginError::Config(e.to_string()))?;

        std::fs::write(self.registry_path(), content)?;

        Ok(())
    }

    /// Install a plugin from a local file.
    pub fn install_from_file(&mut self, path: &Path) -> PluginResult<String> {
        // Read the WASM file
        if !path.exists() {
            return Err(PluginError::NotFound(path.to_path_buf()));
        }

        // Look for manifest in the same directory or embedded
        let manifest_path = path.with_file_name("plugin.toml");
        let manifest = if manifest_path.exists() {
            PluginManifest::from_file(&manifest_path)?
        } else {
            return Err(PluginError::InvalidManifest(
                "plugin.toml not found alongside WASM file".to_string(),
            ));
        };

        // Validate manifest
        manifest.validate()?;

        // Check API compatibility
        if !manifest.is_compatible_with(PLUGIN_API_VERSION) {
            return Err(PluginError::IncompatibleVersion {
                name: manifest.plugin.name.clone(),
                required: manifest.plugin.api_version.clone(),
                available: PLUGIN_API_VERSION.to_string(),
            });
        }

        // Check if already installed
        if self.plugins.contains_key(&manifest.plugin.name) {
            return Err(PluginError::AlreadyInstalled(manifest.plugin.name.clone()));
        }

        // Copy WASM file to plugins directory
        let plugin_name = &manifest.plugin.name;
        let dest_dir = self.plugins_dir.join(plugin_name);
        std::fs::create_dir_all(&dest_dir)?;

        let dest_wasm = dest_dir.join(format!("{plugin_name}.wasm"));
        std::fs::copy(path, &dest_wasm)?;

        // Copy manifest
        let dest_manifest = dest_dir.join("plugin.toml");
        std::fs::write(&dest_manifest, manifest.to_toml()?)?;

        // Register plugin
        let installed = InstalledPlugin {
            manifest,
            wasm_path: dest_wasm,
            state: PluginState::Enabled,
            installed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            last_error: None,
        };

        let name = installed.manifest.plugin.name.clone();
        self.plugins.insert(name.clone(), installed);
        self.save_registry()?;

        Ok(name)
    }

    /// Uninstall a plugin.
    pub fn uninstall(&mut self, name: &str) -> PluginResult<()> {
        if !self.plugins.contains_key(name) {
            return Err(PluginError::NotFound(PathBuf::from(name)));
        }

        // Remove from registry
        self.plugins.remove(name);

        // Remove plugin directory
        let plugin_dir = self.plugins_dir.join(name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(plugin_dir)?;
        }

        // Remove runtime if loaded
        self.runtimes.remove(name);

        self.save_registry()?;

        Ok(())
    }

    /// Enable a plugin.
    pub fn enable(&mut self, name: &str) -> PluginResult<()> {
        let plugin =
            self.plugins.get_mut(name).ok_or_else(|| PluginError::NotFound(PathBuf::from(name)))?;

        plugin.state = PluginState::Enabled;
        plugin.last_error = None;
        self.save_registry()?;

        Ok(())
    }

    /// Disable a plugin.
    pub fn disable(&mut self, name: &str) -> PluginResult<()> {
        let plugin =
            self.plugins.get_mut(name).ok_or_else(|| PluginError::NotFound(PathBuf::from(name)))?;

        plugin.state = PluginState::Disabled;
        self.runtimes.remove(name);
        self.save_registry()?;

        Ok(())
    }

    /// Get an installed plugin by name.
    pub fn get(&self, name: &str) -> Option<&InstalledPlugin> {
        self.plugins.get(name)
    }

    /// List all installed plugins.
    pub fn list(&self) -> impl Iterator<Item = &InstalledPlugin> {
        self.plugins.values()
    }

    /// List plugins by type.
    pub fn list_by_type(&self, plugin_type: PluginType) -> impl Iterator<Item = &InstalledPlugin> {
        self.plugins.values().filter(move |p| p.manifest.plugin.plugin_type == plugin_type)
    }

    /// List enabled plugins.
    pub fn list_enabled(&self) -> impl Iterator<Item = &InstalledPlugin> {
        self.plugins.values().filter(|p| p.state == PluginState::Enabled)
    }

    /// Get the number of installed plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Get the number of enabled plugins.
    pub fn count_enabled(&self) -> usize {
        self.plugins.values().filter(|p| p.state == PluginState::Enabled).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_plugin(dir: &Path, name: &str) -> PathBuf {
        let plugin_dir = dir.join(name);
        std::fs::create_dir_all(&plugin_dir).unwrap();

        // Create a minimal manifest
        let manifest = format!(
            r#"
[plugin]
name = "{name}"
version = "0.1.0"
type = "scanner"
api_version = "0.1.0"
"#
        );

        let manifest_path = plugin_dir.join("plugin.toml");
        std::fs::write(&manifest_path, manifest).unwrap();

        // Create a dummy WASM file (not valid WASM, just for testing)
        let wasm_path = plugin_dir.join(format!("{name}.wasm"));
        std::fs::write(&wasm_path, b"dummy wasm").unwrap();

        wasm_path
    }

    #[test]
    fn test_plugin_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_install_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "test-plugin");

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
        let name = manager.install_from_file(&wasm_path).unwrap();

        assert_eq!(name, "test-plugin");
        assert_eq!(manager.count(), 1);
        assert!(manager.get("test-plugin").is_some());
    }

    #[test]
    fn test_uninstall() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "test-plugin");

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.install_from_file(&wasm_path).unwrap();
        assert_eq!(manager.count(), 1);

        manager.uninstall("test-plugin").unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "test-plugin");

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.install_from_file(&wasm_path).unwrap();

        // Initially enabled
        assert_eq!(manager.get("test-plugin").unwrap().state, PluginState::Enabled);
        assert_eq!(manager.count_enabled(), 1);

        // Disable
        manager.disable("test-plugin").unwrap();
        assert_eq!(manager.get("test-plugin").unwrap().state, PluginState::Disabled);
        assert_eq!(manager.count_enabled(), 0);

        // Re-enable
        manager.enable("test-plugin").unwrap();
        assert_eq!(manager.get("test-plugin").unwrap().state, PluginState::Enabled);
    }

    #[test]
    fn test_already_installed() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "test-plugin");

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.install_from_file(&wasm_path).unwrap();

        // Try to install again
        let result = manager.install_from_file(&wasm_path);
        assert!(matches!(result, Err(PluginError::AlreadyInstalled(_))));
    }

    #[test]
    fn test_list_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "scanner-plugin");

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
        manager.install_from_file(&wasm_path).unwrap();

        let scanners: Vec<_> = manager.list_by_type(PluginType::Scanner).collect();
        assert_eq!(scanners.len(), 1);

        let ai_providers: Vec<_> = manager.list_by_type(PluginType::AiProvider).collect();
        assert_eq!(ai_providers.len(), 0);
    }

    #[test]
    fn test_registry_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = TempDir::new().unwrap();

        let wasm_path = create_test_plugin(source_dir.path(), "test-plugin");

        // Install plugin
        {
            let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
            manager.install_from_file(&wasm_path).unwrap();
        }

        // Create new manager and verify plugin is still there
        {
            let manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();
            assert_eq!(manager.count(), 1);
            assert!(manager.get("test-plugin").is_some());
        }
    }
}
