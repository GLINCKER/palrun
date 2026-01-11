//! Network connectivity utilities.
//!
//! Provides offline detection and network status monitoring.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Network status for graceful degradation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    /// Network is available
    Online,
    /// Network is unavailable
    Offline,
    /// Network status unknown (not yet checked)
    Unknown,
}

impl NetworkStatus {
    /// Check if online.
    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }

    /// Check if offline.
    pub fn is_offline(&self) -> bool {
        matches!(self, Self::Offline)
    }
}

/// Network connectivity checker.
#[derive(Debug, Clone)]
pub struct NetworkChecker {
    /// Cached online status.
    is_online: Arc<AtomicBool>,
    /// Check timeout.
    timeout: Duration,
}

impl Default for NetworkChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkChecker {
    /// Create a new network checker.
    pub fn new() -> Self {
        Self {
            is_online: Arc::new(AtomicBool::new(true)), // Assume online initially
            timeout: Duration::from_secs(2),
        }
    }

    /// Create with custom timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { is_online: Arc::new(AtomicBool::new(true)), timeout }
    }

    /// Check network connectivity.
    ///
    /// Performs a quick DNS lookup or HTTP request to verify connectivity.
    #[cfg(feature = "ai")]
    pub fn check(&self) -> NetworkStatus {
        // Try a simple HTTP request to a reliable endpoint
        let client =
            if let Ok(c) = reqwest::blocking::Client::builder().timeout(self.timeout).build() {
                c
            } else {
                self.is_online.store(false, Ordering::SeqCst);
                return NetworkStatus::Offline;
            };

        // Try multiple endpoints for reliability
        let endpoints = ["https://httpbin.org/status/200", "https://www.google.com/generate_204"];

        for endpoint in endpoints {
            if let Ok(response) = client.head(endpoint).send() {
                if response.status().is_success() || response.status().as_u16() == 204 {
                    self.is_online.store(true, Ordering::SeqCst);
                    return NetworkStatus::Online;
                }
            }
        }

        self.is_online.store(false, Ordering::SeqCst);
        NetworkStatus::Offline
    }

    /// Check network connectivity (async version).
    #[cfg(feature = "ai")]
    pub async fn check_async(&self) -> NetworkStatus {
        let client = if let Ok(c) = reqwest::Client::builder().timeout(self.timeout).build() {
            c
        } else {
            self.is_online.store(false, Ordering::SeqCst);
            return NetworkStatus::Offline;
        };

        let endpoints = ["https://httpbin.org/status/200", "https://www.google.com/generate_204"];

        for endpoint in endpoints {
            if let Ok(response) = client.head(endpoint).send().await {
                if response.status().is_success() || response.status().as_u16() == 204 {
                    self.is_online.store(true, Ordering::SeqCst);
                    return NetworkStatus::Online;
                }
            }
        }

        self.is_online.store(false, Ordering::SeqCst);
        NetworkStatus::Offline
    }

    /// Fallback check without network features.
    #[cfg(not(feature = "ai"))]
    pub fn check(&self) -> NetworkStatus {
        // Without reqwest, assume online
        NetworkStatus::Unknown
    }

    /// Fallback check without network features.
    #[cfg(not(feature = "ai"))]
    pub async fn check_async(&self) -> NetworkStatus {
        NetworkStatus::Unknown
    }

    /// Get cached online status.
    pub fn is_online(&self) -> bool {
        self.is_online.load(Ordering::SeqCst)
    }

    /// Get cached status.
    pub fn status(&self) -> NetworkStatus {
        if self.is_online.load(Ordering::SeqCst) {
            NetworkStatus::Online
        } else {
            NetworkStatus::Offline
        }
    }
}

/// Service availability checker for specific endpoints.
#[derive(Debug)]
pub struct ServiceChecker {
    /// Service name.
    pub name: String,
    /// Health check URL.
    pub url: String,
    /// Timeout for checks.
    pub timeout: Duration,
}

impl ServiceChecker {
    /// Create a new service checker.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self { name: name.into(), url: url.into(), timeout: Duration::from_secs(5) }
    }

    /// Create for Claude API.
    pub fn claude() -> Self {
        Self::new("Claude API", "https://api.anthropic.com/v1/messages")
    }

    /// Create for Ollama.
    pub fn ollama() -> Self {
        Self::new(
            "Ollama",
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string()),
        )
    }

    /// Check if the service is available.
    #[cfg(feature = "ai")]
    pub async fn is_available(&self) -> bool {
        let client = match reqwest::Client::builder().timeout(self.timeout).build() {
            Ok(c) => c,
            Err(_) => return false,
        };

        // For API endpoints, a 401/403 still means the service is up
        match client.head(&self.url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                // 2xx, 401, 403 all indicate the service is reachable
                (200..300).contains(&status) || status == 401 || status == 403
            }
            Err(_) => false,
        }
    }

    /// Fallback without network features.
    #[cfg(not(feature = "ai"))]
    pub async fn is_available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_status() {
        assert!(NetworkStatus::Online.is_online());
        assert!(!NetworkStatus::Online.is_offline());
        assert!(NetworkStatus::Offline.is_offline());
        assert!(!NetworkStatus::Offline.is_online());
    }

    #[test]
    fn test_network_checker_creation() {
        let checker = NetworkChecker::new();
        assert!(checker.is_online()); // Default to online
    }

    #[test]
    fn test_network_checker_custom_timeout() {
        let checker = NetworkChecker::with_timeout(Duration::from_millis(500));
        assert_eq!(checker.timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_service_checker_creation() {
        let checker = ServiceChecker::new("test", "http://localhost:8080");
        assert_eq!(checker.name, "test");
        assert_eq!(checker.url, "http://localhost:8080");
    }

    #[test]
    fn test_service_checker_claude() {
        let checker = ServiceChecker::claude();
        assert_eq!(checker.name, "Claude API");
        assert!(checker.url.contains("anthropic"));
    }

    #[test]
    fn test_service_checker_ollama() {
        let checker = ServiceChecker::ollama();
        assert_eq!(checker.name, "Ollama");
    }
}
