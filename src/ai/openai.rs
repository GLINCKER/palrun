//! OpenAI API integration.
//!
//! Implements the AIProvider trait for OpenAI GPT models.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AIProvider, ProjectContext};

/// OpenAI API provider.
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider.
    ///
    /// Reads API key from OPENAI_API_KEY environment variable.
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY not set"))?;

        Ok(Self {
            client: Client::new(),
            api_key,
            model: "gpt-4o".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// Create with a specific model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Create with a custom base URL (for Azure OpenAI or compatible APIs).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Make a request to the OpenAI API.
    async fn request(&self, system: &str, user_message: &str) -> anyhow::Result<String> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system.to_string() },
                ChatMessage { role: "user".to_string(), content: user_message.to_string() },
            ],
            max_tokens: Some(1024),
            temperature: Some(0.7),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, body);
        }

        let response: OpenAIResponse = response.json().await?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        let system = format!(
            r"You are Palrun, an AI assistant for terminal commands.
Your task is to generate the exact shell command the user needs.

Current directory: {}
Project type: {}
Available commands: {}

Rules:
1. Output ONLY the command, nothing else
2. Use the correct package manager for this project
3. If multiple commands are needed, join with && or ;
4. Never explain, just output the command",
            context.current_directory.display(),
            context.project_type,
            context.available_commands.join(", ")
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
Explain what this command does in plain English.

Current directory: {}
Project type: {}

Be concise but thorough. Explain each part of the command.",
            context.current_directory.display(),
            context.project_type
        );

        self.request(&system, &format!("Explain: {}", command)).await
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

Current directory: {}
Project type: {}

Be concise. Focus on the most likely cause and solution.",
            context.current_directory.display(),
            context.project_type
        );

        let user_message = format!("Command: {}\n\nError:\n{}", command, error);

        self.request(&system, &user_message).await
    }

    fn name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> bool {
        // Check if we can reach the API
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        response.map(|r| r.status().is_success()).unwrap_or(false)
    }
}

// Request/Response types

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(openai_env)]
    fn test_openai_provider_requires_api_key() {
        // Save current value
        let original = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("OPENAI_API_KEY");

        let result = OpenAIProvider::new();

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("OPENAI_API_KEY", val);
        }

        assert!(result.is_err());
    }

    #[test]
    #[serial(openai_env)]
    fn test_openai_provider_with_model() {
        // Save current value
        let original = std::env::var("OPENAI_API_KEY").ok();
        std::env::set_var("OPENAI_API_KEY", "test-key");

        let provider = OpenAIProvider::new().unwrap().with_model("gpt-4-turbo");
        assert_eq!(provider.model, "gpt-4-turbo");

        // Restore or remove
        match original {
            Some(val) => std::env::set_var("OPENAI_API_KEY", val),
            None => std::env::remove_var("OPENAI_API_KEY"),
        }
    }
}
