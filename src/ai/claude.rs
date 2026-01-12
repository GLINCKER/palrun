//! Claude API integration.
//!
//! Implements the AIProvider trait for Claude.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AIProvider, ProjectContext};

/// Claude API provider.
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    /// Create a new Claude provider.
    ///
    /// Reads API key from ANTHROPIC_API_KEY environment variable.
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;

        Ok(Self { client: Client::new(), api_key, model: "claude-sonnet-4-20250514".to_string() })
    }

    /// Create with a specific model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Make a request to the Claude API.
    async fn request(&self, system: &str, user_message: &str) -> anyhow::Result<String> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            system: system.to_string(),
            messages: vec![Message { role: "user".to_string(), content: user_message.to_string() }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let response: ClaudeResponse = response.json().await?;

        response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Claude"))
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn generate_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let system = format!(
            r"You are Palrun, an AI assistant for terminal commands.
Your task is to generate the exact shell command the user needs.

Project: {}
Type: {}
Available commands: {}
Current directory: {}

Rules:
1. Output ONLY the command, no explanation
2. Use available project commands when possible
3. Be precise and safe
4. Do not include any markdown formatting",
            context.project_name,
            context.project_type,
            context.available_commands.join(", "),
            context.current_directory.display()
        );

        self.request(&system, prompt).await
    }

    async fn explain_command(
        &self,
        command: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let system = format!(
            r"You are Palrun, an AI assistant for terminal commands.
Explain what the following command does in 2-3 sentences.

Project context: {} ({})",
            context.project_name, context.project_type
        );

        let prompt = format!("Explain this command: {command}");
        self.request(&system, &prompt).await
    }

    async fn diagnose_error(
        &self,
        command: &str,
        error: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let system = format!(
            r"You are Palrun, an AI assistant for terminal commands.
Diagnose why this command failed and suggest a fix.

Project: {} ({})
Available commands: {}",
            context.project_name,
            context.project_type,
            context.available_commands.join(", ")
        );

        let prompt =
            format!("Command: {command}\n\nError:\n{error}\n\nWhat went wrong and how to fix it?");
        self.request(&system, &prompt).await
    }

    fn name(&self) -> &str {
        "claude"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Claude API request structure.
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

/// Message in a Claude request.
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Claude API response structure.
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

/// Content block in a Claude response.
#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_claude_provider_creation_fails_without_key() {
        // Clear the env var for this test
        std::env::remove_var("ANTHROPIC_API_KEY");
        let result = ClaudeProvider::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_project_context_creation() {
        let mut context = ProjectContext::new("test", PathBuf::from("."));
        context.project_type = "node".to_string();
        context.available_commands = vec!["npm run build".to_string()];

        assert_eq!(context.project_name, "test");
    }
}
