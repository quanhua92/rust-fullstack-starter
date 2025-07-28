# Rust Full-Stack Starter

A modern Rust web application starter template with authentication, background tasks, and comprehensive testing. Built with Axum, SQLx, and PostgreSQL for learning and rapid prototyping.

## Quick Start

### Prerequisites

- Rust 1.75+ ([rustup.rs](https://rustup.rs/))
- Docker and Docker Compose

### 1. Clone and Setup

```bash
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter
```

### 2. Start the Server

```bash
# Start database and HTTP server
./scripts/dev-server.sh
```

### 3. Start the Worker (New Terminal)

```bash
# Start background task worker with log following
./scripts/worker.sh -f
```

### 4. Try It Out

**Check health:**
```bash
curl http://localhost:3000/health
```

**Explore API documentation:**
```bash
open http://localhost:3000/api-docs
# Or visit: http://localhost:3000/api-docs
```

**Full OpenAPI Documentation:**
- 📋 **[Interactive Swagger UI](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/quanhua92/rust-fullstack-starter/refs/heads/main/docs/openapi.json)**
- 📄 **OpenAPI Schema**: [docs/openapi.json](docs/openapi.json)

**Create a user and test authentication:**
```bash
# Register a new user
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "password123"}'

# Login to get a session token
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email": "testuser", "password": "password123"}'
```

**Create and monitor a background task:**
```bash
# Create a task (replace TOKEN with your session token from login)
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type": "email", "payload": {"to": "user@example.com", "subject": "Hello", "body": "Test email"}}'

# Check task status
curl -H "Authorization: Bearer TOKEN" http://localhost:3000/tasks
```

### 5. Explore More

- **API Docs**: http://localhost:3000/api-docs (Interactive Swagger UI)
- **Health Check**: http://localhost:3000/health (System status)
- **Worker Logs**: Check `/tmp/starter-worker.log` for task processing

## Key Features

- **🔐 Authentication System** - Registration, login, session management
- **⚙️ Background Tasks** - Async job processing with retry logic and dead letter queue
- **📊 API Documentation** - Interactive OpenAPI/Swagger docs
- **🧪 Testing Framework** - 53 integration tests + API endpoint testing
- **🔥 Chaos Testing** - Docker-based resilience testing with 7 scenarios
- **⚙️ Admin CLI** - Direct database access for monitoring and maintenance
- **🐳 Docker Support** - Development and production containers

## Development Commands

```bash
# Run tests
cargo nextest run                    # Integration tests (53 tests)
./scripts/test-with-curl.sh         # API endpoint tests (44 tests)
./scripts/test-chaos.sh             # Chaos testing (7 scenarios)

# Quality checks
./scripts/check.sh                  # Format, lint, test (run before commits)

# Background tasks
./scripts/worker.sh -f              # Start task worker with logs

# Admin commands (direct database access)
cargo run -- admin task-stats       # Task statistics
cargo run -- admin list-tasks       # List recent tasks
```

## Project Structure

```
rust-fullstack-starter/
├── scripts/          # Development automation
├── docs/            # Comprehensive documentation
└── starter/         # Main application
    ├── src/
    │   ├── auth/     # Authentication
    │   ├── users/    # User management
    │   ├── tasks/    # Background jobs
    │   └── ...
    ├── migrations/   # Database schema
    └── tests/        # Integration tests
```

## Documentation

**📚 Complete documentation available in [`docs/`](docs/)**

### Quick Links
- **[Getting Started](docs/getting-started.md)** - Detailed setup guide
- **[Development Guide](docs/development.md)** - Daily workflow
- **[Architecture Guides](docs/guides/)** - System design and patterns
- **[API Reference](docs/api-reference.md)** - Complete endpoint docs
- **[Production Deployment](docs/production-deployment.md)** - Docker deployment

### Learning Guides
- [Authentication System](docs/guides/02-authentication.md)
- [Background Tasks](docs/guides/04-background-tasks.md)
- [Testing Framework](docs/guides/07-testing.md)
- [Chaos Testing](docs/guides/08-chaos-testing.md) - **Enhanced with 7 scenarios**

## API Endpoints

Key endpoints (see [full API docs](http://localhost:3000/api-docs)):

- `POST /auth/register` - User registration
- `POST /auth/login` - Authentication
- `POST /tasks` - Create background task
- `GET /tasks/dead-letter` - Failed task queue
- `GET /health` - Health check

## Configuration

Copy `.env.example` to `.env` and set your admin password:

```bash
cp .env.example .env
# Edit .env and set STARTER__INITIAL_ADMIN_PASSWORD
```

## Admin CLI Commands

Direct database access for monitoring and maintenance (bypasses API authentication):

```bash
# Task monitoring
cargo run -- admin task-stats                    # Overall statistics
cargo run -- admin task-stats --tag "baseline"  # Filter by tag

# Task inspection
cargo run -- admin list-tasks --limit 10        # Recent tasks
cargo run -- admin list-tasks --verbose         # Detailed view

# Maintenance
cargo run -- admin clear-completed --dry-run    # Preview cleanup
cargo run -- admin clear-completed              # Clean old tasks
```

**Use cases**: Monitoring during chaos testing, debugging task processing, maintenance operations.

## Production Deployment

```bash
# Docker deployment
cp .env.prod.example .env.prod
# Edit .env.prod with production settings
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

## License

MIT License - see LICENSE file for details.

---

*Ready to build? Start with the [Getting Started Guide](docs/getting-started.md) for detailed setup instructions.*