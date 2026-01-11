//! Resilience infrastructure combining retry, circuit breaker, and degradation.
//!
//! Provides a unified approach to resilient operations that automatically:
//! - Retries transient failures
//! - Opens circuit breakers on persistent failures
//! - Queues operations when offline
//! - Tracks feature degradation

use std::sync::Mutex;
use std::time::Duration;

use super::degradation::{DegradationManager, DegradationReason, Feature};
use super::offline::{OfflineManager, QueuedOperation};
use super::retry::{retry, CircuitBreaker, CircuitState, RetryConfig};

/// Result of a resilient operation.
#[derive(Debug)]
pub struct ResilientResult<T> {
    /// The operation result (if successful).
    pub value: Option<T>,
    /// Whether the operation was queued for later.
    pub queued: bool,
    /// Whether a fallback was used.
    pub used_fallback: bool,
    /// Number of retry attempts made.
    pub attempts: u32,
    /// Error message if failed.
    pub error: Option<String>,
}

impl<T> ResilientResult<T> {
    /// Create a successful result.
    pub fn success(value: T, attempts: u32) -> Self {
        Self { value: Some(value), queued: false, used_fallback: false, attempts, error: None }
    }

    /// Create a queued result (operation will be executed later).
    pub fn queued() -> Self {
        Self { value: None, queued: true, used_fallback: false, attempts: 0, error: None }
    }

    /// Create a fallback result.
    pub fn fallback(value: T) -> Self {
        Self { value: Some(value), queued: false, used_fallback: true, attempts: 0, error: None }
    }

    /// Create a failed result.
    pub fn failed(error: impl Into<String>, attempts: u32) -> Self {
        Self {
            value: None,
            queued: false,
            used_fallback: false,
            attempts,
            error: Some(error.into()),
        }
    }

    /// Check if operation succeeded.
    pub fn is_success(&self) -> bool {
        self.value.is_some()
    }

    /// Get the value if successful.
    pub fn into_value(self) -> Option<T> {
        self.value
    }
}

/// Resilience context for a specific feature.
pub struct FeatureResilience {
    /// The feature this context manages.
    feature: Feature,
    /// Circuit breaker for this feature.
    circuit_breaker: Mutex<CircuitBreaker>,
    /// Retry configuration.
    retry_config: RetryConfig,
}

impl std::fmt::Debug for FeatureResilience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeatureResilience")
            .field("feature", &self.feature)
            .field("circuit_state", &self.circuit_state())
            .finish()
    }
}

impl FeatureResilience {
    /// Create a new feature resilience context.
    pub fn new(feature: Feature) -> Self {
        let (circuit_breaker, retry_config) = match feature {
            Feature::Ai => (CircuitBreaker::new(3, Duration::from_secs(60), 2), RetryConfig::api()),
            Feature::Network => {
                (CircuitBreaker::new(5, Duration::from_secs(30), 3), RetryConfig::network())
            }
            Feature::Sync => {
                (CircuitBreaker::new(3, Duration::from_secs(120), 2), RetryConfig::network())
            }
            Feature::Integrations => {
                (CircuitBreaker::new(3, Duration::from_secs(60), 2), RetryConfig::network())
            }
            Feature::Mcp => {
                (CircuitBreaker::new(2, Duration::from_secs(30), 1), RetryConfig::quick())
            }
            Feature::FuzzySearch | Feature::Scanning => {
                (CircuitBreaker::new(3, Duration::from_secs(10), 2), RetryConfig::quick())
            }
        };

        Self { feature, circuit_breaker: Mutex::new(circuit_breaker), retry_config }
    }

    /// Check if the circuit allows requests.
    pub fn is_available(&self) -> bool {
        self.circuit_breaker.lock().map(|mut cb| cb.allow_request()).unwrap_or(false)
    }

    /// Get the circuit state.
    pub fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.lock().map(|mut cb| cb.state()).unwrap_or(CircuitState::Open)
    }

    /// Record a successful operation.
    pub fn record_success(&self) {
        if let Ok(mut cb) = self.circuit_breaker.lock() {
            cb.record_success();
        }
    }

    /// Record a failed operation.
    pub fn record_failure(&self) {
        if let Ok(mut cb) = self.circuit_breaker.lock() {
            cb.record_failure();
        }
    }

    /// Reset the circuit breaker.
    pub fn reset(&self) {
        if let Ok(mut cb) = self.circuit_breaker.lock() {
            cb.reset();
        }
    }

    /// Execute an operation with retry and circuit breaker.
    pub fn execute<T, E, F>(&self, operation: F) -> ResilientResult<T>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        // Check circuit breaker first
        if !self.is_available() {
            return ResilientResult::failed(
                format!("{} temporarily unavailable (circuit open)", self.feature),
                0,
            );
        }

        // Execute with retry
        let result = retry(&self.retry_config, operation);

        if result.is_ok() {
            self.record_success();
            ResilientResult::success(result.result.unwrap(), result.attempts)
        } else {
            self.record_failure();
            let error = result
                .result
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string());
            ResilientResult::failed(error, result.attempts)
        }
    }

    /// Get the feature this resilience context manages.
    pub fn feature(&self) -> Feature {
        self.feature
    }

    /// Get the retry config.
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
}

/// Manager coordinating resilience across all features.
#[derive(Debug)]
pub struct ResilienceManager {
    /// AI feature resilience.
    pub ai: FeatureResilience,
    /// Network feature resilience.
    pub network: FeatureResilience,
    /// Sync feature resilience.
    pub sync: FeatureResilience,
    /// Integrations feature resilience.
    pub integrations: FeatureResilience,
    /// MCP feature resilience.
    pub mcp: FeatureResilience,
}

impl Default for ResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResilienceManager {
    /// Create a new resilience manager.
    pub fn new() -> Self {
        Self {
            ai: FeatureResilience::new(Feature::Ai),
            network: FeatureResilience::new(Feature::Network),
            sync: FeatureResilience::new(Feature::Sync),
            integrations: FeatureResilience::new(Feature::Integrations),
            mcp: FeatureResilience::new(Feature::Mcp),
        }
    }

    /// Get resilience context for a feature.
    pub fn for_feature(&self, feature: Feature) -> &FeatureResilience {
        match feature {
            Feature::Ai => &self.ai,
            Feature::Network => &self.network,
            Feature::Sync => &self.sync,
            Feature::Integrations => &self.integrations,
            Feature::Mcp => &self.mcp,
            Feature::FuzzySearch | Feature::Scanning => &self.ai, // Use AI config for local features
        }
    }

    /// Execute operation with resilience, updating degradation on failure.
    pub fn execute_with_degradation<T, E, F>(
        &self,
        feature: Feature,
        degradation: &mut DegradationManager,
        operation: F,
    ) -> ResilientResult<T>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let resilience = self.for_feature(feature);
        let result = resilience.execute(operation);

        if result.is_success() {
            // Recover if previously degraded
            degradation.recover(feature);
        } else if resilience.circuit_state() == CircuitState::Open {
            // Mark as degraded when circuit opens
            degradation.degrade(feature, DegradationReason::CircuitOpen);
        }

        result
    }

    /// Execute operation with offline queue fallback.
    pub fn execute_with_queue<T, E, F>(
        &self,
        feature: Feature,
        offline_manager: &mut OfflineManager,
        operation: F,
        queue_operation: impl FnOnce() -> QueuedOperation,
    ) -> ResilientResult<T>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        // If offline, queue immediately
        if offline_manager.is_offline() {
            let op = queue_operation();
            offline_manager.queue_operation(op);
            return ResilientResult::queued();
        }

        let resilience = self.for_feature(feature);
        let result = resilience.execute(operation);

        // If failed and circuit is open, queue for later
        if !result.is_success() && resilience.circuit_state() == CircuitState::Open {
            let op = queue_operation();
            offline_manager.queue_operation(op);
            return ResilientResult::queued();
        }

        result
    }

    /// Reset all circuit breakers.
    pub fn reset_all(&self) {
        self.ai.reset();
        self.network.reset();
        self.sync.reset();
        self.integrations.reset();
        self.mcp.reset();
    }

    /// Get status summary.
    pub fn status_summary(&self) -> Vec<(Feature, CircuitState)> {
        vec![
            (Feature::Ai, self.ai.circuit_state()),
            (Feature::Network, self.network.circuit_state()),
            (Feature::Sync, self.sync.circuit_state()),
            (Feature::Integrations, self.integrations.circuit_state()),
            (Feature::Mcp, self.mcp.circuit_state()),
        ]
    }
}

/// Execute an operation with full resilience: retry, circuit breaker, degradation, and offline queue.
pub fn execute_resilient<T, E, F>(
    feature: Feature,
    resilience: &ResilienceManager,
    degradation: &mut DegradationManager,
    offline: &mut OfflineManager,
    operation: F,
    queue_operation: impl FnOnce() -> QueuedOperation,
) -> ResilientResult<T>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display + std::fmt::Debug,
{
    // If offline, queue immediately
    if offline.is_offline() {
        let op = queue_operation();
        offline.queue_operation(op);
        return ResilientResult::queued();
    }

    let feature_resilience = resilience.for_feature(feature);

    // Check circuit breaker
    if !feature_resilience.is_available() {
        degradation.degrade(feature, DegradationReason::CircuitOpen);
        let op = queue_operation();
        offline.queue_operation(op);
        return ResilientResult::queued();
    }

    // Execute with retry
    let result = feature_resilience.execute(operation);

    if result.is_success() {
        degradation.recover(feature);
    } else if feature_resilience.circuit_state() == CircuitState::Open {
        degradation.degrade(feature, DegradationReason::CircuitOpen);
        // Queue for later execution
        let op = queue_operation();
        offline.queue_operation(op);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resilient_result_success() {
        let result: ResilientResult<i32> = ResilientResult::success(42, 1);
        assert!(result.is_success());
        assert_eq!(result.value, Some(42));
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_resilient_result_queued() {
        let result: ResilientResult<i32> = ResilientResult::queued();
        assert!(!result.is_success());
        assert!(result.queued);
    }

    #[test]
    fn test_resilient_result_failed() {
        let result: ResilientResult<i32> = ResilientResult::failed("error", 3);
        assert!(!result.is_success());
        assert_eq!(result.error, Some("error".to_string()));
        assert_eq!(result.attempts, 3);
    }

    #[test]
    fn test_feature_resilience_creation() {
        let resilience = FeatureResilience::new(Feature::Ai);
        assert_eq!(resilience.feature(), Feature::Ai);
        assert!(resilience.is_available());
    }

    #[test]
    fn test_feature_resilience_execute_success() {
        let resilience = FeatureResilience::new(Feature::Network);
        let result = resilience.execute(|| Ok::<_, &str>(42));
        assert!(result.is_success());
        assert_eq!(result.value, Some(42));
    }

    #[test]
    fn test_feature_resilience_circuit_opens() {
        let resilience = FeatureResilience::new(Feature::Mcp);

        // Record enough failures to open circuit
        for _ in 0..3 {
            resilience.record_failure();
        }

        assert_eq!(resilience.circuit_state(), CircuitState::Open);
        assert!(!resilience.is_available());
    }

    #[test]
    fn test_resilience_manager_creation() {
        let manager = ResilienceManager::new();
        assert!(manager.ai.is_available());
        assert!(manager.network.is_available());
    }

    #[test]
    fn test_resilience_manager_status() {
        let manager = ResilienceManager::new();
        let status = manager.status_summary();

        assert_eq!(status.len(), 5);
        for (_, state) in status {
            assert_eq!(state, CircuitState::Closed);
        }
    }

    #[test]
    fn test_execute_with_degradation_success() {
        let manager = ResilienceManager::new();
        let mut degradation = DegradationManager::new();

        let result = manager
            .execute_with_degradation(Feature::Ai, &mut degradation, || Ok::<_, &str>("success"));

        assert!(result.is_success());
        assert!(!degradation.is_degraded(Feature::Ai));
    }

    #[test]
    fn test_execute_with_queue_offline() {
        let manager = ResilienceManager::new();
        let mut offline = OfflineManager::new();
        offline.set_offline(true);

        let result: ResilientResult<String> = manager.execute_with_queue(
            Feature::Sync,
            &mut offline,
            || Ok::<_, &str>("data".to_string()),
            || QueuedOperation::SyncHistory { entries_count: 5 },
        );

        assert!(result.queued);
        assert_eq!(offline.queue().len(), 1);
    }
}
