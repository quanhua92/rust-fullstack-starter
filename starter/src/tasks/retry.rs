use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RetryStrategy {
    /// Exponential backoff: delay = base_delay * multiplier^attempt
    Exponential {
        base_delay: Duration,
        multiplier: f64,
        max_delay: Duration,
        max_attempts: u32,
    },
    /// Linear backoff: delay = base_delay + (increment * attempt)
    Linear {
        base_delay: Duration,
        increment: Duration,
        max_delay: Duration,
        max_attempts: u32,
    },
    /// Fixed interval: delay = interval for each retry
    Fixed {
        interval: Duration,
        max_attempts: u32,
    },
    /// No retry
    None,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::Exponential {
            base_delay: Duration::from_millis(1000),
            multiplier: 2.0,
            max_delay: Duration::from_secs(300), // 5 minutes
            max_attempts: 5,
        }
    }
}

impl RetryStrategy {
    /// Calculate delay for given attempt number (0-based)
    pub fn calculate_delay(&self, attempt: u32) -> Option<Duration> {
        match self {
            Self::Exponential {
                base_delay,
                multiplier,
                max_delay,
                max_attempts,
            } => {
                if attempt >= *max_attempts {
                    return None;
                }
                let delay = Duration::from_millis(
                    (base_delay.as_millis() as f64 * multiplier.powi(attempt as i32)) as u64,
                );
                Some(delay.min(*max_delay))
            }
            Self::Linear {
                base_delay,
                increment,
                max_delay,
                max_attempts,
            } => {
                if attempt >= *max_attempts {
                    return None;
                }
                let delay = *base_delay + (*increment * attempt);
                Some(delay.min(*max_delay))
            }
            Self::Fixed {
                interval,
                max_attempts,
            } => {
                if attempt >= *max_attempts {
                    None
                } else {
                    Some(*interval)
                }
            }
            Self::None => None,
        }
    }

    /// Get maximum number of attempts
    pub fn max_attempts(&self) -> u32 {
        match self {
            Self::Exponential { max_attempts, .. } => *max_attempts,
            Self::Linear { max_attempts, .. } => *max_attempts,
            Self::Fixed { max_attempts, .. } => *max_attempts,
            Self::None => 0,
        }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if let Some(delay) = self.calculate_delay(attempt) {
                        tracing::warn!(
                            "Operation failed on attempt {}, retrying in {:?}: {:?}",
                            attempt + 1,
                            delay,
                            error
                        );
                        sleep(delay).await;
                        attempt += 1;
                    } else {
                        tracing::error!(
                            "Operation failed after {} attempts, giving up: {:?}",
                            attempt + 1,
                            error
                        );
                        return Err(error);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    /// Check if we should allow the operation to proceed
    pub fn should_allow_operation(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() >= self.timeout {
                        // Move to half-open to test
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    // Recovery confirmed, close circuit
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.last_failure = None;
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                self.success_count = 0;
                self.last_failure = None;
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    // Open circuit due to too many failures
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                // Failure during testing, go back to open
                self.state = CircuitState::Open;
                self.failure_count += 1;
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just increment counter
                self.failure_count += 1;
            }
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, Fut, T, E>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        if !self.should_allow_operation() {
            return Err(CircuitBreakerError::Open);
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(CircuitBreakerError::Operation(error))
            }
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 3, Duration::from_secs(60))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    Open,
    #[error("Operation failed: {0}")]
    Operation(#[from] E),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_exponential_backoff() {
        let strategy = RetryStrategy::Exponential {
            base_delay: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Duration::from_secs(1),
            max_attempts: 3,
        };

        assert_eq!(
            strategy.calculate_delay(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            strategy.calculate_delay(1),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            strategy.calculate_delay(2),
            Some(Duration::from_millis(400))
        );
        assert_eq!(strategy.calculate_delay(3), None);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let mut cb = CircuitBreaker::new(2, 1, Duration::from_secs(1));

        // Initially closed
        assert!(matches!(cb.state(), CircuitState::Closed));
        assert!(cb.should_allow_operation());

        // First failure
        cb.record_failure();
        assert!(matches!(cb.state(), CircuitState::Closed));
        assert!(cb.should_allow_operation());

        // Second failure - should open circuit
        cb.record_failure();
        assert!(matches!(cb.state(), CircuitState::Open));
        assert!(!cb.should_allow_operation());
    }

    #[tokio::test]
    async fn test_retry_strategy_execution() {
        let strategy = RetryStrategy::Fixed {
            interval: Duration::from_millis(10),
            max_attempts: 3,
        };

        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        let call_count = Arc::new(AtomicUsize::new(0));
        let count_clone = call_count.clone();

        let result = strategy
            .execute(move || {
                let count = count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 3 {
                        Err("temporary failure")
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert_eq!(result, Ok("success"));
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}
