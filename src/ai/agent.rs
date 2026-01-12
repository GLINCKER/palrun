//! AI Agent with tool-use capability.
//!
//! Provides an agentic loop that allows the AI to use MCP tools
//! to accomplish tasks autonomously.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ProjectContext;

/// A tool definition for the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Input schema (JSON Schema format)
    pub input_schema: serde_json::Value,
    /// Server that provides this tool (for MCP tools)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
}

/// A tool call requested by the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolResult {
    /// ID of the tool call this is a response to
    pub tool_call_id: String,
    /// Whether the tool execution was successful
    pub success: bool,
    /// Tool output (text content)
    pub output: String,
}

/// A message in the agent conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum AgentMessage {
    /// System message (context/instructions)
    #[serde(rename = "system")]
    System { content: String },

    /// User message
    #[serde(rename = "user")]
    User { content: String },

    /// Assistant message (may include tool calls)
    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<AgentToolCall>>,
    },

    /// Tool result message
    #[serde(rename = "tool")]
    Tool { tool_call_id: String, content: String },
}

/// Agent state for managing the conversation.
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Conversation history
    pub messages: Vec<AgentMessage>,
    /// Available tools
    pub tools: Vec<AgentTool>,
    /// Project context
    pub context: ProjectContext,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Current iteration
    pub current_iteration: usize,
    /// Whether the agent is done
    pub done: bool,
}

impl AgentState {
    /// Create a new agent state.
    pub fn new(context: ProjectContext) -> Self {
        Self {
            messages: Vec::new(),
            tools: Vec::new(),
            context,
            max_iterations: 10,
            current_iteration: 0,
            done: false,
        }
    }

    /// Add tools to the agent.
    pub fn with_tools(mut self, tools: Vec<AgentTool>) -> Self {
        self.tools = tools;
        self
    }

    /// Set the maximum iterations.
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Add a system message.
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(AgentMessage::System { content: content.into() });
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(AgentMessage::User { content: content.into() });
    }

    /// Add an assistant message.
    pub fn add_assistant_message(
        &mut self,
        content: Option<String>,
        tool_calls: Option<Vec<AgentToolCall>>,
    ) {
        self.messages.push(AgentMessage::Assistant { content, tool_calls });
    }

    /// Add a tool result.
    pub fn add_tool_result(&mut self, tool_call_id: String, content: String) {
        self.messages.push(AgentMessage::Tool { tool_call_id, content });
    }

    /// Check if we should continue.
    pub fn should_continue(&self) -> bool {
        !self.done && self.current_iteration < self.max_iterations
    }

    /// Mark the agent as done.
    pub fn finish(&mut self) {
        self.done = true;
    }

    /// Increment the iteration counter.
    pub fn next_iteration(&mut self) {
        self.current_iteration += 1;
    }
}

/// Response from the AI provider for agentic interactions.
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// Text content (if any)
    pub content: Option<String>,
    /// Tool calls requested (if any)
    pub tool_calls: Option<Vec<AgentToolCall>>,
    /// Stop reason
    pub stop_reason: AgentStopReason,
}

/// Reason the agent stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStopReason {
    /// Normal end of response
    EndTurn,
    /// Waiting for tool results
    ToolUse,
    /// Max tokens reached
    MaxTokens,
    /// An error occurred
    Error,
}

/// Trait for AI providers that support tool use.
#[async_trait]
pub trait AgentProvider: Send + Sync {
    /// Run one step of the agent loop.
    ///
    /// Takes the current state and returns the AI's response,
    /// which may include tool calls.
    async fn step(&self, state: &AgentState) -> anyhow::Result<AgentResponse>;

    /// Get the provider name.
    fn name(&self) -> &str;

    /// Check if tool use is supported.
    fn supports_tools(&self) -> bool;
}

/// Trait for executing tools.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool call.
    async fn execute(&mut self, tool_call: &AgentToolCall) -> AgentToolResult;
}

/// Simple agent runner.
pub struct Agent<P: AgentProvider, E: ToolExecutor> {
    provider: P,
    executor: E,
}

impl<P: AgentProvider, E: ToolExecutor> Agent<P, E> {
    /// Create a new agent.
    pub fn new(provider: P, executor: E) -> Self {
        Self { provider, executor }
    }

    /// Run the agent with a user task.
    pub async fn run(&mut self, task: &str, mut state: AgentState) -> anyhow::Result<AgentState> {
        // Build system message
        let system_prompt = build_system_prompt(&state.context, &state.tools);
        state.add_system_message(system_prompt);

        // Add the user task
        state.add_user_message(task);

        // Agent loop
        while state.should_continue() {
            state.next_iteration();

            tracing::debug!(iteration = state.current_iteration, "Agent iteration");

            // Get AI response
            let response = self.provider.step(&state).await?;

            // Add assistant message
            state.add_assistant_message(response.content.clone(), response.tool_calls.clone());

            // Check stop reason
            match response.stop_reason {
                AgentStopReason::EndTurn => {
                    // Agent is done
                    state.finish();
                }
                AgentStopReason::ToolUse => {
                    // Execute tool calls
                    if let Some(tool_calls) = response.tool_calls {
                        for tool_call in tool_calls {
                            tracing::info!(
                                tool = %tool_call.name,
                                "Executing tool"
                            );

                            let result = self.executor.execute(&tool_call).await;
                            state.add_tool_result(result.tool_call_id, result.output);
                        }
                    }
                }
                AgentStopReason::MaxTokens => {
                    tracing::warn!("Max tokens reached");
                    state.finish();
                }
                AgentStopReason::Error => {
                    tracing::error!("Agent error");
                    state.finish();
                }
            }
        }

        Ok(state)
    }

    /// Get the final response from the agent.
    pub fn get_final_response(state: &AgentState) -> Option<String> {
        // Find the last assistant message with content
        for msg in state.messages.iter().rev() {
            if let AgentMessage::Assistant { content: Some(text), .. } = msg {
                if !text.is_empty() {
                    return Some(text.clone());
                }
            }
        }
        None
    }
}

/// Build the system prompt for the agent.
fn build_system_prompt(context: &ProjectContext, tools: &[AgentTool]) -> String {
    let mut prompt = format!(
        r"You are Palrun, an AI-powered terminal assistant that can execute commands and use tools to help users with their tasks.

## Project Context
- Project: {}
- Type: {}
- Directory: {}
- Available commands: {}

## Your Capabilities
You can use the provided tools to:
1. Read and analyze files
2. Execute shell commands
3. Search the codebase
4. Fetch web content
5. And more based on available MCP tools

## Guidelines
1. Think step by step about what you need to do
2. Use tools when you need information or to perform actions
3. Be concise in your explanations
4. Always explain what you're doing and why
5. If a tool fails, try an alternative approach
6. When done, provide a summary of what was accomplished",
        context.project_name,
        context.project_type,
        context.current_directory.display(),
        if context.available_commands.is_empty() {
            "none detected".to_string()
        } else {
            context.available_commands.join(", ")
        }
    );

    if !tools.is_empty() {
        prompt.push_str("\n\n## Available Tools\n");
        for tool in tools {
            prompt.push_str(&format!("- {}", tool.name));
            if let Some(ref desc) = tool.description {
                prompt.push_str(&format!(": {}", desc));
            }
            prompt.push('\n');
        }
    }

    prompt
}

/// Convert MCP tools to agent tools.
pub fn mcp_tools_to_agent_tools(tools: &[crate::mcp::RegisteredTool]) -> Vec<AgentTool> {
    tools
        .iter()
        .map(|t| AgentTool {
            name: t.tool.name.clone(),
            description: t.tool.description.clone(),
            input_schema: serde_json::to_value(&t.tool.input_schema)
                .unwrap_or(serde_json::json!({})),
            server: Some(t.server.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_agent_state_creation() {
        let context = ProjectContext::new("test", PathBuf::from("."));
        let state = AgentState::new(context);

        assert!(state.messages.is_empty());
        assert!(state.tools.is_empty());
        assert!(!state.done);
    }

    #[test]
    fn test_agent_state_messages() {
        let context = ProjectContext::new("test", PathBuf::from("."));
        let mut state = AgentState::new(context);

        state.add_user_message("Hello");
        state.add_assistant_message(Some("Hi!".to_string()), None);

        assert_eq!(state.messages.len(), 2);
    }

    #[test]
    fn test_should_continue() {
        let context = ProjectContext::new("test", PathBuf::from("."));
        let mut state = AgentState::new(context).with_max_iterations(2);

        assert!(state.should_continue());

        state.next_iteration();
        assert!(state.should_continue());

        state.next_iteration();
        assert!(!state.should_continue());
    }

    #[test]
    fn test_build_system_prompt() {
        let mut context = ProjectContext::new("my-app", PathBuf::from("/project"));
        context.project_type = "node".to_string();
        context.available_commands = vec!["npm run build".to_string()];

        let tools = vec![AgentTool {
            name: "read_file".to_string(),
            description: Some("Read a file".to_string()),
            input_schema: serde_json::json!({}),
            server: None,
        }];

        let prompt = build_system_prompt(&context, &tools);

        assert!(prompt.contains("my-app"));
        assert!(prompt.contains("node"));
        assert!(prompt.contains("read_file"));
    }
}
