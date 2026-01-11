//! MCP Server management.
//!
//! Handles spawning, communicating with, and managing MCP server processes.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::protocol::{
    CallToolParams, CallToolResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult,
    MCPInitializeParams, MCPInitializeResult, MCPTool,
};

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// Server name (unique identifier)
    pub name: String,
    /// Command to run
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// MCP server state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCPServerState {
    /// Server not started
    Stopped,
    /// Server is starting
    Starting,
    /// Server is running and ready
    Running,
    /// Server encountered an error
    Error,
}

/// Error type for MCP server operations.
#[derive(Debug, thiserror::Error)]
pub enum MCPServerError {
    #[error("Failed to spawn server process: {0}")]
    SpawnFailed(#[from] std::io::Error),

    #[error("Server initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Server not running")]
    NotRunning,

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpc(#[from] JsonRpcError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout waiting for response")]
    Timeout,
}

/// MCP server instance.
pub struct MCPServer {
    /// Server configuration
    config: MCPServerConfig,
    /// Server state
    state: MCPServerState,
    /// Server process
    process: Option<Child>,
    /// Request ID counter
    request_id: AtomicI64,
    /// Cached tools
    tools: Vec<MCPTool>,
    /// Server info from initialization
    server_info: Option<MCPInitializeResult>,
    /// Stdin handle (wrapped for thread safety)
    stdin: Option<Arc<Mutex<std::process::ChildStdin>>>,
    /// Stdout reader
    stdout: Option<Arc<Mutex<BufReader<std::process::ChildStdout>>>>,
}

impl MCPServer {
    /// Create a new MCP server instance.
    pub fn new(config: MCPServerConfig) -> Self {
        Self {
            config,
            state: MCPServerState::Stopped,
            process: None,
            request_id: AtomicI64::new(1),
            tools: Vec::new(),
            server_info: None,
            stdin: None,
            stdout: None,
        }
    }

    /// Get the server name.
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the current state.
    pub fn state(&self) -> MCPServerState {
        self.state
    }

    /// Get available tools.
    pub fn tools(&self) -> &[MCPTool] {
        &self.tools
    }

    /// Get server info.
    pub fn server_info(&self) -> Option<&MCPInitializeResult> {
        self.server_info.as_ref()
    }

    /// Start the server process.
    pub fn start(&mut self) -> Result<(), MCPServerError> {
        if self.state == MCPServerState::Running {
            return Ok(());
        }

        self.state = MCPServerState::Starting;

        // Build command
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args);

        // Set environment
        for (key, value) in &self.config.env {
            // Expand environment variables in the value
            let expanded = shellexpand::env(value).unwrap_or_else(|_| value.clone().into());
            cmd.env(key, expanded.as_ref());
        }

        // Set working directory if specified
        if let Some(ref cwd) = self.config.cwd {
            cmd.current_dir(cwd);
        }

        // Set up stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit()); // Let stderr go to console for debugging

        // Spawn process
        let mut child = cmd.spawn()?;

        // Take ownership of stdin/stdout
        let stdin = child.stdin.take().ok_or_else(|| {
            MCPServerError::InitializationFailed("Failed to capture stdin".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            MCPServerError::InitializationFailed("Failed to capture stdout".to_string())
        })?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.stdout = Some(Arc::new(Mutex::new(BufReader::new(stdout))));
        self.process = Some(child);

        // Initialize the server
        self.initialize()?;

        // Fetch available tools
        self.refresh_tools()?;

        self.state = MCPServerState::Running;
        Ok(())
    }

    /// Stop the server process.
    pub fn stop(&mut self) -> Result<(), MCPServerError> {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }

        self.stdin = None;
        self.stdout = None;
        self.state = MCPServerState::Stopped;
        self.tools.clear();
        self.server_info = None;

        Ok(())
    }

    /// Check if the server is running.
    pub fn is_running(&self) -> bool {
        self.state == MCPServerState::Running
    }

    /// Send a request and wait for response.
    fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<JsonRpcResponse, MCPServerError> {
        let stdin = self.stdin.as_ref().ok_or(MCPServerError::NotRunning)?;
        let stdout = self.stdout.as_ref().ok_or(MCPServerError::NotRunning)?;

        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        // Serialize and send
        let request_json = serde_json::to_string(&request)?;
        tracing::debug!("MCP {} <- {}", self.config.name, request_json);

        {
            let mut stdin_guard = stdin.lock().map_err(|e| {
                MCPServerError::CommunicationError(format!("Failed to lock stdin: {}", e))
            })?;
            writeln!(stdin_guard, "{}", request_json)?;
            stdin_guard.flush()?;
        }

        // Read response
        let response = {
            let mut stdout_guard = stdout.lock().map_err(|e| {
                MCPServerError::CommunicationError(format!("Failed to lock stdout: {}", e))
            })?;

            let mut line = String::new();
            stdout_guard.read_line(&mut line)?;
            tracing::debug!("MCP {} -> {}", self.config.name, line.trim());
            line
        };

        let response: JsonRpcResponse = serde_json::from_str(&response)?;
        Ok(response)
    }

    /// Initialize the server with MCP handshake.
    fn initialize(&mut self) -> Result<(), MCPServerError> {
        let params = MCPInitializeParams::default();
        let response = self.send_request("initialize", Some(serde_json::to_value(&params)?))?;

        let result: MCPInitializeResult = response.into_result()?;
        self.server_info = Some(result);

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        if let Some(ref stdin) = self.stdin {
            let mut stdin_guard = stdin.lock().map_err(|e| {
                MCPServerError::CommunicationError(format!("Failed to lock stdin: {}", e))
            })?;
            writeln!(stdin_guard, "{}", notification)?;
            stdin_guard.flush()?;
        }

        Ok(())
    }

    /// Refresh the list of available tools.
    pub fn refresh_tools(&mut self) -> Result<(), MCPServerError> {
        let response = self.send_request("tools/list", None)?;
        let result: ListToolsResult = response.into_result()?;
        self.tools = result.tools;
        Ok(())
    }

    /// Call a tool.
    pub fn call_tool(
        &mut self,
        name: &str,
        arguments: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CallToolResult, MCPServerError> {
        let params = CallToolParams { name: name.to_string(), arguments };

        let response = self.send_request("tools/call", Some(serde_json::to_value(&params)?))?;

        let result: CallToolResult = response.into_result()?;
        Ok(result)
    }

    /// Get a tool by name.
    pub fn get_tool(&self, name: &str) -> Option<&MCPTool> {
        self.tools.iter().find(|t| t.name == name)
    }

    /// Check if the server has a specific tool.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }
}

impl Drop for MCPServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = MCPServerConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
            cwd: None,
        };

        assert_eq!(config.name, "test");
        assert_eq!(config.command, "echo");
    }

    #[test]
    fn test_server_initial_state() {
        let config = MCPServerConfig {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };

        let server = MCPServer::new(config);
        assert_eq!(server.state(), MCPServerState::Stopped);
        assert!(server.tools().is_empty());
    }
}
