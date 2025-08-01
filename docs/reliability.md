# Reliability Guide

*This guide explains the reliability patterns and testing strategies included in the starter to help you learn foundation concepts for building robust applications.*

## Overview

Building reliable applications requires handling failures gracefully and testing thoroughly. This starter demonstrates common reliability patterns in a learning-friendly way, giving you practical experience with concepts you'll use in real applications.

**Important**: This is a starter framework designed for learning and development. The patterns demonstrated here are educational examples that you should adapt and extend for production use.

## Reliability Patterns

### 1. Circuit Breaker Pattern

The circuit breaker protects your application from cascading failures by temporarily blocking requests to failing services.

**How it works**:
```
Normal Operation → Failures Detected → Circuit Opens (blocks requests)
                                    ↓
Periodic Testing ← Circuit Half-Open ← Timeout Expires
     ↓
Success → Circuit Closes (normal operation)
```

**Implementation in the starter**:
```rust
// Example from src/tasks/retry.rs
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    timeout: Duration,
}

// Used in task processing to protect external services
```

**Use cases demonstrated**:
- Email service protection
- External API calls
- Database operation protection

### 2. Retry Strategies

The starter implements multiple retry strategies for different failure scenarios:

**Exponential Backoff**: Wait progressively longer between retries
```
Attempt 1: Immediate
Attempt 2: Wait 1s → retry
Attempt 3: Wait 2s → retry  
Attempt 4: Wait 4s → retry
```

**Linear Backoff**: Fixed increment between retries
```
Attempt 1: Immediate
Attempt 2: Wait 1s → retry
Attempt 3: Wait 2s → retry
Attempt 4: Wait 3s → retry
```

**Fixed Interval**: Same delay for all retries
```
Attempt 1: Immediate
Attempt 2: Wait 30s → retry
Attempt 3: Wait 30s → retry
```

### 3. Error Handling

Structured error handling throughout the application:

```rust
// Centralized error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Validation failed for {field}: {message}")]
    ValidationError { field: String, message: String },
}

// Consistent JSON error responses
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Password must be at least 8 characters"
  }
}
```

## Testing Strategy

### Testing Philosophy

The starter emphasizes **integration testing** because it:
- Tests real interactions between components
- Catches configuration issues
- Verifies database operations
- Tests the full HTTP request/response cycle
- Is easier to understand for learning

### Test Architecture

#### 1. TestApp Pattern
```rust
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: reqwest::Client,
    pub db: TestDatabase,
}

// Spawns real server for testing
let app = spawn_app().await;
let response = app.post_json("/auth/register", &data).await;
```

#### 2. Database Isolation
Each test gets its own PostgreSQL database:
```
Template Database (created once)
├── All migrations applied
├── Used as template for test databases
└── 10x faster than running migrations per test

Test Database (per test)
├── Cloned from template
├── Isolated from other tests
└── Automatically cleaned up
```

#### 3. Test Data Factories
Consistent test data creation:
```rust
let factory = TestDataFactory::new(app.clone());

// Create user without auth
let user = factory.create_user("testuser").await;

// Create user with auth token
let (user, token) = factory.create_authenticated_user("testuser").await;

// Create background task
let task = factory.create_task("send_email", payload).await;
```

### Test Coverage

The starter includes 119 integration tests covering:

#### Authentication Tests
- User registration with validation
- Login flow with correct/incorrect credentials
- Token-based authentication
- Session management
- Password strength requirements

#### API Standards Tests
- CORS header configuration
- Security headers (X-Frame-Options, X-Content-Type-Options)
- Request ID headers
- JSON response format consistency
- Error response structure

#### Task System Tests
- Background task creation
- Task status tracking
- Task listing and filtering
- Priority handling
- Authentication requirements for task operations

#### Health Check Tests
- Basic health endpoint
- Detailed health with dependency status
- Database connectivity verification
- Version and uptime reporting

#### User Management Tests
- User profile operations
- Authorization requirements
- User lookup by ID
- Profile updates

### Performance Optimization

#### Template Database Pattern
Traditional approach vs. optimized approach:

```
Traditional (migrations per test):
Test 1: 2.5s (run migrations)
Test 2: 2.5s (run migrations)
Test 3: 2.5s (run migrations)
Total: 7.5s for 3 tests

Template approach:
Setup: 2.5s (create template once)
Test 1: 0.3s (clone from template)
Test 2: 0.3s (clone from template)
Test 3: 0.3s (clone from template)
Total: 3.4s for 3 tests (55% faster)
```

For 119 tests, this saves significant development time.

#### Parallel Test Execution
Tests run safely in parallel because:
- Each test has isolated database
- Each test spawns on random port
- No shared state between tests
- Atomic synchronization with OnceCell

## Monitoring and Observability

### Health Checks

The starter implements standard health check patterns:

```rust
// Basic health - always available
GET /health
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": 1234.56
  }
}

// Detailed health - includes dependencies
GET /health/detailed
{
  "success": true,
  "data": {
    "status": "healthy",
    "checks": {
      "database": {
        "status": "healthy",
        "message": "Database connection successful"
      }
    }
  }
}

// Kubernetes health probes for container orchestration
GET /health/live         // Liveness probe - minimal checks
GET /health/ready        // Readiness probe - dependency checks  
GET /health/startup      // Startup probe - initialization checks
```

### Request Tracing

Basic request tracking:
- Request ID headers on all responses
- Structured logging with tracing
- Error context preservation

### Security Headers

Standard security headers included:
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-Request-ID: <unique-id>
```

## Development Workflow

### Testing During Development

```bash
# Run tests frequently during development
cargo nextest run

# Run specific test categories
cargo nextest run auth::
cargo nextest run tasks::

# Run with output for debugging
TEST_LOG=1 cargo test test_name -- --nocapture

# Check all tests pass before committing
cargo nextest run --no-fail-fast
```

### Adding New Features

1. **Write Tests First** (TDD approach):
   ```rust
   #[tokio::test]
   async fn test_new_feature() {
       let app = spawn_app().await;
       // Test the feature you're about to build
   }
   ```

2. **Implement Feature**:
   - Add the actual functionality
   - Run tests to verify behavior

3. **Verify Integration**:
   - Run full test suite
   - Check health endpoints
   - Verify error handling

### Debugging Failures

```bash
# Run single test with full output
cargo test test_user_registration -- --nocapture

# Enable debug logging
TEST_LOG=1 cargo test test_create_task -- --nocapture

# Check database state
psql $DATABASE_URL -c "SELECT * FROM users LIMIT 5"

# View test database template
psql postgresql://starter_user:starter_pass@localhost:5432/starter_test_template
```

## Configuration for Different Environments

### Development Configuration
```rust
// Relaxed settings for fast development
CircuitBreaker::new(
    failure_threshold: 10,    // More tolerant
    timeout: Duration::from_secs(30),  // Shorter timeout
)

ExponentialBackoff {
    base_delay: Duration::from_millis(100),  // Fast retries
    max_attempts: 3,  // Fewer attempts
}
```

### Testing Configuration
```rust
// Fast settings for test execution
CircuitBreaker::new(
    failure_threshold: 2,     // Fail fast in tests
    timeout: Duration::from_secs(1),  // Very short timeout
)

FixedInterval {
    interval: Duration::from_millis(10),  // Minimal delays
    max_attempts: 2,  // Quick failure
}
```

### Production Considerations
When adapting for production, consider:

```rust
// More conservative settings
CircuitBreaker::new(
    failure_threshold: 5,     // Balanced threshold
    success_threshold: 3,     // Require multiple successes
    timeout: Duration::from_secs(60),  // Longer recovery time
)

ExponentialBackoff {
    base_delay: Duration::from_secs(1),    // Longer initial delay
    multiplier: 2.0,           // Standard doubling
    max_delay: Duration::from_secs(300),   // 5 minute max
    max_attempts: 8,           // More retry attempts
}
```

## Limitations and Learning Opportunities

### Current Limitations
- **Simplified Error Handling**: Basic error types for learning
- **No Distributed Tracing**: Single-service focus
- **Limited Metrics**: Basic health checks only
- **Simple Circuit Breaker**: Doesn't persist state across restarts
- **In-Memory Session Storage**: Not suitable for multiple instances

### Learning Opportunities
This starter demonstrates:
- **Foundation Patterns**: Circuit breakers, retries, health checks
- **Testing Strategies**: Integration testing, database isolation
- **Error Handling**: Structured errors, consistent responses
- **Authentication**: Token-based auth, session management
- **API Design**: REST conventions, security headers

### Next Steps for Production
Consider adding:
- **Distributed Tracing**: OpenTelemetry or similar
- **Metrics Collection**: Prometheus, StatsD
- **Persistent Circuit Breakers**: Redis or database storage
- **Load Balancing**: Multiple instance support
- **Rate Limiting**: Request throttling
- **Audit Logging**: Security event tracking
- **Chaos Engineering**: See [Chaos Testing Guide](guides/09-chaos-testing.md) for systematic resilience testing with 6-level difficulty progression

## Best Practices Demonstrated

### 1. Fail Fast
```rust
// Quick validation to avoid wasted processing
if task.task_type.trim().is_empty() {
    return Err(Error::validation("task_type", "Cannot be empty"));
}
```

### 2. Graceful Degradation
```rust
// Health check continues even if some checks fail
let db_status = check_database().await.unwrap_or_else(|_| ComponentHealth {
    status: "unhealthy".to_string(),
    message: Some("Database unavailable".to_string()),
});
```

### 3. Observable Failures
```rust
// Structured logging for debugging
tracing::error!(
    task_id = %task.id,
    error = %error,
    attempt = task.current_attempt,
    "Task execution failed"
);
```

### 4. Test Coverage
Every public endpoint and major code path has corresponding tests:
```rust
// Test both success and failure cases
#[tokio::test]
async fn test_user_registration_success() { /* ... */ }

#[tokio::test]
async fn test_user_registration_duplicate_username() { /* ... */ }
```

## Conclusion

This starter provides a foundation for understanding reliability patterns and testing strategies. The implementations are designed for learning and development use.

For production applications:
- Extend these patterns based on your specific requirements
- Add comprehensive monitoring and alerting
- Implement proper secret management
- Add load testing and chaos engineering
- Consider distributed system patterns

The testing framework and reliability patterns demonstrated here give you practical experience with concepts that scale to production systems.

---

*Remember: This is a starter framework for learning. Use these patterns as a foundation, but adapt them to your specific needs and requirements.*