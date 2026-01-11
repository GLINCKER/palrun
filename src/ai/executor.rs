//! MCP Tool Executor for AI Agent.
//!
//! Executes MCP tools on behalf of the AI agent.

use std::collections::HashMap;

use async_trait::async_trait;

use super::agent::{AgentToolCall, AgentToolResult, ToolExecutor};
use crate::mcp::{MCPManager, MCPServerConfig};

/// MCP-based tool executor.
///
/// Connects to MCP servers and executes tools on behalf of the AI agent.
pub struct MCPToolExecutor {
    manager: MCPManager,
    /// Mapping from tool name to server name
    tool_servers: HashMap<String, String>,
}

impl MCPToolExecutor {
    /// Create a new MCP tool executor.
    pub fn new() -> Self {
        Self { manager: MCPManager::new(), tool_servers: HashMap::new() }
    }

    /// Add an MCP server.
    pub fn add_server(&mut self, config: MCPServerConfig) -> anyhow::Result<()> {
        self.manager.add_server(config)?;
        Ok(())
    }

    /// Start all servers and discover tools.
    pub fn start(&mut self) -> anyhow::Result<()> {
        self.manager.start_all()?;

        // Build tool -> server mapping
        self.tool_servers.clear();
        for tool in self.manager.list_tools() {
            self.tool_servers.insert(tool.tool.name.clone(), tool.server.clone());
        }

        Ok(())
    }

    /// Stop all servers.
    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.manager.stop_all()?;
        Ok(())
    }

    /// Get the list of available tools.
    pub fn available_tools(&self) -> Vec<super::agent::AgentTool> {
        super::agent::mcp_tools_to_agent_tools(&self.manager.list_tools())
    }

    /// Get the underlying manager.
    pub fn manager(&self) -> &MCPManager {
        &self.manager
    }

    /// Get mutable access to the underlying manager.
    pub fn manager_mut(&mut self) -> &mut MCPManager {
        &mut self.manager
    }
}

impl Default for MCPToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for MCPToolExecutor {
    async fn execute(&mut self, tool_call: &AgentToolCall) -> AgentToolResult {
        tracing::debug!(
            tool = %tool_call.name,
            args = ?tool_call.arguments,
            "Executing MCP tool"
        );

        // Call the tool through the manager
        match self.manager.call_tool(&tool_call.name, Some(tool_call.arguments.clone())) {
            Ok(result) => {
                // Extract text content from result
                let output = result
                    .content
                    .iter()
                    .filter_map(|c| c.as_text())
                    .collect::<Vec<_>>()
                    .join("\n");

                let success = !result.is_error.unwrap_or(false);

                AgentToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success,
                    output: if output.is_empty() {
                        if success {
                            "Tool executed successfully (no output)".to_string()
                        } else {
                            "Tool execution failed (no details)".to_string()
                        }
                    } else {
                        output
                    },
                }
            }
            Err(e) => AgentToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                output: format!("Tool execution error: {}", e),
            },
        }
    }
}

/// Shell command executor for running shell commands.
///
/// This is a simple executor that runs shell commands and returns the output.
pub struct ShellExecutor;

impl ShellExecutor {
    /// Create a new shell executor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for ShellExecutor {
    async fn execute(&mut self, tool_call: &AgentToolCall) -> AgentToolResult {
        // For shell executor, we expect a "command" argument
        let command = tool_call.arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");

        if command.is_empty() {
            return AgentToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                output: "No command provided".to_string(),
            };
        }

        tracing::debug!(command = %command, "Executing shell command");

        // Execute the command
        let output = std::process::Command::new("sh").arg("-c").arg(command).output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else if stdout.is_empty() {
                    stderr.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };

                AgentToolResult {
                    tool_call_id: tool_call.id.clone(),
                    success: output.status.success(),
                    output: if combined.is_empty() {
                        if output.status.success() {
                            "Command executed successfully (no output)".to_string()
                        } else {
                            format!("Command failed with exit code: {:?}", output.status.code())
                        }
                    } else {
                        combined
                    },
                }
            }
            Err(e) => AgentToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                output: format!("Failed to execute command: {}", e),
            },
        }
    }
}

/// Composite executor that can use multiple executors.
pub struct CompositeExecutor {
    mcp: Option<MCPToolExecutor>,
    shell: ShellExecutor,
}

impl CompositeExecutor {
    /// Create a new composite executor.
    pub fn new() -> Self {
        Self { mcp: None, shell: ShellExecutor::new() }
    }

    /// Create with an MCP executor.
    pub fn with_mcp(mut self, mcp: MCPToolExecutor) -> Self {
        self.mcp = Some(mcp);
        self
    }

    /// Get the MCP executor if available.
    pub fn mcp(&self) -> Option<&MCPToolExecutor> {
        self.mcp.as_ref()
    }

    /// Get mutable access to the MCP executor.
    pub fn mcp_mut(&mut self) -> Option<&mut MCPToolExecutor> {
        self.mcp.as_mut()
    }
}

impl Default for CompositeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for CompositeExecutor {
    async fn execute(&mut self, tool_call: &AgentToolCall) -> AgentToolResult {
        // Check if this is a shell command
        if tool_call.name == "execute_command" || tool_call.name == "shell" {
            return self.shell.execute(tool_call).await;
        }

        // Try MCP executor
        if let Some(ref mut mcp) = self.mcp {
            return mcp.execute(tool_call).await;
        }

        // Unknown tool
        AgentToolResult {
            tool_call_id: tool_call.id.clone(),
            success: false,
            output: format!("Unknown tool: {}", tool_call.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_executor_creation() {
        let executor = MCPToolExecutor::new();
        assert!(executor.available_tools().is_empty());
    }

    #[tokio::test]
    async fn test_shell_executor() {
        let mut executor = ShellExecutor::new();

        let tool_call = AgentToolCall {
            id: "test".to_string(),
            name: "shell".to_string(),
            arguments: {
                let mut args = HashMap::new();
                args.insert("command".to_string(), serde_json::json!("echo hello"));
                args
            },
        };

        let result = executor.execute(&tool_call).await;
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[test]
    fn test_composite_executor_creation() {
        let executor = CompositeExecutor::new();
        assert!(executor.mcp().is_none());
    }
}
