//! Notification services integration.
//!
//! Provides webhook-based notifications to Slack, Discord, and custom endpoints.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Notification service type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    /// Slack incoming webhook
    Slack,
    /// Discord webhook
    Discord,
    /// Generic HTTP webhook
    Webhook,
}

impl NotificationType {
    /// Get the display name for this notification type.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Slack => "Slack",
            Self::Discord => "Discord",
            Self::Webhook => "Webhook",
        }
    }

    /// Get an icon for this notification type.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Slack => "#",
            Self::Discord => "D",
            Self::Webhook => "W",
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Event that can trigger a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationEvent {
    /// Command execution started
    CommandStart,
    /// Command execution completed (success or failure)
    CommandComplete,
    /// Command execution succeeded
    CommandSuccess,
    /// Command execution failed
    CommandFailure,
    /// Background command completed
    BackgroundComplete,
    /// CI status changed
    CiStatusChange,
    /// Custom event
    Custom,
}

impl NotificationEvent {
    /// Get the display name for this event.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::CommandStart => "command_start",
            Self::CommandComplete => "command_complete",
            Self::CommandSuccess => "command_success",
            Self::CommandFailure => "command_failure",
            Self::BackgroundComplete => "background_complete",
            Self::CiStatusChange => "ci_status_change",
            Self::Custom => "custom",
        }
    }
}

impl std::fmt::Display for NotificationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Unique name for this notification endpoint.
    pub name: String,

    /// Type of notification service.
    #[serde(rename = "type")]
    pub notification_type: NotificationType,

    /// Webhook URL.
    pub webhook_url: String,

    /// Events that trigger this notification.
    #[serde(default)]
    pub events: Vec<NotificationEvent>,

    /// Optional filter pattern for commands (glob pattern).
    #[serde(default)]
    pub filter: Option<String>,

    /// Whether this notification is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Custom headers for webhook requests.
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl NotificationConfig {
    /// Create a new Slack notification config.
    pub fn slack(name: impl Into<String>, webhook_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notification_type: NotificationType::Slack,
            webhook_url: webhook_url.into(),
            events: vec![NotificationEvent::CommandComplete],
            filter: None,
            enabled: true,
            headers: HashMap::new(),
        }
    }

    /// Create a new Discord notification config.
    pub fn discord(name: impl Into<String>, webhook_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notification_type: NotificationType::Discord,
            webhook_url: webhook_url.into(),
            events: vec![NotificationEvent::CommandComplete],
            filter: None,
            enabled: true,
            headers: HashMap::new(),
        }
    }

    /// Create a new generic webhook notification config.
    pub fn webhook(name: impl Into<String>, webhook_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            notification_type: NotificationType::Webhook,
            webhook_url: webhook_url.into(),
            events: vec![NotificationEvent::CommandComplete],
            filter: None,
            enabled: true,
            headers: HashMap::new(),
        }
    }

    /// Set the events that trigger this notification.
    pub fn with_events(mut self, events: Vec<NotificationEvent>) -> Self {
        self.events = events;
        self
    }

    /// Set a filter pattern for commands.
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Check if this notification matches an event and command.
    pub fn matches(&self, event: NotificationEvent, command: Option<&str>) -> bool {
        if !self.enabled {
            return false;
        }

        if !self.events.contains(&event) {
            return false;
        }

        if let (Some(filter), Some(cmd)) = (&self.filter, command) {
            // Simple glob matching
            if filter.contains('*') {
                let pattern = filter.replace('*', "");
                if filter.starts_with('*') && filter.ends_with('*') {
                    return cmd.contains(&pattern);
                } else if filter.starts_with('*') {
                    return cmd.ends_with(&pattern);
                } else if filter.ends_with('*') {
                    return cmd.starts_with(&pattern);
                }
            }
            return cmd.contains(filter);
        }

        true
    }
}

/// Notification message to send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// Message title (used in rich messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Message text content.
    pub text: String,

    /// Message color (hex color code).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Additional fields (key-value pairs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<MessageField>,

    /// Whether this is an error/failure message.
    #[serde(default)]
    pub is_error: bool,
}

/// A field in a notification message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageField {
    /// Field name.
    pub name: String,
    /// Field value.
    pub value: String,
    /// Whether to display inline (for rich messages).
    #[serde(default)]
    pub inline: bool,
}

impl NotificationMessage {
    /// Create a simple text message.
    pub fn text(text: impl Into<String>) -> Self {
        Self { title: None, text: text.into(), color: None, fields: Vec::new(), is_error: false }
    }

    /// Create a message with a title.
    pub fn with_title(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            text: text.into(),
            color: None,
            fields: Vec::new(),
            is_error: false,
        }
    }

    /// Set the message color.
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Add a field to the message.
    pub fn add_field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        self.fields.push(MessageField { name: name.into(), value: value.into(), inline });
        self
    }

    /// Mark this as an error message.
    pub fn error(mut self) -> Self {
        self.is_error = true;
        if self.color.is_none() {
            self.color = Some("#dc3545".to_string()); // Red
        }
        self
    }

    /// Mark this as a success message.
    pub fn success(mut self) -> Self {
        self.is_error = false;
        if self.color.is_none() {
            self.color = Some("#28a745".to_string()); // Green
        }
        self
    }

    /// Create a command completion message.
    pub fn command_completed(command: &str, success: bool, duration: Option<Duration>) -> Self {
        let status = if success { "succeeded" } else { "failed" };
        let emoji = if success { ":white_check_mark:" } else { ":x:" };

        let mut msg = Self::with_title(
            format!("Command {}", status),
            format!("{} `{}` {}", emoji, command, status),
        );

        if let Some(dur) = duration {
            msg = msg.add_field("Duration", format!("{:.2}s", dur.as_secs_f64()), true);
        }

        if success {
            msg.success()
        } else {
            msg.error()
        }
    }
}

/// Error type for notification operations.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Webhook returned an error.
    #[error("Webhook error: {status} - {message}")]
    Webhook { status: u16, message: String },

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for notification operations.
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification client for sending messages.
pub struct NotificationClient {
    /// HTTP client.
    client: reqwest::blocking::Client,
}

impl NotificationClient {
    /// Create a new notification client.
    pub fn new() -> NotificationResult<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(format!("palrun/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { client })
    }

    /// Send a notification.
    pub fn send(
        &self,
        config: &NotificationConfig,
        message: &NotificationMessage,
    ) -> NotificationResult<()> {
        if !config.enabled {
            return Ok(());
        }

        match config.notification_type {
            NotificationType::Slack => self.send_slack(config, message),
            NotificationType::Discord => self.send_discord(config, message),
            NotificationType::Webhook => self.send_webhook(config, message),
        }
    }

    /// Send a Slack message.
    fn send_slack(
        &self,
        config: &NotificationConfig,
        message: &NotificationMessage,
    ) -> NotificationResult<()> {
        // Build Slack message payload
        let payload = self.build_slack_payload(message);

        let mut request = self.client.post(&config.webhook_url).json(&payload);

        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        let response = request.send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::Webhook { status, message })
        }
    }

    /// Build Slack message payload.
    fn build_slack_payload(&self, message: &NotificationMessage) -> serde_json::Value {
        if message.fields.is_empty() && message.title.is_none() && message.color.is_none() {
            // Simple text message
            serde_json::json!({
                "text": message.text
            })
        } else {
            // Rich message with attachment
            let mut attachment = serde_json::json!({
                "text": message.text,
                "mrkdwn_in": ["text"]
            });

            if let Some(ref title) = message.title {
                attachment["title"] = serde_json::json!(title);
            }

            if let Some(ref color) = message.color {
                attachment["color"] = serde_json::json!(color);
            }

            if !message.fields.is_empty() {
                let fields: Vec<serde_json::Value> = message
                    .fields
                    .iter()
                    .map(|f| {
                        serde_json::json!({
                            "title": f.name,
                            "value": f.value,
                            "short": f.inline
                        })
                    })
                    .collect();
                attachment["fields"] = serde_json::json!(fields);
            }

            serde_json::json!({
                "attachments": [attachment]
            })
        }
    }

    /// Send a Discord message.
    fn send_discord(
        &self,
        config: &NotificationConfig,
        message: &NotificationMessage,
    ) -> NotificationResult<()> {
        // Build Discord message payload
        let payload = self.build_discord_payload(message);

        let mut request = self.client.post(&config.webhook_url).json(&payload);

        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        let response = request.send()?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::Webhook { status, message })
        }
    }

    /// Build Discord message payload.
    fn build_discord_payload(&self, message: &NotificationMessage) -> serde_json::Value {
        if message.fields.is_empty() && message.title.is_none() && message.color.is_none() {
            // Simple text message
            serde_json::json!({
                "content": message.text
            })
        } else {
            // Rich embed message
            let mut embed = serde_json::json!({
                "description": message.text
            });

            if let Some(ref title) = message.title {
                embed["title"] = serde_json::json!(title);
            }

            if let Some(ref color) = message.color {
                // Convert hex color to decimal
                if let Some(decimal) = hex_to_decimal(color) {
                    embed["color"] = serde_json::json!(decimal);
                }
            }

            if !message.fields.is_empty() {
                let fields: Vec<serde_json::Value> = message
                    .fields
                    .iter()
                    .map(|f| {
                        serde_json::json!({
                            "name": f.name,
                            "value": f.value,
                            "inline": f.inline
                        })
                    })
                    .collect();
                embed["fields"] = serde_json::json!(fields);
            }

            serde_json::json!({
                "embeds": [embed]
            })
        }
    }

    /// Send a generic webhook.
    fn send_webhook(
        &self,
        config: &NotificationConfig,
        message: &NotificationMessage,
    ) -> NotificationResult<()> {
        let payload = serde_json::json!({
            "event": "notification",
            "title": message.title,
            "text": message.text,
            "color": message.color,
            "is_error": message.is_error,
            "fields": message.fields,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let mut request = self.client.post(&config.webhook_url).json(&payload);

        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        let response = request.send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(NotificationError::Webhook { status, message })
        }
    }

    /// Send a notification to multiple endpoints.
    pub fn send_to_all(
        &self,
        configs: &[NotificationConfig],
        event: NotificationEvent,
        command: Option<&str>,
        message: &NotificationMessage,
    ) -> Vec<(String, NotificationResult<()>)> {
        configs
            .iter()
            .filter(|c| c.matches(event, command))
            .map(|c| {
                let result = self.send(c, message);
                (c.name.clone(), result)
            })
            .collect()
    }
}

impl Default for NotificationClient {
    fn default() -> Self {
        Self::new().expect("Failed to create notification client")
    }
}

/// Convert hex color to decimal (for Discord).
fn hex_to_decimal(hex: &str) -> Option<u32> {
    let hex = hex.trim_start_matches('#');
    u32::from_str_radix(hex, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_display() {
        assert_eq!(NotificationType::Slack.name(), "Slack");
        assert_eq!(NotificationType::Discord.name(), "Discord");
        assert_eq!(NotificationType::Webhook.name(), "Webhook");
    }

    #[test]
    fn test_notification_config_matches() {
        let config = NotificationConfig::slack("test", "https://example.com")
            .with_events(vec![NotificationEvent::CommandComplete])
            .with_filter("npm run*");

        assert!(config.matches(NotificationEvent::CommandComplete, Some("npm run build")));
        assert!(config.matches(NotificationEvent::CommandComplete, Some("npm run test")));
        assert!(!config.matches(NotificationEvent::CommandComplete, Some("cargo build")));
        assert!(!config.matches(NotificationEvent::CommandStart, Some("npm run build")));
    }

    #[test]
    fn test_notification_message_creation() {
        let msg = NotificationMessage::text("Hello, World!");
        assert_eq!(msg.text, "Hello, World!");
        assert!(msg.title.is_none());
        assert!(!msg.is_error);

        let msg = NotificationMessage::with_title("Title", "Text").success();
        assert_eq!(msg.title, Some("Title".to_string()));
        assert_eq!(msg.color, Some("#28a745".to_string()));

        let msg = NotificationMessage::text("Error!").error();
        assert!(msg.is_error);
        assert_eq!(msg.color, Some("#dc3545".to_string()));
    }

    #[test]
    fn test_command_completed_message() {
        let msg = NotificationMessage::command_completed(
            "npm run build",
            true,
            Some(Duration::from_secs(5)),
        );
        assert!(msg.title.unwrap().contains("succeeded"));
        assert!(!msg.is_error);
        assert_eq!(msg.fields.len(), 1);

        let msg = NotificationMessage::command_completed("npm run test", false, None);
        assert!(msg.title.unwrap().contains("failed"));
        assert!(msg.is_error);
    }

    #[test]
    fn test_hex_to_decimal() {
        assert_eq!(hex_to_decimal("#28a745"), Some(2664261));
        assert_eq!(hex_to_decimal("dc3545"), Some(14431557));
        assert_eq!(hex_to_decimal("#ffffff"), Some(16777215));
    }

    #[test]
    fn test_slack_payload_simple() {
        let client = NotificationClient::new().unwrap();
        let message = NotificationMessage::text("Hello");
        let payload = client.build_slack_payload(&message);

        assert_eq!(payload["text"], "Hello");
        assert!(payload.get("attachments").is_none());
    }

    #[test]
    fn test_discord_payload_simple() {
        let client = NotificationClient::new().unwrap();
        let message = NotificationMessage::text("Hello");
        let payload = client.build_discord_payload(&message);

        assert_eq!(payload["content"], "Hello");
        assert!(payload.get("embeds").is_none());
    }
}
