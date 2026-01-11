//! Ollama local LLM integration.
//!
//! Implements the AIProvider trait for Ollama (local LLM).

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

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

/// Ollama API request structure.
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama API response structure.
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.name(), "ollama");
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
        let context = ProjectContext {
            project_name: "test-project".to_string(),
            project_type: "node".to_string(),
            available_commands: vec!["npm run build".to_string(), "npm test".to_string()],
            current_directory: PathBuf::from("/project"),
            recent_commands: vec![],
        };

        let prompt = OllamaProvider::build_command_prompt("run tests", &context);

        assert!(prompt.contains("test-project"));
        assert!(prompt.contains("npm run build"));
        assert!(prompt.contains("run tests"));
    }
}
