# Development Scripts

This directory contains comprehensive scripts for developing, testing, and managing the starter project with full background worker support.

## 🎯 Quick Start Scripts

### `dev-server.sh [port]`
**Complete development environment with database, migrations, and server startup (recommended).**
- **Default port:** 3000
- **Includes:** PostgreSQL startup, migrations, server startup, health checks
- **Auto-creates:** .env file if missing
- **Validates:** Prerequisites and working directory
```bash
./scripts/dev-server.sh         # Start on port 3000
./scripts/dev-server.sh 8080    # Start on port 8080
```

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


### `check-prereqs.sh`
**Validate all required dependencies before starting development.**
- **Checks:** Docker, Docker Compose, Rust, sqlx-cli
- **Auto-installs:** sqlx-cli if missing
- **Reports:** Optional tools status
```bash
./scripts/check-prereqs.sh      # Check all prerequisites
```

## 🏗️ Infrastructure Management

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
Comprehensive test suite with 53 integration tests covering all functionality:
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (~12 seconds)
cargo nextest run

# Run specific test categories
cargo nextest run auth::     # Authentication tests (6 tests)
cargo nextest run tasks::    # Task system tests (18 tests including TDD metadata tests)
cargo nextest run health::   # Health check tests
cargo nextest run api::      # API standards tests
```

Benefits of the Rust test suite:
- **Fast**: 10x speedup with database template pattern
- **Isolated**: Each test gets its own database
- **Comprehensive**: 53 tests covering all functionality including metadata persistence
- **Reliable**: Uses proper HTTP client and test harness
- **TDD**: Includes Test-Driven Development tests for critical system behaviors

### Chaos Testing Framework (Advanced)
Docker-based resilience testing with 6 difficulty levels and container isolation:

```bash
# Basic resilience testing with Docker containers
./scripts/test-chaos.sh --difficulty 1

# Advanced chaos scenarios with resource constraints
./scripts/test-chaos.sh --difficulty 6 --verbose

# Specific container-based scenarios
./scripts/test-chaos.sh --scenarios "multi-worker-chaos,db-failure"

# Container scaling and worker resilience testing
./scripts/test-chaos.sh --scenarios "baseline,server-restart,worker-restart"

# Dynamic worker scaling with 4-phase testing
./scripts/test-chaos.sh --scenarios "dynamic-scaling"
```

**API Endpoint Testing:**
```bash
./scripts/test-with-curl.sh             # 40+ endpoint tests (~5 seconds)
./scripts/test-with-curl.sh localhost 8080  # Custom host/port
```

**Chaos Testing Levels (Redesigned):**
- **Level 1** - Basic Resilience: 2 workers, 10 tasks, ≥90% completion (baseline functionality)
- **Level 2** - Light Disruption: 2 workers, 15 tasks, ≥85% completion (introduction of failures)
- **Level 3** - Load Testing: 3 workers, 25 tasks, ≥80% completion (increased task volume)
- **Level 4** - Resource Pressure: 3 workers, 35 tasks, ≥75% completion (challenging workload)
- **Level 5** - Extreme Chaos: 4 workers, 30 tasks, ≥60% completion (high-pressure scenarios)
- **Level 6** - Catastrophic Load: 2 workers, 40 tasks, 20-50% completion (stress test limits)

**Chaos Testing Features:**
- **Multi-worker Resilience**: Docker Compose scaling with container failure simulation
- **Task Completion Monitoring**: Real-time progress tracking with metadata validation
- **Deadline Enforcement**: Validates system performance under time pressure
- **Container Failure Injection**: Systematic container kills and automatic restarts
- **Comprehensive Reporting**: Detailed logs, statistics, and failure analysis

**Docker Container Features:**
- **Container Isolation**: Each service runs in isolated Docker containers for realistic testing
- **Resource Constraints**: CPU and memory limits simulate deployment constraints
- **Container Failures**: Docker kill/restart scenarios mirror real deployment issues
- **Horizontal Scaling**: Multi-container worker testing with Docker Compose scaling
- **Fresh Code Testing**: Automatically rebuilds containers ensuring tests use latest changes

## 📊 Complete Workflow Examples

### Development Startup
```bash
# Method 1: Complete environment (Recommended)
./scripts/dev-server.sh                    # Complete setup with database and server

# Method 2: Full reset and test
./scripts/reset-all.sh --reset-database   # Clean slate
cargo nextest run                         # Complete system test

# Method 3: Manual step-by-step
./scripts/check-prereqs.sh                # Validate dependencies
docker compose up -d postgres             # Start database
docker compose up --wait                  # Wait for services
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

- Scripts assume PostgreSQL is running (use `docker compose up -d postgres`)
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