//! MCP (Model Context Protocol) integration.
//!
//! This module implements an MCP client that can connect to MCP servers
//! and expose their tools to the Palrun AI agent. This enables dynamic
//! tool discovery and execution from external services.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                 Palrun AI Agent                  │
//! │  ┌─────────────────────────────────────────┐    │
//! │  │           MCPManager                     │    │
//! │  │  • Manages multiple MCP servers         │    │
//! │  │  • Discovers tools from all servers     │    │
//! │  │  • Routes tool calls to correct server  │    │
//! │  └─────────────────────────────────────────┘    │
//! │                      │                           │
//! │      ┌───────────────┼───────────────┐          │
//! │      ▼               ▼               ▼          │
//! │  MCPServer       MCPServer       MCPServer      │
//! │  (GitHub)        (Linear)        (Custom)       │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use palrun::mcp::MCPManager;
//!
//! let mut manager = MCPManager::new();
//!
//! // Add a server from config
//! manager.add_server(MCPServerConfig {
//!     name: "github".to_string(),
//!     command: "npx".to_string(),
//!     args: vec!["-y", "@modelcontextprotocol/server-github"],
//!     env: HashMap::new(),
//! })?;
//!
//! // Start all servers
//! manager.start_all().await?;
//!
//! // Get available tools
//! let tools = manager.list_tools().await?;
//!
//! // Call a tool
//! let result = manager.call_tool("github", "create_issue", args).await?;
//! ```

mod client;
mod manager;
mod protocol;
mod server;
mod tools;

pub use client::{MCPClient, MCPClientError};
pub use manager::{MCPManager, MCPManagerError, RegisteredTool};
pub use protocol::{
    CallToolParams, CallToolResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, ListToolsResult,
    MCPCapabilities, MCPInitializeParams, MCPInitializeResult, MCPTool, MCPToolInputSchema,
    ToolContent,
};
pub use server::{MCPServer, MCPServerConfig, MCPServerError, MCPServerState};
pub use tools::{ToolCall, ToolRegistry, ToolResult};
