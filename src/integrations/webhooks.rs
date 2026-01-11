//! Outgoing webhooks integration.
//!
//! Sends events to external webhook URLs when specific actions occur in Palrun.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Unique name for this webhook.
    pub name: String,

    /// URL to send webhooks to.
    pub url: String,

    /// HTTP method to use (default: POST).
    #[serde(default = "default_method")]
    pub method: String,

    /// Events that trigger this webhook.
    #[serde(default)]
    pub events: Vec<WebhookEvent>,

    /// Optional filter for commands (glob pattern).
    #[serde(default)]
    pub command_filter: Option<String>,

    /// Custom headers to include.
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Optional secret for signing payloads.
    #[serde(default)]
    pub secret: Option<String>,

    /// Whether the webhook is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Retry count on failure.
    #[serde(default = "default_retries")]
    pub retries: u32,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_enabled() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

/// Events that can trigger a webhook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Command execution started.
    CommandStart,

    /// Command execution completed (success or failure).
    CommandComplete,

    /// Command execution succeeded.
    CommandSuccess,

    /// Command execution failed.
    CommandFailure,

    /// Runbook execution started.
    RunbookStart,

    /// Runbook execution completed.
    RunbookComplete,

    /// AI agent task started.
    AgentStart,

    /// AI agent task completed.
    AgentComplete,

    /// MCP tool called.
    McpToolCall,

    /// All events.
    All,
}

impl WebhookEvent {
    /// Get the event name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CommandStart => "command.start",
            Self::CommandComplete => "command.complete",
            Self::CommandSuccess => "command.success",
            Self::CommandFailure => "command.failure",
            Self::RunbookStart => "runbook.start",
            Self::RunbookComplete => "runbook.complete",
            Self::AgentStart => "agent.start",
            Self::AgentComplete => "agent.complete",
            Self::McpToolCall => "mcp.tool_call",
            Self::All => "*",
        }
    }

    /// Check if this event matches a target event.
    #[must_use]
    pub fn matches(&self, target: Self) -> bool {
        *self == Self::All || *self == target
    }
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Webhook payload sent to the URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type.
    pub event: String,

    /// Timestamp of the event (Unix timestamp).
    pub timestamp: u64,

    /// Event data.
    pub data: WebhookData,

    /// Palrun version.
    pub version: String,
}

/// Event-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookData {
    /// Command event data.
    Command(CommandEventData),

    /// Runbook event data.
    Runbook(RunbookEventData),

    /// Agent event data.
    Agent(AgentEventData),

    /// MCP tool call data.
    McpTool(McpToolEventData),
}

/// Command event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEventData {
    /// Command that was executed.
    pub command: String,

    /// Working directory.
    pub working_dir: String,

    /// Exit code (if completed).
    pub exit_code: Option<i32>,

    /// Duration in milliseconds (if completed).
    pub duration_ms: Option<u64>,

    /// Whether the command succeeded.
    pub success: Option<bool>,

    /// Error message (if failed).
    pub error: Option<String>,

    /// Output (truncated to 10KB).
    pub output: Option<String>,
}

/// Runbook event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookEventData {
    /// Runbook name.
    pub runbook: String,

    /// Number of steps.
    pub total_steps: usize,

    /// Completed steps (if in progress).
    pub completed_steps: Option<usize>,

    /// Whether the runbook succeeded.
    pub success: Option<bool>,

    /// Error message (if failed).
    pub error: Option<String>,
}

/// Agent event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEventData {
    /// Task description.
    pub task: String,

    /// Number of iterations.
    pub iterations: usize,

    /// Tool calls made.
    pub tool_calls: Vec<String>,

    /// Final result.
    pub result: Option<String>,

    /// Whether the task succeeded.
    pub success: Option<bool>,
}

/// MCP tool call event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolEventData {
    /// Tool name.
    pub tool: String,

    /// Server name.
    pub server: String,

    /// Arguments (redacted if sensitive).
    pub arguments: HashMap<String, serde_json::Value>,

    /// Result (truncated).
    pub result: Option<String>,

    /// Whether the call succeeded.
    pub success: bool,
}

/// Webhook delivery result.
#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    /// Webhook name.
    pub webhook_name: String,

    /// Event that triggered the webhook.
    pub event: WebhookEvent,

    /// HTTP status code.
    pub status_code: Option<u16>,

    /// Whether the delivery succeeded.
    pub success: bool,

    /// Error message (if failed).
    pub error: Option<String>,

    /// Duration in milliseconds.
    pub duration_ms: u64,

    /// Number of retry attempts.
    pub retries: u32,
}

/// Webhook error types.
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Webhook URL is invalid: {0}")]
    InvalidUrl(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Webhook not found: {0}")]
    NotFound(String),

    #[error("Webhook disabled: {0}")]
    Disabled(String),

    #[error("All retries exhausted")]
    RetriesExhausted,
}

/// Result type for webhook operations.
pub type WebhookResult<T> = Result<T, WebhookError>;

/// Webhook manager for sending events.
pub struct WebhookManager {
    /// Configured webhooks.
    webhooks: Vec<WebhookConfig>,

    /// HTTP client.
    client: reqwest::blocking::Client,

    /// Delivery history (recent).
    history: Vec<WebhookDelivery>,

    /// Maximum history size.
    max_history: usize,
}

impl WebhookManager {
    /// Create a new webhook manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            webhooks: Vec::new(),
            client: reqwest::blocking::Client::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Create with specific webhooks.
    #[must_use]
    pub fn with_webhooks(webhooks: Vec<WebhookConfig>) -> Self {
        Self {
            webhooks,
            client: reqwest::blocking::Client::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Add a webhook configuration.
    pub fn add_webhook(&mut self, config: WebhookConfig) {
        self.webhooks.push(config);
    }

    /// Remove a webhook by name.
    pub fn remove_webhook(&mut self, name: &str) -> bool {
        let len_before = self.webhooks.len();
        self.webhooks.retain(|w| w.name != name);
        self.webhooks.len() < len_before
    }

    /// Get all configured webhooks.
    #[must_use]
    pub fn webhooks(&self) -> &[WebhookConfig] {
        &self.webhooks
    }

    /// Get webhooks for a specific event.
    #[must_use]
    pub fn webhooks_for_event(&self, event: WebhookEvent) -> Vec<&WebhookConfig> {
        self.webhooks
            .iter()
            .filter(|w| w.enabled && w.events.iter().any(|e| e.matches(event)))
            .collect()
    }

    /// Send an event to all matching webhooks.
    pub fn send_event(&mut self, event: WebhookEvent, data: WebhookData) -> Vec<WebhookDelivery> {
        let matching = self.webhooks_for_event(event);
        let mut deliveries = Vec::new();

        for webhook in matching {
            let delivery = self.send_to_webhook(webhook, event, data.clone());
            deliveries.push(delivery);
        }

        // Add to history
        for delivery in &deliveries {
            self.history.push(delivery.clone());
            if self.history.len() > self.max_history {
                self.history.remove(0);
            }
        }

        deliveries
    }

    /// Send to a specific webhook.
    fn send_to_webhook(
        &self,
        webhook: &WebhookConfig,
        event: WebhookEvent,
        data: WebhookData,
    ) -> WebhookDelivery {
        let start = std::time::Instant::now();
        let mut retries = 0;
        let mut last_error = None;
        let mut status_code = None;

        let payload = WebhookPayload {
            event: event.name().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            data,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let body = match serde_json::to_string(&payload) {
            Ok(b) => b,
            Err(e) => {
                return WebhookDelivery {
                    webhook_name: webhook.name.clone(),
                    event,
                    status_code: None,
                    success: false,
                    error: Some(format!("Serialization failed: {}", e)),
                    duration_ms: start.elapsed().as_millis() as u64,
                    retries: 0,
                };
            }
        };

        // Retry loop
        while retries <= webhook.retries {
            let mut request = match webhook.method.to_uppercase().as_str() {
                "POST" => self.client.post(&webhook.url),
                "PUT" => self.client.put(&webhook.url),
                "PATCH" => self.client.patch(&webhook.url),
                _ => self.client.post(&webhook.url),
            };

            // Add headers
            request = request.header("Content-Type", "application/json");
            request = request.header("User-Agent", format!("palrun/{}", env!("CARGO_PKG_VERSION")));
            request = request.header("X-Palrun-Event", event.name());

            for (key, value) in &webhook.headers {
                request = request.header(key.as_str(), value.as_str());
            }

            // Add signature if secret is configured
            if let Some(ref secret) = webhook.secret {
                let signature = compute_signature(secret, &body);
                request = request.header("X-Palrun-Signature", signature);
            }

            // Set timeout
            request = request.timeout(Duration::from_secs(webhook.timeout_secs));

            // Send request
            match request.body(body.clone()).send() {
                Ok(response) => {
                    status_code = Some(response.status().as_u16());
                    if response.status().is_success() {
                        return WebhookDelivery {
                            webhook_name: webhook.name.clone(),
                            event,
                            status_code,
                            success: true,
                            error: None,
                            duration_ms: start.elapsed().as_millis() as u64,
                            retries,
                        };
                    } else {
                        last_error = Some(format!("HTTP {}", response.status()));
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }

            retries += 1;
            if retries <= webhook.retries {
                // Exponential backoff
                std::thread::sleep(Duration::from_millis(100 * 2u64.pow(retries - 1)));
            }
        }

        WebhookDelivery {
            webhook_name: webhook.name.clone(),
            event,
            status_code,
            success: false,
            error: last_error,
            duration_ms: start.elapsed().as_millis() as u64,
            retries: retries.saturating_sub(1),
        }
    }

    /// Get delivery history.
    #[must_use]
    pub fn history(&self) -> &[WebhookDelivery] {
        &self.history
    }

    /// Clear delivery history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Test a webhook with a test payload.
    pub fn test_webhook(&self, name: &str) -> WebhookResult<WebhookDelivery> {
        let webhook = self
            .webhooks
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| WebhookError::NotFound(name.to_string()))?;

        if !webhook.enabled {
            return Err(WebhookError::Disabled(name.to_string()));
        }

        let test_data = WebhookData::Command(CommandEventData {
            command: "echo 'test webhook'".to_string(),
            working_dir: "/tmp".to_string(),
            exit_code: Some(0),
            duration_ms: Some(100),
            success: Some(true),
            error: None,
            output: Some("test webhook".to_string()),
        });

        Ok(self.send_to_webhook(webhook, WebhookEvent::CommandComplete, test_data))
    }
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute HMAC-SHA256 signature for payload.
fn compute_signature(secret: &str, body: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(body.as_bytes());
    let result = hasher.finalize();

    format!("sha256={:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_names() {
        assert_eq!(WebhookEvent::CommandStart.name(), "command.start");
        assert_eq!(WebhookEvent::CommandComplete.name(), "command.complete");
        assert_eq!(WebhookEvent::All.name(), "*");
    }

    #[test]
    fn test_webhook_event_matches() {
        assert!(WebhookEvent::All.matches(WebhookEvent::CommandStart));
        assert!(WebhookEvent::CommandStart.matches(WebhookEvent::CommandStart));
        assert!(!WebhookEvent::CommandStart.matches(WebhookEvent::CommandComplete));
    }

    #[test]
    fn test_webhook_manager_creation() {
        let manager = WebhookManager::new();
        assert!(manager.webhooks().is_empty());
    }

    #[test]
    fn test_webhook_config_defaults() {
        let json = r#"{
            "name": "test",
            "url": "https://example.com/webhook",
            "events": ["command_complete"]
        }"#;

        let config: WebhookConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.method, "POST");
        assert!(config.enabled);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.retries, 3);
    }

    #[test]
    fn test_add_remove_webhook() {
        let mut manager = WebhookManager::new();

        manager.add_webhook(WebhookConfig {
            name: "test".to_string(),
            url: "https://example.com".to_string(),
            method: "POST".to_string(),
            events: vec![WebhookEvent::CommandComplete],
            command_filter: None,
            headers: HashMap::new(),
            secret: None,
            enabled: true,
            timeout_secs: 30,
            retries: 3,
        });

        assert_eq!(manager.webhooks().len(), 1);

        assert!(manager.remove_webhook("test"));
        assert!(manager.webhooks().is_empty());
    }

    #[test]
    fn test_webhooks_for_event() {
        let mut manager = WebhookManager::new();

        manager.add_webhook(WebhookConfig {
            name: "command-hook".to_string(),
            url: "https://example.com/commands".to_string(),
            method: "POST".to_string(),
            events: vec![WebhookEvent::CommandComplete],
            command_filter: None,
            headers: HashMap::new(),
            secret: None,
            enabled: true,
            timeout_secs: 30,
            retries: 3,
        });

        manager.add_webhook(WebhookConfig {
            name: "all-hook".to_string(),
            url: "https://example.com/all".to_string(),
            method: "POST".to_string(),
            events: vec![WebhookEvent::All],
            command_filter: None,
            headers: HashMap::new(),
            secret: None,
            enabled: true,
            timeout_secs: 30,
            retries: 3,
        });

        let matching = manager.webhooks_for_event(WebhookEvent::CommandComplete);
        assert_eq!(matching.len(), 2);

        let matching = manager.webhooks_for_event(WebhookEvent::RunbookStart);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].name, "all-hook");
    }

    #[test]
    fn test_compute_signature() {
        let sig = compute_signature("secret", "body");
        assert!(sig.starts_with("sha256="));
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event: "command.complete".to_string(),
            timestamp: 1704067200,
            data: WebhookData::Command(CommandEventData {
                command: "npm test".to_string(),
                working_dir: "/project".to_string(),
                exit_code: Some(0),
                duration_ms: Some(5000),
                success: Some(true),
                error: None,
                output: Some("All tests passed".to_string()),
            }),
            version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("command.complete"));
        assert!(json.contains("npm test"));
    }
}
