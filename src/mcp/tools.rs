//! MCP Tool utilities.
//!
//! Provides utilities for working with MCP tools including
//! registry, formatting, and conversion to AI tool formats.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::protocol::{CallToolResult, MCPTool, ToolContent};

/// Represents a tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: HashMap<String, serde_json::Value>,
    /// Optional tool ID (for tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl ToolCall {
    /// Create a new tool call.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), arguments: HashMap::new(), id: None }
    }

    /// Add an argument.
    pub fn arg(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.arguments.insert(key.into(), value.into());
        self
    }

    /// Set the tool ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID (if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Whether the call was successful
    pub success: bool,
    /// Text content from the result
    pub content: String,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result.
    pub fn success(content: impl Into<String>) -> Self {
        Self { id: None, success: true, content: content.into(), error: None }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self { id: None, success: false, content: String::new(), error: Some(message.into()) }
    }

    /// Set the tool call ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl From<CallToolResult> for ToolResult {
    fn from(result: CallToolResult) -> Self {
        let success = !result.is_error.unwrap_or(false);
        let content =
            result.content.iter().filter_map(|c| c.as_text()).collect::<Vec<_>>().join("\n");

        if success {
            ToolResult::success(content)
        } else {
            ToolResult::error(content)
        }
    }
}

/// Tool registry for tracking available tools.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    /// Tools indexed by name
    tools: HashMap<String, RegisteredToolInfo>,
}

/// Information about a registered tool.
#[derive(Debug, Clone)]
pub struct RegisteredToolInfo {
    /// The tool definition
    pub tool: MCPTool,
    /// Source server name
    pub server: String,
    /// Whether the tool is enabled
    pub enabled: bool,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool.
    pub fn register(&mut self, server: &str, tool: MCPTool) {
        self.tools.insert(
            tool.name.clone(),
            RegisteredToolInfo { tool, server: server.to_string(), enabled: true },
        );
    }

    /// Unregister a tool.
    pub fn unregister(&mut self, name: &str) {
        self.tools.remove(name);
    }

    /// Unregister all tools from a server.
    pub fn unregister_server(&mut self, server: &str) {
        self.tools.retain(|_, info| info.server != server);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&RegisteredToolInfo> {
        self.tools.get(name)
    }

    /// Check if a tool exists.
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tools.
    pub fn all(&self) -> impl Iterator<Item = &RegisteredToolInfo> {
        self.tools.values()
    }

    /// Get tools from a specific server.
    pub fn from_server<'a>(
        &'a self,
        server: &'a str,
    ) -> impl Iterator<Item = &'a RegisteredToolInfo> {
        self.tools.values().filter(move |t| t.server == server)
    }

    /// Get enabled tools.
    pub fn enabled(&self) -> impl Iterator<Item = &RegisteredToolInfo> {
        self.tools.values().filter(|t| t.enabled)
    }

    /// Enable a tool.
    pub fn enable(&mut self, name: &str) {
        if let Some(info) = self.tools.get_mut(name) {
            info.enabled = true;
        }
    }

    /// Disable a tool.
    pub fn disable(&mut self, name: &str) {
        if let Some(info) = self.tools.get_mut(name) {
            info.enabled = false;
        }
    }

    /// Get count of tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Get count of enabled tools.
    pub fn enabled_count(&self) -> usize {
        self.tools.values().filter(|t| t.enabled).count()
    }

    /// Convert to AI tool format (for Claude/OpenAI).
    pub fn to_ai_tools(&self) -> Vec<serde_json::Value> {
        self.enabled()
            .map(|info| {
                serde_json::json!({
                    "name": info.tool.name,
                    "description": info.tool.description,
                    "input_schema": info.tool.input_schema,
                })
            })
            .collect()
    }
}

/// Format a tool for display.
pub fn format_tool(tool: &MCPTool, server: Option<&str>) -> String {
    let mut output = String::new();

    // Tool name and server
    if let Some(srv) = server {
        output.push_str(&format!("{} (from {})", tool.name, srv));
    } else {
        output.push_str(&tool.name);
    }

    // Description
    if let Some(ref desc) = tool.description {
        output.push_str(&format!("\n  {}", desc));
    }

    // Parameters
    if let Some(ref props) = tool.input_schema.properties {
        if !props.is_empty() {
            output.push_str("\n  Parameters:");
            for (name, schema) in props {
                let type_str = schema.get("type").and_then(|v| v.as_str()).unwrap_or("any");
                let desc = schema.get("description").and_then(|v| v.as_str()).unwrap_or("");
                output.push_str(&format!("\n    - {} ({}): {}", name, type_str, desc));
            }
        }
    }

    // Required parameters
    if let Some(ref required) = tool.input_schema.required {
        if !required.is_empty() {
            output.push_str(&format!("\n  Required: {}", required.join(", ")));
        }
    }

    output
}

/// Extract text content from tool result.
pub fn extract_text_content(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| match c {
            ToolContent::Text { text } => Some(text.as_str()),
            ToolContent::Resource { text, .. } => text.as_deref(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::MCPToolInputSchema;

    fn create_test_tool(name: &str) -> MCPTool {
        MCPTool {
            name: name.to_string(),
            description: Some(format!("Test tool: {}", name)),
            input_schema: MCPToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
            },
        }
    }

    #[test]
    fn test_tool_call_builder() {
        let call = ToolCall::new("test_tool").arg("foo", "bar").arg("count", 42).with_id("call-1");

        assert_eq!(call.name, "test_tool");
        assert_eq!(call.arguments.get("foo"), Some(&serde_json::json!("bar")));
        assert_eq!(call.arguments.get("count"), Some(&serde_json::json!(42)));
        assert_eq!(call.id, Some("call-1".to_string()));
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("Hello, world!");
        assert!(success.success);
        assert_eq!(success.content, "Hello, world!");

        let error = ToolResult::error("Something went wrong");
        assert!(!error.success);
        assert_eq!(error.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        registry.register("server1", create_test_tool("tool1"));
        registry.register("server1", create_test_tool("tool2"));
        registry.register("server2", create_test_tool("tool3"));

        assert_eq!(registry.count(), 3);
        assert!(registry.has("tool1"));
        assert!(!registry.has("nonexistent"));

        // Check server filtering
        assert_eq!(registry.from_server("server1").count(), 2);
        assert_eq!(registry.from_server("server2").count(), 1);

        // Unregister server
        registry.unregister_server("server1");
        assert_eq!(registry.count(), 1);
        assert!(!registry.has("tool1"));
        assert!(registry.has("tool3"));
    }

    #[test]
    fn test_enable_disable() {
        let mut registry = ToolRegistry::new();
        registry.register("server", create_test_tool("tool"));

        assert_eq!(registry.enabled_count(), 1);

        registry.disable("tool");
        assert_eq!(registry.enabled_count(), 0);

        registry.enable("tool");
        assert_eq!(registry.enabled_count(), 1);
    }
}
