# Development Guide

This guide covers the development workflow, tools, and best practices for the Rust Full-Stack Starter project.

## Development Environment Setup

### Quick Start
```bash
# Start all development services
./scripts/dev.sh

# In another terminal, start the application
cargo run -- server
```

### Manual Setup
```bash
# Start database
docker compose up -d postgres

# Wait for database health check
docker compose up --wait

# Run migrations
sqlx migrate run

# Start development
cargo run -- server --port 3000
```

## Project Structure

```
rust-fullstack-starter/
├── Cargo.toml                 # Workspace configuration
├── docker-compose.yaml        # Database infrastructure
├── .env.example               # Environment template
├── scripts/
│   └── dev.sh                 # Development startup script
├── docs/                      # Documentation
└── starter/                   # Main application
    ├── Cargo.toml             # Application dependencies
    ├── src/
    │   ├── main.rs             # CLI entry point
    │   ├── lib.rs              # Library exports
    │   ├── config.rs           # Configuration management
    │   ├── database.rs         # Database connection and migrations
    │   ├── error.rs            # Error handling
    │   ├── types.rs            # Common type definitions
    │   └── models.rs           # Database models
    └── migrations/
        └── 001_initial.sql     # Database schema
```

## Development Workflow

### 1. Feature Development
```bash
# Start with fresh database (optional)
docker compose down -v
docker compose up -d postgres
sqlx migrate run

# Make your changes
# Test compilation
cargo check

# Run tests (when available)
cargo test

# Test the application
cargo run -- server
```

### 2. Database Changes
```bash
# Create new migration
sqlx migrate add your_migration_name

# Edit the generated SQL file in starter/migrations/

# Apply migration
sqlx migrate run

# Revert if needed
sqlx migrate revert
```

### 3. Testing Changes
```bash
# Check compilation
cargo check

# Run with different configurations
STARTER__SERVER__PORT=3001 cargo run -- server

# Test worker mode
cargo run -- worker
```

## CLI Commands

### Server Mode
```bash
# Default port (8080)
cargo run -- server

# Custom port
cargo run -- server --port 3000

# With specific config
STARTER__SERVER__HOST=localhost cargo run -- server --port 8080
```

### Worker Mode
```bash
# Start background worker
cargo run -- worker

# With custom concurrency
STARTER__WORKER__CONCURRENCY=2 cargo run -- worker
```

## Database Operations

### Migrations
```bash
# Check migration status
sqlx migrate info

# Run pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Reset database (careful!)
docker compose down -v
docker compose up -d postgres
sqlx migrate run
```

### Database Access
```bash
# Connect via psql
psql $DATABASE_URL

# Or using individual components
psql -h localhost -p 5432 -U starter_user -d starter_db

# View tables
\dt

# View specific table
\d users
```

## Environment Configuration

### Development .env
```bash
# Copy template
cp .env.example .env

# Key development settings
STARTER__SERVER__HOST=0.0.0.0        # Allow external connections
STARTER__SERVER__PORT=8080            # Default port
STARTER__DATABASE__HOST=localhost     # Database host
STARTER__DATABASE__PORT=5432          # Database port
```

### Testing Different Configurations
```bash
# Test with different database
STARTER__DATABASE__DATABASE=test_db cargo run -- server

# Test with different worker settings
STARTER__WORKER__CONCURRENCY=1 cargo run -- worker

# Test with admin user creation
STARTER__INITIAL_ADMIN_PASSWORD=admin123 cargo run -- server
```

## Code Quality

### Formatting
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting
```bash
# Run clippy
cargo clippy

# Fix issues automatically
cargo clippy --fix
```

### Compilation Checks
```bash
# Fast compilation check
cargo check

# Full build
cargo build

# Release build
cargo build --release
```

## Docker Development

### Database Management
```bash
# View database logs
docker compose logs -f postgres

# Restart database
docker compose restart postgres

# Connect to database container
docker compose exec postgres psql -U starter_user -d starter_db

# View database stats
docker compose exec postgres pg_isready -U starter_user -d starter_db
```

### Cleanup
```bash
# Stop services
docker compose down

# Remove volumes (data loss!)
docker compose down -v

# Remove images
docker compose down --rmi all
```

## Debugging

### Application Debugging
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- server

# Enable trace logging
RUST_LOG=trace cargo run -- server

# Application-specific logging
RUST_LOG=starter=debug cargo run -- server
```

### Database Debugging
```bash
# Check database connection
psql $DATABASE_URL -c "SELECT 1"

# View active connections
docker compose exec postgres psql -U starter_user -d starter_db -c "SELECT * FROM pg_stat_activity"

# Check database size
docker compose exec postgres psql -U starter_user -d starter_db -c "SELECT pg_size_pretty(pg_database_size('starter_db'))"
```

## Performance

### Build Performance
```bash
# Use multiple cores
cargo build -j$(nproc)

# Faster linker (if available)
cargo build --config target.x86_64-unknown-linux-gnu.linker=\"lld\"
```

### Database Performance
```bash
# Check query performance
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'admin@example.com';

# View database stats
SELECT * FROM pg_stat_user_tables;
```

## Common Development Tasks

### Reset Everything
```bash
docker compose down -v
docker compose up -d postgres
sqlx migrate run
cargo run -- server
```

### Quick Health Check
```bash
# Check all components
docker compose ps
cargo check
psql $DATABASE_URL -c "SELECT 1"
```

### Add New Dependencies
```bash
# Add to workspace (preferred)
# Edit Cargo.toml [workspace.dependencies]

# Add to starter package
# Edit starter/Cargo.toml [dependencies]
# Use .workspace = true

# Update dependencies
cargo update
```

## IDE Setup

### VS Code
Recommended extensions:
- rust-analyzer
- Better TOML
- Docker
- PostgreSQL

### Environment Variables
Create `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true
}
```