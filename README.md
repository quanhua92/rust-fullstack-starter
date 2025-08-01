# Rust Full-Stack Starter

A modern Rust web application starter template with authentication, user management, background tasks, and comprehensive testing. Built with Axum, SQLx, and PostgreSQL for learning and rapid prototyping.

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

# Or multiple concurrent workers
./scripts/worker.sh --id 1 -f    # Terminal 1
./scripts/worker.sh --id 2 -f    # Terminal 2
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

**Register and test user management:**
```bash
# Register a new user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "SecurePass123!"}'

# Login to get a session token
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "SecurePass123!"}'

# Update your profile (using token from login)
curl -X PUT http://localhost:3000/api/v1/users/me/profile \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{"email": "updated@example.com"}'
```

**Create and monitor a background task:**
```bash
# Create a task (replace TOKEN with your session token from login)
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type": "email", "payload": {"to": "user@example.com", "subject": "Hello", "body": "Test email"}}'

# Check task status (users see only their tasks, admins/moderators see all)
curl -H "Authorization: Bearer TOKEN" http://localhost:3000/api/v1/tasks
```

### 5. Explore More

- **API Docs**: http://localhost:3000/api-docs (Interactive Swagger UI)
- **Health Check**: http://localhost:3000/health (System status)
- **Worker Logs**: Check `/tmp/starter-worker-0.log` for task processing (or `/tmp/starter-worker-{ID}.log` for specific worker ID)

## Key Features

- **🔐 Authentication & Authorization** - Session-based auth with Role-Based Access Control (RBAC)
- **👥 User Management System** - Complete user lifecycle with 12 endpoints (profile, admin, analytics)
- **🔑 Role-Based Access Control** - Three-tier system (User/Moderator/Admin) with hierarchical permissions
- **⚙️ Background Tasks** - Async job processing with retry logic and dead letter queue
- **📊 API Documentation** - Interactive OpenAPI/Swagger docs
- **🧪 Testing Framework** - 95 integration tests + comprehensive API endpoint testing
- **🔥 Chaos Testing** - Docker-based resilience testing with 7 scenarios
- **⚙️ Admin CLI** - Direct database access for monitoring and maintenance
- **🐳 Docker Support** - Development and production containers

## Development Commands

```bash
# Run tests
cargo nextest run                    # Integration tests (95 tests)
./scripts/test-with-curl.sh         # API endpoint tests (44+ tests)
./scripts/test-chaos.sh             # Chaos testing (7 scenarios)

# Quality checks
./scripts/check.sh                  # Format, lint, test (run before commits)

# Background tasks
./scripts/worker.sh -f              # Start task worker with logs (ID 0)
./scripts/worker.sh --id 1 -f       # Start concurrent worker (ID 1)
./scripts/worker.sh --id 2          # Start background worker (ID 2)

# Admin commands (direct database access)
cargo run -- admin task-stats       # Task statistics (bypasses API auth)
cargo run -- admin list-tasks       # List recent tasks (all users)
cargo run -- admin clear-completed  # Cleanup maintenance
```

## Project Structure

```
rust-fullstack-starter/
├── scripts/          # Development automation (13 scripts + helpers)
├── docs/            # Comprehensive documentation (12 guides + references)
├── web/             # React/TypeScript frontend (TanStack Router + shadcn/ui)
└── starter/         # Main Rust application
    ├── src/
    │   ├── auth/     # Session-based authentication
    │   ├── users/    # User management (12 endpoints)
    │   ├── rbac/     # Role-based access control
    │   ├── cli/      # Admin command-line interface
    │   ├── tasks/    # Background job processing
    │   └── ...       # Health, errors, database, server
    ├── migrations/   # Database schema evolution
    └── tests/        # Integration tests (95 tests)
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
- [Authentication & Authorization](docs/guides/02-authentication-and-authorization.md) - **Session-based auth with RBAC**
- [User Management System](docs/guides/12-user-management.md) - **Complete user lifecycle with 12 endpoints**
- [Background Tasks](docs/guides/04-background-tasks.md)
- [Testing Framework](docs/guides/08-testing.md)
- [Chaos Testing](docs/guides/09-chaos-testing.md) - **Enhanced with 7 scenarios**

## API Endpoints

Key endpoints (see [full API docs](http://localhost:3000/api-docs)):

**Authentication:**
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - Authentication
- `POST /api/v1/auth/refresh` - Token refresh

**User Management:**
- `PUT /api/v1/users/me/profile` - Update own profile
- `PUT /api/v1/users/me/password` - Change password
- `GET /api/v1/users` - List users (Moderator+)
- `GET /api/v1/admin/users/stats` - User analytics (Admin)

**Tasks:**
- `POST /api/v1/tasks` - Create background task
- `GET /api/v1/tasks/dead-letter` - Failed task queue
- `GET /api/v1/tasks/types` - Task type registry

**System:**
- `GET /api/v1/health` - Health check

## Configuration

Copy `.env.example` to `.env` and set your admin password:

```bash
cp .env.example .env
# Edit .env and set STARTER__INITIAL_ADMIN_PASSWORD
```

### Admin Account Setup

The system automatically creates an admin account on first startup if `STARTER__INITIAL_ADMIN_PASSWORD` is set:

```bash
# In .env file
STARTER__INITIAL_ADMIN_PASSWORD=your_secure_admin_password

# Admin account will be created with:
# Username: admin
# Email: admin@example.com  
# Role: Admin (full system access)
```

**Admin Capabilities:**
- Access all user tasks and profiles
- Use admin-only endpoints
- Full RBAC permissions for system management

## Admin CLI Commands

The application includes a modular CLI system with direct database access for monitoring and maintenance (bypasses API authentication):

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

**CLI Architecture**: The CLI functionality is organized in `starter/src/cli/` with dedicated modules:
- `api.rs` - Command execution and application entry point
- `models.rs` - Command definitions and data structures (using Clap)
- `services.rs` - Business logic and database operations
- Comprehensive testing in `starter/tests/cli/` (19 tests: 11 unit + 8 integration)

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