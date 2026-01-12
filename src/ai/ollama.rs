//! Ollama local LLM integration.
//!
//! Implements the AIProvider trait for Ollama (local LLM).
//!
//! Supports both simple text generation and agentic tool use.

use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::agent::{
    AgentMessage, AgentProvider, AgentResponse, AgentState, AgentStopReason, AgentTool,
    AgentToolCall,
};
use super::{AIProvider, ProjectContext};

/// Ollama API provider for local LLM.
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider with default settings.
    ///
    /// Uses localhost:11434 by default.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
        }
    }

    /// Create with a specific base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Create with a specific model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Make a request to the Ollama API.
    async fn request(&self, prompt: &str) -> anyhow::Result<String> {
        let request =
            OllamaRequest { model: self.model.clone(), prompt: prompt.to_string(), stream: false };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, body);
        }

        let response: OllamaResponse = response.json().await?;
        Ok(response.response)
    }

    /// Build a system prompt for command generation.
    fn build_command_prompt(prompt: &str, context: &ProjectContext) -> String {
        format!(
            r"You are Palrun, an AI assistant for terminal commands.
Your task is to generate the exact shell command the user needs.

Project: {}
Type: {}
Available commands: {}
Current directory: {}

User request: {}

Rules:
1. Output ONLY the command, no explanation or markdown
2. Use available project commands when possible
3. Be precise and safe
4. Single line output only",
            context.project_name,
            context.project_type,
            context.available_commands.join(", "),
            context.current_directory.display(),
            prompt
        )
    }

    /// Build a prompt for command explanation.
    fn build_explain_prompt(command: &str, context: &ProjectContext) -> String {
        format!(
            r"You are Palrun, an AI assistant for terminal commands.
Explain what the following command does in 2-3 sentences.

Project context: {} ({})
Command: {}

Provide a clear, concise explanation.",
            context.project_name, context.project_type, command
        )
    }

    /// Build a prompt for error diagnosis.
    fn build_diagnose_prompt(command: &str, error: &str, context: &ProjectContext) -> String {
        format!(
            r"You are Palrun, an AI assistant for terminal commands.
Diagnose why this command failed and suggest a fix.

Project: {} ({})
Available commands: {}

Command: {}

Error:
{}

What went wrong and how to fix it? Be concise.",
            context.project_name,
            context.project_type,
            context.available_commands.join(", "),
            command,
            error
        )
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn generate_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let full_prompt = Self::build_command_prompt(prompt, context);
        let response = self.request(&full_prompt).await?;

        // Clean up the response - remove any markdown or extra whitespace
        let command = response
            .lines()
            .find(|line| !line.trim().is_empty() && !line.starts_with("```"))
            .unwrap_or(&response)
            .trim()
            .to_string();

        Ok(command)
    }

    async fn explain_command(
        &self,
        command: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let full_prompt = Self::build_explain_prompt(command, context);
        self.request(&full_prompt).await
    }

    async fn diagnose_error(
        &self,
        command: &str,
        error: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let full_prompt = Self::build_diagnose_prompt(command, error, context);
        self.request(&full_prompt).await
    }

    fn name(&self) -> &str {
        "ollama"
    }

    async fn is_available(&self) -> bool {
        // Try to reach the Ollama API
        let result = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        result.is_ok()
    }
}

/// Ollama API request structure (generate endpoint).
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response structure (generate endpoint).
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

// ============================================================================
// Agentic Tool Use Support (Chat API)
// ============================================================================

/// Ollama chat API request with tool support.
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool>>,
    stream: bool,
}

/// Ollama chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Ollama tool definition.
#[derive(Debug, Serialize)]
struct OllamaTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OllamaFunction,
}

/// Ollama function definition.
#[derive(Debug, Serialize)]
struct OllamaFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Ollama tool call in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaToolCall {
    function: OllamaFunctionCall,
}

/// Ollama function call details.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaFunctionCall {
    name: String,
    arguments: serde_json::Value,
}

/// Ollama chat API response.
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    done_reason: Option<String>,
}

impl OllamaProvider {
    /// Make a chat request with optional tool support.
    async fn chat_request(
        &self,
        messages: Vec<OllamaChatMessage>,
        tools: Option<Vec<OllamaTool>>,
    ) -> anyhow::Result<OllamaChatResponse> {
        let request =
            OllamaChatRequest { model: self.model.clone(), messages, tools, stream: false };

        let response =
            self.client.post(format!("{}/api/chat", self.base_url)).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, body);
        }

        let response: OllamaChatResponse = response.json().await?;
        Ok(response)
    }
}

/// Convert AgentTool to OllamaTool.
fn agent_tool_to_ollama(tool: &AgentTool) -> OllamaTool {
    OllamaTool {
        tool_type: "function".to_string(),
        function: OllamaFunction {
            name: tool.name.clone(),
            description: tool.description.clone().unwrap_or_default(),
            parameters: tool.input_schema.clone(),
        },
    }
}

/// Convert AgentMessage to OllamaChatMessage.
fn agent_message_to_ollama(msg: &AgentMessage) -> OllamaChatMessage {
    match msg {
        AgentMessage::System { content } => OllamaChatMessage {
            role: "system".to_string(),
            content: content.clone(),
            tool_calls: None,
        },
        AgentMessage::User { content } => OllamaChatMessage {
            role: "user".to_string(),
            content: content.clone(),
            tool_calls: None,
        },
        AgentMessage::Assistant { content, tool_calls } => OllamaChatMessage {
            role: "assistant".to_string(),
            content: content.clone().unwrap_or_default(),
            tool_calls: tool_calls.as_ref().map(|calls| {
                calls
                    .iter()
                    .map(|tc| OllamaToolCall {
                        function: OllamaFunctionCall {
                            name: tc.name.clone(),
                            arguments: serde_json::to_value(&tc.arguments)
                                .unwrap_or(serde_json::json!({})),
                        },
                    })
                    .collect()
            }),
        },
        AgentMessage::Tool { tool_call_id: _, content } => OllamaChatMessage {
            role: "tool".to_string(),
            content: content.clone(),
            tool_calls: None,
        },
    }
}

#[async_trait]
impl AgentProvider for OllamaProvider {
    async fn step(&self, state: &AgentState) -> anyhow::Result<AgentResponse> {
        // Convert messages
        let messages: Vec<OllamaChatMessage> =
            state.messages.iter().map(agent_message_to_ollama).collect();

        // Convert tools
        let tools: Option<Vec<OllamaTool>> = if state.tools.is_empty() {
            None
        } else {
            Some(state.tools.iter().map(agent_tool_to_ollama).collect())
        };

        // Make request
        let response = self.chat_request(messages, tools).await?;

        // Convert tool calls
        let tool_calls: Option<Vec<AgentToolCall>> = response.message.tool_calls.map(|calls| {
            calls
                .into_iter()
                .enumerate()
                .map(|(i, tc)| {
                    let arguments: HashMap<String, serde_json::Value> =
                        if let serde_json::Value::Object(map) = tc.function.arguments {
                            map.into_iter().collect()
                        } else if let serde_json::Value::String(s) = tc.function.arguments {
                            // Some models return JSON as a string
                            serde_json::from_str(&s).unwrap_or_default()
                        } else {
                            HashMap::new()
                        };

                    AgentToolCall { id: format!("call_{}", i), name: tc.function.name, arguments }
                })
                .collect()
        });

        // Determine stop reason
        let stop_reason = if tool_calls.is_some() {
            AgentStopReason::ToolUse
        } else if response.done_reason.as_deref() == Some("length") {
            AgentStopReason::MaxTokens
        } else {
            AgentStopReason::EndTurn
        };

        Ok(AgentResponse {
            content: if response.message.content.is_empty() {
                None
            } else {
                Some(response.message.content)
            },
            tool_calls,
            stop_reason,
        })
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn supports_tools(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new();
        assert_eq!(AIProvider::name(&provider), "ollama");
    }

    #[test]
    fn test_ollama_with_custom_url() {
        let provider = OllamaProvider::new().with_base_url("http://custom:8080");
        assert_eq!(provider.base_url, "http://custom:8080");
    }

    #[test]
    fn test_ollama_with_custom_model() {
        let provider = OllamaProvider::new().with_model("codellama");
        assert_eq!(provider.model, "codellama");
    }

    #[test]
    fn test_command_prompt_building() {
        let mut context = ProjectContext::new("test-project", PathBuf::from("/project"));
        context.project_type = "node".to_string();
        context.available_commands = vec!["npm run build".to_string(), "npm test".to_string()];

        let prompt = OllamaProvider::build_command_prompt("run tests", &context);

        assert!(prompt.contains("test-project"));
        assert!(prompt.contains("npm run build"));
        assert!(prompt.contains("run tests"));
    }
}
