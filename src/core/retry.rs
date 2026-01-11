//! Retry logic with exponential backoff and jitter.
//!
//! Provides resilient execution of fallible operations with configurable
//! retry strategies, circuit breakers, and timeout handling.

use std::future::Future;
use std::time::Duration;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_attempts: u32,

    /// Initial delay before first retry.
    pub initial_delay: Duration,

    /// Maximum delay between retries.
    pub max_delay: Duration,

    /// Multiplier for exponential backoff (e.g., 2.0 = double each time).
    pub backoff_multiplier: f64,

    /// Whether to add jitter to delays (prevents thundering herd).
    pub jitter: bool,

    /// Timeout for each individual attempt.
    pub attempt_timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
            attempt_timeout: Some(Duration::from_secs(30)),
        }
    }
}

impl RetryConfig {
    /// Create a config with no retries (fail fast).
    pub fn no_retry() -> Self {
        Self { max_attempts: 0, ..Default::default() }
    }

    /// Create a config for quick retries (UI operations).
    pub fn quick() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            jitter: true,
            attempt_timeout: Some(Duration::from_secs(5)),
        }
    }

    /// Create a config for network operations.
    pub fn network() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
            attempt_timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create a config for AI/API operations (longer timeouts).
    pub fn api() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
            attempt_timeout: Some(Duration::from_secs(120)),
        }
    }

    /// Calculate delay for the given attempt number.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let base_delay = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32 - 1);
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (rand_jitter() * 0.25);
            capped_delay * jitter_factor
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }
}

/// Simple pseudo-random jitter (0.0 to 1.0) without external deps.
fn rand_jitter() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

/// Result of a retry operation.
#[derive(Debug)]
pub struct RetryResult<T, E> {
    /// The final result (success or last error).
    pub result: Result<T, E>,

    /// Number of attempts made.
    pub attempts: u32,

    /// Total time spent (including delays).
    pub total_time: Duration,

    /// Whether the operation was retried.
    pub was_retried: bool,
}

impl<T, E> RetryResult<T, E> {
    /// Check if the operation succeeded.
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Unwrap the result, panicking on error.
    pub fn unwrap(self) -> T
    where
        E: std::fmt::Debug,
    {
        self.result.unwrap()
    }

    /// Get the result.
    pub fn into_result(self) -> Result<T, E> {
        self.result
    }
}

/// Retry a synchronous operation with the given configuration.
pub fn retry<T, E, F>(config: &RetryConfig, mut operation: F) -> RetryResult<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let start = std::time::Instant::now();
    let mut attempts = 0;
    let max_attempts = config.max_attempts + 1; // +1 for initial attempt

    loop {
        attempts += 1;
        let result = operation();

        if result.is_ok() || attempts >= max_attempts {
            return RetryResult {
                result,
                attempts,
                total_time: start.elapsed(),
                was_retried: attempts > 1,
            };
        }

        // Sleep before next attempt
        let delay = config.delay_for_attempt(attempts);
        std::thread::sleep(delay);
    }
}

/// Retry an async operation with the given configuration.
pub async fn retry_async<T, E, F, Fut>(config: &RetryConfig, mut operation: F) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let start = std::time::Instant::now();
    let mut attempts = 0;
    let max_attempts = config.max_attempts + 1;

    loop {
        attempts += 1;
        let result = operation().await;

        if result.is_ok() || attempts >= max_attempts {
            return RetryResult {
                result,
                attempts,
                total_time: start.elapsed(),
                was_retried: attempts > 1,
            };
        }

        // Sleep before next attempt
        let delay = config.delay_for_attempt(attempts);
        tokio::time::sleep(delay).await;
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, operations proceed normally.
    Closed,
    /// Circuit is open, operations fail immediately.
    Open,
    /// Circuit is half-open, testing if operations succeed.
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures.
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state.
    state: CircuitState,

    /// Number of consecutive failures.
    failure_count: u32,

    /// Failure threshold to open circuit.
    failure_threshold: u32,

    /// Time to wait before transitioning to half-open.
    reset_timeout: Duration,

    /// Time when circuit was opened.
    opened_at: Option<std::time::Instant>,

    /// Success threshold in half-open to close circuit.
    success_threshold: u32,

    /// Current success count in half-open state.
    half_open_successes: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30), 2)
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(failure_threshold: u32, reset_timeout: Duration, success_threshold: u32) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            reset_timeout,
            opened_at: None,
            success_threshold,
            half_open_successes: 0,
        }
    }

    /// Get current state (may transition to half-open).
    pub fn state(&mut self) -> CircuitState {
        if self.state == CircuitState::Open {
            if let Some(opened_at) = self.opened_at {
                if opened_at.elapsed() >= self.reset_timeout {
                    self.state = CircuitState::HalfOpen;
                    self.half_open_successes = 0;
                }
            }
        }
        self.state
    }

    /// Check if the circuit allows operations.
    pub fn allow_request(&mut self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    /// Record a successful operation.
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed operation.
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.opened_at = Some(std::time::Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.opened_at = Some(std::time::Instant::now());
            }
            CircuitState::Open => {}
        }
    }

    /// Reset the circuit breaker.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.opened_at = None;
        self.half_open_successes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_no_retry() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_attempts, 0);
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
            attempt_timeout: None,
        };

        assert_eq!(config.delay_for_attempt(0), Duration::ZERO);
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(400));
    }

    #[test]
    fn test_delay_capped_at_max() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 10.0,
            jitter: false,
            attempt_timeout: None,
        };

        // After a few attempts, should be capped at max
        let delay = config.delay_for_attempt(5);
        assert!(delay <= config.max_delay);
    }

    #[test]
    fn test_retry_success_first_attempt() {
        let config = RetryConfig::default();
        let result = retry(&config, || Ok::<_, &str>("success"));

        assert!(result.is_ok());
        assert_eq!(result.attempts, 1);
        assert!(!result.was_retried);
    }

    #[test]
    fn test_retry_success_after_failures() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            jitter: false,
            ..Default::default()
        };

        let mut attempts = 0;
        let result = retry(&config, || {
            attempts += 1;
            if attempts < 3 {
                Err("transient error")
            } else {
                Ok("success")
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.attempts, 3);
        assert!(result.was_retried);
    }

    #[test]
    fn test_retry_all_failures() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            jitter: false,
            ..Default::default()
        };

        let result = retry(&config, || Err::<(), _>("persistent error"));

        assert!(!result.is_ok());
        assert_eq!(result.attempts, 3); // 1 initial + 2 retries
        assert!(result.was_retried);
    }

    #[test]
    fn test_circuit_breaker_closed() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1), 1);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60), 1);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure(); // Third failure opens circuit
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60), 1);

        cb.record_failure();
        cb.record_failure();
        cb.record_success(); // Resets failure count

        assert_eq!(cb.failure_count, 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_manual_reset() {
        let mut cb = CircuitBreaker::new(1, Duration::from_secs(60), 1);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }
}
