# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture Overview

This is a Rust fullstack starter built around a **single binary, multiple modes** pattern:
- **Server mode**: HTTP API server with Axum framework
- **Worker mode**: Background job processor (future implementation)
- **Shared foundation**: Database, configuration, and error handling

**Key Design Patterns:**
- Workspace-based dependency management with version inheritance
- Environment-based configuration with `STARTER__` prefix
- Database-first approach with SQLx and migrations
- Clean error handling with custom types and HTTP conversion
- Session-based authentication architecture

## Essential Commands

### Development Setup
```bash
# Complete development environment (recommended)
./scripts/dev-server.sh [port]

# Step-by-step workflow
./scripts/dev.sh                    # Start infrastructure
./scripts/server.sh [port]          # Start server in background
./scripts/test-server.sh [port]     # Test health endpoints

# Manual commands (fallback)
docker compose up -d postgres       # Start database
sqlx migrate run                    # Run migrations
cargo run -- server                # Start server (foreground)
cargo run -- server --port 3000    # Custom port
cargo run -- worker                # Worker mode
```

### Development Scripts
The `scripts/` directory provides automated workflow management:

**Key Features:**
- **Background execution** - scripts return control immediately  
- **PID tracking** - saves to `/tmp/starter-server-{PORT}.pid`
- **Log management** - saves to `/tmp/starter-server-{PORT}.log` with 50MB rotation
- **Process cleanup** - auto-kills existing processes on port
- **Health validation** - automated endpoint testing

```bash
# Complete workflow (infrastructure + server + testing)
./scripts/dev-server.sh [port]
# - Starts PostgreSQL with docker compose
# - Starts server in background with PID tracking
# - Runs health tests automatically
# - Graceful shutdown with Ctrl+C

# Server management
./scripts/server.sh [port]          # Start server (default port: 3000)
./scripts/test-server.sh [port]     # Test /health endpoints (30s timeout)
./scripts/stop-server.sh [port]     # Stop server with graceful shutdown
./scripts/dev.sh                    # Infrastructure only (postgres)

# Log monitoring
tail -f /tmp/starter-server-3000.log    # View server logs
ls /tmp/starter-server-*                # List all server files
```

### Database Operations
```bash
# Create new migration
sqlx migrate add migration_name

# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Connect to database
psql $DATABASE_URL
# or
psql -h localhost -p 5432 -U starter_user -d starter_db
```

### Build and Quality
```bash
# Fast compilation check
cargo check

# Format code
cargo fmt

# Lint with clippy  
cargo clippy

# Release build
cargo build --release
```

## Configuration System

**Environment Variables:**
- Use `STARTER__` prefix with double underscore `__` separators
- App reads individual database components, NOT `DATABASE_URL`
- `DATABASE_URL` is only for sqlx CLI tools
- CORS origins support comma-separated strings: `"http://localhost:5173,http://localhost:3000"`

**Key Configuration:**
```bash
# Server
STARTER__SERVER__HOST=0.0.0.0
STARTER__SERVER__PORT=8080
STARTER__SERVER__CORS_ORIGINS="http://localhost:5173,http://localhost:3000"

# Database (for app)
STARTER__DATABASE__USER=starter_user
STARTER__DATABASE__PASSWORD=starter_pass
STARTER__DATABASE__HOST=localhost
STARTER__DATABASE__PORT=5432
STARTER__DATABASE__DATABASE=starter_db

# Database URL (for sqlx CLI only)
DATABASE_URL=postgres://starter_user:starter_pass@localhost:5432/starter_db

# Optional admin user creation
STARTER__INITIAL_ADMIN_PASSWORD=your_secure_admin_password
```

## Database Schema

**Core Tables:**
- `users`: Authentication with Argon2 password hashing
- `sessions`: Session-based auth with expiration
- `api_keys`: Machine-to-machine authentication  
- `tasks`: Background job queue with retry logic

**Key Features:**
- UUID primary keys for distributed compatibility
- Automatic `updated_at` triggers
- Performance-optimized indexes
- JSONB for flexible data (permissions, task payloads)

## Code Organization

**Main Application (`starter/src/`):**
- `main.rs`: CLI entry point with clap
- `config.rs`: Environment-based configuration with custom deserializers
- `database.rs`: Connection pooling, migrations, health checks
- `error.rs`: Custom error types with HTTP response conversion
- `models.rs`: Domain models with validation
- `types.rs`: Common types (AppState, pagination, API responses)
- `server.rs`: Axum server setup with middleware
- `api/`: HTTP handlers (currently health endpoints)

**Key Implementation Details:**
- Custom serde deserializer for comma-separated CORS origins
- Manual SQLx error conversion to avoid trait conflicts  
- Clone-able Database wrapper for AppState sharing
- Comprehensive error handling with HTTP status mapping

## Health Endpoints

```bash
# Basic health check
GET /health
# Returns: {"success":true,"data":"OK"}

# Detailed health with database status
GET /health/detailed  
# Returns: {"success":true,"data":{"status":"healthy","checks":{"database":...}}}
```

## Development Workflow

### Recommended Workflow (Using Scripts)
```bash
# 1. Start complete development environment
./scripts/dev-server.sh 3000

# 2. During development - restart server
./scripts/server.sh 3000

# 3. Test changes
./scripts/test-server.sh 3000

# 4. Monitor logs
tail -f /tmp/starter-server-3000.log

# 5. Stop when done
./scripts/stop-server.sh 3000
```

### Manual Workflow (Fallback)
1. **Start infrastructure**: `docker compose up -d postgres`
2. **Run migrations**: `sqlx migrate run`
3. **Start server**: `cargo run -- server --port 3000`
4. **Make changes**: Edit code, migrations auto-run on server start
5. **Test endpoints**: Manual curl or browser testing

### Database Changes
1. `sqlx migrate add new_feature`
2. Edit generated SQL file in `starter/migrations/`
3. `sqlx migrate run` (or restart server - auto-runs migrations)
4. Update models/queries as needed

### Script Benefits vs Manual
- ✅ **No hanging** - scripts return control immediately
- ✅ **Auto-cleanup** - kills existing processes on port
- ✅ **Health testing** - automated endpoint validation
- ✅ **Log persistence** - persistent logs in `/tmp/` with rotation
- ✅ **PID tracking** - proper background process management

## Debugging

### Server Logs
```bash
# View live logs (when using scripts)
tail -f /tmp/starter-server-3000.log

# Enable debug logging (manual mode)
RUST_LOG=debug cargo run -- server

# Enable trace logging (manual mode)  
RUST_LOG=trace cargo run -- server

# Application-specific logging (manual mode)
RUST_LOG=starter=debug cargo run -- server
```

### Process Debugging
```bash
# Check if server is running
ps aux | grep "starter server"

# Check PID file
cat /tmp/starter-server-3000.pid

# Check port usage
lsof -i :3000

# Force kill if needed
./scripts/stop-server.sh 3000
```

### Database Debugging
```bash
# Check database connection
psql $DATABASE_URL -c "SELECT 1"

# View database logs
docker compose logs -f postgres

# Connect to database directly
psql -h localhost -p 5432 -U starter_user -d starter_db
```

### Health Check Debugging
```bash
# Test endpoints manually
curl http://localhost:3000/health
curl http://localhost:3000/health/detailed

# Or use the test script
./scripts/test-server.sh 3000
```