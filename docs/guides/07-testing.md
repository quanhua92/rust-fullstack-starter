# Testing Guide

*This guide explains the comprehensive testing framework included in the starter, designed to help you learn testing patterns while building reliable applications.*

## 🤔 Why This Testing Approach? (First Principles)

### The Fundamental Problem: Confidence in Code

**What developers need to know**:
- Does my code actually work?
- Will it work in production conditions?
- Can I change code without breaking things?
- How do I debug when things go wrong?

**What traditional testing often provides**:
- Unit tests that mock everything (doesn't test real interactions)
- End-to-end tests that are flaky and slow
- Complex setup that discourages writing tests
- False confidence from tests that don't match production

### Testing Strategy Comparison

| Strategy | What It Tests | Pros | Cons | When to Use |
|----------|---------------|------|------|-------------|
| **Unit Tests** | Individual functions in isolation | Fast, focused | Misses integration issues | Pure functions, algorithms |
| **Integration Tests** ⭐ | Multiple components working together | Realistic, catches real bugs | Slower setup | Web APIs, database operations |
| **End-to-End Tests** | Full user workflows | Most realistic | Slow, flaky, complex | Critical user journeys |
| **Contract Tests** | API contracts between services | Catches breaking changes | Extra infrastructure | Microservices, APIs |
| **Property Tests** | Random inputs, invariants | Finds edge cases | Hard to write, understand | Validation logic, parsers |

### Why Integration Tests for This Starter?

**Our First Principles Decision**:

**Principle 1: Test What Matters**
- Web applications are about HTTP requests, database operations, and business logic working together
- Isolated unit tests miss the most common failure points (serialization, database queries, middleware)
- Integration tests catch real bugs that users experience

**Principle 2: Learning Value**
- Shows how to test real application behavior
- Demonstrates database isolation patterns
- Teaches debugging skills through test failures
- Builds confidence in the full system

**Principle 3: Development Speed**
- Faster feedback than manual testing
- Catches regressions before deployment
- Documents expected behavior
- Enables confident refactoring

### 🧠 Mental Model: Testing Pyramid (Inverted for Web Apps)

```mermaid
graph TD
    subgraph "🧪 Testing Strategy for Web Applications"
        E2E[🌐 E2E Tests<br/><i>Few, critical paths</i><br/>~5% of tests<br/>💡 User journeys]
        
        INTEGRATION[🔌 Integration Tests<br/><b>Main focus - 53 tests</b><br/>~80% of tests<br/>💡 HTTP + Database + Business Logic]
        
        UNIT[⚙️ Unit Tests<br/><i>Pure functions only</i><br/>~15% of tests<br/>💡 Algorithms, validation]
    end
    
    subgraph "📊 What Each Layer Provides"
        CONFIDENCE[📈 Confidence<br/>Real behavior]
        SPEED[⚡ Feedback Speed<br/>Quick iteration]
        COVERAGE[🎯 Coverage<br/>Edge cases]
    end
    
    E2E --> CONFIDENCE
    INTEGRATION --> CONFIDENCE
    INTEGRATION --> SPEED
    UNIT --> SPEED
    UNIT --> COVERAGE
    
    classDef main fill:#e8f5e8,stroke:#1b5e20,stroke-width:3px
    classDef support fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef benefit fill:#fff3e0,stroke:#e65100,stroke-width:2px
    
    class INTEGRATION main
    class E2E,UNIT support
    class CONFIDENCE,SPEED,COVERAGE benefit
```

**Key Insight**: For web applications, integration tests provide the best balance of confidence, speed, and maintenance cost.

## Testing Philosophy

### Why Integration Tests?

Integration tests verify that different parts of your system work together correctly:
- **Real HTTP Requests**: Test actual server responses
- **Database Operations**: Verify data persistence and retrieval  
- **Authentication Flow**: Test login, authorization, and security
- **Error Handling**: Ensure proper error responses and codes

### Testing Pyramid (Simplified)

```mermaid
graph TD
    subgraph "🧪 Testing Strategy for Learning"
        E2E[🌐 E2E Tests<br/><i>Manual Testing</i><br/>🚫 Not automated<br/>💡 Too complex for starter]
        
        INTEGRATION[🔌 Integration Tests<br/><b>53 comprehensive tests</b><br/>✅ Main focus<br/>💡 Test real interactions]
        
        CHAOS[🔥 Chaos Tests<br/><b>7 resilience scenarios</b><br/>🐳 Docker-based testing<br/>💡 Test failure handling]
        
        UNIT[⚙️ Unit Tests<br/><i>Some in modules</i><br/>📝 Function-level<br/>💡 Fast feedback]
    end
    
    subgraph "📊 Test Coverage Areas"
        AUTH_T[🔐 Authentication<br/>Login, Sessions, Security]
        API_T[🌐 API Standards<br/>CORS, Headers, Errors]
        TASK_T[⚙️ Task Processing<br/>Jobs, Registry, Dead Letter]
        HEALTH_T[💓 Health Checks<br/>Database, Services]
    end
    
    E2E -.->|🎭 User Journeys| AUTH_T
    INTEGRATION -->|📡 HTTP Testing| AUTH_T
    INTEGRATION -->|📡 HTTP Testing| API_T
    INTEGRATION -->|📡 HTTP Testing| TASK_T
    INTEGRATION -->|📡 HTTP Testing| HEALTH_T
    UNIT -->|⚡ Quick Validation| AUTH_T
    UNIT -->|⚡ Quick Validation| TASK_T
    
    classDef focusBox fill:#e8f5e8,stroke:#1b5e20,stroke-width:3px
    classDef supportBox fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef manualBox fill:#fff3e0,stroke:#e65100,stroke-width:2px
    
    class INTEGRATION focusBox
    class UNIT supportBox
    class E2E manualBox
```

**Why Focus on Integration Tests?**
- **🎯 Real World**: Test how components work together
- **🚀 Fast Setup**: No complex browser automation
- **💡 Educational**: See how HTTP, database, and business logic connect
- **🔄 Reliable**: Isolated test databases prevent flaky tests

This starter focuses on integration tests because they:
- Catch real-world bugs
- Test the full request/response cycle
- Verify database interactions
- Are easier to understand for beginners

## Test Architecture

### 1. TestApp Pattern

The `TestApp` spawns real server instances for testing:

```rust
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub client: reqwest::Client,
    pub db: TestDatabase,
}

// Usage in tests
#[tokio::test]
async fn test_user_registration() {
    let app = spawn_app().await;  // Starts real server
    
    let response = app.post_json("/auth/register", &user_data).await;
    assert_status(&response, StatusCode::OK);
}
```

**Benefits**:
- Tests real HTTP behavior
- Catches routing issues
- Verifies middleware behavior
- Tests serialization/deserialization

### 2. Template Database Pattern

Each test gets an isolated database for complete test independence:

```rust
// One-time setup: Create template with migrations
CREATE DATABASE starter_test_template;
-- Run all migrations on template

// Per test: Clone from template (10x faster)
CREATE DATABASE test_uuid_12345 WITH TEMPLATE starter_test_template;
```

**Performance**: 
- Template creation: ~2-3 seconds (one time setup)
- Test database clone: ~200-300ms (per test)
- Traditional migrations: ~2-3 seconds per test
- **Total speedup**: 10x faster test execution

**Benefits**:
- Complete test isolation
- 10x faster than running migrations per test
- No data contamination between tests
- Parallel test execution safe

### 3. Test Data Factories

Consistent test data creation with authentication support:

```rust
pub struct TestDataFactory {
    pub app: TestApp,
}

impl TestDataFactory {
    // Create user without authentication
    pub async fn create_user(&self, username: &str) -> User { ... }
    
    // Create user and return auth token
    pub async fn create_authenticated_user(&self, username: &str) -> (User, AuthToken) { ... }
    
    // Create background tasks
    pub async fn create_task(&self, task_type: &str, payload: Value) -> Value { ... }
}
```

**Usage**:
```rust
#[tokio::test]
async fn test_protected_endpoint() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // This handles user creation + login + token extraction
    let (user, token) = factory.create_authenticated_user("testuser").await;
    
    let response = app.get_auth("/auth/me", &token.token).await;
    assert_status(&response, StatusCode::OK);
}
```

### 4. Helper Utilities

Common test operations made simple:

```rust
// HTTP assertions
assert_status(&response, StatusCode::OK);
assert_status(&response, StatusCode::UNAUTHORIZED);

// JSON assertions  
assert_json_field_exists(&json, "data");
assert_eq!(json["data"]["status"], "healthy");

// Async waiting
let success = wait_for(|| async {
    check_some_condition().await
}, 5000).await;
```

## Test Categories

### Authentication Tests (`auth/mod.rs`)

Tests the complete authentication flow:

```rust
#[tokio::test]
async fn test_user_registration_success() {
    let app = spawn_app().await;
    
    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com", 
        "password": "SecurePass123!"
    });

    let response = app.post_json("/auth/register", &user_data).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "success");
    assert_json_field_exists(&json, "data");
}
```

**Covers**:
- User registration with validation
- Login with correct/incorrect credentials
- Token-based authentication
- Duplicate user handling
- Password strength requirements

### Task System Tests (`tasks/mod.rs`)

Tests the background job system:

```rust
#[tokio::test]
async fn test_create_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Tasks require authentication
    let task_response = factory.create_task("email", json!({
        "to": "test@example.com",
        "subject": "Test",
        "body": "Hello"
    })).await;
    
    assert_json_field_exists(&task_response, "data");
    assert_eq!(task_response["data"]["task_type"], "email");
}
```

**Covers**:
- Task creation with different types
- Task status tracking
- Task listing and filtering
- Priority handling
- Authentication requirements

### Health Check Tests (`health/mod.rs`)

Tests monitoring endpoints:

```rust
#[tokio::test]
async fn test_health_endpoint() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["status"], "healthy");
}
```

**Covers**:
- Basic health checks
- Detailed health with database status
- Version information
- Uptime reporting

### API Standards Tests (`api/mod.rs`)

Tests cross-cutting concerns:

```rust
#[tokio::test]
async fn test_api_security_headers() {
    let app = spawn_app().await;
    
    let response = app.get("/health").await;
    
    assert_status(&response, StatusCode::OK);
    let headers = response.headers();
    
    // Security headers
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
}
```

**Covers**:
- CORS configuration
- Security headers (X-Frame-Options, X-Content-Type-Options)
- Content-Type headers
- Error response format consistency
- Request ID headers

### User Management Tests (`users/mod.rs`)

Tests user operations:

```rust
#[tokio::test]
async fn test_get_user_profile_authenticated() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    let (user, token) = factory.create_authenticated_user("testuser").await;
    
    let response = app.get_auth("/auth/me", &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["username"], "testuser");
}
```

**Covers**:
- Profile retrieval with authentication
- User lookup by ID
- Authorization requirements
- Nonexistent user handling

## Running Tests

### Basic Commands

```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (38 integration tests, ~10 seconds)
cargo nextest run

# Run without stopping on first failure
cargo nextest run --no-fail-fast

# Traditional cargo test (slower)
cargo test
```

### API Endpoint Testing

The starter includes a comprehensive curl-based test script for API validation:

```bash
# Test default server (localhost:3000)
./scripts/test-with-curl.sh

# Test custom host and port
./scripts/test-with-curl.sh localhost 8080
./scripts/test-with-curl.sh api.example.com 443  # Auto-detects HTTPS

# Example output
🧪 Comprehensive API Testing with curl
==================================================
Testing: localhost:3000 (http)
Base URL: http://localhost:3000

📊 Health Endpoints
✅ PASS GET /health (Status: 200)
✅ PASS GET /health/detailed (Status: 200)
✅ PASS GET /health/live (Status: 200)
✅ PASS GET /health/ready (Status: 200)
✅ PASS GET /health/startup (Status: 200)

🔐 Authentication Flow  
✅ PASS POST /auth/register (Status: 200)
✅ PASS POST /auth/login (Token obtained)
...

📊 Test Results Summary
Total tests: 26
Passed: 26
Success rate: 100%
```

**Features**:
- Tests all 15 documented API endpoints
- Validates input/output formats against documentation
- Tests authentication flows and error cases
- Supports custom server configurations
- Non-interactive automated execution

### Targeted Testing

```bash
# Test specific modules
cargo nextest run auth::
cargo nextest run tasks::
cargo nextest run health::

# Test specific functions
cargo nextest run test_user_registration
cargo nextest run test_create_task

# Test with output for debugging
TEST_LOG=1 cargo test test_health_endpoint -- --nocapture
```

### Environment Variables

```bash
# Enable debug logging in tests
TEST_LOG=1 cargo nextest run

# Use different database for testing
TEST_DATABASE_URL=postgres://user:pass@localhost/test_db cargo test
```

## Test Performance

### Database Template Optimization

The template pattern provides significant speedup:

```
Traditional approach:
- Test 1: 2.5s (migrations)
- Test 2: 2.5s (migrations)  
- Test 3: 2.5s (migrations)
Total: 7.5s for 3 tests

Template approach:
- Setup: 2.5s (create template)
- Test 1: 0.3s (clone template)
- Test 2: 0.3s (clone template)
- Test 3: 0.3s (clone template)  
Total: 3.4s for 3 tests (55% faster)
```

With 40 tests, this saves significant development time.

### Parallel Execution

Tests run in parallel safely because:
- Each test has isolated database
- Each test spawns on random port
- No shared state between tests

## Writing New Tests

### 1. Basic Test Structure

```rust
use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_my_feature() {
    // 1. Setup
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // 2. Prepare data (if needed)
    let (user, token) = factory.create_authenticated_user("testuser").await;
    
    // 3. Execute
    let response = app.post_json_auth("/my/endpoint", &data, &token.token).await;
    
    // 4. Assert
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["result"], "expected");
}
```

### 2. Testing Authenticated Endpoints

```rust
#[tokio::test] 
async fn test_protected_feature() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Test without authentication (should fail)
    let response = app.get("/protected/endpoint").await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
    
    // Test with authentication (should succeed)
    let (_user, token) = factory.create_authenticated_user("user").await;
    let response = app.get_auth("/protected/endpoint", &token.token).await;
    assert_status(&response, StatusCode::OK);
}
```

### 3. Testing Error Cases

```rust
#[tokio::test]
async fn test_validation_errors() {
    let app = spawn_app().await;
    
    // Test invalid data
    let invalid_data = json!({
        "email": "not-an-email",
        "password": "weak"
    });
    
    let response = app.post_json("/auth/register", &invalid_data).await;
    assert_status(&response, StatusCode::BAD_REQUEST);
    
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "error");
}
```

### 4. Testing with Multiple Users

```rust
#[tokio::test]
async fn test_user_isolation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create multiple users
    let (user1, token1) = factory.create_authenticated_user("user1").await;
    let (user2, token2) = factory.create_authenticated_user("user2").await;
    
    // Each user should only see their own data
    let response1 = app.get_auth("/auth/me", &token1.token).await;
    let response2 = app.get_auth("/auth/me", &token2.token).await;
    
    assert_status(&response1, StatusCode::OK);
    assert_status(&response2, StatusCode::OK);
    
    // Verify user isolation
    let json1: serde_json::Value = response1.json().await.unwrap();
    let json2: serde_json::Value = response2.json().await.unwrap();
    
    assert_eq!(json1["data"]["username"], "user1");
    assert_eq!(json2["data"]["username"], "user2");
}
```

## Debugging Tests

### Common Issues

1. **Database Connection Errors**:
   ```bash
   # Check if PostgreSQL is running
   docker compose ps postgres
   
   # Check connection
   psql $DATABASE_URL -c "SELECT 1"
   ```

2. **Port Conflicts**:
   ```bash
   # Tests use random ports, but check for conflicts
   netstat -tulpn | grep :808
   ```

3. **Migration Issues**:
   ```bash
   # Reset database
   docker compose down -v
   docker compose up -d postgres
   sqlx migrate run
   ```

### Debugging Specific Tests

```bash
# Run single test with output
cargo test test_user_registration -- --nocapture

# Run with debug logging
TEST_LOG=1 cargo test test_create_task -- --nocapture

# Run with Rust backtrace
RUST_BACKTRACE=1 cargo test test_failing_test
```

### Test Database Inspection

```bash
# Connect to template database
psql postgresql://starter_user:starter_pass@localhost:5432/starter_test_template

# List test databases (they're cleaned up automatically)
psql postgresql://starter_user:starter_pass@localhost:5432/postgres -c "\l" | grep test_
```

## Best Practices

### 1. Test Independence
- Each test should work in isolation
- Don't rely on test execution order
- Clean up is automatic (database per test)

### 2. Meaningful Assertions
```rust
// Good: Specific assertion
assert_eq!(json["data"]["status"], "healthy");

// Bad: Generic assertion
assert!(json["data"].is_object());
```

### 3. Test Data
```rust
// Good: Use factories for consistency
let (user, token) = factory.create_authenticated_user("testuser").await;

// Avoid: Manual data creation in every test
```

### 4. Error Testing
```rust
// Test both success and failure cases
#[tokio::test]
async fn test_login_success() { /* ... */ }

#[tokio::test] 
async fn test_login_invalid_credentials() { /* ... */ }
```

### 5. Documentation
```rust
#[tokio::test]
async fn test_user_registration_duplicate_username() {
    // This test verifies that attempting to register
    // a user with an existing username returns a 409 Conflict
    // rather than allowing duplicate users
}
```

## Limitations & Future Improvements

### Current Limitations
- **No E2E Tests**: Only integration tests, no browser automation
- **Limited Load Testing**: Not designed for performance testing
- **Simple Error Cases**: Basic error handling testing
- **No Mocking**: Tests against real database (by design)

### Learning Opportunities
This testing framework demonstrates:
- Database isolation patterns
- Authentication testing strategies  
- HTTP testing approaches
- Test data management
- Assertion patterns

Use these patterns as a foundation for your own testing strategies.

### Extending the Framework
Consider adding:
- Property-based tests for validation logic
- Performance benchmarks for critical paths
- Contract testing for API consumers
- Chaos testing for reliability (see [Chaos Testing Guide](08-chaos-testing.md))

## Next Steps

Now that you understand the testing framework:

1. **Run the Tests**: `cargo nextest run` to see all 40 tests pass
2. **Explore Test Code**: Read through `starter/tests/` to understand patterns
3. **Write New Tests**: Add tests for any new features you build
4. **Learn from Failures**: When tests fail, use them to understand the system

The testing framework is designed to grow with your application. As you add features, add corresponding tests using these established patterns.

---

*This testing framework provides a solid foundation for reliable development. While it's designed for learning and starter projects, the patterns demonstrated here scale to production applications.*