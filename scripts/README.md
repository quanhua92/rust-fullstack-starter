# Development Scripts

This directory contains comprehensive scripts for developing, testing, and managing the starter project with full background worker support.

## 🎯 Quick Start Scripts

### `dev-server.sh [options]`
**Complete full-stack development environment with database, web build, and unified server (recommended).**
- **Default port:** 3000
- **Includes:** PostgreSQL startup, web frontend build, unified API + static serving, migrations
- **Optional:** Background worker startup with `-w` flag
- **Auto-creates:** .env file if missing
- **Validates:** Prerequisites and working directory
- **Options:** `-f` (foreground), `-w` (with worker), `-p PORT`, `--api-only`, `-h` (help)
```bash
./scripts/dev-server.sh              # Database + web + API
./scripts/dev-server.sh -w           # Complete setup: database + web + API + worker
./scripts/dev-server.sh -f           # Foreground mode
./scripts/dev-server.sh --api-only   # Skip web build (API only)
./scripts/dev-server.sh -p 8080      # Custom port
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

### `server.sh [port] [-f|--foreground]`
Start HTTP API server with static file serving in background or foreground mode.
- **Default port:** 3000
- **Static serving:** Automatically serves React frontend from `web/dist` with SPA fallback
- **Auto-build:** Detects missing web build and builds frontend automatically
- **Background mode:** PID tracking, log files, process management
- **Foreground mode (-f):** Direct exec, Ctrl+C kills process cleanly
- **PID file:** `/tmp/starter-server-{PORT}.pid` (background only)
- **Log file:** `/tmp/starter-server-{PORT}.log` (background only, auto-rotated at 50MB)
- **Auto-cleanup:** Always kills existing processes on port first
```bash
# Background mode (default) - serves API + frontend
./scripts/server.sh         # Port 3000, auto-builds web if needed
./scripts/server.sh 8080     # Port 8080, auto-builds web if needed

# Foreground mode (direct control) - unified full-stack server
./scripts/server.sh -f       # Port 3000, direct exec, Ctrl+C to stop
./scripts/server.sh 3000 -f  # Port 3000, direct exec, Ctrl+C to stop
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

## 🌐 Web Frontend Management

### `build-web.sh`
Build React/TypeScript frontend with comprehensive quality checks.
- **Output:** Production build to `web/dist/` directory
- **Quality checks:** Dependencies, TypeScript, linting, build validation
- **Auto-install:** pnpm if not available
- **Fallback:** Regular install if frozen lockfile fails
```bash
./scripts/build-web.sh              # Build with quality checks
```


## ⚙️ Background Worker Management

### `worker.sh [--id ID] [-f|--foreground]`
Start background task worker in background or foreground mode with support for concurrent workers.
- **Background mode:** PID tracking, log files, process management
- **Foreground mode (-f):** Direct exec, Ctrl+C kills process cleanly
- **Concurrent workers (--id):** Run multiple workers with different IDs (default: 0)
- **PID file:** `/tmp/starter-worker-{ID}.pid` (background only)
- **Log file:** `/tmp/starter-worker-{ID}.log` (background only, auto-rotated at 50MB)
- **Auto-cleanup:** Always kills existing worker processes with same ID first
- **Processes:** Email, data processing, webhooks, file cleanup, reports
```bash
# Background mode (default, ID 0)
./scripts/worker.sh          # Creates PID file, logs to file

# Concurrent workers with different IDs
./scripts/worker.sh --id 1   # Worker ID 1
./scripts/worker.sh --id 2   # Worker ID 2 (runs alongside ID 1)

# Foreground mode (direct control)
./scripts/worker.sh -f       # Direct exec, Ctrl+C to stop
./scripts/worker.sh --id 3 -f # Foreground worker with ID 3
```

### `stop-worker.sh [--id ID] [--all]`
Gracefully stop background worker(s).
- **Specific worker:** Stop worker with specific ID (default: 0)
- **All workers:** Stop all workers by finding all PID files
```bash
# Stop default worker (ID 0)
./scripts/stop-worker.sh

# Stop specific worker by ID
./scripts/stop-worker.sh --id 1
./scripts/stop-worker.sh --id 2

# Stop all workers
./scripts/stop-worker.sh --all
```

## 🔧 Database Management

### `prepare-sqlx.sh`
**Update SQLx query cache for offline compilation.**
- **Purpose:** Generates query metadata for offline Rust compilation
- **Requirement:** Database must be running (use `docker compose up -d`)
- **Auto-directory:** Changes to starter/ directory automatically
- **Comprehensive:** Uses `--all -- --all-targets` for complete caching
```bash
./scripts/prepare-sqlx.sh      # Update SQLx query cache
```

**When to use:**
- After adding/modifying database queries in your code
- Before committing changes (ensures offline compilation works)
- When SQLx compilation fails with "no cached data" errors
- After database schema changes or migrations

## 🧪 Testing & Integration

### Rust Integration Tests (Recommended)
Comprehensive test suite with 119 integration tests covering all functionality:
```bash
# Install faster test runner (recommended)
cargo install cargo-nextest

# Run all tests (~12 seconds)
cargo nextest run

# Run specific test categories
cargo nextest run auth::     # Authentication tests (12 tests)
cargo nextest run users::    # User management tests (17 tests)
cargo nextest run tasks::    # Task system tests (18 tests including TDD metadata tests)
cargo nextest run health::   # Health check tests
cargo nextest run api::      # API standards tests
```

Benefits of the Rust test suite:
- **Fast**: 10x speedup with database template pattern
- **Isolated**: Each test gets its own database
- **Comprehensive**: 119 tests covering all functionality including user management and metadata persistence
- **Reliable**: Uses proper HTTP client and test harness
- **TDD**: Includes Test-Driven Development tests for critical system behaviors
- **RBAC Testing**: Complete role-based access control validation

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
./scripts/test-with-curl.sh             # 44+ endpoint tests (~5 seconds)
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
- **Admin CLI Integration**: Direct database monitoring bypassing API authentication (RBAC-aware)
- **Foundation Checks**: Pre-scenario validation ensuring monitoring infrastructure works
- **Fail-Fast Patterns**: Early exit on detected failures to save testing time
- **Enhanced Baseline**: Fixed degradation from 100% to 58% success rate
- **ANSI-Aware Parsing**: Proper handling of colored CLI output for accurate statistics
- **RBAC Testing**: Validates role-based access controls under chaos conditions

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
./scripts/worker.sh                       # Start worker (ID 0)
./scripts/worker.sh --id 1                # Start concurrent worker (ID 1)
./scripts/test-server.sh 3000             # Test health
./scripts/status.sh                       # Check everything
```

### Daily Development
```bash
# Option 1: Background mode (traditional)
./scripts/server.sh 3000
./scripts/worker.sh                       # Worker ID 0
./scripts/worker.sh --id 1                # Worker ID 1 (concurrent)

# Monitor logs
tail -f /tmp/starter-server-3000.log
tail -f /tmp/starter-worker-0.log   # Default worker (ID 0)
# tail -f /tmp/starter-worker-1.log # Worker ID 1

# Option 2: Foreground mode (direct control)
# Terminal 1: Server in foreground
./scripts/server.sh 3000 -f

# Terminal 2: Worker in foreground  
./scripts/worker.sh -f                    # Worker ID 0 foreground
# Or: ./scripts/worker.sh --id 1 -f       # Worker ID 1 foreground

# Test changes
cargo nextest run
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"email","payload":{"to":"test@example.com","subject":"Test","body":"Hello"}}'

# Stop when done
./scripts/stop-server.sh 3000
./scripts/stop-worker.sh                  # Stop default worker (ID 0)
./scripts/stop-worker.sh --id 1           # Stop worker ID 1
./scripts/stop-worker.sh --all            # Stop all workers
```

### Debugging
```bash
# Check what's running
./scripts/status.sh

# View logs
tail -f /tmp/starter-server-3000.log
tail -f /tmp/starter-worker-0.log   # Default worker (ID 0)
# tail -f /tmp/starter-worker-*.log # All workers

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
- `GET /tasks/types` - List registered task types
- `POST /tasks/types` - Register task type

**Admin CLI Commands (Direct Database Access):**
- `admin task-stats` - Task statistics bypassing API (shows all tasks regardless of ownership)
- `admin task-stats --tag "baseline"` - Filter task statistics by metadata tag
- `admin list-tasks` - List tasks with verbose details (shows all users' tasks)
- `admin list-tasks --verbose` - Include detailed task information and user context
- `admin clear-completed` - Maintenance operations (cleans completed tasks from all users)

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