//! REST API for Palrun.
//!
//! Provides an optional HTTP server for remote control and integration.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether the API is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// Host to bind to (default: 127.0.0.1).
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to (default: 8765).
    #[serde(default = "default_port")]
    pub port: u16,

    /// API key for authentication (required).
    pub api_key: Option<String>,

    /// Whether to enable CORS.
    #[serde(default)]
    pub cors_enabled: bool,

    /// Allowed CORS origins.
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Rate limit per minute.
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8765
}

fn default_rate_limit() -> u32 {
    60
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_host(),
            port: default_port(),
            api_key: None,
            cors_enabled: false,
            cors_origins: Vec::new(),
            rate_limit: default_rate_limit(),
        }
    }
}

impl ApiConfig {
    /// Get the socket address.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.host, self.port).parse()
    }
}

/// API error types.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("API is not enabled")]
    NotEnabled,

    #[error("API key is required")]
    ApiKeyRequired,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// API request for executing a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    /// Command ID or name.
    pub command: String,

    /// Optional arguments to pass.
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether to run in background.
    #[serde(default)]
    pub background: bool,
}

/// API response for command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    /// Execution ID.
    pub id: String,

    /// Whether the command was started.
    pub started: bool,

    /// Exit code (if not background).
    pub exit_code: Option<i32>,

    /// stdout output (if not background).
    pub stdout: Option<String>,

    /// stderr output (if not background).
    pub stderr: Option<String>,

    /// Error message (if failed to start).
    pub error: Option<String>,
}

/// Command information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Command ID.
    pub id: String,

    /// Command name.
    pub name: String,

    /// Full command string.
    pub command: String,

    /// Description.
    pub description: Option<String>,

    /// Source (npm, cargo, makefile, etc.).
    pub source: String,

    /// Working directory.
    pub working_dir: Option<String>,

    /// Tags.
    pub tags: Vec<String>,
}

/// Palrun status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Palrun version.
    pub version: String,

    /// Whether Palrun is healthy.
    pub healthy: bool,

    /// Number of commands available.
    pub command_count: usize,

    /// Current working directory.
    pub working_dir: String,

    /// Active background jobs.
    pub active_jobs: usize,

    /// API uptime in seconds.
    pub uptime_secs: u64,
}

/// History entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Entry ID.
    pub id: String,

    /// Command that was run.
    pub command: String,

    /// Timestamp (Unix).
    pub timestamp: u64,

    /// Exit code.
    pub exit_code: Option<i32>,

    /// Duration in milliseconds.
    pub duration_ms: Option<u64>,

    /// Whether it succeeded.
    pub success: bool,
}

/// Rate limiter for API requests.
#[derive(Debug)]
pub struct RateLimiter {
    /// Requests per minute limit.
    limit: u32,

    /// Request counts by client.
    requests: HashMap<String, Vec<u64>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(limit: u32) -> Self {
        Self { limit, requests: HashMap::new() }
    }

    /// Check if a request is allowed.
    pub fn check(&mut self, client: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let window_start = now - 60; // 1 minute window

        // Get or create client entry
        let timestamps = self.requests.entry(client.to_string()).or_default();

        // Remove old requests
        timestamps.retain(|&t| t >= window_start);

        // Check limit
        if timestamps.len() >= self.limit as usize {
            return false;
        }

        // Add this request
        timestamps.push(now);
        true
    }

    /// Clear all rate limit data.
    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

/// API state shared across handlers.
#[derive(Debug)]
pub struct ApiState {
    /// Configuration.
    pub config: ApiConfig,

    /// Rate limiter.
    pub rate_limiter: Mutex<RateLimiter>,

    /// Start time.
    pub start_time: std::time::Instant,

    /// Commands available.
    pub commands: Mutex<Vec<CommandInfo>>,

    /// Execution history.
    pub history: Mutex<Vec<HistoryEntry>>,
}

impl ApiState {
    /// Create new API state.
    pub fn new(config: ApiConfig) -> Self {
        let rate_limit = config.rate_limit;
        Self {
            config,
            rate_limiter: Mutex::new(RateLimiter::new(rate_limit)),
            start_time: std::time::Instant::now(),
            commands: Mutex::new(Vec::new()),
            history: Mutex::new(Vec::new()),
        }
    }

    /// Check API key.
    pub fn check_api_key(&self, key: Option<&str>) -> ApiResult<()> {
        match (&self.config.api_key, key) {
            (Some(expected), Some(provided)) if expected == provided => Ok(()),
            (Some(_), Some(_)) => Err(ApiError::InvalidApiKey),
            (Some(_), None) => Err(ApiError::ApiKeyRequired),
            (None, _) => Ok(()), // No key required
        }
    }

    /// Check rate limit.
    pub fn check_rate_limit(&self, client: &str) -> ApiResult<()> {
        let mut limiter = self.rate_limiter.lock().unwrap();
        if limiter.check(client) {
            Ok(())
        } else {
            Err(ApiError::RateLimitExceeded)
        }
    }

    /// Get uptime.
    pub fn uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Add command.
    pub fn add_command(&self, cmd: CommandInfo) {
        let mut commands = self.commands.lock().unwrap();
        commands.push(cmd);
    }

    /// Get commands.
    pub fn get_commands(&self) -> Vec<CommandInfo> {
        self.commands.lock().unwrap().clone()
    }

    /// Add history entry.
    pub fn add_history(&self, entry: HistoryEntry) {
        let mut history = self.history.lock().unwrap();
        history.push(entry);

        // Keep only last 1000 entries
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get history.
    pub fn get_history(&self, limit: usize) -> Vec<HistoryEntry> {
        let history = self.history.lock().unwrap();
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// API server handle.
pub struct ApiServer {
    /// Server state.
    state: Arc<ApiState>,

    /// Whether the server is running.
    running: bool,
}

impl ApiServer {
    /// Create a new API server (not started).
    pub fn new(config: ApiConfig) -> Self {
        Self { state: Arc::new(ApiState::new(config)), running: false }
    }

    /// Get server state.
    pub fn state(&self) -> Arc<ApiState> {
        Arc::clone(&self.state)
    }

    /// Check if server is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the status response.
    pub fn status(&self) -> StatusResponse {
        StatusResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            healthy: true,
            command_count: self.state.get_commands().len(),
            working_dir: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            active_jobs: 0, // TODO: track active jobs
            uptime_secs: self.state.uptime(),
        }
    }

    /// Start the server (placeholder - actual server requires async runtime).
    ///
    /// In a full implementation, this would use `axum` or `actix-web`.
    /// For now, we provide the API types and state management.
    pub fn start(&mut self) -> ApiResult<()> {
        if !self.state.config.enabled {
            return Err(ApiError::NotEnabled);
        }

        // In a full implementation:
        // 1. Create axum router with routes
        // 2. Bind to socket address
        // 3. Spawn server task

        self.running = true;
        Ok(())
    }

    /// Stop the server.
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// API route handler types (for documentation).
pub mod routes {
    //! API routes documentation.
    //!
    //! Available endpoints:
    //!
    //! - `GET /` - API information
    //! - `GET /status` - Palrun status
    //! - `GET /commands` - List available commands
    //! - `GET /commands/:id` - Get command details
    //! - `POST /commands/:id/execute` - Execute a command
    //! - `GET /history` - Get execution history
    //! - `POST /ai/generate` - Generate command with AI
    //! - `POST /ai/explain` - Explain a command
    //! - `POST /ai/diagnose` - Diagnose an error
    //!
    //! All endpoints require the `X-API-Key` header if configured.

    /// Route: GET /
    pub const ROOT: &str = "/";

    /// Route: GET /status
    pub const STATUS: &str = "/status";

    /// Route: GET /commands
    pub const COMMANDS: &str = "/commands";

    /// Route: GET /commands/:id
    pub const COMMAND: &str = "/commands/:id";

    /// Route: POST /commands/:id/execute
    pub const EXECUTE: &str = "/commands/:id/execute";

    /// Route: GET /history
    pub const HISTORY: &str = "/history";

    /// Route: POST /ai/generate
    pub const AI_GENERATE: &str = "/ai/generate";

    /// Route: POST /ai/explain
    pub const AI_EXPLAIN: &str = "/ai/explain";

    /// Route: POST /ai/diagnose
    pub const AI_DIAGNOSE: &str = "/ai/diagnose";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_defaults() {
        let config = ApiConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8765);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_socket_addr() {
        let config = ApiConfig::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8765");
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);

        assert!(limiter.check("client1"));
        assert!(limiter.check("client1"));
        assert!(!limiter.check("client1")); // Should be rate limited

        // Different client should work
        assert!(limiter.check("client2"));
    }

    #[test]
    fn test_api_state_api_key() {
        let config = ApiConfig { api_key: Some("secret".to_string()), ..Default::default() };

        let state = ApiState::new(config);

        assert!(state.check_api_key(Some("secret")).is_ok());
        assert!(state.check_api_key(Some("wrong")).is_err());
        assert!(state.check_api_key(None).is_err());
    }

    #[test]
    fn test_api_state_no_key() {
        let config = ApiConfig::default();
        let state = ApiState::new(config);

        assert!(state.check_api_key(None).is_ok());
        assert!(state.check_api_key(Some("any")).is_ok());
    }

    #[test]
    fn test_api_server_creation() {
        let config = ApiConfig::default();
        let server = ApiServer::new(config);

        assert!(!server.is_running());
    }

    #[test]
    fn test_api_state_commands() {
        let config = ApiConfig::default();
        let state = ApiState::new(config);

        state.add_command(CommandInfo {
            id: "1".to_string(),
            name: "test".to_string(),
            command: "npm test".to_string(),
            description: Some("Run tests".to_string()),
            source: "npm".to_string(),
            working_dir: None,
            tags: vec![],
        });

        let commands = state.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "test");
    }

    #[test]
    fn test_api_state_history() {
        let config = ApiConfig::default();
        let state = ApiState::new(config);

        state.add_history(HistoryEntry {
            id: "1".to_string(),
            command: "npm test".to_string(),
            timestamp: 1704067200,
            exit_code: Some(0),
            duration_ms: Some(5000),
            success: true,
        });

        let history = state.get_history(10);
        assert_eq!(history.len(), 1);
        assert!(history[0].success);
    }

    #[test]
    fn test_execute_request_serialization() {
        let request = ExecuteRequest {
            command: "build".to_string(),
            args: vec!["--release".to_string()],
            env: HashMap::new(),
            background: false,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("build"));
        assert!(json.contains("--release"));
    }

    #[test]
    fn test_status_response() {
        let config = ApiConfig::default();
        let server = ApiServer::new(config);
        let status = server.status();

        assert!(status.healthy);
        assert_eq!(status.command_count, 0);
    }
}
