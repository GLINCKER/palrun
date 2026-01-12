//! Configuration management for Palrun.
//!
//! Handles loading and saving configuration from TOML files.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,

    /// UI/TUI settings
    pub ui: UiConfig,

    /// Scanner settings
    pub scanner: ScannerConfig,

    /// AI settings
    #[cfg(feature = "ai")]
    pub ai: AiConfig,

    /// Keybinding overrides
    pub keys: KeyConfig,

    /// Git hooks configuration
    #[cfg(feature = "git")]
    pub hooks: HooksConfig,

    /// Command aliases
    #[serde(default)]
    pub aliases: Vec<AliasConfig>,

    /// MCP (Model Context Protocol) configuration
    #[serde(default)]
    pub mcp: MCPConfig,
}

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Whether to show hidden commands
    pub show_hidden: bool,

    /// Whether to confirm before executing dangerous commands
    pub confirm_dangerous: bool,

    /// Maximum number of history entries to keep
    pub max_history: usize,

    /// Default shell to use for command execution
    pub shell: Option<String>,
}

/// UI/TUI settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Color theme name (built-in: default, dracula, nord, solarized-dark, etc.)
    pub theme: String,

    /// Whether to show command preview
    pub show_preview: bool,

    /// Whether to show command source icons
    pub show_icons: bool,

    /// Maximum number of commands to display
    pub max_display: usize,

    /// Whether to enable mouse support
    pub mouse: bool,

    /// Custom theme color overrides (hex format: "#RRGGBB")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_colors: Option<CustomColorsConfig>,
}

/// Custom color configuration for theme overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomColorsConfig {
    /// Primary accent color (headers, selected items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    /// Secondary accent color (command prompts, success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
    /// Tertiary accent color (highlights, warnings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// Highlight color for search matches
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
    /// Main text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Dimmed text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_dim: Option<String>,
    /// Muted text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_muted: Option<String>,
    /// Background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// Selected item background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_bg: Option<String>,
    /// Border color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    /// Success color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<String>,
    /// Warning color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    /// Error color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Scanner settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScannerConfig {
    /// Enabled scanners
    pub enabled: Vec<String>,

    /// Directories to ignore when scanning
    pub ignore_dirs: Vec<String>,

    /// Maximum scan depth for monorepos
    pub max_depth: usize,

    /// Whether to scan recursively
    pub recursive: bool,
}

/// AI integration settings.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Whether AI features are enabled
    pub enabled: bool,

    /// Default AI provider (claude, ollama, openai, azure, grok)
    pub provider: String,

    /// Model to use (overrides provider-specific model)
    pub model: Option<String>,

    /// Enable automatic fallback if primary provider fails
    #[serde(default = "default_true")]
    pub fallback_enabled: bool,

    /// Fallback order
    #[serde(default)]
    pub fallback_chain: Vec<String>,

    /// Ollama-specific settings
    #[serde(default)]
    pub ollama: OllamaConfig,

    /// Claude-specific settings
    #[serde(default)]
    pub claude: ClaudeConfig,

    /// OpenAI-specific settings
    #[serde(default)]
    pub openai: OpenAIConfig,

    /// Azure OpenAI-specific settings
    #[serde(default)]
    pub azure: AzureOpenAIConfig,

    /// Grok-specific settings
    #[serde(default)]
    pub grok: GrokConfig,
}

#[cfg(feature = "ai")]
fn default_true() -> bool {
    true
}

/// Ollama configuration.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OllamaConfig {
    /// Ollama server URL
    pub base_url: String,

    /// Model to use
    pub model: String,
}

/// Claude (Anthropic) configuration.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeConfig {
    /// API key (prefer env var ANTHROPIC_API_KEY)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Model to use
    #[serde(default = "default_claude_model")]
    pub model: String,
}

#[cfg(feature = "ai")]
fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

/// OpenAI configuration.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenAIConfig {
    /// API key (prefer env var OPENAI_API_KEY)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Model to use
    #[serde(default = "default_openai_model")]
    pub model: String,

    /// Base URL (for OpenAI-compatible APIs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[cfg(feature = "ai")]
fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

/// Azure OpenAI configuration.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AzureOpenAIConfig {
    /// Azure OpenAI endpoint (e.g., https://your-resource.openai.azure.com)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// API key (prefer env var AZURE_OPENAI_API_KEY)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Deployment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,

    /// API version
    #[serde(default = "default_azure_api_version")]
    pub api_version: String,
}

#[cfg(feature = "ai")]
fn default_azure_api_version() -> String {
    "2024-02-01".to_string()
}

/// Grok (xAI) configuration.
#[cfg(feature = "ai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrokConfig {
    /// API key (prefer env var XAI_API_KEY)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Model to use
    #[serde(default = "default_grok_model")]
    pub model: String,
}

#[cfg(feature = "ai")]
fn default_grok_model() -> String {
    "grok-beta".to_string()
}

/// Keybinding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyConfig {
    /// Key to quit (default: "esc" or "q")
    pub quit: String,

    /// Key to execute/select (default: "enter")
    pub select: String,

    /// Key to move up (default: "up")
    pub up: String,

    /// Key to move down (default: "down")
    pub down: String,

    /// Key to clear input (default: "ctrl+u")
    pub clear: String,

    /// Key to toggle favorite (default: "ctrl+s")
    pub favorite: String,

    /// Key to run in background (default: "ctrl+b")
    pub background: String,

    /// Key to toggle multi-select mode (default: "ctrl+space")
    pub multi_select: String,

    /// Key to show help (default: "?")
    pub help: String,

    /// Key to page up (default: "pageup")
    pub page_up: String,

    /// Key to page down (default: "pagedown")
    pub page_down: String,

    /// Key to toggle AI mode
    #[cfg(feature = "ai")]
    pub ai_mode: String,
}

/// Git hooks configuration.
#[cfg(feature = "git")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HooksConfig {
    /// Pre-commit hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<String>,

    /// Prepare-commit-msg hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_commit_msg: Option<String>,

    /// Commit-msg hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_msg: Option<String>,

    /// Post-commit hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_commit: Option<String>,

    /// Pre-rebase hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_rebase: Option<String>,

    /// Post-checkout hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_checkout: Option<String>,

    /// Post-merge hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_merge: Option<String>,

    /// Pre-push hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_push: Option<String>,

    /// Pre-auto-gc hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_auto_gc: Option<String>,

    /// Post-rewrite hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_rewrite: Option<String>,
}

#[cfg(feature = "git")]
impl HooksConfig {
    /// Get all configured hooks as (hook_name, command) pairs.
    pub fn get_configured_hooks(&self) -> Vec<(String, String)> {
        let mut hooks = Vec::new();

        if let Some(ref cmd) = self.pre_commit {
            hooks.push(("pre-commit".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.prepare_commit_msg {
            hooks.push(("prepare-commit-msg".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.commit_msg {
            hooks.push(("commit-msg".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.post_commit {
            hooks.push(("post-commit".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.pre_rebase {
            hooks.push(("pre-rebase".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.post_checkout {
            hooks.push(("post-checkout".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.post_merge {
            hooks.push(("post-merge".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.pre_push {
            hooks.push(("pre-push".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.pre_auto_gc {
            hooks.push(("pre-auto-gc".to_string(), cmd.clone()));
        }
        if let Some(ref cmd) = self.post_rewrite {
            hooks.push(("post-rewrite".to_string(), cmd.clone()));
        }

        hooks
    }

    /// Check if any hooks are configured.
    pub fn has_configured_hooks(&self) -> bool {
        self.pre_commit.is_some()
            || self.prepare_commit_msg.is_some()
            || self.commit_msg.is_some()
            || self.post_commit.is_some()
            || self.pre_rebase.is_some()
            || self.post_checkout.is_some()
            || self.post_merge.is_some()
            || self.pre_push.is_some()
            || self.pre_auto_gc.is_some()
            || self.post_rewrite.is_some()
    }
}

impl Config {
    /// Load configuration with hierarchical merging.
    ///
    /// Loading order (later overrides earlier):
    /// 1. Defaults
    /// 2. `~/.config/palrun/palrun.toml` (system - can have secrets)
    /// 3. `palrun.toml` in current directory (project - NO secrets)
    /// 4. `.palrun.local.toml` in current directory (local - can have secrets, gitignored)
    /// 5. Environment variables (highest priority)
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        // 1. Load system config (can have secrets)
        if let Some(config_dir) = dirs::config_dir() {
            let system_config = config_dir.join("palrun").join("palrun.toml");
            if system_config.exists() {
                if let Ok(system) = Self::load_from_file(&system_config) {
                    config = config.merge(system);
                    tracing::debug!("Loaded system config from {}", system_config.display());
                }
            }
            // Also check legacy path
            let legacy_config = config_dir.join("palrun").join("config.toml");
            if legacy_config.exists() && !system_config.exists() {
                if let Ok(legacy) = Self::load_from_file(&legacy_config) {
                    config = config.merge(legacy);
                    tracing::debug!("Loaded legacy config from {}", legacy_config.display());
                }
            }
        }

        // 2. Load project config (NO secrets - may be committed)
        let project_config = PathBuf::from("palrun.toml");
        if project_config.exists() {
            if let Ok(project) = Self::load_from_file(&project_config) {
                config = config.merge(project);
                tracing::debug!("Loaded project config from palrun.toml");
            }
        }
        // Also check .palrun.toml (legacy project config)
        let legacy_project = PathBuf::from(".palrun.toml");
        if legacy_project.exists() {
            if let Ok(legacy) = Self::load_from_file(&legacy_project) {
                config = config.merge(legacy);
                tracing::debug!("Loaded legacy project config from .palrun.toml");
            }
        }

        // 3. Load local config (can have secrets - gitignored)
        let local_config = PathBuf::from(".palrun.local.toml");
        if local_config.exists() {
            if let Ok(local) = Self::load_from_file(&local_config) {
                config = config.merge(local);
                tracing::debug!("Loaded local config from .palrun.local.toml");
            }
        }

        // 4. Apply environment variable overrides
        #[cfg(feature = "ai")]
        {
            config = config.apply_env_overrides();
        }

        Ok(config)
    }

    /// Load configuration from a specific file.
    pub fn load_from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Merge another config into this one (other takes precedence).
    pub fn merge(mut self, other: Self) -> Self {
        // General - use other's values if they differ from default
        if other.general.show_hidden {
            self.general.show_hidden = true;
        }
        if !other.general.confirm_dangerous {
            self.general.confirm_dangerous = false;
        }
        if other.general.max_history != 1000 {
            self.general.max_history = other.general.max_history;
        }
        if other.general.shell.is_some() {
            self.general.shell = other.general.shell;
        }

        // UI
        if other.ui.theme != "default" {
            self.ui.theme = other.ui.theme;
        }
        if !other.ui.show_preview {
            self.ui.show_preview = false;
        }
        if !other.ui.show_icons {
            self.ui.show_icons = false;
        }
        if other.ui.max_display != 50 {
            self.ui.max_display = other.ui.max_display;
        }
        if !other.ui.mouse {
            self.ui.mouse = false;
        }
        if other.ui.custom_colors.is_some() {
            self.ui.custom_colors = other.ui.custom_colors;
        }

        // Scanner
        if !other.scanner.enabled.is_empty() {
            self.scanner.enabled = other.scanner.enabled;
        }
        if !other.scanner.ignore_dirs.is_empty() {
            self.scanner.ignore_dirs = other.scanner.ignore_dirs;
        }

        // AI config
        #[cfg(feature = "ai")]
        {
            self.ai = self.ai.merge(other.ai);
        }

        // Keys - use other if different from default
        let default_keys = KeyConfig::default();
        if other.keys.quit != default_keys.quit {
            self.keys.quit = other.keys.quit;
        }
        if other.keys.select != default_keys.select {
            self.keys.select = other.keys.select;
        }

        // Aliases - append
        if !other.aliases.is_empty() {
            self.aliases.extend(other.aliases);
        }

        // MCP
        if other.mcp.enabled {
            self.mcp.enabled = true;
        }
        if !other.mcp.servers.is_empty() {
            self.mcp.servers.extend(other.mcp.servers);
        }

        // Hooks
        #[cfg(feature = "git")]
        {
            if other.hooks.pre_commit.is_some() {
                self.hooks.pre_commit = other.hooks.pre_commit;
            }
            if other.hooks.commit_msg.is_some() {
                self.hooks.commit_msg = other.hooks.commit_msg;
            }
            // ... other hooks follow the same pattern
        }

        self
    }

    /// Apply environment variable overrides to AI config.
    #[cfg(feature = "ai")]
    fn apply_env_overrides(mut self) -> Self {
        // Claude
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            self.ai.claude.api_key = Some(key);
        }

        // OpenAI
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            self.ai.openai.api_key = Some(key);
        }

        // Azure OpenAI
        if let Ok(key) = std::env::var("AZURE_OPENAI_API_KEY") {
            self.ai.azure.api_key = Some(key);
        }
        if let Ok(endpoint) = std::env::var("AZURE_OPENAI_ENDPOINT") {
            self.ai.azure.endpoint = Some(endpoint);
        }
        if let Ok(deployment) = std::env::var("AZURE_OPENAI_DEPLOYMENT") {
            self.ai.azure.deployment = Some(deployment);
        }

        // Grok
        if let Ok(key) = std::env::var("XAI_API_KEY") {
            self.ai.grok.api_key = Some(key);
        }

        // Ollama
        if let Ok(url) = std::env::var("OLLAMA_HOST") {
            self.ai.ollama.base_url = url;
        }

        self
    }

    /// Save configuration to the global config file.
    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        let palrun_dir = config_dir.join("palrun");
        std::fs::create_dir_all(&palrun_dir)?;

        let config_path = palrun_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;

        Ok(())
    }

    /// Get the config directory path.
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("palrun"))
    }

    /// Get the data directory path (for history, cache, etc.).
    pub fn data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("palrun"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            scanner: ScannerConfig::default(),
            #[cfg(feature = "ai")]
            ai: AiConfig::default(),
            keys: KeyConfig::default(),
            #[cfg(feature = "git")]
            hooks: HooksConfig::default(),
            aliases: Vec::new(),
            mcp: MCPConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { show_hidden: false, confirm_dangerous: true, max_history: 1000, shell: None }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_preview: true,
            show_icons: true,
            max_display: 50,
            mouse: true,
            custom_colors: None,
        }
    }
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "npm".to_string(),
                "nx".to_string(),
                "turbo".to_string(),
                "make".to_string(),
                "cargo".to_string(),
                "docker".to_string(),
            ],
            ignore_dirs: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
            max_depth: 5,
            recursive: true,
        }
    }
}

#[cfg(feature = "ai")]
impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "claude".to_string(),
            model: None,
            fallback_enabled: true,
            fallback_chain: vec![
                "claude".to_string(),
                "openai".to_string(),
                "azure".to_string(),
                "grok".to_string(),
                "ollama".to_string(),
            ],
            ollama: OllamaConfig::default(),
            claude: ClaudeConfig::default(),
            openai: OpenAIConfig::default(),
            azure: AzureOpenAIConfig::default(),
            grok: GrokConfig::default(),
        }
    }
}

#[cfg(feature = "ai")]
impl Default for ClaudeConfig {
    fn default() -> Self {
        Self { api_key: None, model: default_claude_model() }
    }
}

#[cfg(feature = "ai")]
impl Default for OpenAIConfig {
    fn default() -> Self {
        Self { api_key: None, model: default_openai_model(), base_url: None }
    }
}

#[cfg(feature = "ai")]
impl Default for AzureOpenAIConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            api_key: None,
            deployment: None,
            api_version: default_azure_api_version(),
        }
    }
}

#[cfg(feature = "ai")]
impl Default for GrokConfig {
    fn default() -> Self {
        Self { api_key: None, model: default_grok_model() }
    }
}

#[cfg(feature = "ai")]
impl AiConfig {
    /// Merge another AI config into this one (other takes precedence for non-None values).
    pub fn merge(mut self, other: Self) -> Self {
        // Basic settings
        if !other.enabled {
            self.enabled = false;
        }
        if other.provider != "claude" {
            self.provider = other.provider;
        }
        if other.model.is_some() {
            self.model = other.model;
        }
        if !other.fallback_enabled {
            self.fallback_enabled = false;
        }
        if !other.fallback_chain.is_empty() {
            self.fallback_chain = other.fallback_chain;
        }

        // Ollama
        if other.ollama.base_url != "http://localhost:11434" {
            self.ollama.base_url = other.ollama.base_url;
        }
        if other.ollama.model != "codellama:7b" {
            self.ollama.model = other.ollama.model;
        }

        // Claude
        if other.claude.api_key.is_some() {
            self.claude.api_key = other.claude.api_key;
        }
        if other.claude.model != default_claude_model() {
            self.claude.model = other.claude.model;
        }

        // OpenAI
        if other.openai.api_key.is_some() {
            self.openai.api_key = other.openai.api_key;
        }
        if other.openai.model != default_openai_model() {
            self.openai.model = other.openai.model;
        }
        if other.openai.base_url.is_some() {
            self.openai.base_url = other.openai.base_url;
        }

        // Azure
        if other.azure.endpoint.is_some() {
            self.azure.endpoint = other.azure.endpoint;
        }
        if other.azure.api_key.is_some() {
            self.azure.api_key = other.azure.api_key;
        }
        if other.azure.deployment.is_some() {
            self.azure.deployment = other.azure.deployment;
        }
        if other.azure.api_version != default_azure_api_version() {
            self.azure.api_version = other.azure.api_version;
        }

        // Grok
        if other.grok.api_key.is_some() {
            self.grok.api_key = other.grok.api_key;
        }
        if other.grok.model != default_grok_model() {
            self.grok.model = other.grok.model;
        }

        self
    }

    /// Check if a provider has credentials configured.
    pub fn has_credentials(&self, provider: &str) -> bool {
        match provider {
            "claude" => self.claude.api_key.is_some(),
            "openai" => self.openai.api_key.is_some(),
            "azure" => {
                self.azure.api_key.is_some()
                    && self.azure.endpoint.is_some()
                    && self.azure.deployment.is_some()
            }
            "grok" => self.grok.api_key.is_some(),
            "ollama" => true, // Ollama doesn't need credentials
            _ => false,
        }
    }

    /// Get the API key for a provider (from config, not env).
    pub fn get_api_key(&self, provider: &str) -> Option<&str> {
        match provider {
            "claude" => self.claude.api_key.as_deref(),
            "openai" => self.openai.api_key.as_deref(),
            "azure" => self.azure.api_key.as_deref(),
            "grok" => self.grok.api_key.as_deref(),
            _ => None,
        }
    }
}

#[cfg(feature = "ai")]
impl Default for OllamaConfig {
    fn default() -> Self {
        Self { base_url: "http://localhost:11434".to_string(), model: "codellama:7b".to_string() }
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            quit: "esc".to_string(),
            select: "enter".to_string(),
            up: "up".to_string(),
            down: "down".to_string(),
            clear: "ctrl+u".to_string(),
            favorite: "ctrl+s".to_string(),
            background: "ctrl+b".to_string(),
            multi_select: "ctrl+space".to_string(),
            help: "?".to_string(),
            page_up: "pageup".to_string(),
            page_down: "pagedown".to_string(),
            #[cfg(feature = "ai")]
            ai_mode: "/".to_string(),
        }
    }
}

/// Command alias configuration.
///
/// Allows users to define shortcuts for frequently used commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfig {
    /// Short name for the alias (used in command palette)
    pub name: String,

    /// The actual command to execute
    pub command: String,

    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Whether to confirm before running
    #[serde(default)]
    pub confirm: bool,

    /// Working directory (relative to project root)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<PathBuf>,

    /// Environment variables to set
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<(String, String)>,

    /// Branch patterns this alias is available on
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<String>,
}

impl AliasConfig {
    /// Create a new alias with the minimum required fields.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            description: None,
            tags: Vec::new(),
            confirm: false,
            working_dir: None,
            env: Vec::new(),
            branches: Vec::new(),
        }
    }
}

/// MCP (Model Context Protocol) configuration.
///
/// Configures connections to MCP servers for dynamic tool discovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MCPConfig {
    /// Whether MCP is enabled
    pub enabled: bool,

    /// MCP servers to connect to
    #[serde(default)]
    pub servers: Vec<MCPServerEntry>,
}

/// Configuration for a single MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerEntry {
    /// Server name (unique identifier)
    pub name: String,

    /// Command to run the server
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Working directory (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

impl MCPServerEntry {
    /// Create a new MCP server entry.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            cwd: None,
        }
    }

    /// Add arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add environment variables.
    pub fn with_env(mut self, env: std::collections::HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.general.show_hidden);
        assert!(config.general.confirm_dangerous);
        assert_eq!(config.ui.theme, "default");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[ui]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [general]
            show_hidden = true
            max_history = 500

            [ui]
            theme = "dark"
            show_preview = false
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.general.show_hidden);
        assert_eq!(config.general.max_history, 500);
        assert_eq!(config.ui.theme, "dark");
        assert!(!config.ui.show_preview);
    }

    #[test]
    fn test_alias_config_creation() {
        let alias = AliasConfig::new("deploy-dev", "npm run build && npm run deploy:dev");
        assert_eq!(alias.name, "deploy-dev");
        assert_eq!(alias.command, "npm run build && npm run deploy:dev");
        assert!(alias.description.is_none());
        assert!(alias.tags.is_empty());
        assert!(!alias.confirm);
    }

    #[test]
    fn test_alias_config_deserialization() {
        let toml_str = r#"
            [[aliases]]
            name = "deploy"
            command = "npm run deploy"
            description = "Deploy to production"
            tags = ["deploy", "prod"]
            confirm = true

            [[aliases]]
            name = "test-all"
            command = "npm run test && npm run test:e2e"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.aliases.len(), 2);

        let deploy = &config.aliases[0];
        assert_eq!(deploy.name, "deploy");
        assert_eq!(deploy.command, "npm run deploy");
        assert_eq!(deploy.description, Some("Deploy to production".to_string()));
        assert_eq!(deploy.tags, vec!["deploy", "prod"]);
        assert!(deploy.confirm);

        let test_all = &config.aliases[1];
        assert_eq!(test_all.name, "test-all");
        assert_eq!(test_all.command, "npm run test && npm run test:e2e");
        assert!(test_all.description.is_none());
        assert!(test_all.tags.is_empty());
        assert!(!test_all.confirm);
    }

    #[test]
    fn test_alias_with_branches() {
        let toml_str = r#"
            [[aliases]]
            name = "deploy-prod"
            command = "npm run deploy:prod"
            branches = ["main", "release/*"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.aliases.len(), 1);

        let alias = &config.aliases[0];
        assert_eq!(alias.branches, vec!["main", "release/*"]);
    }

    #[test]
    fn test_alias_serialization() {
        let mut config = Config::default();
        config.aliases.push(AliasConfig {
            name: "test".to_string(),
            command: "npm test".to_string(),
            description: Some("Run tests".to_string()),
            tags: vec!["test".to_string()],
            confirm: false,
            working_dir: None,
            env: Vec::new(),
            branches: Vec::new(),
        });

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[[aliases]]"));
        assert!(toml_str.contains("name = \"test\""));
        assert!(toml_str.contains("command = \"npm test\""));
    }
}
