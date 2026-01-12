//! AI Model Routing Engine.
//!
//! Intelligently routes tasks to the optimal AI model based on task type,
//! cost, performance, and availability.

use serde::{Deserialize, Serialize};

use super::{AIProvider, ClaudeProvider, GrokProvider, OllamaProvider, OpenAIProvider};

/// Task category for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskCategory {
    /// Strategic planning (roadmap, architecture)
    Planning,

    /// Writing new code
    CodeGeneration,

    /// Reviewing/analyzing existing code
    CodeReview,

    /// Quick tasks (simple queries, short responses)
    QuickTask,

    /// Writing documentation
    Documentation,

    /// Error diagnosis
    ErrorDiagnosis,
}

impl TaskCategory {
    /// Infer category from prompt content.
    pub fn from_prompt(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();

        if lower.contains("plan") || lower.contains("roadmap") || lower.contains("architect") {
            Self::Planning
        } else if lower.contains("review") || lower.contains("analyze") || lower.contains("check") {
            Self::CodeReview
        } else if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("explain")
        {
            Self::Documentation
        } else if lower.contains("error") || lower.contains("fix") || lower.contains("debug") {
            Self::ErrorDiagnosis
        } else if lower.contains("write") || lower.contains("implement") || lower.contains("create")
        {
            Self::CodeGeneration
        } else if prompt.len() < 100 {
            Self::QuickTask
        } else {
            Self::CodeGeneration
        }
    }

    /// Get the default model for this category.
    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Planning => "claude",
            Self::CodeGeneration => "claude",
            Self::CodeReview => "claude",
            Self::QuickTask => "ollama",
            Self::Documentation => "claude",
            Self::ErrorDiagnosis => "claude",
        }
    }
}

/// Routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Model for planning tasks
    #[serde(default = "default_planning")]
    pub planning: String,

    /// Model for code generation
    #[serde(default = "default_code_generation")]
    pub code_generation: String,

    /// Model for code review
    #[serde(default = "default_code_review")]
    pub code_review: String,

    /// Model for quick tasks
    #[serde(default = "default_quick_tasks")]
    pub quick_tasks: String,

    /// Model for documentation
    #[serde(default = "default_documentation")]
    pub documentation: String,

    /// Model for error diagnosis
    #[serde(default = "default_error_diagnosis")]
    pub error_diagnosis: String,

    /// Fallback model when primary fails
    #[serde(default = "default_fallback")]
    pub fallback: String,

    /// Local model for offline/budget mode
    #[serde(default = "default_local")]
    pub local: String,
}

fn default_planning() -> String {
    "claude".to_string()
}
fn default_code_generation() -> String {
    "claude".to_string()
}
fn default_code_review() -> String {
    "claude".to_string()
}
fn default_quick_tasks() -> String {
    "ollama".to_string()
}
fn default_documentation() -> String {
    "claude".to_string()
}
fn default_error_diagnosis() -> String {
    "claude".to_string()
}
fn default_fallback() -> String {
    "openai".to_string()
}
fn default_local() -> String {
    "ollama".to_string()
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            planning: default_planning(),
            code_generation: default_code_generation(),
            code_review: default_code_review(),
            quick_tasks: default_quick_tasks(),
            documentation: default_documentation(),
            error_diagnosis: default_error_diagnosis(),
            fallback: default_fallback(),
            local: default_local(),
        }
    }
}

impl RoutingConfig {
    /// Get the model name for a task category.
    pub fn model_for(&self, category: TaskCategory) -> &str {
        match category {
            TaskCategory::Planning => &self.planning,
            TaskCategory::CodeGeneration => &self.code_generation,
            TaskCategory::CodeReview => &self.code_review,
            TaskCategory::QuickTask => &self.quick_tasks,
            TaskCategory::Documentation => &self.documentation,
            TaskCategory::ErrorDiagnosis => &self.error_diagnosis,
        }
    }
}

/// Model router that selects the best provider for each task.
pub struct ModelRouter {
    config: RoutingConfig,
    providers: Vec<(String, Box<dyn AIProvider>)>,
}

impl ModelRouter {
    /// Create a new router with default configuration.
    pub async fn new() -> Self {
        Self::with_config(RoutingConfig::default()).await
    }

    /// Create a router with custom configuration.
    pub async fn with_config(config: RoutingConfig) -> Self {
        let mut providers: Vec<(String, Box<dyn AIProvider>)> = Vec::new();

        // Try to initialize each provider
        if let Ok(claude) = ClaudeProvider::new() {
            if claude.is_available().await {
                providers.push(("claude".to_string(), Box::new(claude)));
            }
        }

        if let Ok(openai) = OpenAIProvider::new() {
            if openai.is_available().await {
                providers.push(("openai".to_string(), Box::new(openai)));
            }
        }

        if let Ok(grok) = GrokProvider::new() {
            if grok.is_available().await {
                providers.push(("grok".to_string(), Box::new(grok)));
            }
        }

        let ollama = OllamaProvider::new();
        if ollama.is_available().await {
            providers.push(("ollama".to_string(), Box::new(ollama)));
        }

        Self { config, providers }
    }

    /// Select the best provider for a task category.
    pub fn select(&self, category: TaskCategory) -> Option<&dyn AIProvider> {
        let model_name = self.config.model_for(category);
        self.get_provider(model_name)
    }

    /// Get a provider by name.
    pub fn get_provider(&self, name: &str) -> Option<&dyn AIProvider> {
        self.providers.iter().find(|(n, _)| n == name).map(|(_, p)| p.as_ref())
    }

    /// Get a fallback chain for a task category.
    pub fn fallback_chain(&self, category: TaskCategory) -> FallbackChain<'_> {
        let mut chain = Vec::new();

        // Primary model for this category
        if let Some(primary) = self.select(category) {
            chain.push(primary);
        }

        // Fallback model
        if let Some(fallback) = self.get_provider(&self.config.fallback) {
            if !chain.iter().any(|p| p.name() == fallback.name()) {
                chain.push(fallback);
            }
        }

        // Local model as last resort
        if let Some(local) = self.get_provider(&self.config.local) {
            if !chain.iter().any(|p| p.name() == local.name()) {
                chain.push(local);
            }
        }

        FallbackChain::new(chain)
    }

    /// List available providers.
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Check if a specific provider is available.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.iter().any(|(n, _)| n == name)
    }
}

/// A chain of providers to try in order.
pub struct FallbackChain<'a> {
    providers: Vec<&'a dyn AIProvider>,
    current_index: usize,
}

impl<'a> FallbackChain<'a> {
    /// Create a new fallback chain.
    pub fn new(providers: Vec<&'a dyn AIProvider>) -> Self {
        Self { providers, current_index: 0 }
    }

    /// Get the current provider.
    pub fn current(&self) -> Option<&'a dyn AIProvider> {
        self.providers.get(self.current_index).copied()
    }

    /// Move to the next provider.
    pub fn next(&mut self) -> Option<&'a dyn AIProvider> {
        self.current_index += 1;
        self.current()
    }

    /// Reset to the first provider.
    pub fn reset(&mut self) {
        self.current_index = 0;
    }

    /// Get all providers in the chain.
    pub fn providers(&self) -> &[&'a dyn AIProvider] {
        &self.providers
    }

    /// Execute a request with fallback.
    pub async fn execute<F, T, E>(&mut self, mut f: F) -> Result<T, E>
    where
        F: FnMut(
            &'a dyn AIProvider,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>>,
        E: std::fmt::Display,
    {
        self.reset();
        while let Some(provider) = self.current() {
            match f(provider).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(
                        provider = provider.name(),
                        error = %e,
                        "Provider failed, trying next"
                    );
                    if self.next().is_none() {
                        return Err(e);
                    }
                }
            }
        }
        unreachable!("Chain should have at least one provider")
    }
}

/// Routing decision with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Selected provider name
    pub provider: String,

    /// Task category
    pub category: TaskCategory,

    /// Reason for selection
    pub reason: String,

    /// Alternative providers available
    pub alternatives: Vec<String>,
}

impl RoutingDecision {
    /// Create a new routing decision.
    pub fn new(provider: &str, category: TaskCategory, alternatives: Vec<&str>) -> Self {
        Self {
            provider: provider.to_string(),
            category,
            reason: format!("{} is the configured model for {:?} tasks", provider, category),
            alternatives: alternatives.into_iter().map(String::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_category_from_prompt() {
        assert_eq!(TaskCategory::from_prompt("plan the architecture"), TaskCategory::Planning);
        assert_eq!(TaskCategory::from_prompt("review this code"), TaskCategory::CodeReview);
        assert_eq!(TaskCategory::from_prompt("write documentation"), TaskCategory::Documentation);
        assert_eq!(TaskCategory::from_prompt("fix this error"), TaskCategory::ErrorDiagnosis);
        assert_eq!(TaskCategory::from_prompt("implement feature"), TaskCategory::CodeGeneration);
        assert_eq!(TaskCategory::from_prompt("hi"), TaskCategory::QuickTask);
    }

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.planning, "claude");
        assert_eq!(config.quick_tasks, "ollama");
        assert_eq!(config.fallback, "openai");
    }

    #[test]
    fn test_routing_config_model_for() {
        let config = RoutingConfig::default();
        assert_eq!(config.model_for(TaskCategory::Planning), "claude");
        assert_eq!(config.model_for(TaskCategory::QuickTask), "ollama");
    }

    #[test]
    fn test_task_category_default_model() {
        assert_eq!(TaskCategory::Planning.default_model(), "claude");
        assert_eq!(TaskCategory::QuickTask.default_model(), "ollama");
    }

    #[test]
    fn test_routing_decision() {
        let decision =
            RoutingDecision::new("claude", TaskCategory::Planning, vec!["openai", "ollama"]);
        assert_eq!(decision.provider, "claude");
        assert_eq!(decision.category, TaskCategory::Planning);
        assert_eq!(decision.alternatives.len(), 2);
    }
}
