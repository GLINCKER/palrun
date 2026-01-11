//! MCP Manager for handling multiple MCP servers.
//!
//! Provides a unified interface for managing multiple MCP servers
//! and routing tool calls to the appropriate server.

use std::collections::HashMap;

use super::client::MCPClient;
use super::protocol::{CallToolResult, MCPTool};
use super::server::MCPServerConfig;

/// Error type for MCP manager operations.
#[derive(Debug, thiserror::Error)]
pub enum MCPManagerError {
    #[error("Client error: {0}")]
    Client(#[from] super::client::MCPClientError),

    #[error("Server already exists: {0}")]
    ServerExists(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Represents a tool with its source server.
#[derive(Debug, Clone)]
pub struct RegisteredTool {
    /// The tool definition
    pub tool: MCPTool,
    /// Server name that provides this tool
    pub server: String,
}

/// MCP Manager for handling multiple servers.
pub struct MCPManager {
    /// Connected clients by name
    clients: HashMap<String, MCPClient>,
    /// Tool registry (tool name -> server name)
    tool_registry: HashMap<String, String>,
}

impl MCPManager {
    /// Create a new MCP manager.
    pub fn new() -> Self {
        Self { clients: HashMap::new(), tool_registry: HashMap::new() }
    }

    /// Add a server configuration.
    pub fn add_server(&mut self, config: MCPServerConfig) -> Result<(), MCPManagerError> {
        let name = config.name.clone();

        if self.clients.contains_key(&name) {
            return Err(MCPManagerError::ServerExists(name));
        }

        let client = MCPClient::new(config);
        self.clients.insert(name, client);
        Ok(())
    }

    /// Remove a server.
    pub fn remove_server(&mut self, name: &str) -> Result<(), MCPManagerError> {
        if let Some(mut client) = self.clients.remove(name) {
            let _ = client.stop();

            // Remove tools from registry
            self.tool_registry.retain(|_, server| server != name);
        }
        Ok(())
    }

    /// Start all servers.
    pub fn start_all(&mut self) -> Result<(), MCPManagerError> {
        let names: Vec<String> = self.clients.keys().cloned().collect();

        for name in names {
            self.start_server(&name)?;
        }

        Ok(())
    }

    /// Start a specific server.
    pub fn start_server(&mut self, name: &str) -> Result<(), MCPManagerError> {
        let client = self
            .clients
            .get_mut(name)
            .ok_or_else(|| MCPManagerError::ServerNotFound(name.to_string()))?;

        client.start()?;

        // Register tools
        for tool in client.tools() {
            self.tool_registry.insert(tool.name.clone(), name.to_string());
        }

        Ok(())
    }

    /// Stop all servers.
    pub fn stop_all(&mut self) -> Result<(), MCPManagerError> {
        for client in self.clients.values_mut() {
            let _ = client.stop();
        }
        self.tool_registry.clear();
        Ok(())
    }

    /// Stop a specific server.
    pub fn stop_server(&mut self, name: &str) -> Result<(), MCPManagerError> {
        let client = self
            .clients
            .get_mut(name)
            .ok_or_else(|| MCPManagerError::ServerNotFound(name.to_string()))?;

        client.stop()?;

        // Unregister tools
        self.tool_registry.retain(|_, server| server != name);

        Ok(())
    }

    /// Get all available tools from all servers.
    pub fn list_tools(&self) -> Vec<RegisteredTool> {
        let mut tools = Vec::new();

        for (name, client) in &self.clients {
            if client.is_connected() {
                for tool in client.tools() {
                    tools.push(RegisteredTool { tool: tool.clone(), server: name.clone() });
                }
            }
        }

        tools
    }

    /// Get a specific tool.
    pub fn get_tool(&self, name: &str) -> Option<RegisteredTool> {
        let server_name = self.tool_registry.get(name)?;
        let client = self.clients.get(server_name)?;
        let tool = client.get_tool(name)?;

        Some(RegisteredTool { tool: tool.clone(), server: server_name.clone() })
    }

    /// Call a tool by name.
    pub fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, MCPManagerError> {
        let server_name = self
            .tool_registry
            .get(tool_name)
            .ok_or_else(|| MCPManagerError::ToolNotFound(tool_name.to_string()))?
            .clone();

        let client = self
            .clients
            .get_mut(&server_name)
            .ok_or(MCPManagerError::ServerNotFound(server_name))?;

        let result = client.call_tool(tool_name, arguments)?;
        Ok(result)
    }

    /// Get server names.
    pub fn server_names(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a server is connected.
    pub fn is_server_connected(&self, name: &str) -> bool {
        self.clients.get(name).map(|c| c.is_connected()).unwrap_or(false)
    }

    /// Get the number of connected servers.
    pub fn connected_count(&self) -> usize {
        self.clients.values().filter(|c| c.is_connected()).count()
    }

    /// Get tools for AI tool-use format.
    pub fn get_tools_for_ai(&self) -> Vec<serde_json::Value> {
        self.list_tools()
            .into_iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.tool.name,
                    "description": t.tool.description,
                    "input_schema": t.tool.input_schema,
                    "server": t.server,
                })
            })
            .collect()
    }

    /// Refresh tools from all connected servers.
    pub fn refresh_all_tools(&mut self) -> Result<(), MCPManagerError> {
        self.tool_registry.clear();

        for (name, client) in &mut self.clients {
            if client.is_connected() {
                client.refresh_tools()?;
                for tool in client.tools() {
                    self.tool_registry.insert(tool.name.clone(), name.clone());
                }
            }
        }

        Ok(())
    }
}

impl Default for MCPManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MCPManager {
    fn drop(&mut self) {
        let _ = self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = MCPManager::new();
        assert_eq!(manager.connected_count(), 0);
        assert!(manager.list_tools().is_empty());
    }

    #[test]
    fn test_add_duplicate_server() {
        let mut manager = MCPManager::new();

        let config = MCPServerConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };

        manager.add_server(config.clone()).unwrap();
        assert!(manager.add_server(config).is_err());
    }
}
