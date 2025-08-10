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
# Complete full-stack environment (database + web + API)
./scripts/dev-server.sh

# Or with options:
./scripts/dev-server.sh -w          # Also start worker (complete setup)
./scripts/dev-server.sh -f          # Foreground mode
./scripts/dev-server.sh --api-only  # API only (skip web build)
./scripts/dev-server.sh -p 8080     # Custom port
```

### 3. Start the Worker

**Option A: Included with server** (easiest):
```bash
./scripts/dev-server.sh -w    # Starts everything: database + web + API + worker
```

**Option B: Separate terminal**:
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
curl http://localhost:3000/api/v1/health
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

# Monitor events with tag filtering
curl -H "Authorization: Bearer TOKEN" \
  "http://localhost:3000/api/v1/monitoring/events?tags=task_type:email,status:completed"

# Access the monitoring dashboard (requires moderator+ role)
open http://localhost:3000/admin/monitoring     # Full-featured monitoring interface
```

### 5. Build and Serve Full-Stack Application

**Option A: Complete development environment** (recommended):
```bash
# Full setup: database + web build + unified server
./scripts/dev-server.sh

# Or quick restart: auto-detects and builds web if needed  
./scripts/server.sh 3000       # Smart server with auto-build
```

**Option B: Separate development servers**:
```bash
# Terminal 1: API server only
./scripts/server.sh 3000

# Terminal 2: React dev server  
cd web && pnpm dev
```

**Option C: Manual build then serve**:
```bash
./scripts/build-web.sh         # Build React frontend  
./scripts/server.sh 3000       # Serve API + static files
```

### 6. Explore the Application

- **🌐 Frontend**: http://localhost:3000 (React app served by Rust)
- **🔌 API**: http://localhost:3000/api/v1 (REST API)
- **📚 API Docs**: http://localhost:3000/api-docs (Interactive Swagger UI)
- **📊 Monitoring Dashboard**: http://localhost:3000/admin/monitoring (Admin/Moderator interface with full CRUD capabilities)
- **❤️ Health Check**: http://localhost:3000/api/v1/health (System status)
- **🔧 Worker Logs**: Check `/tmp/starter-worker-0.log` for task processing

## Key Features

- **🌐 Full-Stack Integration** - React frontend served directly by Rust server with unified deployment
- **🔐 Authentication & Authorization** - Session-based auth with ownership-based RBAC and transaction safety
- **👥 User Management System** - Complete user lifecycle with 10 endpoints (profile, admin, analytics)
- **🔑 Role-Based Access Control** - Advanced ownership-based security: users own their data, Admin/Moderator access all
- **⚙️ Background Tasks** - Async job processing with retry logic and dead letter queue
- **🏗️ Module Generator** - Secure template-based code generation with ownership patterns, transaction safety, and comprehensive testing
- **📊 API Documentation** - Interactive OpenAPI/Swagger docs
- **🧪 Testing Framework** - 184 integration tests + 194 frontend tests (135 unit + 46 integration + 13 E2E) + comprehensive API endpoint testing (37 endpoints) including security vulnerability tests
- **📊 Monitoring & Observability** - Complete monitoring system with 9 API endpoints, comprehensive security protections, web UI dashboard, advanced tag filtering, RBAC-integrated interface with ownership-based authorization
- **🔥 Chaos Testing** - Docker-based resilience testing with 10 scenarios
- **⚙️ Admin CLI** - Direct database access for monitoring and maintenance
- **🐳 Docker Support** - Development and production containers with multi-stage builds

## Development Commands

```bash
# Full-stack development
./scripts/dev-server.sh             # Complete environment: database + web + API
./scripts/build-web.sh              # Build React frontend only

# Module generation
cargo run -- generate module books --template basic      # Generate CRUD module  
cargo run -- revert module books --dry-run               # Preview revert
./scripts/test-template-with-curl.sh books               # Test generated API

# Run tests
cargo nextest run                    # Integration tests (184 tests)
./scripts/test-with-curl.sh         # API endpoint tests (37 endpoints)
./scripts/test-generate.sh          # Module generator system tests
./scripts/test-chaos.sh             # Chaos testing (10 scenarios)

# Quality checks
./scripts/check.sh                  # Backend: format, lint, test (run before commits)
cd web && ./scripts/check-web.sh    # Frontend: 135 unit + 46 integration + 13 E2E tests

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
    │   ├── users/    # User management (10 endpoints)
    │   ├── rbac/     # Role-based access control
    │   ├── cli/      # Admin command-line interface
    │   ├── tasks/    # Background job processing
    │   ├── monitoring/ # Observability system (9 endpoints)
    │   └── ...       # Health, errors, database, server
    ├── migrations/   # Database schema evolution (6 migrations)
    └── tests/        # Integration tests (184 tests)
```

## Documentation

**📚 Complete documentation available in [`docs/`](docs/)**

### Quick Navigation
- **[Quick Start](docs/QUICK-START.md)** - 2-minute setup guide
- **[Architecture Guide](docs/ARCHITECTURE-GUIDE.md)** - System design and patterns
- **[Implementation Guide](docs/IMPLEMENTATION-GUIDE.md)** - Core systems and testing
- **[API Reference](docs/API-REFERENCE.md)** - All 37 endpoints with examples
- **[Production Deployment](docs/PRODUCTION-DEPLOYMENT.md)** - Docker deployment
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Debugging and diagnostics
- **[Advanced Topics](docs/ADVANCED-TOPICS.md)** - Chaos testing and frontend patterns

### Additional Resources
- **[AI-First Development Ideas](docs/ideas/ai-first.md)** - Conceptual approaches and future enhancements

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

**Monitoring:**
- `POST /api/v1/monitoring/events` - Create monitoring events
- `GET /api/v1/monitoring/events?tags=key:value` - Query events with tag filtering
- `GET /api/v1/monitoring/metrics/prometheus` - Prometheus metrics export
- `POST /api/v1/monitoring/incidents` - Create incidents

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

## Testing

**378 Total Tests** - Comprehensive coverage across the full stack:

```bash
# Backend Tests (215 tests)
cargo test                           # 31 unit tests  
cargo nextest run                    # 184 integration tests

# Frontend Tests (194 tests)  
cd web && pnpm test:unit             # 135 unit tests (~2s)
cd web && pnpm test:integration      # 46 integration tests (real server)
cd web && pnpm test:e2e              # 13 E2E tests (auth flow, API health)

# API Testing
./scripts/test-with-curl.sh          # 37 endpoints tested
./scripts/test-chaos.sh              # Docker resilience testing
```

**Key Features:**
- **Stateless Design** - Tests use unique data (no cleanup dependencies)
- **Resilient Patterns** - Handle race conditions and async operations gracefully  
- **Full Coverage** - Unit, integration, E2E, API endpoints, security vulnerabilities
- **CI-Ready** - Fast unit tests for development, comprehensive tests for validation

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
cp .env.example .env.prod
# Edit .env.prod with production settings
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

## License

MIT License - see LICENSE file for details.

---

*Ready to build? Start with the [Quick Start Guide](docs/QUICK-START.md) for 2-minute setup or explore the [complete documentation](docs/) for deeper learning.*