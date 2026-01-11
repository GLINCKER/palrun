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

mod agent;
mod claude;
mod context;
mod executor;
mod ollama;

pub use agent::{
    mcp_tools_to_agent_tools, Agent, AgentMessage, AgentProvider, AgentResponse, AgentState,
    AgentStopReason, AgentTool, AgentToolCall, AgentToolResult, ToolExecutor,
};
pub use claude::ClaudeProvider;
pub use context::ProjectContext;
pub use executor::{CompositeExecutor, MCPToolExecutor, ShellExecutor};
pub use ollama::OllamaProvider;

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
/// Tries providers in order: Claude (if API key available) -> Ollama (if running) -> None
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

        // Then Ollama (local LLM)
        let ollama = OllamaProvider::new();
        if ollama.is_available().await {
            providers.push(Box::new(ollama));
        }

        Self { providers }
    }

    /// Create with only Ollama (for local-only usage).
    pub fn ollama_only() -> Self {
        Self { providers: vec![Box::new(OllamaProvider::new())] }
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
}
