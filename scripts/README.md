# Development Scripts

This directory contains comprehensive scripts for developing, testing, and managing the starter project with full background worker support.

## 🎯 Quick Start Scripts

### `rename-project.sh <name>`
**Rename project from "starter" to your custom name (recommended first step).**
```bash
./scripts/rename-project.sh my_awesome_project
```

### `deploy-prod.sh`
**Production deployment with Docker Compose.**
```bash
./scripts/deploy-prod.sh            # Deploy to production
```

### `start-dev.sh`
**Complete one-command setup for new developers.**
```bash
./scripts/start-dev.sh         # Start on port 3000
./scripts/start-dev.sh 8080     # Start on port 8080
```

### `dev-server.sh [port]`
**Complete development environment with validation and testing.**
- **Default port:** 3000
- **Includes:** Infrastructure, migrations, server startup, health checks
- **Auto-creates:** .env file if missing
- **Validates:** Prerequisites and working directory
```bash
./scripts/dev-server.sh         # Start on port 3000
./scripts/dev-server.sh 8080    # Start on port 8080
```

### `check-prereqs.sh`
**Validate all required dependencies before starting development.**
- **Checks:** Docker, Docker Compose, Rust, sqlx-cli
- **Auto-installs:** sqlx-cli if missing
- **Reports:** Optional tools status
```bash
./scripts/check-prereqs.sh      # Check all prerequisites
```

## 🏗️ Infrastructure Management

### `dev.sh`
Start PostgreSQL database and wait for services to be ready.
```bash
./scripts/dev.sh
```

### `reset-all.sh` 
Complete environment reset - stops all processes, cleans ports, resets database.
```bash
./scripts/reset-all.sh
```

### `status.sh`
Check status of all services, ports, PID files, and connectivity.
```bash
./scripts/status.sh
```

## 🖥️ Server Management

### `server.sh [port]`
Start HTTP API server in background with PID tracking and log management.
- **Default port:** 3000
- **PID file:** `/tmp/starter-server-{PORT}.pid`
- **Log file:** `/tmp/starter-server-{PORT}.log` (auto-rotated at 50MB)
- **Auto-cleanup:** Kills existing processes on port
```bash
./scripts/server.sh         # Start on port 3000
./scripts/server.sh 8080     # Start on port 8080
```

### `stop-server.sh [port]`
Gracefully stop server with proper cleanup.
```bash
./scripts/stop-server.sh 3000
```

### `test-server.sh [port]`
Test server health endpoints with 30-second timeout.
```bash
./scripts/test-server.sh 3000
```

## ⚙️ Background Worker Management

### `worker.sh`
Start background worker for task processing.
- **PID file:** `/tmp/starter-worker.pid`
- **Log file:** `/tmp/starter-worker.log` (auto-rotated at 50MB)
- **Processes:** Email, data processing, webhooks, file cleanup, reports
```bash
./scripts/worker.sh
```

### `stop-worker.sh`
Gracefully stop background worker.
```bash
./scripts/stop-worker.sh
```

## 🧪 Testing & Integration

### Rust Integration Tests (Recommended)
Comprehensive test suite with 38 integration tests covering all functionality:
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (~10 seconds)
cargo nextest run

# Run specific test categories
cargo nextest run auth::     # Authentication tests (6 tests)
cargo nextest run tasks::    # Task system tests (11 tests)
cargo nextest run health::   # Health check tests
cargo nextest run api::      # API standards tests
```

Benefits of the Rust test suite:
- **Fast**: 10x speedup with database template pattern
- **Isolated**: Each test gets its own database
- **Comprehensive**: 38 tests covering all functionality
- **Reliable**: Uses proper HTTP client and test harness

## 📊 Complete Workflow Examples

### Development Startup
```bash
# Method 1: One-command setup (Recommended)
./scripts/start-dev.sh 3000               # Complete setup for new developers

# Method 2: Complete environment with validation
./scripts/dev-server.sh 3000              # Start everything with testing

# Method 3: Full reset and test
./scripts/reset-all.sh --reset-database   # Clean slate
cargo nextest run                         # Complete system test

# Method 4: Manual step-by-step
./scripts/check-prereqs.sh                # Validate dependencies
./scripts/dev.sh                          # Start infrastructure
./scripts/server.sh 3000                  # Start server
./scripts/worker.sh                       # Start worker
./scripts/test-server.sh 3000             # Test health
./scripts/status.sh                       # Check everything
```

### Daily Development
```bash
# Start services
./scripts/server.sh 3000
./scripts/worker.sh

# Monitor logs
tail -f /tmp/starter-server-3000.log
tail -f /tmp/starter-worker.log

# Test changes
cargo nextest run
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"email","payload":{"to":"test@example.com","subject":"Test","body":"Hello"}}'

# Stop when done
./scripts/stop-server.sh 3000
./scripts/stop-worker.sh
```

### Debugging
```bash
# Check what's running
./scripts/status.sh

# View logs
tail -f /tmp/starter-server-3000.log
tail -f /tmp/starter-worker.log

# Clean reset if issues
./scripts/reset-all.sh
```

## 🔧 Script Features

**Process Management:**
- ✅ PID tracking and cleanup
- ✅ Graceful shutdown with SIGTERM
- ✅ Force kill fallback
- ✅ Port conflict resolution
- ✅ Stale file cleanup

**Logging:**
- ✅ Persistent logs in `/tmp/`
- ✅ Auto-rotation at 50MB
- ✅ Timestamped entries
- ✅ Structured output

**Testing:**
- ✅ Health endpoint validation
- ✅ Authentication flow testing
- ✅ End-to-end task processing
- ✅ Statistics verification
- ✅ Error scenario coverage

**Reliability:**
- ✅ Automated cleanup on exit
- ✅ Timeout handling
- ✅ Database reset capability
- ✅ Comprehensive status checks

## 📝 Adding New Scripts

When adding new scripts:
1. **Make executable:** `chmod +x script_name.sh`
2. **Follow patterns:** Use consistent error handling and cleanup
3. **Document usage:** Include examples in comments and this README
4. **Test thoroughly:** Ensure graceful cleanup and error scenarios
5. **Update documentation:** Add to this README and test workflows

## 🚀 Background Worker System

The background worker system processes tasks asynchronously:

**Supported Task Types:**
- `email` - Send notifications
- `data_processing` - Process datasets (count, sum, etc.)
- `webhook` - HTTP callbacks
- `file_cleanup` - Clean temporary files
- `report_generation` - Generate reports

**Task Features:**
- Priority-based processing (Critical → High → Normal → Low)
- Retry strategies (exponential backoff, linear, fixed interval)
- Circuit breaker protection
- Task status tracking
- Comprehensive error handling

**API Endpoints:**
- `POST /tasks` - Create task
- `GET /tasks` - List tasks
- `GET /tasks/stats` - Get statistics
- `GET /tasks/{id}` - Get task details
- `POST /tasks/{id}/cancel` - Cancel task

## 🗄️ Database Notes

- Scripts assume PostgreSQL is running (use `./scripts/dev.sh`)
- Database auto-resets in `reset-all.sh` and `test_tasks_integration.sh`
- Migrations run automatically on server startup
- Default connection: `postgres://starter_user:starter_pass@localhost:5432/starter_db`

## 🔗 Dependencies

**Required:**
- **Docker & Docker Compose** - For PostgreSQL database
- **Rust 1.75+** - Install from https://rustup.rs
- **curl** - For HTTP testing (usually pre-installed)
- **lsof** - For port checking (usually pre-installed)

**Auto-installed:**
- **sqlx-cli** - Installed automatically by scripts when needed

**Optional:**
- **jq** - For JSON parsing (scripts fall back to python3)
- **python3** - For JSON parsing fallback
- **cargo-nextest** - For faster test execution

**Validation:**
Run `./scripts/check-prereqs.sh` to validate all dependencies.