# Configuration Guide

This document explains all configuration options and environment variables used in the Rust Full-Stack Starter project.

## Configuration System Overview

The application uses a hierarchical configuration system:

1. **Default values** - Built into the application
2. **Environment variables** - Override defaults using `STARTER__*` prefix
3. **Validation** - All configuration is validated at startup

## Environment Variables

### Server Configuration

Controls the HTTP server behavior.

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTER__SERVER__HOST` | `127.0.0.1` | Server bind address |
| `STARTER__SERVER__PORT` | `8080` | Server port (overridden by CLI) |
| `STARTER__SERVER__CORS_ORIGINS` | `["http://localhost:5173"]` | Allowed CORS origins (JSON array) |
| `STARTER__SERVER__REQUEST_TIMEOUT_SECS` | `30` | HTTP request timeout in seconds |

**Examples:**
```bash
# Bind to all interfaces
STARTER__SERVER__HOST=0.0.0.0

# Multiple CORS origins
STARTER__SERVER__CORS_ORIGINS=["http://localhost:3000","http://localhost:5173"]

# Longer timeout for slow requests
STARTER__SERVER__REQUEST_TIMEOUT_SECS=60
```

### Database Configuration

Controls PostgreSQL connection and pooling.

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTER__DATABASE__USER` | `starter_user` | Database username |
| `STARTER__DATABASE__PASSWORD` | `starter_pass` | Database password |
| `STARTER__DATABASE__HOST` | `localhost` | Database host |
| `STARTER__DATABASE__PORT` | `5432` | Database port |
| `STARTER__DATABASE__DATABASE` | `starter_db` | Database name |
| `STARTER__DATABASE__MAX_CONNECTIONS` | `10` | Maximum connections in pool |
| `STARTER__DATABASE__MIN_CONNECTIONS` | `2` | Minimum connections in pool |
| `STARTER__DATABASE__CONNECT_TIMEOUT_SECS` | `30` | Connection timeout |
| `STARTER__DATABASE__IDLE_TIMEOUT_SECS` | `300` | Idle connection timeout |
| `STARTER__DATABASE__MAX_LIFETIME_SECS` | `600` | Maximum connection lifetime |

**Examples:**
```bash
# Production database
STARTER__DATABASE__HOST=prod-db.example.com
STARTER__DATABASE__USER=app_user
STARTER__DATABASE__PASSWORD=secure_password
STARTER__DATABASE__DATABASE=app_production

# High-traffic pool settings
STARTER__DATABASE__MAX_CONNECTIONS=50
STARTER__DATABASE__MIN_CONNECTIONS=10
```

### Authentication Configuration

Controls session management and security.

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTER__AUTH__SESSION_DURATION_HOURS` | `24` | Session lifetime in hours |
| `STARTER__AUTH__CLEANUP_INTERVAL_SECS` | `3600` | Expired session cleanup interval |

**Examples:**
```bash
# Shorter sessions for security
STARTER__AUTH__SESSION_DURATION_HOURS=8

# More frequent cleanup
STARTER__AUTH__CLEANUP_INTERVAL_SECS=1800
```

### Worker Configuration

Controls background job processing.

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTER__WORKER__CONCURRENCY` | `4` | Number of concurrent workers |
| `STARTER__WORKER__POLL_INTERVAL_SECS` | `5` | Job polling interval |
| `STARTER__WORKER__MAX_RETRIES` | `3` | Maximum job retry attempts |
| `STARTER__WORKER__RETRY_BACKOFF_BASE_SECS` | `2` | Base retry backoff time |

**Examples:**
```bash
# High-performance worker
STARTER__WORKER__CONCURRENCY=16
STARTER__WORKER__POLL_INTERVAL_SECS=1

# Conservative retry policy
STARTER__WORKER__MAX_RETRIES=5
STARTER__WORKER__RETRY_BACKOFF_BASE_SECS=5
```

### Security Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `STARTER__INITIAL_ADMIN_PASSWORD` | None | Creates admin user on first startup if set |

**Examples:**
```bash
# Create initial admin user
STARTER__INITIAL_ADMIN_PASSWORD=SecureAdminPassword123!
```

## Special Environment Variables

### DATABASE_URL (sqlx CLI only)

Used by sqlx CLI tools for migrations and code generation. **The application does not read this variable.**

```bash
DATABASE_URL=postgres://starter_user:starter_pass@localhost:5432/starter_db
```

### RUST_LOG (Logging)

Controls application logging levels.

```bash
# Debug level for all modules
RUST_LOG=debug

# Trace level for starter application only
RUST_LOG=starter=trace

# Multiple modules
RUST_LOG=starter=debug,sqlx=info
```

## Configuration Examples

### Development Environment

```bash
# .env for development
STARTER__SERVER__HOST=0.0.0.0
STARTER__SERVER__PORT=8080
STARTER__SERVER__CORS_ORIGINS=["http://localhost:3000","http://localhost:5173"]

STARTER__DATABASE__USER=starter_user
STARTER__DATABASE__PASSWORD=starter_pass
STARTER__DATABASE__HOST=localhost
STARTER__DATABASE__PORT=5432
STARTER__DATABASE__DATABASE=starter_db

STARTER__INITIAL_ADMIN_PASSWORD=admin123

# For sqlx CLI
DATABASE_URL=postgres://starter_user:starter_pass@localhost:5432/starter_db

# Debug logging
RUST_LOG=starter=debug
```

### Production Environment

```bash
# Production configuration
STARTER__SERVER__HOST=0.0.0.0
STARTER__SERVER__PORT=80
STARTER__SERVER__REQUEST_TIMEOUT_SECS=30
STARTER__SERVER__CORS_ORIGINS=["https://app.example.com"]

STARTER__DATABASE__HOST=prod-db.internal
STARTER__DATABASE__USER=app_prod
STARTER__DATABASE__PASSWORD=${DATABASE_PASSWORD_FROM_SECRETS}
STARTER__DATABASE__DATABASE=app_production
STARTER__DATABASE__MAX_CONNECTIONS=25
STARTER__DATABASE__MIN_CONNECTIONS=5

STARTER__AUTH__SESSION_DURATION_HOURS=8
STARTER__AUTH__CLEANUP_INTERVAL_SECS=1800

STARTER__WORKER__CONCURRENCY=8
STARTER__WORKER__POLL_INTERVAL_SECS=5

# Production logging
RUST_LOG=starter=info,warn
```

### Testing Environment

```bash
# Test configuration
STARTER__DATABASE__USER=test_user
STARTER__DATABASE__PASSWORD=test_pass
STARTER__DATABASE__DATABASE=starter_test
STARTER__DATABASE__MAX_CONNECTIONS=5

STARTER__AUTH__SESSION_DURATION_HOURS=1
STARTER__WORKER__CONCURRENCY=1

# Test database for sqlx
DATABASE_URL=postgres://test_user:test_pass@localhost:5432/starter_test

# Test logging
RUST_LOG=starter=debug
```

## Configuration Validation

The application validates all configuration at startup and will fail with helpful error messages:

### Common Validation Errors

```bash
# Missing required values
Database user cannot be empty
Database password cannot be empty
Database host cannot be empty
Database name cannot be empty

# Invalid ranges
Server port must be specified
max_connections must be >= min_connections
min_connections must be > 0
Worker concurrency must be > 0
```

### Validation Override

You can bypass validation for testing (not recommended):

```bash
# This will still fail validation
STARTER__DATABASE__USER=""
```

## Docker Compose Integration

The `docker-compose.yaml` reads the same environment variables:

```yaml
environment:
  POSTGRES_USER: ${STARTER__DATABASE__USER}
  POSTGRES_PASSWORD: ${STARTER__DATABASE__PASSWORD}
  POSTGRES_DB: ${STARTER__DATABASE__DATABASE}
```

This ensures consistency between the application and infrastructure.

## Configuration Loading Order

1. **Application defaults** (hardcoded in `config.rs`)
2. **Environment variables** with `STARTER__` prefix
3. **Validation** - ensures all required values are present and valid
4. **Runtime overrides** - CLI arguments can override some values

## Best Practices

### Security
- Never commit real passwords to version control
- Use strong passwords in production
- Rotate database credentials regularly
- Use environment-specific configurations

### Performance
- Tune connection pool sizes based on load
- Monitor connection usage
- Adjust timeouts based on network conditions
- Scale worker concurrency with CPU cores

### Development
- Use `.env` for local development
- Keep `.env.example` updated with new variables
- Document any new configuration options
- Test configuration changes locally first

### Production
- Use container orchestration secrets
- Monitor configuration drift
- Document environment-specific settings
- Implement configuration backup/restore