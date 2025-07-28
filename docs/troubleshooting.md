# Troubleshooting Guide

This guide covers common issues and their solutions.

## Quick Diagnostics

```bash
# Check everything at once
./scripts/check-prereqs.sh
./scripts/status.sh
```

## Common Issues

### 1. Migration Errors

#### Error: "No such file or directory"
```bash
# WRONG - from project root
sqlx migrate run

# CORRECT - from starter/ directory  
cd starter && sqlx migrate run
```

**Solution**: Use `./scripts/dev-server.sh` which handles this automatically.

#### Error: "Connection refused"
```bash
# Check if database is running
docker compose ps postgres

# If not running, start it
docker compose up -d postgres

# Check logs for errors
docker compose logs postgres
```

#### Error: "Password authentication failed"

**For Development**:
```bash
# Development uses simplified auth - clean restart usually fixes it
docker compose down -v
docker compose up -d
cd starter && sqlx migrate run
```

**For Production**:
```bash
# Production uses SCRAM-SHA-256 - clean restart required
docker-compose -f docker-compose.prod.yaml --env-file .env.prod down -v
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

**Note**: The application handles both authentication methods automatically. Authentication method differences are transparent to your Rust code.

### 2. Docker Issues

#### Error: "Docker daemon not running"
**Solution**: Start Docker Desktop and wait for it to fully load.

#### Error: "Port 5432 already in use"
```bash
# Find what's using the port
lsof -i :5432

# Kill conflicting process or use different port
docker compose down -v
```

### 3. Compilation Issues

#### Error: "Package not found"
```bash
# Update Rust and dependencies
rustup update
cargo clean
cargo build
```

#### Error: "sqlx-cli not found"
```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

### 4. Server Startup Issues

#### Error: "Address already in use"
```bash
# Find and stop conflicting process
lsof -ti:3000 | xargs kill -9

# Or use different port
./scripts/server.sh 3001
```

#### Error: "Database connection failed"
```bash
# Check database status
docker compose ps postgres

# Reset database
docker compose down -v
docker compose up -d postgres && docker compose up --wait
```

### 5. Test Issues

#### Error: "Tests hang or fail"
```bash
# Ensure database is running
docker compose up -d postgres

# Run tests from correct directory
cd starter
cargo test

# For faster tests
cargo install cargo-nextest
cargo nextest run
```

## Complete Reset

When all else fails:
```bash
# Nuclear option - reset everything
./scripts/reset-all.sh
./scripts/check-prereqs.sh
./scripts/dev-server.sh 3000
```

## Getting Help

1. Check logs: `tail -f /tmp/starter-server-*.log`
2. Verify environment: `cat .env`
3. Check GitHub issues: [Project Issues](link-to-issues)
4. Review documentation: `docs/guides/`

## Reporting Issues

When reporting issues, include:
1. Output of `./scripts/check-prereqs.sh`
2. Output of `./scripts/status.sh`
3. Relevant log files from `/tmp/starter-*.log`
4. Your operating system and version
5. Steps to reproduce the issue