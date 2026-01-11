//! MCP Scanner for discovering commands from MCP servers.
//!
//! This scanner connects to configured MCP servers and exposes
//! their tools as commands in the Palrun command palette.

use std::path::Path;

use crate::core::{Command, CommandSource};
use crate::mcp::{MCPManager, MCPServerConfig};

use super::Scanner;

/// Scanner for MCP server tools.
pub struct MCPScanner {
    /// MCP manager
    manager: MCPManager,
    /// Whether the manager has been initialized
    initialized: bool,
}

impl MCPScanner {
    /// Create a new MCP scanner.
    pub fn new() -> Self {
        Self { manager: MCPManager::new(), initialized: false }
    }

    /// Create from existing server configurations.
    pub fn with_servers(configs: Vec<MCPServerConfig>) -> Self {
        let mut scanner = Self::new();
        for config in configs {
            let _ = scanner.manager.add_server(config);
        }
        scanner
    }

    /// Add a server configuration.
    pub fn add_server(&mut self, config: MCPServerConfig) -> anyhow::Result<()> {
        self.manager.add_server(config)?;
        Ok(())
    }

    /// Initialize and connect to all servers.
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.manager.start_all()?;
        self.initialized = true;
        Ok(())
    }

    /// Get the MCP manager.
    pub fn manager(&self) -> &MCPManager {
        &self.manager
    }

    /// Get mutable access to the MCP manager.
    pub fn manager_mut(&mut self) -> &mut MCPManager {
        &mut self.manager
    }
}

impl Default for MCPScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner for MCPScanner {
    fn name(&self) -> &str {
        "mcp"
    }

    fn scan(&self, _path: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Get all tools from connected servers
        for tool in self.manager.list_tools() {
            let name = format!("mcp:{}/{}", tool.server, tool.tool.name);
            let description = tool
                .tool
                .description
                .clone()
                .unwrap_or_else(|| format!("MCP tool from {} server", tool.server));

            // Build a command that would invoke this tool
            // For now, we create a placeholder command that shows in the palette
            let cmd_str = format!("pal mcp call {} {}", tool.server, tool.tool.name);

            let mut command = Command::new(&name, &cmd_str)
                .with_description(&description)
                .with_source(CommandSource::Mcp { server: tool.server.clone() });

            // Add tags for the tool
            command.tags.push(format!("mcp:{}", tool.server));
            command.tags.push("mcp".to_string());

            // Store tool schema in metadata for later use
            if let Ok(schema_json) = serde_json::to_string(&tool.tool.input_schema) {
                command.metadata.insert("mcp_schema".to_string(), schema_json);
            }

            commands.push(command);
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_scanner_creation() {
        let scanner = MCPScanner::new();
        assert_eq!(scanner.name(), "mcp");
        assert!(!scanner.initialized);
    }

    #[test]
    fn test_scan_empty() {
        let scanner = MCPScanner::new();
        let commands = scanner.scan(Path::new(".")).unwrap();
        assert!(commands.is_empty());
    }
}
