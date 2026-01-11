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

    /// AI provider (claude, ollama, openai)
    pub provider: String,

    /// Model to use
    pub model: Option<String>,

    /// Ollama-specific settings
    pub ollama: OllamaConfig,
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
    /// Load configuration from the default location.
    ///
    /// Looks for config in:
    /// 1. `.palrun.toml` in current directory
    /// 2. `~/.config/palrun/config.toml`
    /// 3. Falls back to defaults
    pub fn load() -> anyhow::Result<Self> {
        // Try local config first
        let local_config = PathBuf::from(".palrun.toml");
        if local_config.exists() {
            return Self::load_from_file(&local_config);
        }

        // Try global config
        if let Some(config_dir) = dirs::config_dir() {
            let global_config = config_dir.join("palrun").join("config.toml");
            if global_config.exists() {
                return Self::load_from_file(&global_config);
            }
        }

        // Return defaults
        Ok(Self::default())
    }

    /// Load configuration from a specific file.
    pub fn load_from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
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
            ollama: OllamaConfig::default(),
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
