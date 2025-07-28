# Getting Started

This guide will help you set up and run the Rust Full-Stack Starter project locally.

## Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Docker 20.10+** and **Docker Compose 2.0+** - For database infrastructure
- **PostgreSQL client tools** (optional) - For database inspection

> **Performance Note**: Setup typically takes 2-3 seconds, test execution ~12 seconds for all 53 tests

## Quick Setup

> 📁 **Working Directory Guide**  
> - **Scripts**: Run from project root (`./scripts/dev-server.sh`)  
> - **Cargo commands**: Run from project root (`cargo check`, `cargo run`)  
> - **sqlx commands**: Auto-handled by scripts (or run from `starter/` directory)  
> - **Tests**: Run from `starter/` directory (`cd starter && cargo test`)

### Quick Start (Recommended)

```bash
# Check prerequisites and start server
./scripts/check-prereqs.sh
./scripts/dev-server.sh 3000

# In a new terminal, start the background worker with log following
./scripts/worker.sh -f
```

### Manual Step-by-Step Setup

For learning purposes, here's the manual process:

### 1. Validate Prerequisites

```bash
./scripts/check-prereqs.sh
```

### 2. Clone and Setup Environment

```bash
git clone https://github.com/quanhua92/rust-fullstack-starter
cd rust-fullstack-starter

# Copy environment template (default values work for development)
# Note: .env is auto-created by dev scripts if missing
cp .env.example .env
```

### 3. Start Database Infrastructure

```bash
# Run from project root
docker compose up -d postgres

# Wait for database to be ready
docker compose logs -f postgres
# Look for "database system is ready to accept connections"
```

### 4. Run Database Migrations

```bash
# Install sqlx CLI if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations from starter directory
cd starter && sqlx migrate run
```

### 5. Test the Application

```bash
# Check compilation (from project root)
cargo check

# Test server mode (from project root)
./scripts/server.sh 3000
./scripts/test-server.sh 3000
./scripts/stop-server.sh 3000
```

## Verify Setup

### Database Connection
```bash
# Connect to database directly
psql postgres://starter_user:starter_pass@localhost:5432/starter_db

# List tables (should show: users, sessions, api_keys, tasks)
\dt
```

### Setup Initial Admin User

After copying `.env.example` to `.env`, set a strong password for the initial admin user:

```bash
# In .env file - use a strong password (min 8 chars, mix of letters/numbers/symbols)
STARTER__INITIAL_ADMIN_PASSWORD=YourSecureAdminPassword123!
```

**Security Note**: Remove or comment out this line after first startup for security.

The admin user will be created automatically when the server first starts.

## Development Workflow

### Start Development Environment
```bash
# Check prerequisites and start server
./scripts/check-prereqs.sh
./scripts/dev-server.sh

# In a new terminal, start the background worker with log following
./scripts/worker.sh -f
```

### Stop Everything
```bash
# Stop all Docker services
docker compose down
```

## Common Issues

For detailed troubleshooting, see **[Troubleshooting Guide](./troubleshooting.md)**.

### Quick Fixes

**Database Connection Failed**
```bash
docker compose up -d postgres && docker compose up --wait  # Restart database
```

**Password Authentication Failed**
```bash
# Clean restart fixes auth issues
docker compose down -v
docker compose up -d
cd starter && sqlx migrate run
```

**Compilation Errors**
```bash
cargo clean && cargo build
```

**Migration Errors**
```bash
# Run from starter directory, not project root
cd starter && sqlx migrate run
```

**Complete Reset**
```bash
./scripts/reset-all.sh --reset-database
./scripts/dev-server.sh 3000
```

## Next Steps

Now that you have the system running, follow these guides to understand and extend the starter:

### 📚 Learning Path (Read in Order)
1. **[Architecture Overview](./guides/01-architecture.md)** - System design and why it's built this way
2. **[Authentication System](./guides/02-authentication.md)** - How secure user sessions work  
3. **[Foundation Patterns](./guides/03-patterns.md)** - Circuit breakers, retry strategies, dead letter queues
4. **[Background Tasks](./guides/04-background-tasks.md)** - Async task processing system
5. **[Custom Task Types](./guides/05-task-types.md)** - Creating your own background tasks
6. **[Task Registry](./guides/06-task-registry.md)** - Organizing and managing task handlers

### 🔧 Development Resources
- **[Development Workflow](./development.md)** - Daily development process
- **[Configuration Reference](./configuration.md)** - All environment variables
- **[API Reference](./api-reference.md)** - Complete endpoint documentation
- **[Production Deployment](./production-deployment.md)** - Docker production setup
- **[CI/CD Guide](./cicd.md)** - GitHub Actions workflows and automation

### 🆘 When You Need Help
- **[Troubleshooting](./troubleshooting.md)** - Common issues and solutions
- **[Built-in Handlers](./reference/task-handlers.md)** - Reference for included task types

## Quick Commands Reference

### Using Development Scripts (Recommended)
```bash
# Complete development setup
./scripts/dev-server.sh [port]

# Start background worker with logs
./scripts/worker.sh -f

# Start server with auto-restart
./scripts/server.sh [port]

# Test server health
./scripts/test-server.sh [port]

# Stop server
./scripts/stop-server.sh [port]

# Infrastructure only
docker compose up -d postgres && docker compose up --wait

# Comprehensive API testing
./scripts/test-with-curl.sh [host] [port]
```

### Testing Commands
```bash
# Run all integration tests (~12 seconds)
cargo nextest run

# Test API endpoints (44 tests)
./scripts/test-with-curl.sh

# Test custom server configuration
./scripts/test-with-curl.sh localhost 8080

# Full validation workflow  
cargo nextest run && ./scripts/test-with-curl.sh
```

### 📋 API Documentation
```bash
# Start server and access interactive documentation
./scripts/server.sh 3000
open http://localhost:3000/api-docs

# View OpenAPI schema
curl http://localhost:3000/api-docs/openapi.json | jq

# Quick health check with docs links
curl http://localhost:3000/health | jq '.data.documentation'
```

**Full OpenAPI Documentation:**
- 📋 **[Interactive Swagger UI](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/quanhua92/rust-fullstack-starter/refs/heads/main/docs/openapi.json)**
- 📄 **OpenAPI Schema**: [docs/openapi.json](../docs/openapi.json)

**Features:**
- **Interactive Testing**: Full Swagger UI integration for testing endpoints
- **Complete Schema**: OpenAPI 3.0 specification with all endpoints and models
- **Authentication Support**: Test protected endpoints with session tokens
- **Request Examples**: Copy-paste ready curl commands for all endpoints

### Manual Commands
```bash
# Start infrastructure
docker compose up -d

# Run migrations (from project root)
sqlx migrate run

# Start server
cargo run -- server

# Start worker
cargo run -- worker

# Stop everything
docker compose down
```

### Script Features
- **Auto-kill existing processes** on port
- **Background execution** with PID tracking
- **Log management** with 50MB rotation in `/tmp/`
- **Health endpoint testing** with timeout
- **Graceful shutdown** with cleanup

## Testing Your Application

This starter includes a comprehensive testing framework with 53 integration tests.

### Running Tests
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (53 integration tests) - takes ~12 seconds
cargo nextest run

# Run specific test categories
cargo nextest run auth::     # Authentication tests
cargo nextest run tasks::    # Background task tests
cargo nextest run health::   # Health check tests
cargo nextest run api::      # API standards tests

# Run with debug output
TEST_LOG=1 cargo test -- --nocapture
```

### Testing Features
- **Database Isolation**: Each test gets its own PostgreSQL database
- **10x Performance**: Template database pattern for fast test setup
- **Real HTTP Testing**: TestApp spawns actual server instances
- **Authentication Support**: Test data factories with auth tokens
- **Comprehensive Coverage**: Authentication, tasks, health, API standards

### Example Test
```rust
#[tokio::test]
async fn test_user_registration() {
    let app = spawn_app().await;
    
    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "SecurePass123!"
    });

    let response = app.post_json("/auth/register", &user_data).await;
    assert_status(&response, StatusCode::OK);
}
```

See the **[Testing Guide](./guides/07-testing.md)** for detailed documentation on testing patterns and best practices.

## Next Steps

Start exploring the guides based on your interests:

- **[Architecture Overview](./guides/01-architecture.md)** - Understand the overall system design
- **[Authentication System](./guides/02-authentication.md)** - Learn the auth patterns
- **[Reliability Patterns](./guides/03-patterns.md)** - Circuit breakers and retry strategies  
- **[Background Tasks](./guides/04-background-tasks.md)** - Async job processing
- **[Testing Guide](./guides/07-testing.md)** - Comprehensive testing strategies
- **[Development Workflow](./development.md)** - Daily development practices

## Project Customization

If you want to customize the project name and branding, see the **[Project Customization Guide](./project-customization.md)** for automated renaming tools and customization options.

---

*This starter is designed for learning and development. While the patterns demonstrated here are production-worthy, you should adapt and extend them based on your specific requirements.*