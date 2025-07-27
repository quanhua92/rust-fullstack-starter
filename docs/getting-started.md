# Getting Started

This guide will help you set up and run the Rust Full-Stack Starter project locally.

## Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Docker & Docker Compose** - For database infrastructure
- **PostgreSQL client tools** (optional) - For database inspection

## Quick Setup

### 1. Clone and Setup Environment

```bash
git clone <repository-url>
cd rust-fullstack-starter

# Copy environment template
cp .env.example .env

# Edit .env if needed (default values should work for development)
```

### 2. Start Database Infrastructure

```bash
# Start PostgreSQL with Docker Compose
docker compose up -d postgres

# Wait for database to be ready
docker compose logs -f postgres
# Look for "database system is ready to accept connections"
```

### 3. Run Database Migrations

```bash
# Install sqlx CLI (if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations to create database schema
sqlx migrate run
```

### 4. Test the Application

```bash
# Check that everything compiles
cargo check

# Test server mode
cargo run -- server
# Should output: "Server starting on port 8080 (config: 0.0.0.0)"

# Test worker mode (in another terminal)
cargo run -- worker  
# Should output: "Worker starting with 4 concurrency"

# View CLI help
cargo run -- --help
```

## Verify Setup

### Database Connection
```bash
# Connect to database directly
psql postgres://starter_user:starter_pass@localhost:5432/starter_db

# List tables (should show: users, sessions, api_keys, tasks)
\dt
```

### Check Initial Admin User
If you set `STARTER__INITIAL_ADMIN_PASSWORD` in your `.env`, an admin user will be created automatically on first server startup.

## Development Workflow

### Start Development Environment
```bash
# Use the development script
./scripts/dev.sh
```

### Stop Everything
```bash
# Stop all Docker services
docker compose down

# Stop with data cleanup (careful!)
docker compose down -v
```

## Common Issues

### Database Connection Failed
- Ensure Docker is running: `docker ps`
- Check database logs: `docker compose logs postgres`
- Verify .env DATABASE_URL matches docker-compose.yaml settings

### Compilation Errors
- Update Rust: `rustup update`
- Clean build: `cargo clean && cargo build`

### Migration Errors
- Ensure database is running before running migrations
- Check DATABASE_URL environment variable
- Reset database: `docker compose down -v && docker compose up -d`

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

### 🆘 When You Need Help
- **[Troubleshooting](./reference/troubleshooting.md)** - Common issues and solutions
- **[Built-in Handlers](./reference/task-handlers.md)** - Reference for included task types

## Quick Commands Reference

### Using Development Scripts (Recommended)
```bash
# Complete development setup
./scripts/dev-server.sh [port]

# Start server with auto-restart
./scripts/server.sh [port]

# Test server health
./scripts/test-server.sh [port]

# Stop server
./scripts/stop-server.sh [port]

# Infrastructure only
./scripts/dev.sh
```

### Manual Commands
```bash
# Start infrastructure
docker compose up -d

# Run migrations
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

This starter includes a comprehensive testing framework with 38 integration tests.

### Running Tests
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (38 integration tests)
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

---

*This starter is designed for learning and development. While the patterns demonstrated here are production-worthy, you should adapt and extend them based on your specific requirements.*