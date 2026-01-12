//! Azure OpenAI API integration.
//!
//! Implements the AIProvider trait for Azure OpenAI deployments.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{AIProvider, ProjectContext};

/// Azure OpenAI API provider.
pub struct AzureOpenAIProvider {
    client: Client,
    endpoint: String,
    api_key: String,
    deployment: String,
    api_version: String,
}

impl AzureOpenAIProvider {
    /// Create a new Azure OpenAI provider from config or environment variables.
    ///
    /// Environment variables:
    /// - AZURE_OPENAI_ENDPOINT
    /// - AZURE_OPENAI_API_KEY
    /// - AZURE_OPENAI_DEPLOYMENT
    pub fn new() -> anyhow::Result<Self> {
        let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
            .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_ENDPOINT not set"))?;
        let api_key = std::env::var("AZURE_OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_API_KEY not set"))?;
        let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT")
            .map_err(|_| anyhow::anyhow!("AZURE_OPENAI_DEPLOYMENT not set"))?;

        Ok(Self {
            client: Client::new(),
            endpoint,
            api_key,
            deployment,
            api_version: "2024-02-01".to_string(),
        })
    }

    /// Create from explicit config values.
    pub fn from_config(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        deployment: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.into(),
            api_key: api_key.into(),
            deployment: deployment.into(),
            api_version: "2024-02-01".to_string(),
        }
    }

    /// Set the API version.
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    /// Make a request to the Azure OpenAI API.
    async fn request(&self, system: &str, user_message: &str) -> anyhow::Result<String> {
        let request = AzureOpenAIRequest {
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system.to_string() },
                ChatMessage { role: "user".to_string(), content: user_message.to_string() },
            ],
            max_tokens: Some(1024),
            temperature: Some(0.7),
        };

        // Azure OpenAI URL format:
        // {endpoint}/openai/deployments/{deployment}/chat/completions?api-version={api_version}
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.deployment,
            self.api_version
        );

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key) // Azure uses api-key header, not Bearer
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure OpenAI API error ({}): {}", status, body);
        }

        let response: AzureOpenAIResponse = response.json().await?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Azure OpenAI"))
    }
}

#[async_trait]
impl AIProvider for AzureOpenAIProvider {
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
        "azure"
    }

    async fn is_available(&self) -> bool {
        // Check if we can reach the API by making a simple request
        // Azure doesn't have a /models endpoint like OpenAI, so we just check connectivity
        let url = format!(
            "{}/openai/deployments?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.api_version
        );

        let response = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        // Accept 200 OK or 404 (deployment list may not be accessible)
        // The key thing is the API responds, not a network error
        response.map(|r| r.status().is_success() || r.status().as_u16() == 404).unwrap_or(false)
    }
}

// Request/Response types

#[derive(Debug, Serialize)]
struct AzureOpenAIRequest {
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
struct AzureOpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_provider_from_config() {
        let provider = AzureOpenAIProvider::from_config(
            "https://my-resource.openai.azure.com",
            "test-key",
            "gpt-4",
        );
        assert_eq!(provider.endpoint, "https://my-resource.openai.azure.com");
        assert_eq!(provider.deployment, "gpt-4");
        assert_eq!(provider.api_version, "2024-02-01");
    }

    #[test]
    fn test_azure_provider_with_api_version() {
        let provider = AzureOpenAIProvider::from_config(
            "https://my-resource.openai.azure.com",
            "test-key",
            "gpt-4",
        )
        .with_api_version("2024-06-01");
        assert_eq!(provider.api_version, "2024-06-01");
    }
}
