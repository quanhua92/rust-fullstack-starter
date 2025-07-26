# Development Scripts

This directory contains comprehensive scripts for developing, testing, and managing the starter project with full background worker support.

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

### `test_auth.sh`
Comprehensive authentication system testing - registration, login, sessions, security.
```bash
./scripts/test_auth.sh
```

### `test_tasks_integration.sh`
Complete end-to-end integration test for the entire system:
- Server startup and health checks
- User authentication flow
- Task creation via API (email, data processing)
- Background worker processing
- Task status verification
- Statistics reporting
```bash
./scripts/test_tasks_integration.sh
```

## 📊 Complete Workflow Examples

### Development Startup
```bash
# Method 1: Full reset and test
./scripts/reset-all.sh                    # Clean slate
./scripts/test_tasks_integration.sh       # Complete system test

# Method 2: Manual step-by-step
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
./scripts/test_auth.sh
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

- **Docker Compose** - For PostgreSQL
- **curl** - For HTTP testing
- **jq** - For JSON parsing (fallback to python3)
- **lsof** - For port checking
- **python3** - For JSON parsing