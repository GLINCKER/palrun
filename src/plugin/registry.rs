//! Plugin registry for discovering and installing plugins from remote sources.
//!
//! The registry is a JSON index hosted on GitHub that lists all available plugins
//! with their metadata, download URLs, and compatibility information.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{PluginError, PluginResult, PluginType, PLUGIN_API_VERSION};

/// Default registry URL (GitHub-hosted JSON index in the same repo)
pub const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/GLINCKER/palrun/main/plugins/registry.json";

/// Cache duration for registry data (1 hour)
const CACHE_DURATION_SECS: u64 = 3600;

/// A plugin entry in the remote registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPlugin {
    /// Plugin name (unique identifier)
    pub name: String,

    /// Display name
    #[serde(default)]
    pub display_name: Option<String>,

    /// Plugin description
    pub description: String,

    /// Plugin version
    pub version: String,

    /// Plugin author
    #[serde(default)]
    pub author: Option<String>,

    /// Plugin type
    #[serde(rename = "type")]
    pub plugin_type: PluginType,

    /// Minimum API version required
    pub api_version: String,

    /// Download URL for the plugin package
    pub download_url: String,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// License
    #[serde(default)]
    pub license: Option<String>,

    /// Tags for categorization and search
    #[serde(default)]
    pub tags: Vec<String>,

    /// Download count (for popularity sorting)
    #[serde(default)]
    pub downloads: u64,

    /// Star count (for popularity sorting)
    #[serde(default)]
    pub stars: u64,

    /// Last updated timestamp
    #[serde(default)]
    pub updated_at: Option<u64>,

    /// SHA256 checksum of the download
    #[serde(default)]
    pub checksum: Option<String>,
}

impl RegistryPlugin {
    /// Check if the plugin is compatible with the current API version.
    pub fn is_compatible(&self) -> bool {
        // Simple semver comparison - plugin API must match major version
        let parts: Vec<&str> = self.api_version.split('.').collect();
        let current_parts: Vec<&str> = PLUGIN_API_VERSION.split('.').collect();

        if parts.is_empty() || current_parts.is_empty() {
            return false;
        }

        // Major versions must match for compatibility
        parts[0] == current_parts[0]
    }

    /// Get the display name (or fallback to name).
    pub fn display(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// The remote plugin registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRegistry {
    /// Registry version
    pub version: String,

    /// Registry description
    #[serde(default)]
    pub description: Option<String>,

    /// Last updated timestamp
    pub updated_at: u64,

    /// Plugins in the registry
    pub plugins: Vec<RegistryPlugin>,
}

impl Default for RemoteRegistry {
    fn default() -> Self {
        Self { version: "1.0.0".to_string(), description: None, updated_at: 0, plugins: Vec::new() }
    }
}

impl RemoteRegistry {
    /// Search plugins by name, description, or tags.
    pub fn search(&self, query: &str) -> Vec<&RegistryPlugin> {
        let query_lower = query.to_lowercase();

        self.plugins
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    || p.display_name
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Get plugins by type.
    pub fn by_type(&self, plugin_type: PluginType) -> Vec<&RegistryPlugin> {
        self.plugins.iter().filter(|p| p.plugin_type == plugin_type).collect()
    }

    /// Get plugins sorted by popularity (downloads + stars).
    pub fn by_popularity(&self) -> Vec<&RegistryPlugin> {
        let mut plugins: Vec<_> = self.plugins.iter().collect();
        plugins.sort_by(|a, b| {
            let score_a = a.downloads + a.stars * 10;
            let score_b = b.downloads + b.stars * 10;
            score_b.cmp(&score_a)
        });
        plugins
    }

    /// Get only compatible plugins.
    pub fn compatible(&self) -> Vec<&RegistryPlugin> {
        self.plugins.iter().filter(|p| p.is_compatible()).collect()
    }

    /// Find a plugin by exact name.
    pub fn find(&self, name: &str) -> Option<&RegistryPlugin> {
        self.plugins.iter().find(|p| p.name == name)
    }
}

/// Cached registry data.
#[derive(Debug, Serialize, Deserialize)]
struct CachedRegistry {
    /// When the cache was last updated
    cached_at: u64,

    /// The registry data
    registry: RemoteRegistry,
}

/// Plugin registry client for fetching and caching registry data.
pub struct RegistryClient {
    /// Cache directory
    cache_dir: PathBuf,

    /// Registry URL
    registry_url: String,

    /// HTTP client
    client: reqwest::blocking::Client,

    /// Cached registry (in memory)
    cache: Option<RemoteRegistry>,
}

impl RegistryClient {
    /// Create a new registry client.
    pub fn new(cache_dir: PathBuf) -> PluginResult<Self> {
        std::fs::create_dir_all(&cache_dir)?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("palrun/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| PluginError::Network(e.to_string()))?;

        Ok(Self { cache_dir, registry_url: DEFAULT_REGISTRY_URL.to_string(), client, cache: None })
    }

    /// Create a client with a custom registry URL.
    pub fn with_url(cache_dir: PathBuf, url: &str) -> PluginResult<Self> {
        let mut client = Self::new(cache_dir)?;
        client.registry_url = url.to_string();
        Ok(client)
    }

    /// Get the path to the cache file.
    fn cache_path(&self) -> PathBuf {
        self.cache_dir.join("registry_cache.json")
    }

    /// Check if the cache is still valid.
    fn is_cache_valid(&self) -> bool {
        let cache_path = self.cache_path();

        if !cache_path.exists() {
            return false;
        }

        // Check cache age
        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<CachedRegistry>(&content) {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());

                return now - cached.cached_at < CACHE_DURATION_SECS;
            }
        }

        false
    }

    /// Load registry from cache.
    fn load_cache(&mut self) -> Option<RemoteRegistry> {
        let cache_path = self.cache_path();

        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<CachedRegistry>(&content) {
                self.cache = Some(cached.registry.clone());
                return Some(cached.registry);
            }
        }

        None
    }

    /// Save registry to cache.
    fn save_cache(&self, registry: &RemoteRegistry) -> PluginResult<()> {
        let cached = CachedRegistry {
            cached_at: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()),
            registry: registry.clone(),
        };

        let content = serde_json::to_string_pretty(&cached)
            .map_err(|e| PluginError::Config(e.to_string()))?;

        std::fs::write(self.cache_path(), content)?;

        Ok(())
    }

    /// Fetch the registry from the remote source.
    pub fn fetch(&mut self, force_refresh: bool) -> PluginResult<&RemoteRegistry> {
        // Check cache first (unless forced refresh)
        if !force_refresh && self.is_cache_valid() {
            if let Some(registry) = self.load_cache() {
                self.cache = Some(registry);
                return Ok(self.cache.as_ref().unwrap());
            }
        }

        // Fetch from remote
        let response = self
            .client
            .get(&self.registry_url)
            .send()
            .map_err(|e| PluginError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PluginError::Network(format!(
                "Failed to fetch registry: HTTP {}",
                response.status()
            )));
        }

        let registry: RemoteRegistry = response
            .json()
            .map_err(|e| PluginError::Network(format!("Invalid registry format: {e}")))?;

        // Save to cache
        self.save_cache(&registry)?;

        self.cache = Some(registry);
        Ok(self.cache.as_ref().unwrap())
    }

    /// Get the cached registry (without fetching).
    pub fn cached(&self) -> Option<&RemoteRegistry> {
        self.cache.as_ref()
    }

    /// Download a plugin from the registry.
    pub fn download(&self, plugin: &RegistryPlugin, dest_dir: &Path) -> PluginResult<PathBuf> {
        std::fs::create_dir_all(dest_dir)?;

        let response = self
            .client
            .get(&plugin.download_url)
            .send()
            .map_err(|e| PluginError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PluginError::Network(format!(
                "Failed to download plugin: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().map_err(|e| PluginError::Network(e.to_string()))?;

        // Verify checksum if available
        if let Some(ref expected) = plugin.checksum {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual = format!("{:x}", hasher.finalize());

            if &actual != expected {
                return Err(PluginError::Validation(format!(
                    "Checksum mismatch: expected {expected}, got {actual}"
                )));
            }
        }

        // Save to destination
        let filename = format!("{}.wasm", plugin.name);
        let dest_path = dest_dir.join(&filename);
        std::fs::write(&dest_path, bytes)?;

        Ok(dest_path)
    }

    /// Search the registry.
    pub fn search(&mut self, query: &str) -> PluginResult<Vec<&RegistryPlugin>> {
        let registry = self.fetch(false)?;
        Ok(registry.search(query))
    }

    /// Get plugins by type.
    pub fn by_type(&mut self, plugin_type: PluginType) -> PluginResult<Vec<&RegistryPlugin>> {
        let registry = self.fetch(false)?;
        Ok(registry.by_type(plugin_type))
    }

    /// Clear the cache.
    pub fn clear_cache(&self) -> PluginResult<()> {
        let cache_path = self.cache_path();
        if cache_path.exists() {
            std::fs::remove_file(cache_path)?;
        }
        Ok(())
    }
}

/// Search result with match score.
#[derive(Debug)]
pub struct SearchResult<'a> {
    /// The plugin
    pub plugin: &'a RegistryPlugin,

    /// Match score (higher is better)
    pub score: u32,
}

impl<'a> SearchResult<'a> {
    /// Create a new search result.
    pub fn new(plugin: &'a RegistryPlugin, score: u32) -> Self {
        Self { plugin, score }
    }
}

/// Calculate search score for a plugin.
fn calculate_score(plugin: &RegistryPlugin, query: &str) -> u32 {
    let query_lower = query.to_lowercase();
    let mut score = 0;

    // Exact name match
    if plugin.name.to_lowercase() == query_lower {
        score += 100;
    } else if plugin.name.to_lowercase().starts_with(&query_lower) {
        score += 50;
    } else if plugin.name.to_lowercase().contains(&query_lower) {
        score += 25;
    }

    // Description match
    if plugin.description.to_lowercase().contains(&query_lower) {
        score += 10;
    }

    // Tag match
    for tag in &plugin.tags {
        if tag.to_lowercase() == query_lower {
            score += 30;
        } else if tag.to_lowercase().contains(&query_lower) {
            score += 15;
        }
    }

    // Popularity bonus
    score += (plugin.downloads / 100) as u32;
    score += (plugin.stars * 2) as u32;

    score
}

/// Advanced search with scoring.
pub fn search_with_score<'a>(registry: &'a RemoteRegistry, query: &str) -> Vec<SearchResult<'a>> {
    let mut results: Vec<_> = registry
        .plugins
        .iter()
        .map(|p| SearchResult::new(p, calculate_score(p, query)))
        .filter(|r| r.score > 0)
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> RemoteRegistry {
        RemoteRegistry {
            version: "1.0.0".to_string(),
            description: Some("Test registry".to_string()),
            updated_at: 1704067200,
            plugins: vec![
                RegistryPlugin {
                    name: "gradle-scanner".to_string(),
                    display_name: Some("Gradle Scanner".to_string()),
                    description: "Scans Gradle projects for tasks".to_string(),
                    version: "0.1.0".to_string(),
                    author: Some("Palrun".to_string()),
                    plugin_type: PluginType::Scanner,
                    api_version: "0.1.0".to_string(),
                    download_url: "https://example.com/gradle-scanner.wasm".to_string(),
                    homepage: None,
                    repository: Some("https://github.com/palrun/gradle-scanner".to_string()),
                    license: Some("MIT".to_string()),
                    tags: vec!["java".to_string(), "gradle".to_string(), "build".to_string()],
                    downloads: 1000,
                    stars: 50,
                    updated_at: Some(1704067200),
                    checksum: None,
                },
                RegistryPlugin {
                    name: "maven-scanner".to_string(),
                    display_name: Some("Maven Scanner".to_string()),
                    description: "Scans Maven projects for goals".to_string(),
                    version: "0.1.0".to_string(),
                    author: Some("Community".to_string()),
                    plugin_type: PluginType::Scanner,
                    api_version: "0.1.0".to_string(),
                    download_url: "https://example.com/maven-scanner.wasm".to_string(),
                    homepage: None,
                    repository: None,
                    license: Some("MIT".to_string()),
                    tags: vec!["java".to_string(), "maven".to_string()],
                    downloads: 500,
                    stars: 25,
                    updated_at: None,
                    checksum: None,
                },
                RegistryPlugin {
                    name: "slack-notify".to_string(),
                    display_name: Some("Slack Notifications".to_string()),
                    description: "Send notifications to Slack".to_string(),
                    version: "0.2.0".to_string(),
                    author: Some("Palrun".to_string()),
                    plugin_type: PluginType::Integration,
                    api_version: "0.1.0".to_string(),
                    download_url: "https://example.com/slack-notify.wasm".to_string(),
                    homepage: None,
                    repository: None,
                    license: Some("MIT".to_string()),
                    tags: vec!["notifications".to_string(), "slack".to_string()],
                    downloads: 2000,
                    stars: 100,
                    updated_at: None,
                    checksum: None,
                },
            ],
        }
    }

    #[test]
    fn test_registry_search() {
        let registry = create_test_registry();

        // Search by name
        let results = registry.search("gradle");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "gradle-scanner");

        // Search by description
        let results = registry.search("maven");
        assert_eq!(results.len(), 1);

        // Search by tag
        let results = registry.search("java");
        assert_eq!(results.len(), 2);

        // Search by partial match
        let results = registry.search("scan");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_registry_by_type() {
        let registry = create_test_registry();

        let scanners = registry.by_type(PluginType::Scanner);
        assert_eq!(scanners.len(), 2);

        let integrations = registry.by_type(PluginType::Integration);
        assert_eq!(integrations.len(), 1);
    }

    #[test]
    fn test_registry_by_popularity() {
        let registry = create_test_registry();

        let popular = registry.by_popularity();
        // slack-notify should be first (2000 downloads + 100*10 stars = 3000)
        assert_eq!(popular[0].name, "slack-notify");
    }

    #[test]
    fn test_plugin_compatibility() {
        let plugin = RegistryPlugin {
            name: "test".to_string(),
            display_name: None,
            description: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            plugin_type: PluginType::Scanner,
            api_version: "0.1.0".to_string(),
            download_url: "".to_string(),
            homepage: None,
            repository: None,
            license: None,
            tags: vec![],
            downloads: 0,
            stars: 0,
            updated_at: None,
            checksum: None,
        };

        // Should be compatible with 0.x.x
        assert!(plugin.is_compatible());
    }

    #[test]
    fn test_registry_find() {
        let registry = create_test_registry();

        assert!(registry.find("gradle-scanner").is_some());
        assert!(registry.find("nonexistent").is_none());
    }

    #[test]
    fn test_search_with_score() {
        let registry = create_test_registry();

        let results = search_with_score(&registry, "gradle");
        assert!(!results.is_empty());

        // Exact match should score higher
        assert!(results[0].score > 0);
    }
}
