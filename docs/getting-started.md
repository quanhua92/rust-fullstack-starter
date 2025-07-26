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

- **Development**: See [development.md](./development.md) for detailed workflow
- **API Reference**: See [api-endpoints.md](./api-endpoints.md) for complete endpoint documentation
- **Authentication**: See [authentication.md](./authentication.md) for auth system guide
- **Configuration**: See [configuration.md](./configuration.md) for all environment options
- **Architecture**: See [architecture.md](./architecture.md) for system overview

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