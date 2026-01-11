//! Core plugin types.

use serde::{Deserialize, Serialize};

/// Type of plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// Scanner plugin - detects commands from project files.
    Scanner,
    /// AI provider plugin - custom LLM integration.
    AiProvider,
    /// Integration plugin - external service connection.
    Integration,
    /// UI plugin - custom TUI components.
    Ui,
}

impl PluginType {
    /// Get the display name for this plugin type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Scanner => "Scanner",
            Self::AiProvider => "AI Provider",
            Self::Integration => "Integration",
            Self::Ui => "UI",
        }
    }

    /// Get the icon for this plugin type.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Scanner => "🔍",
            Self::AiProvider => "🤖",
            Self::Integration => "🔗",
            Self::Ui => "🎨",
        }
    }
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Information about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin author.
    pub author: Option<String>,
    /// Plugin description.
    pub description: Option<String>,
    /// Plugin type.
    pub plugin_type: PluginType,
    /// Plugin homepage URL.
    pub homepage: Option<String>,
    /// Plugin repository URL.
    pub repository: Option<String>,
    /// Plugin license.
    pub license: Option<String>,
}

/// A command returned by a scanner plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name.
    pub name: String,
    /// Command to execute.
    pub command: String,
    /// Optional description.
    pub description: Option<String>,
    /// Working directory (relative to project root).
    pub working_dir: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Plugin API version.
pub const PLUGIN_API_VERSION: &str = "0.1.0";

/// Plugin file extension.
pub const PLUGIN_EXTENSION: &str = "wasm";

/// Plugin manifest file name.
pub const MANIFEST_FILE: &str = "plugin.toml";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type_display() {
        assert_eq!(PluginType::Scanner.display_name(), "Scanner");
        assert_eq!(PluginType::AiProvider.display_name(), "AI Provider");
        assert_eq!(PluginType::Integration.display_name(), "Integration");
        assert_eq!(PluginType::Ui.display_name(), "UI");
    }

    #[test]
    fn test_plugin_type_icon() {
        assert_eq!(PluginType::Scanner.icon(), "🔍");
        assert_eq!(PluginType::AiProvider.icon(), "🤖");
    }

    #[test]
    fn test_plugin_info() {
        let info = PluginInfo {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("A test plugin".to_string()),
            plugin_type: PluginType::Scanner,
            homepage: None,
            repository: None,
            license: Some("MIT".to_string()),
        };

        assert_eq!(info.name, "test-plugin");
        assert_eq!(info.plugin_type, PluginType::Scanner);
    }

    #[test]
    fn test_plugin_command() {
        let cmd = PluginCommand {
            name: "build".to_string(),
            command: "gradle build".to_string(),
            description: Some("Build the project".to_string()),
            working_dir: None,
            tags: vec!["build".to_string()],
        };

        assert_eq!(cmd.name, "build");
        assert_eq!(cmd.tags.len(), 1);
    }
}
