//! MCP Client for communicating with MCP servers.
//!
//! Provides a high-level interface for MCP operations.

use std::collections::HashMap;

use super::protocol::{CallToolResult, MCPTool};
use super::server::{MCPServer, MCPServerConfig, MCPServerError};

/// Error type for MCP client operations.
#[derive(Debug, thiserror::Error)]
pub enum MCPClientError {
    #[error("Server error: {0}")]
    Server(#[from] MCPServerError),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
}

/// MCP Client for interacting with a single server.
pub struct MCPClient {
    /// The underlying server
    server: MCPServer,
}

impl MCPClient {
    /// Create a new MCP client from a server configuration.
    pub fn new(config: MCPServerConfig) -> Self {
        Self { server: MCPServer::new(config) }
    }

    /// Get the server name.
    pub fn name(&self) -> &str {
        self.server.name()
    }

    /// Start the client (connects to server).
    pub fn start(&mut self) -> Result<(), MCPClientError> {
        self.server.start()?;
        Ok(())
    }

    /// Stop the client (disconnects from server).
    pub fn stop(&mut self) -> Result<(), MCPClientError> {
        self.server.stop()?;
        Ok(())
    }

    /// Check if the client is connected.
    pub fn is_connected(&self) -> bool {
        self.server.is_running()
    }

    /// Get available tools.
    pub fn tools(&self) -> &[MCPTool] {
        self.server.tools()
    }

    /// Refresh the list of available tools.
    pub fn refresh_tools(&mut self) -> Result<(), MCPClientError> {
        self.server.refresh_tools()?;
        Ok(())
    }

    /// Get a specific tool by name.
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.server.get_tool(name)
    }

    /// Check if a tool exists.
    pub fn has_tool(&self, name: &str) -> bool {
        self.server.has_tool(name)
    }

    /// Call a tool with the given arguments.
    pub fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, MCPClientError> {
        if !self.has_tool(name) {
            return Err(MCPClientError::ToolNotFound(name.to_string()));
        }

        let result = self.server.call_tool(name, arguments)?;
        Ok(result)
    }

    /// Get the server version if available.
    pub fn server_version(&self) -> Option<&str> {
        self.server.server_info().and_then(|info| info.server_info.version.as_deref())
    }

    /// Get the server name from server info.
    pub fn server_name(&self) -> Option<&str> {
        self.server.server_info().map(|info| info.server_info.name.as_str())
    }
}

/// Format tools for display.
pub fn format_tools(tools: &[MCPTool]) -> String {
    let mut output = String::new();

    for tool in tools {
        output.push_str(&format!("  {}", tool.name));
        if let Some(ref desc) = tool.description {
            output.push_str(&format!(" - {}", desc));
        }
        output.push('\n');

        // Show required parameters
        if let Some(ref required) = tool.input_schema.required {
            if !required.is_empty() {
                output.push_str(&format!("    Required: {}\n", required.join(", ")));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tools() {
        let tools = vec![MCPTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: super::super::protocol::MCPToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: Some(vec!["arg1".to_string()]),
            },
        }];

        let output = format_tools(&tools);
        assert!(output.contains("test_tool"));
        assert!(output.contains("A test tool"));
        assert!(output.contains("arg1"));
    }
}
