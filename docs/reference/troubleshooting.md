# Troubleshooting Guide

*Solutions for common issues when developing with the Rust Full-Stack Starter.*

## Quick Diagnostics

### Check System Status
```bash
# Check all services
./scripts/status.sh

# Check specific components
docker compose ps                    # Database status
ps aux | grep "starter"             # Running processes
lsof -i :3000                       # Port usage
tail -f /tmp/starter-*.log          # Recent logs
```

### Health Endpoints
```bash
# Basic health check
curl http://localhost:3000/health

# Detailed health with database
curl http://localhost:3000/health/detailed

# Task statistics
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tasks/stats
```

## Database Issues

### Database Connection Failed

**Symptoms:**
- Server fails to start with database connection errors
- "connection refused" or "database does not exist" errors

**Diagnosis:**
```bash
# Check if PostgreSQL is running
docker compose ps postgres

# Check database logs
docker compose logs postgres

# Test direct connection
psql postgres://starter_user:starter_pass@localhost:5432/starter_db -c "SELECT 1"
```

**Solutions:**

1. **Start PostgreSQL:**
   ```bash
   docker compose up -d postgres
   # Wait for startup message
   docker compose logs -f postgres
   ```

2. **Reset database completely:**
   ```bash
   docker compose down -v
   docker compose up -d postgres
   sqlx migrate run
   ```

3. **Check environment variables:**
   ```bash
   # Verify .env file exists and has correct values
   cat .env | grep DATABASE
   
   # Check individual database components (not DATABASE_URL)
   echo $STARTER__DATABASE__HOST
   echo $STARTER__DATABASE__PORT
   echo $STARTER__DATABASE__USER
   ```

### Migration Errors

**Symptoms:**
- "migration failed" errors on server startup
- Schema version conflicts

**Solutions:**

1. **Check migration status:**
   ```bash
   sqlx migrate info
   ```

2. **Reset migrations:**
   ```bash
   # CAREFUL: This deletes all data
   docker compose down -v
   docker compose up -d postgres
   sqlx migrate run
   ```

3. **Manual migration check:**
   ```bash
   psql $DATABASE_URL -c "\dt"  # List tables
   psql $DATABASE_URL -c "SELECT version FROM _sqlx_migrations ORDER BY version;"
   ```

## Server Issues

### Server Won't Start

**Symptoms:**
- Compilation errors
- Port already in use
- Permission denied errors

**Diagnosis:**
```bash
# Check if port is in use
lsof -i :3000

# Check for compilation errors
cargo check

# Check file permissions
ls -la scripts/
```

**Solutions:**

1. **Kill existing processes:**
   ```bash
   ./scripts/stop-server.sh 3000
   # Or force kill
   pkill -f "starter server"
   ```

2. **Use different port:**
   ```bash
   ./scripts/server.sh 8080
   ```

3. **Fix compilation:**
   ```bash
   cargo clean
   cargo build
   ```

4. **Make scripts executable:**
   ```bash
   chmod +x scripts/*.sh
   ```

### Server Crashes or Hangs

**Symptoms:**
- Server process dies unexpectedly
- Requests hang without response
- High CPU or memory usage

**Diagnosis:**
```bash
# Check server logs
tail -f /tmp/starter-server-3000.log

# Check system resources
top | grep starter
ps aux | grep starter

# Check for zombie processes
ps aux | grep -E "(starter|Z)"
```

**Solutions:**

1. **Restart server with debugging:**
   ```bash
   ./scripts/stop-server.sh 3000
   RUST_LOG=debug ./scripts/server.sh 3000
   ```

2. **Check database connections:**
   ```bash
   # Look for connection pool exhaustion
   grep -i "pool" /tmp/starter-server-3000.log
   ```

3. **Reduce concurrency:**
   ```bash
   # Lower database connection limits
   STARTER__DATABASE__MAX_CONNECTIONS=5 ./scripts/server.sh 3000
   ```

## Background Task Issues

### Tasks Not Processing

**Symptoms:**
- Tasks remain in "pending" status
- Worker logs show no activity
- Task queue builds up

**Diagnosis:**
```bash
# Check if worker is running
./scripts/status.sh

# Check worker logs
tail -f /tmp/starter-worker.log

# Check task queue
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks?status=pending&limit=5"

# Check task statistics
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/tasks/stats
```

**Solutions:**

1. **Start worker if not running:**
   ```bash
   ./scripts/worker.sh
   ```

2. **Restart worker:**
   ```bash
   ./scripts/stop-worker.sh
   ./scripts/worker.sh
   ```

3. **Check worker configuration:**
   ```bash
   # Verify worker settings
   echo $STARTER__WORKER__CONCURRENCY
   echo $STARTER__WORKER__POLL_INTERVAL_SECS
   ```

4. **Reduce worker load:**
   ```bash
   # Start with lower concurrency
   STARTER__WORKER__CONCURRENCY=1 ./scripts/worker.sh
   ```

### High Task Failure Rate

**Symptoms:**
- Many tasks in "failed" status
- Repeated retry attempts
- Error messages in worker logs

**Diagnosis:**
```bash
# Check failed tasks
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks?status=failed&limit=10"

# Look for error patterns
grep "ERROR" /tmp/starter-worker.log | tail -20

# Check specific task details
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:3000/tasks/TASK_ID_HERE"
```

**Solutions:**

1. **Check task payloads:**
   ```bash
   # Look for invalid task data
   grep -i "invalid.*payload" /tmp/starter-worker.log
   ```

2. **Verify task types:**
   ```bash
   # Check for unregistered task types
   grep -i "unknown.*task.*type" /tmp/starter-worker.log
   ```

3. **Test task creation:**
   ```bash
   # Create simple test task
   curl -X POST http://localhost:3000/tasks \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "task_type": "email",
       "payload": {
         "to": "test@example.com",
         "subject": "Test",
         "body": "Hello"
       }
     }'
   ```

### Worker Memory Issues

**Symptoms:**
- Worker process consuming excessive memory
- Out of memory errors
- System becomes unresponsive

**Diagnosis:**
```bash
# Check memory usage
ps aux | grep "starter worker"
top -p $(pgrep -f "starter worker")

# Check for memory leaks
grep -i "memory\|oom" /tmp/starter-worker.log
```

**Solutions:**

1. **Reduce worker concurrency:**
   ```bash
   STARTER__WORKER__CONCURRENCY=2 ./scripts/worker.sh
   ```

2. **Restart worker periodically:**
   ```bash
   # Add to cron for automatic restart
   0 2 * * * /path/to/scripts/stop-worker.sh && /path/to/scripts/worker.sh
   ```

3. **Check task payload sizes:**
   ```bash
   # Look for large payloads in database
   psql $DATABASE_URL -c "
     SELECT id, task_type, length(payload::text) as payload_size 
     FROM tasks 
     ORDER BY payload_size DESC 
     LIMIT 10;
   "
   ```

## Authentication Issues

### Login Failures

**Symptoms:**
- "Invalid credentials" for correct passwords
- "User not found" errors
- Session token rejected

**Diagnosis:**
```bash
# Check user exists
psql $DATABASE_URL -c "SELECT username, email, is_active FROM users WHERE email = 'user@example.com';"

# Check password hash
psql $DATABASE_URL -c "SELECT username, length(password_hash) FROM users WHERE email = 'user@example.com';"

# Check session
psql $DATABASE_URL -c "SELECT token, expires_at, is_active FROM sessions WHERE token = 'your-token-here';"
```

**Solutions:**

1. **Create test user:**
   ```bash
   curl -X POST http://localhost:3000/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username":"test","email":"test@example.com","password":"password123"}'
   ```

2. **Reset user password:**
   ```sql
   -- Connect to database and reset password hash
   psql $DATABASE_URL -c "
     UPDATE users 
     SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$...' -- New hash
     WHERE email = 'user@example.com';
   "
   ```

3. **Clean up expired sessions:**
   ```sql
   psql $DATABASE_URL -c "DELETE FROM sessions WHERE expires_at < NOW();"
   ```

### Token Issues

**Symptoms:**
- "Unauthorized" errors with valid tokens
- Tokens expire immediately
- Session not found errors

**Solutions:**

1. **Check token format:**
   ```bash
   # Token should be 64 characters, alphanumeric
   echo $TOKEN | wc -c  # Should be 65 (64 + newline)
   ```

2. **Verify session in database:**
   ```sql
   psql $DATABASE_URL -c "
     SELECT s.token, s.expires_at, s.is_active, u.username
     FROM sessions s
     JOIN users u ON s.user_id = u.id
     WHERE s.token = 'your-token-here';
   "
   ```

3. **Check system time:**
   ```bash
   # Ensure server time is correct
   date
   psql $DATABASE_URL -c "SELECT NOW();"
   ```

## Development Environment Issues

### Scripts Not Working

**Symptoms:**
- Permission denied errors
- Scripts can't find binaries
- Environment variables not loaded

**Solutions:**

1. **Make scripts executable:**
   ```bash
   chmod +x scripts/*.sh
   ```

2. **Check PATH and environment:**
   ```bash
   which cargo
   which docker
   which psql
   echo $PATH
   ```

3. **Load environment manually:**
   ```bash
   source .env
   export $(cat .env | xargs)
   ```

### Port Conflicts

**Symptoms:**
- "Address already in use" errors
- Can't bind to port
- Services conflict

**Solutions:**

1. **Find what's using the port:**
   ```bash
   lsof -i :3000
   lsof -i :5432
   lsof -i :8080
   ```

2. **Kill processes on port:**
   ```bash
   # Kill specific process
   kill $(lsof -t -i:3000)
   
   # Or use the script
   ./scripts/stop-server.sh 3000
   ```

3. **Use different ports:**
   ```bash
   ./scripts/server.sh 8080
   STARTER__SERVER__PORT=8080 cargo run -- server
   ```

### Docker Issues

**Symptoms:**
- Docker commands fail
- Database container won't start
- Permission errors with volumes

**Solutions:**

1. **Check Docker is running:**
   ```bash
   docker ps
   docker compose version
   ```

2. **Reset Docker environment:**
   ```bash
   docker compose down -v
   docker system prune -f
   docker compose up -d postgres
   ```

3. **Fix permissions:**
   ```bash
   # On Linux, fix Docker socket permissions
   sudo usermod -aG docker $USER
   # Logout and login again
   ```

## Performance Issues

### Slow Database Queries

**Symptoms:**
- API requests timeout
- High database CPU usage
- Slow task processing

**Solutions:**

1. **Check database performance:**
   ```sql
   -- Check active queries
   SELECT query, state, query_start 
   FROM pg_stat_activity 
   WHERE state = 'active';
   
   -- Check slow queries
   SELECT query, mean_exec_time, calls 
   FROM pg_stat_statements 
   ORDER BY mean_exec_time DESC 
   LIMIT 10;
   ```

2. **Add missing indexes:**
   ```sql
   -- Check if indexes exist
   SELECT tablename, indexname 
   FROM pg_indexes 
   WHERE tablename IN ('users', 'sessions', 'tasks');
   ```

3. **Clean up old data:**
   ```sql
   -- Remove old sessions
   DELETE FROM sessions WHERE expires_at < NOW() - INTERVAL '7 days';
   
   -- Archive old completed tasks
   DELETE FROM tasks 
   WHERE status = 'completed' 
     AND completed_at < NOW() - INTERVAL '30 days';
   ```

### High Memory Usage

**Symptoms:**
- System becomes slow
- Out of memory errors
- Processes get killed

**Solutions:**

1. **Check memory usage:**
   ```bash
   free -h
   ps aux --sort=-%mem | head -20
   ```

2. **Reduce connection pools:**
   ```bash
   # Lower database connections
   STARTER__DATABASE__MAX_CONNECTIONS=5 ./scripts/server.sh 3000
   ```

3. **Limit worker concurrency:**
   ```bash
   STARTER__WORKER__CONCURRENCY=2 ./scripts/worker.sh
   ```

## Testing Issues

### Test Scripts Fail

**Symptoms:**
- Integration tests timeout
- Authentication tests fail
- Database tests conflict

**Solutions:**

1. **Clean test environment:**
   ```bash
   ./scripts/reset-all.sh
   ./scripts/test_tasks_integration.sh
   ```

2. **Run tests individually:**
   ```bash
   ./scripts/test_auth.sh
   ./scripts/test-server.sh 3000
   ```

3. **Check test dependencies:**
   ```bash
   # Ensure curl, jq, python3 are available
   which curl jq python3
   ```

### Cargo Test Failures

**Symptoms:**
- Unit tests fail
- Compilation errors in tests
- Database connection errors in tests

**Solutions:**

1. **Run specific tests:**
   ```bash
   cargo test auth::tests
   cargo test tasks::tests
   ```

2. **Use test database:**
   ```bash
   # Set test database URL
   export DATABASE_URL="postgres://starter_user:starter_pass@localhost:5432/starter_test_db"
   cargo test
   ```

3. **Run tests sequentially:**
   ```bash
   cargo test -- --test-threads=1
   ```

## Common Error Messages

### "Task type 'xyz' not registered"
**Cause:** Worker doesn't have handler for task type  
**Solution:** Check worker logs for handler registration, restart worker

### "Circuit breaker is open"
**Cause:** Too many failures triggered circuit breaker  
**Solution:** Wait for circuit breaker timeout, check external services

### "Database connection pool exhausted"
**Cause:** Too many concurrent database operations  
**Solution:** Increase pool size or reduce concurrency

### "Invalid payload for task type"
**Cause:** Task payload doesn't match expected schema  
**Solution:** Check payload format against handler requirements

### "Session expired or invalid"
**Cause:** Authentication token expired or malformed  
**Solution:** Login again to get new token, check token format

## Getting More Help

### Enable Debug Logging
```bash
# Server with debug logs
RUST_LOG=debug ./scripts/server.sh 3000

# Worker with debug logs  
RUST_LOG=debug ./scripts/worker.sh

# Specific module logging
RUST_LOG=starter::tasks=debug ./scripts/worker.sh
```

### Database Inspection
```bash
# Connect to database
psql $DATABASE_URL

# Useful queries
\dt                     # List tables
\d users                # Describe users table
SELECT * FROM tasks WHERE status = 'failed' LIMIT 5;
SELECT task_type, COUNT(*) FROM tasks GROUP BY task_type;
```

### Log Analysis
```bash
# Server logs
tail -f /tmp/starter-server-3000.log

# Worker logs
tail -f /tmp/starter-worker.log

# Search for errors
grep -i error /tmp/starter-*.log

# Search for specific task
grep "task-id-here" /tmp/starter-*.log
```

### System Information
```bash
# Check system resources
df -h                   # Disk space
free -h                 # Memory usage
ps aux | grep starter   # Process status
netstat -tlnp | grep :3000  # Network connections
```

## Prevention Tips

1. **Regular Cleanup:**
   ```bash
   # Add to cron
   0 2 * * * psql $DATABASE_URL -c "DELETE FROM sessions WHERE expires_at < NOW() - INTERVAL '7 days';"
   ```

2. **Monitor Resources:**
   ```bash
   # Check status regularly
   ./scripts/status.sh
   ```

3. **Test Changes:**
   ```bash
   # Always test after changes
   ./scripts/test_tasks_integration.sh
   ```

4. **Keep Logs Rotated:**
   ```bash
   # Scripts automatically rotate at 50MB
   ls -lh /tmp/starter-*.log
   ```

---
*When in doubt, try the "reset everything" approach: `./scripts/reset-all.sh` followed by `./scripts/test_tasks_integration.sh`*