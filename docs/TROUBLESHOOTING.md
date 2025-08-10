# Troubleshooting Guide

*Systematic problem-solving for development, deployment, and operational issues. From quick fixes to comprehensive debugging strategies.*

## 🚨 Quick Fixes

### Server Won't Start
```bash
# Kill conflicting processes
lsof -ti:3000 | xargs kill -9

# Reset everything
./scripts/reset-all.sh --reset-database

# Start fresh
./scripts/dev-server.sh --port 3000
```

### Database Connection Failed
```bash
# Check Docker status
docker ps | grep postgres

# Reset database completely
docker compose down -v
docker compose up -d postgres && sleep 5
cd starter && sqlx migrate run
```

### Frontend Build Fails
```bash
cd web
rm -rf node_modules dist .next
pnpm install
pnpm run generate-api
pnpm run build
```

### Permission Errors
```bash
chmod +x scripts/*.sh
sudo chown -R $(whoami) .
```

## 🔍 Systematic Debugging

### Problem Isolation Framework

**1. Layer-Based Debugging**:
```mermaid
graph TD
    USER[👤 User Action] --> UI[🌐 Frontend Layer]
    UI --> API[🔌 API Layer] 
    API --> BIZ[🏗️ Business Logic]
    BIZ --> DB[🗄️ Database Layer]
    
    UI -.-> UI_DEBUG[Debug: Browser DevTools<br/>Network tab, Console logs]
    API -.-> API_DEBUG[Debug: HTTP status codes<br/>Request/response logs]
    BIZ -.-> BIZ_DEBUG[Debug: Application logs<br/>Rust error traces]
    DB -.-> DB_DEBUG[Debug: SQL logs<br/>Connection pool status]
    
    classDef layer fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef debug fill:#fff3e0,stroke:#e65100,stroke-width:2px
    
    class USER,UI,API,BIZ,DB layer
    class UI_DEBUG,API_DEBUG,BIZ_DEBUG,DB_DEBUG debug
```

**2. Information Gathering**:
```bash
# System state
./scripts/status.sh

# Application logs
tail -f /tmp/starter-server-*.log
tail -f /tmp/starter-worker-*.log

# Database connectivity
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "\l"

# Resource usage
docker stats
df -h
free -h
```

### Authentication Issues

**Problem: Login fails with valid credentials**

**Debugging steps**:
```bash
# 1. Check task stats (available admin command)
cd starter && cargo run -- admin task-stats

# 2. Verify session creation
grep "session created" /tmp/starter-server-*.log

# 3. Test direct database query
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT username, created_at FROM users WHERE username = 'testuser';"

# 4. Check for timing attacks protection
grep "constant_time" /tmp/starter-server-*.log
```

**Common causes**:
- Password hash mismatch (check bcrypt compatibility)
- Database connection issues during auth
- Session cleanup timing issues
- Case sensitivity in username

### Task Processing Issues

**Problem: Background tasks stuck in pending**

**Debugging steps**:
```bash
# 1. Check worker status
./scripts/status.sh | grep worker

# 2. Check task type registration
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/tasks/types

# 3. Check task queue
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT task_type, status, COUNT(*) FROM tasks GROUP BY task_type, status;"

# 4. Worker logs
grep "processing task" /tmp/starter-worker-*.log
```

**Common causes**:
- Worker not running (`./scripts/worker.sh -f`)
- Task type not registered
- Circuit breaker open
- Database deadlock

### Database Issues

**Problem: Migration failures**

**Debugging steps**:
```bash
# 1. Check current migration state
cd starter && sqlx migrate info

# 2. Check database permissions
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "\du"

# 3. Manual migration step
sqlx migrate run --source migrations

# 4. Reset if corrupted
./scripts/reset-all.sh --reset-database
```

**Problem: Connection pool exhausted**

**Debugging steps**:
```bash
# 1. Check active connections
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT count(*) FROM pg_stat_activity;"

# 2. Check pool configuration
grep "max_connections\|min_connections" /tmp/starter-server-*.log

# 3. Find long-running queries
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT query, state, query_start FROM pg_stat_activity WHERE state = 'active';"
```

### API Issues

**Problem: 500 Internal Server Error**

**Debugging steps**:
```bash
# 1. Check server logs
tail -50 /tmp/starter-server-*.log | grep ERROR

# 2. Check specific error
curl -v http://localhost:3000/api/v1/problematic-endpoint

# 3. Database query errors
grep "sqlx::Error" /tmp/starter-server-*.log

# 4. Test with minimal payload
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"test","payload":{}}'
```

### Frontend Issues

**Problem: TypeScript errors after API changes**

**Solution**:
```bash
# 1. Regenerate API types
./scripts/prepare-openapi.sh

# 2. Verify OpenAPI schema
curl http://localhost:3000/api-docs/openapi.json | jq .

# 3. Check for breaking changes
cd web && pnpm run check
```

**Problem: React Query cache collisions**

**Solution**:
```typescript
// ❌ Bad: Manual query keys
const { data } = useQuery({
  queryKey: ['tasks', 'list'], // Collision risk
  queryFn: () => apiClient.getTasks()
});

// ✅ Good: Centralized query keys
import { useHealthBasic } from '@/hooks/useApiQueries';
const { data } = useHealthBasic(15000); // Auto-refresh every 15s
```

## 🏥 Health Check Debugging

### Health Check Hierarchy

```bash
# 1. Basic connectivity
curl http://localhost:3000/api/v1/health

# 2. System components
curl http://localhost:3000/api/v1/health/detailed

# 3. Database-specific
curl http://localhost:3000/api/v1/health/ready

# 4. Application startup
curl http://localhost:3000/api/v1/health/startup
```

### Health Status Interpretation

```json
{
  "status": "degraded",
  "checks": {
    "database": "healthy",
    "task_processor": "unhealthy",  // ← Problem here
    "memory": "healthy"
  }
}
```

**Degraded status debugging**:
- `database: unhealthy` → Check PostgreSQL connection, pool exhaustion
- `task_processor: unhealthy` → Check worker registration, circuit breaker
- `memory: unhealthy` → Check for memory leaks, increase container limits

## 🐳 Docker Issues

### Container Problems

**Problem: Container won't start**

```bash
# 1. Check container logs
docker logs container_name

# 2. Check resource constraints
docker stats

# 3. Test image directly
docker run -it --rm image_name /bin/sh

# 4. Check Docker daemon
docker system info
```

**Problem: Permission denied in container**

```bash
# 1. Check user context
docker exec container_name id

# 2. Check file permissions
docker exec container_name ls -la /path/to/file

# 3. Fix ownership (if needed)
docker exec --user root container_name chown app:app /path/to/file
```

### Volume Issues

**Problem: Data not persisting**

```bash
# 1. Check volume mounts
docker inspect container_name | grep -A 10 "Mounts"

# 2. Verify volume permissions
docker exec container_name ls -la /data/volume/path

# 3. Test volume directly
docker run --rm -v volume_name:/test alpine ls -la /test
```

## 🚀 Performance Issues

### Slow API Responses

**Debugging steps**:
```bash
# 1. Check response times
curl -w "@curl-format.txt" -s -o /dev/null http://localhost:3000/api/v1/endpoint

# 2. Database query performance
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 5;"

# 3. Connection pool metrics
grep "connection_pool" /tmp/starter-server-*.log

# 4. Memory usage
docker stats --no-stream
```

### High Memory Usage

**Debugging steps**:
```bash
# 1. Check container memory
docker exec container_name cat /proc/meminfo

# 2. Rust memory profiling (in development)
RUST_LOG=debug cargo run server

# 3. Database memory usage
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT setting FROM pg_settings WHERE name = 'shared_buffers';"

# 4. Identify memory leaks
valgrind ./target/debug/starter server  # Linux only
```

### Slow Task Processing

**Debugging steps**:
```bash
# 1. Check worker concurrency
grep "worker_count\|concurrent" /tmp/starter-worker-*.log

# 2. Task processing times
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/api/v1/monitoring/events?source=task-processor&limit=20"

# 3. Circuit breaker status
grep "circuit.*open\|circuit.*closed" /tmp/starter-worker-*.log

# 4. Database transaction times
grep "query_duration" /tmp/starter-server-*.log
```

## 🔒 Security Issues

### Authentication Bypass Attempts

**Monitoring**:
```bash
# 1. Failed login attempts
grep "authentication.*failed\|invalid.*credentials" /tmp/starter-server-*.log

# 2. Suspicious API calls
grep "401\|403" /tmp/starter-server-*.log | tail -20

# 3. Check nginx rate limiting (if using nginx profile)
grep "limiting" /var/log/nginx/access.log
```

### Session Management Issues

**Debugging steps**:
```bash
# 1. Session cleanup
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT COUNT(*) FROM sessions WHERE expires_at < NOW();"

# 2. Active sessions
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT user_id, COUNT(*) FROM sessions WHERE expires_at > NOW() GROUP BY user_id;"

# 3. Session fixation prevention
grep "session.*created\|session.*invalidated" /tmp/starter-server-*.log
```

## 🧪 Testing Issues

### Integration Test Failures

**Problem: Tests fail intermittently**

**Debugging**:
```bash
# 1. Run single test with logs
RUST_LOG=debug cargo test test_name -- --nocapture

# 2. Check database isolation
cargo test --test integration -- --show-output | grep "test_db"

# 3. Check for race conditions
cargo test -- --test-threads=1

# 4. Database cleanup between tests
grep "dropping.*database" target/debug/deps/integration-*.log
```

**Problem: Database test interference**

**Solution**:
```rust
// Ensure each test gets unique database
#[tokio::test]
async fn test_something() {
    let app = spawn_app().await;  // Creates unique test database
    // Test implementation...
    // Database automatically cleaned up when app drops
}
```

## 🔧 Development Environment

### Rust Compilation Issues

**Problem: sqlx compile-time errors**

**Solution**:
```bash
# 1. Prepare query cache
./scripts/prepare-sqlx.sh

# 2. Force regeneration
rm -rf .sqlx/ && ./scripts/prepare-sqlx.sh

# 3. Check database schema
sqlx migrate info --source starter/migrations
```

**Problem: Dependency conflicts**

**Solution**:
```bash
# 1. Clean build
cargo clean && cargo build

# 2. Update dependencies
cargo update

# 3. Check for duplicate versions
cargo tree --duplicates
```

### IDE Integration Issues

**Problem: rust-analyzer not working**

**Solution**:
```bash
# 1. Restart rust-analyzer
# In VS Code: Ctrl+Shift+P > "Rust Analyzer: Restart Server"

# 2. Clean workspace
cargo clean && rm -rf target/

# 3. Check rust-analyzer logs
# In VS Code: Output panel > "Rust Analyzer Language Server"
```

## 📋 Diagnostic Commands

### System Information
```bash
# Complete system status
./scripts/status.sh

# Environment validation  
./scripts/check-prereqs.sh

# Resource usage
docker system df
docker system events --since 1h
```

### Application State
```bash
# Admin CLI (bypasses API authentication)
cargo run -- admin task-stats
cargo run -- admin list-tasks --limit 5 --verbose
cargo run -- admin clear-completed --dry-run

# API testing
./scripts/test-with-curl.sh localhost 3000

# Database state
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "\dt"  # List tables
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT COUNT(*) FROM tasks WHERE status = 'pending';"
```

### Performance Metrics
```bash
# API response times
curl -w "@curl-format.txt" -s -o /dev/null http://localhost:3000/api/v1/health

# Database performance
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del FROM pg_stat_user_tables;"

# System resources
top -p $(pgrep starter)
```

---

*Remember: When debugging complex issues, work systematically through the layers, gather information before making changes, and document what works for future reference.*