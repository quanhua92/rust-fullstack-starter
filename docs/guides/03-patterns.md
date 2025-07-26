# Foundation Patterns

*This guide explains the core reliability patterns used throughout the system: circuit breakers, retry strategies, and dead letter queues.*

## Why Reliability Patterns?

When building systems that handle failures gracefully, certain patterns emerge repeatedly. This starter implements these patterns so you can:
- **Learn by Example**: See how they work in real code
- **Understand Trade-offs**: When to use each pattern
- **Build Confidence**: Handle failures without system crashes
- **Apply Elsewhere**: Use these patterns in your own projects

## Pattern 1: Circuit Breaker

### The Problem
Imagine your task system tries to send emails, but the email service is down. Without protection:
```
Task 1: Try email → Timeout (30 seconds)
Task 2: Try email → Timeout (30 seconds) 
Task 3: Try email → Timeout (30 seconds)
...
```
This wastes resources and creates cascading delays.

### The Solution: Circuit Breaker
```
Circuit States:
┌─────────┐    failures    ┌─────────┐    timeout     ┌─────────┐
│ CLOSED  │──────────────→ │  OPEN   │───────────────→│HALF_OPEN│
│(normal) │                │(blocked)│                │ (test)  │
└─────────┘                └─────────┘                └─────────┘
     ↑                                                      │
     │                     successes                        │
     └──────────────────────────────────────────────────────┘
```

### How It Works
```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,    // Open after N failures
    success_threshold: u32,    // Close after N successes  
    last_failure_time: Option<Instant>,
    timeout: Duration,         // How long to stay open
}

pub enum CircuitState {
    Closed,    // Normal operation - allow all requests
    Open,      // Failing - block all requests immediately  
    HalfOpen,  // Testing - allow limited requests
}
```

### Implementation Example
```rust
impl CircuitBreaker {
    pub async fn call<F, T, E>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitState::Closed | CircuitState::HalfOpen => {}
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(CircuitBreakerError::Inner(error))
            }
        }
    }

    fn on_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.reset_counts();
                }
            }
            CircuitState::Closed => {
                // Already healthy, reset failure count
                self.failure_count = 0;
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset to half-open
                self.state = CircuitState::HalfOpen;
            }
        }
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }
}
```

### When to Use Circuit Breakers
- **External Services**: API calls, database connections, email services
- **Resource Protection**: Prevent overwhelming failing systems
- **Fast Failure**: Return errors immediately instead of waiting for timeouts
- **System Stability**: Prevent cascading failures

### Configuration Example
```rust
// For email service
CircuitBreaker::new(
    failure_threshold: 5,    // Open after 5 failures
    success_threshold: 3,    // Close after 3 successes
    timeout: Duration::from_secs(60), // Stay open for 60 seconds
)

// For database operations  
CircuitBreaker::new(
    failure_threshold: 3,    // More sensitive
    success_threshold: 2,    // Recover faster
    timeout: Duration::from_secs(30),
)
```

## Pattern 2: Retry Strategies

### The Problem
Networks are unreliable. Services have hiccups. Sometimes the first try fails, but the second succeeds. How do you retry intelligently?

### Strategy 1: Exponential Backoff
**Concept**: Wait longer between each retry attempt.
```
Attempt 1: Immediate
Attempt 2: Wait 1 second → retry
Attempt 3: Wait 2 seconds → retry  
Attempt 4: Wait 4 seconds → retry
Attempt 5: Wait 8 seconds → retry
```

**Implementation**:
```rust
pub struct ExponentialBackoff {
    pub base_delay: Duration,
    pub multiplier: f64,
    pub max_delay: Duration,
    pub max_attempts: u32,
}

impl ExponentialBackoff {
    pub fn calculate_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None; // No more retries
        }

        let delay = self.base_delay.as_millis() as f64 
            * self.multiplier.powi(attempt as i32);
        
        let delay = Duration::from_millis(delay as u64);
        Some(delay.min(self.max_delay))
    }

    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut attempt = 0;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if let Some(delay) = self.calculate_delay(attempt) {
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                    } else {
                        return Err(error); // Max attempts reached
                    }
                }
            }
        }
    }
}
```

**When to Use**: 
- Network requests (avoid overwhelming servers)
- Database operations (temporary connection issues)
- File operations (temporary locks)

### Strategy 2: Linear Backoff  
**Concept**: Fixed increase between retries.
```
Attempt 1: Immediate
Attempt 2: Wait 1 second → retry
Attempt 3: Wait 2 seconds → retry
Attempt 4: Wait 3 seconds → retry
```

**Implementation**:
```rust
pub struct LinearBackoff {
    pub base_delay: Duration,
    pub increment: Duration,
    pub max_delay: Duration,
    pub max_attempts: u32,
}

impl LinearBackoff {
    pub fn calculate_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let delay = self.base_delay + self.increment * attempt;
        Some(delay.min(self.max_delay))
    }
}
```

**When to Use**:
- Rate-limited APIs (predictable backoff)
- Queue processing (steady pressure)
- Resource contention (gradual backing off)

### Strategy 3: Fixed Interval
**Concept**: Same delay between all retries.
```
Attempt 1: Immediate  
Attempt 2: Wait 30 seconds → retry
Attempt 3: Wait 30 seconds → retry
Attempt 4: Wait 30 seconds → retry
```

**Implementation**:
```rust
pub struct FixedInterval {
    pub interval: Duration,
    pub max_attempts: u32,
}

impl FixedInterval {
    pub fn calculate_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            None
        } else {
            Some(self.interval)
        }
    }
}
```

**When to Use**:
- Scheduled operations (cron-like behavior)
- Monitoring checks (consistent intervals)
- Simple retry logic (easy to reason about)

### Retry Strategy Selection Guide
```rust
// For external API calls
ExponentialBackoff {
    base_delay: Duration::from_millis(500),
    multiplier: 2.0,
    max_delay: Duration::from_secs(30),
    max_attempts: 5,
}

// For database operations
ExponentialBackoff {
    base_delay: Duration::from_millis(100),
    multiplier: 1.5,
    max_delay: Duration::from_secs(5),
    max_attempts: 3,
}

// For rate-limited services
LinearBackoff {
    base_delay: Duration::from_secs(1),
    increment: Duration::from_secs(5),
    max_delay: Duration::from_secs(60),
    max_attempts: 10,
}

// For periodic health checks
FixedInterval {
    interval: Duration::from_secs(30),
    max_attempts: 3,
}
```

## Pattern 3: Dead Letter Queue

### The Problem
Some tasks will always fail, no matter how many times you retry:
- Invalid email addresses
- Malformed data that can't be processed
- Requests to services that no longer exist

These tasks shouldn't retry forever, but you also shouldn't lose them completely.

### The Solution: Dead Letter Queue
```
┌─────────────┐    process    ┌─────────────┐    retry     ┌─────────────┐
│   Pending   │──────────────→│   Failed    │─────────────→│  Retrying   │
│   Tasks     │               │   Tasks     │              │   Tasks     │
└─────────────┘               └─────────────┘              └─────────────┘
                                     │                             │
                                     │ max retries exceeded        │
                                     ▼                             │
                               ┌─────────────┐                     │
                               │ Dead Letter │◀────────────────────┘
                               │   Queue     │
                               └─────────────┘
```

### Implementation
```rust
pub async fn process_failed_task(
    conn: &mut DbConn,
    task: &Task,
    error: &TaskError,
) -> Result<()> {
    // Increment attempt counter
    let new_attempt = task.current_attempt + 1;
    
    if new_attempt >= task.max_attempts {
        // Move to dead letter queue
        mark_task_as_dead_letter(conn, task.id, error).await?;
        log::warn!("Task {} moved to dead letter queue after {} attempts", 
                   task.id, new_attempt);
    } else {
        // Schedule for retry
        let retry_at = calculate_retry_time(&task.retry_strategy, new_attempt);
        schedule_task_retry(conn, task.id, retry_at, new_attempt, error).await?;
        log::info!("Task {} scheduled for retry {} at {}", 
                   task.id, new_attempt, retry_at);
    }
    
    Ok(())
}

async fn mark_task_as_dead_letter(
    conn: &mut DbConn,
    task_id: Uuid,
    error: &TaskError,
) -> Result<()> {
    sqlx::query!(
        "UPDATE tasks 
         SET status = 'failed', 
             last_error = $1, 
             updated_at = NOW()
         WHERE id = $2",
        error.to_string(),
        task_id
    )
    .execute(conn)
    .await?;
    
    Ok(())
}
```

### Dead Letter Queue Management
```rust
// Find tasks in dead letter queue
pub async fn get_dead_letter_tasks(
    conn: &mut DbConn,
    limit: i32,
) -> Result<Vec<Task>> {
    let tasks = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks 
         WHERE status = 'failed' 
           AND current_attempt >= max_attempts
         ORDER BY updated_at DESC 
         LIMIT $1",
        limit
    )
    .fetch_all(conn)
    .await?;
    
    Ok(tasks)
}

// Retry a dead letter task (manual intervention)
pub async fn retry_dead_letter_task(
    conn: &mut DbConn,
    task_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        "UPDATE tasks 
         SET status = 'pending',
             current_attempt = 0,
             last_error = NULL,
             updated_at = NOW()
         WHERE id = $1 
           AND status = 'failed'",
        task_id
    )
    .execute(conn)
    .await?;
    
    Ok(())
}
```

### When to Use Dead Letter Queues
- **Poison Messages**: Tasks that crash the processor
- **Invalid Data**: Malformed payloads that can't be fixed automatically
- **External Failures**: Services that are permanently unavailable
- **Debugging**: Investigate why certain tasks consistently fail

## Combining Patterns

### Circuit Breaker + Retry Strategy
```rust
pub struct ReliableTaskProcessor {
    circuit_breaker: CircuitBreaker,
    retry_strategy: ExponentialBackoff,
}

impl ReliableTaskProcessor {
    pub async fn process_task(&mut self, task: Task) -> Result<TaskResult> {
        let result = self.retry_strategy.execute(|| async {
            // Circuit breaker protects the actual operation
            self.circuit_breaker.call(|| async {
                self.execute_task_handler(&task).await
            }).await
        }).await;

        match result {
            Ok(success) => Ok(success),
            Err(error) => {
                // Send to dead letter queue if max retries exceeded
                self.handle_permanent_failure(task, error).await
            }
        }
    }
}
```

### Task System Integration
In our background job system, these patterns work together:

1. **Circuit Breaker**: Protects each task type (email, webhook, etc.)
2. **Retry Strategy**: Configured per task with exponential backoff
3. **Dead Letter Queue**: Failed tasks after max retries

```rust
// Task configuration example
Task {
    task_type: "email",
    retry_strategy: json!({
        "type": "exponential",
        "base_delay_ms": 1000,
        "multiplier": 2.0,
        "max_delay_ms": 300000,
        "max_attempts": 5
    }),
    max_attempts: 5,
    // ... other fields
}
```

## Configuration Examples

### Conservative (High Reliability)
```rust
// Circuit breaker: Very sensitive to failures
CircuitBreaker::new(
    failure_threshold: 2,
    success_threshold: 5,
    timeout: Duration::from_secs(120),
)

// Retry: Many attempts with long delays  
ExponentialBackoff {
    base_delay: Duration::from_secs(2),
    multiplier: 2.0,
    max_delay: Duration::from_secs(300),
    max_attempts: 8,
}
```

### Aggressive (High Performance)
```rust
// Circuit breaker: Tolerates more failures
CircuitBreaker::new(
    failure_threshold: 10,
    success_threshold: 2, 
    timeout: Duration::from_secs(30),
)

// Retry: Fewer attempts with short delays
ExponentialBackoff {
    base_delay: Duration::from_millis(200),
    multiplier: 1.5,
    max_delay: Duration::from_secs(10),
    max_attempts: 3,
}
```

### Balanced (Good Default)
```rust
// Circuit breaker: Reasonable thresholds
CircuitBreaker::new(
    failure_threshold: 5,
    success_threshold: 3,
    timeout: Duration::from_secs(60),
)

// Retry: Standard exponential backoff
ExponentialBackoff {
    base_delay: Duration::from_secs(1),
    multiplier: 2.0,
    max_delay: Duration::from_secs(60),
    max_attempts: 5,
}
```

## Testing Patterns

### Circuit Breaker Tests
```rust
#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    let mut cb = CircuitBreaker::new(2, 1, Duration::from_secs(1));
    
    // First failure
    let result = cb.call(|| async { Err::<(), _>("error") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Inner(_))));
    
    // Second failure - should open circuit
    let result = cb.call(|| async { Err::<(), _>("error") }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Inner(_))));
    
    // Third call - should be blocked
    let result = cb.call(|| async { Ok::<(), String>(()) }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Open)));
}
```

### Retry Strategy Tests
```rust
#[tokio::test]
async fn test_exponential_backoff_delays() {
    let strategy = ExponentialBackoff {
        base_delay: Duration::from_millis(100),
        multiplier: 2.0,
        max_delay: Duration::from_secs(1),
        max_attempts: 4,
    };

    assert_eq!(strategy.calculate_delay(0), Some(Duration::from_millis(100)));
    assert_eq!(strategy.calculate_delay(1), Some(Duration::from_millis(200)));
    assert_eq!(strategy.calculate_delay(2), Some(Duration::from_millis(400)));
    assert_eq!(strategy.calculate_delay(3), Some(Duration::from_millis(800)));
    assert_eq!(strategy.calculate_delay(4), None); // Max attempts reached
}
```

## Next Steps

Now that you understand these reliability patterns, see how they're used in practice:

- **[Background Jobs →](./04-background-jobs.md)** - How the task system uses these patterns
- **[Custom Task Types →](./05-task-types.md)** - Applying patterns to your own tasks

---
*These patterns form the reliability foundation for the entire system. Understanding them helps you build robust, fault-tolerant applications.*