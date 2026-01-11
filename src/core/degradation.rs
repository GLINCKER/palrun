//! Graceful degradation and fallback strategies.
//!
//! Provides mechanisms for Palrun to continue operating when
//! certain features or services are unavailable.

use std::collections::HashSet;
use std::fmt;

/// Features that can be degraded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// AI-powered features (command generation, suggestions)
    Ai,
    /// Network connectivity
    Network,
    /// Cloud sync
    Sync,
    /// External integrations (GitHub, Linear, etc.)
    Integrations,
    /// MCP server connections
    Mcp,
    /// Fuzzy search (falls back to basic search)
    FuzzySearch,
    /// Project scanning
    Scanning,
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Feature::Ai => write!(f, "AI features"),
            Feature::Network => write!(f, "Network connectivity"),
            Feature::Sync => write!(f, "Cloud sync"),
            Feature::Integrations => write!(f, "External integrations"),
            Feature::Mcp => write!(f, "MCP servers"),
            Feature::FuzzySearch => write!(f, "Fuzzy search"),
            Feature::Scanning => write!(f, "Project scanning"),
        }
    }
}

/// Reason why a feature is degraded.
#[derive(Debug, Clone)]
pub enum DegradationReason {
    /// Feature is disabled in config
    Disabled,
    /// Required service is offline
    ServiceOffline(String),
    /// API key or credentials missing
    MissingCredentials,
    /// Network unavailable
    NetworkUnavailable,
    /// Feature failed and circuit breaker opened
    CircuitOpen,
    /// Unknown/other reason
    Other(String),
}

impl fmt::Display for DegradationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DegradationReason::Disabled => write!(f, "disabled in configuration"),
            DegradationReason::ServiceOffline(s) => write!(f, "{} is offline", s),
            DegradationReason::MissingCredentials => write!(f, "missing API key or credentials"),
            DegradationReason::NetworkUnavailable => write!(f, "no network connection"),
            DegradationReason::CircuitOpen => {
                write!(f, "temporarily unavailable (too many failures)")
            }
            DegradationReason::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Information about a degraded feature.
#[derive(Debug, Clone)]
pub struct DegradedFeature {
    /// The feature that is degraded
    pub feature: Feature,
    /// Why it's degraded
    pub reason: DegradationReason,
    /// What fallback is being used (if any)
    pub fallback: Option<String>,
    /// How to recover/fix
    pub recovery_hint: Option<String>,
}

impl DegradedFeature {
    /// Create a new degraded feature info.
    pub fn new(feature: Feature, reason: DegradationReason) -> Self {
        let (fallback, recovery_hint) = Self::default_fallback_and_hint(&feature, &reason);
        Self { feature, reason, fallback, recovery_hint }
    }

    /// Get default fallback and recovery hint for a feature.
    fn default_fallback_and_hint(
        feature: &Feature,
        reason: &DegradationReason,
    ) -> (Option<String>, Option<String>) {
        match feature {
            Feature::Ai => {
                let fallback = Some("Manual command entry".to_string());
                let hint = match reason {
                    DegradationReason::MissingCredentials => Some(
                        "Set ANTHROPIC_API_KEY or OPENAI_API_KEY, or run Ollama locally"
                            .to_string(),
                    ),
                    DegradationReason::NetworkUnavailable => Some(
                        "Check your internet connection, or use Ollama for local AI".to_string(),
                    ),
                    DegradationReason::ServiceOffline(s) => {
                        Some(format!("Wait for {} to come back online", s))
                    }
                    _ => None,
                };
                (fallback, hint)
            }
            Feature::Network => {
                let fallback = Some("Offline mode with cached data".to_string());
                let hint = Some("Check your internet connection".to_string());
                (fallback, hint)
            }
            Feature::Sync => {
                let fallback = Some("Local-only mode".to_string());
                let hint = Some("Changes will sync when connection is restored".to_string());
                (fallback, hint)
            }
            Feature::Integrations => {
                let fallback = Some("Core features only".to_string());
                let hint = Some("External services unavailable, try again later".to_string());
                (fallback, hint)
            }
            Feature::Mcp => {
                let fallback = Some("Built-in scanners only".to_string());
                let hint = Some("Check MCP server status with 'palrun mcp status'".to_string());
                (fallback, hint)
            }
            Feature::FuzzySearch => {
                let fallback = Some("Basic substring search".to_string());
                let hint = None;
                (fallback, hint)
            }
            Feature::Scanning => {
                let fallback = Some("Manual command entry or cached commands".to_string());
                let hint =
                    Some("Check file permissions and try 'palrun scan --verbose'".to_string());
                (fallback, hint)
            }
        }
    }

    /// Set a custom fallback description.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    /// Set a custom recovery hint.
    pub fn with_recovery_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }
}

/// Manager for tracking degraded features and providing fallbacks.
#[derive(Debug, Default)]
pub struct DegradationManager {
    /// Currently degraded features
    degraded: HashSet<Feature>,
    /// Detailed info about degraded features
    details: Vec<DegradedFeature>,
}

impl DegradationManager {
    /// Create a new degradation manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a feature as degraded.
    pub fn degrade(&mut self, feature: Feature, reason: DegradationReason) {
        if self.degraded.insert(feature) {
            self.details.push(DegradedFeature::new(feature, reason));
        }
    }

    /// Mark a feature as recovered.
    pub fn recover(&mut self, feature: Feature) {
        self.degraded.remove(&feature);
        self.details.retain(|d| d.feature != feature);
    }

    /// Check if a feature is degraded.
    pub fn is_degraded(&self, feature: Feature) -> bool {
        self.degraded.contains(&feature)
    }

    /// Check if any features are degraded.
    pub fn has_degradations(&self) -> bool {
        !self.degraded.is_empty()
    }

    /// Get all degraded features.
    pub fn degraded_features(&self) -> &[DegradedFeature] {
        &self.details
    }

    /// Get a summary of degraded features for display.
    pub fn summary(&self) -> String {
        if self.details.is_empty() {
            return String::new();
        }

        let features: Vec<_> = self.details.iter().map(|d| d.feature.to_string()).collect();
        format!("Degraded: {}", features.join(", "))
    }

    /// Get recovery hints for all degraded features.
    pub fn recovery_hints(&self) -> Vec<String> {
        self.details.iter().filter_map(|d| d.recovery_hint.clone()).collect()
    }

    /// Clear all degradations.
    pub fn clear(&mut self) {
        self.degraded.clear();
        self.details.clear();
    }
}

/// Fallback result indicating what fallback was used.
#[derive(Debug)]
pub struct FallbackResult<T> {
    /// The result value
    pub value: T,
    /// Whether a fallback was used
    pub used_fallback: bool,
    /// Description of the fallback used
    pub fallback_description: Option<String>,
}

impl<T> FallbackResult<T> {
    /// Create a primary (non-fallback) result.
    pub fn primary(value: T) -> Self {
        Self { value, used_fallback: false, fallback_description: None }
    }

    /// Create a fallback result.
    pub fn fallback(value: T, description: impl Into<String>) -> Self {
        Self { value, used_fallback: true, fallback_description: Some(description.into()) }
    }
}

/// Execute an operation with fallback.
///
/// Tries the primary operation first, then falls back to the secondary
/// if the primary fails.
pub fn with_fallback<T, E, F1, F2>(
    primary: F1,
    fallback: F2,
    fallback_description: &str,
) -> FallbackResult<Result<T, E>>
where
    F1: FnOnce() -> Result<T, E>,
    F2: FnOnce() -> Result<T, E>,
{
    match primary() {
        Ok(v) => FallbackResult::primary(Ok(v)),
        Err(_) => FallbackResult::fallback(fallback(), fallback_description),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degradation_manager() {
        let mut mgr = DegradationManager::new();
        assert!(!mgr.has_degradations());

        mgr.degrade(Feature::Ai, DegradationReason::MissingCredentials);
        assert!(mgr.has_degradations());
        assert!(mgr.is_degraded(Feature::Ai));
        assert!(!mgr.is_degraded(Feature::Network));

        mgr.recover(Feature::Ai);
        assert!(!mgr.has_degradations());
    }

    #[test]
    fn test_degraded_feature_info() {
        let df = DegradedFeature::new(Feature::Ai, DegradationReason::NetworkUnavailable);
        assert!(df.fallback.is_some());
        assert!(df.recovery_hint.is_some());
    }

    #[test]
    fn test_fallback_result() {
        let primary: FallbackResult<i32> = FallbackResult::primary(42);
        assert!(!primary.used_fallback);

        let fallback: FallbackResult<i32> = FallbackResult::fallback(0, "default value");
        assert!(fallback.used_fallback);
        assert_eq!(fallback.fallback_description, Some("default value".to_string()));
    }

    #[test]
    fn test_with_fallback() {
        // Primary succeeds
        let result = with_fallback(|| Ok::<_, &str>(42), || Ok(0), "fallback");
        assert!(!result.used_fallback);
        assert_eq!(result.value.unwrap(), 42);

        // Primary fails, fallback succeeds
        let result = with_fallback(|| Err::<i32, _>("primary failed"), || Ok(99), "used default");
        assert!(result.used_fallback);
        assert_eq!(result.value.unwrap(), 99);
    }

    #[test]
    fn test_summary() {
        let mut mgr = DegradationManager::new();
        mgr.degrade(Feature::Ai, DegradationReason::MissingCredentials);
        mgr.degrade(Feature::Network, DegradationReason::NetworkUnavailable);

        let summary = mgr.summary();
        assert!(summary.contains("AI features"));
        assert!(summary.contains("Network"));
    }
}
