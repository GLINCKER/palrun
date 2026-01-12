//! AI integration module.
//!
//! Provides natural language to command translation using Claude or local LLMs.
//!
//! ## Features
//!
//! - Command generation from natural language
//! - Command explanation
//! - Error diagnosis
//! - **Agentic tool use** - AI can use MCP tools autonomously
//! - **Model routing** - Route tasks to optimal models

mod agent;
mod azure;
mod claude;
mod context;
mod executor;
mod grok;
mod ollama;
mod openai;
mod routing;

pub use agent::{
    mcp_tools_to_agent_tools, Agent, AgentMessage, AgentProvider, AgentResponse, AgentState,
    AgentStopReason, AgentTool, AgentToolCall, AgentToolResult, ToolExecutor,
};
pub use azure::AzureOpenAIProvider;
pub use claude::ClaudeProvider;
pub use context::ProjectContext;
pub use executor::{CompositeExecutor, MCPToolExecutor, ShellExecutor};
pub use grok::GrokProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use routing::{FallbackChain, ModelRouter, RoutingConfig, RoutingDecision, TaskCategory};

use async_trait::async_trait;

/// Trait for AI providers.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a command from natural language.
    async fn generate_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String>;

    /// Explain what a command does.
    async fn explain_command(
        &self,
        command: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String>;

    /// Diagnose why a command failed.
    async fn diagnose_error(
        &self,
        command: &str,
        error: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String>;

    /// Get the provider name.
    fn name(&self) -> &str;

    /// Check if the provider is available.
    async fn is_available(&self) -> bool;
}

/// AI error types.
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limited, retry after {0}s")]
    RateLimited(u64),

    #[error("Context too large")]
    ContextTooLarge,

    #[error("No response from AI")]
    NoResponse,
}

/// AI provider manager with fallback support.
///
/// Tries providers in order based on availability:
/// 1. Claude (if ANTHROPIC_API_KEY set)
/// 2. OpenAI (if OPENAI_API_KEY set)
/// 3. Azure (if AZURE_OPENAI_* vars set)
/// 4. Grok (if XAI_API_KEY set)
/// 5. Ollama (if running locally)
pub struct AIManager {
    providers: Vec<Box<dyn AIProvider>>,
}

impl AIManager {
    /// Create a new AI manager with default provider chain.
    pub async fn new() -> Self {
        let mut providers: Vec<Box<dyn AIProvider>> = Vec::new();

        // Try Claude first (requires API key)
        if let Ok(claude) = ClaudeProvider::new() {
            if claude.is_available().await {
                providers.push(Box::new(claude));
            }
        }

        // Then OpenAI (requires API key)
        if let Ok(openai) = OpenAIProvider::new() {
            if openai.is_available().await {
                providers.push(Box::new(openai));
            }
        }

        // Then Azure OpenAI (requires endpoint + key + deployment)
        if let Ok(azure) = AzureOpenAIProvider::new() {
            if azure.is_available().await {
                providers.push(Box::new(azure));
            }
        }

        // Then Grok (requires API key)
        if let Ok(grok) = GrokProvider::new() {
            if grok.is_available().await {
                providers.push(Box::new(grok));
            }
        }

        // Finally Ollama (local LLM, always available if running)
        let ollama = OllamaProvider::new();
        if ollama.is_available().await {
            providers.push(Box::new(ollama));
        }

        Self { providers }
    }

    /// Create with a specific provider.
    pub fn with_provider(provider: impl Into<String>) -> anyhow::Result<Self> {
        let provider_name = provider.into();
        let provider: Box<dyn AIProvider> = match provider_name.as_str() {
            "claude" => Box::new(ClaudeProvider::new()?),
            "openai" => Box::new(OpenAIProvider::new()?),
            "azure" => Box::new(AzureOpenAIProvider::new()?),
            "grok" => Box::new(GrokProvider::new()?),
            "ollama" => Box::new(OllamaProvider::new()),
            other => anyhow::bail!("Unknown provider: {}", other),
        };
        Ok(Self { providers: vec![provider] })
    }

    /// Create with only Ollama (for local-only usage).
    pub fn ollama_only() -> Self {
        Self { providers: vec![Box::new(OllamaProvider::new())] }
    }

    /// List all available providers.
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Check if any AI provider is available.
    pub fn is_available(&self) -> bool {
        !self.providers.is_empty()
    }

    /// Get the active provider name.
    pub fn active_provider(&self) -> Option<&str> {
        self.providers.first().map(|p| p.name())
    }

    /// Generate a command from natural language.
    pub async fn generate_command(
        &self,
        prompt: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        for provider in &self.providers {
            match provider.generate_command(prompt, context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(provider = provider.name(), error = %e, "Provider failed, trying next");
                }
            }
        }

        Err(AIError::ProviderNotAvailable("No AI provider available".to_string()).into())
    }

    /// Explain what a command does.
    pub async fn explain_command(
        &self,
        command: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        for provider in &self.providers {
            match provider.explain_command(command, context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(provider = provider.name(), error = %e, "Provider failed, trying next");
                }
            }
        }

        Err(AIError::ProviderNotAvailable("No AI provider available".to_string()).into())
    }

    /// Diagnose why a command failed.
    pub async fn diagnose_error(
        &self,
        command: &str,
        error: &str,
        context: &ProjectContext,
    ) -> anyhow::Result<String> {
        for provider in &self.providers {
            match provider.diagnose_error(command, error, context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(provider = provider.name(), error = %e, "Provider failed, trying next");
                }
            }
        }

        Err(AIError::ProviderNotAvailable("No AI provider available".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_manager_ollama_only() {
        let manager = AIManager::ollama_only();
        assert_eq!(manager.active_provider(), Some("ollama"));
    }

    #[test]
    fn test_ai_manager_with_provider() {
        // Test with ollama (doesn't require API key)
        let manager = AIManager::with_provider("ollama").unwrap();
        assert_eq!(manager.active_provider(), Some("ollama"));
    }

    #[test]
    fn test_ai_manager_with_invalid_provider() {
        let result = AIManager::with_provider("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_available_providers() {
        let manager = AIManager::ollama_only();
        let providers = manager.available_providers();
        assert_eq!(providers, vec!["ollama"]);
    }
}
