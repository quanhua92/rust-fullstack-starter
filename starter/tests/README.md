# Integration Tests

**📖 See [../../docs/TESTING-GUIDE.md](../../docs/TESTING-GUIDE.md) for comprehensive testing documentation including the complete 7-layer testing architecture, workflows, and best practices.**

This directory contains comprehensive integration tests for the Rust starter application, following the excellent patterns from the production codebase.

## Test Architecture

- **TestApp Pattern**: Spawns real server instances on random ports
- **Template Database Pattern**: Fast database setup using PostgreSQL templates (10x speedup)
- **Test Data Factories**: Consistent test data generation with builder patterns
- **Helper Utilities**: Common assertions and utilities
- **Atomic Synchronization**: Safe concurrent test execution

## Structure

```
tests/
├── helpers/
│   ├── test_app.rs     # TestApp structure for spawning test servers
│   ├── db.rs           # Database template optimization with atomic synchronization
│   ├── test_data.rs    # Test data factories and builders
│   └── utils.rs        # Common test utilities and assertions
├── auth/
│   └── mod.rs          # Authentication integration tests
├── users/
│   └── mod.rs          # User management integration tests
├── tasks/
│   └── mod.rs          # Task processing integration tests
├── cli/
│   └── mod.rs          # CLI admin commands integration tests
├── health/
│   └── mod.rs          # Health check integration tests
├── api/
│   └── mod.rs          # API integration tests (CORS, security, etc.)
└── lib.rs              # Test module exports
```

## Running Tests

```bash
# Run all tests
cargo test

# Run with test output
TEST_LOG=1 cargo test -- --nocapture

# Run specific test module
cargo test auth::

# Run specific test
cargo test test_user_registration_success

# Run tests in parallel (default)
cargo test

# Run tests sequentially (if needed)
cargo test -- --test-threads=1
```

## Test Features

### Database Isolation
Each test gets its own isolated PostgreSQL database created from a template. This ensures:
- Complete test isolation
- No data contamination between tests
- Fast setup (template cloning is 10x faster than running migrations)

### Test Data Factories
The `TestDataFactory` provides consistent test data creation:
- `create_user()` - Creates a test user
- `create_authenticated_user()` - Creates user and returns auth token
- `create_multiple_users()` - Creates multiple users for pagination tests
- `create_task()` - Creates test tasks

### Builder Pattern
Use the builder pattern for custom test data:
```rust
let user_request = UserBuilder::new()
    .with_username("custom_user")
    .with_email("custom@example.com")
    .with_role("admin")
    .build();
```

### Test Utilities
Common test utilities include:
- `assert_status()` - Assert HTTP status codes
- `assert_json_field()` - Assert JSON field values
- `assert_json_field_exists()` - Assert JSON fields exist
- `wait_for()` - Wait for conditions with timeout
- `random_string()` / `random_email()` - Generate random test data

## Test Categories

### Authentication Tests (`auth/mod.rs`)
- User registration (success, validation errors)
- User login (success, invalid credentials)
- Token validation
- Duplicate user handling

### User Management Tests (`users/mod.rs`)
- User listing and pagination
- User profile retrieval
- User profile updates
- Authentication requirements

### Task Tests (`tasks/mod.rs`)
- Task creation and validation
- Task status tracking
- Task listing and filtering
- Priority handling
- Retry mechanisms

### CLI Tests (`cli/mod.rs`)
- Admin service database operations
- Task statistics retrieval
- Task listing and filtering
- Task cleanup operations
- CLI command validation (8 integration tests)

### Health Tests (`health/mod.rs`)
- Basic health endpoint
- Readiness probes
- Liveness probes
- Database health checks

### API Tests (`api/mod.rs`)
- CORS headers
- Content-Type headers
- Error response format
- Security headers
- Rate limiting
- Authentication requirements

## Environment Variables

- `TEST_LOG=1` - Enable debug logging in tests
- `DATABASE_URL` - Test database connection string (uses isolated test databases)

## Performance

The template database pattern provides significant performance improvements:
- Template creation: ~2-3 seconds (one-time setup)
- Test database creation: ~200-300ms (from template)
- Traditional migration: ~2-3 seconds per test

This results in a 10x speedup for database-heavy integration tests.

## Best Practices

1. **Use factories** for consistent test data
2. **Use builders** for custom test scenarios
3. **Use helpers** for common assertions
4. **Test isolation** - each test should be independent
5. **Cleanup** - databases are automatically cleaned up
6. **Meaningful assertions** - use specific assertion helpers
7. **Test documentation** - add comments for complex test scenarios