# Chaos Testing Guide

This guide explains how to use the chaos testing framework to validate system resilience under various failure conditions.

## 🎯 Quick Start

```bash
# Basic chaos testing
./scripts/test-chaos.sh

# Advanced testing with higher difficulty
./scripts/test-chaos.sh --difficulty 3

# Test specific scenarios
./scripts/test-chaos.sh --scenarios "db-failure,task-flood"

# Verbose output with detailed logs
./scripts/test-chaos.sh --difficulty 5 --verbose
```

## 📊 Difficulty Levels

### Level 1 - Basic (Recommended for Development)
**Scenario:** Simple service restarts and basic failure recovery
- **Task Load:** 20 tasks with 0.5s delay
- **Chaos Duration:** 10 seconds
- **Use Case:** Daily development testing, CI/CD validation
- **Expected:** Quick recovery, minimal impact

```bash
./scripts/test-chaos.sh --difficulty 1
```

### Level 2 - Moderate (Integration Testing)
**Scenario:** Service interruptions with moderate load
- **Task Load:** 50 tasks with 0.2s delay  
- **Chaos Duration:** 20 seconds
- **Use Case:** Pre-production validation, staging environment
- **Expected:** Graceful degradation, circuit breaker activation

```bash
./scripts/test-chaos.sh --difficulty 2
```

### Level 3 - Advanced (Load Testing)
**Scenario:** High load with circuit breaker triggers
- **Task Load:** 100 tasks with 0.1s delay
- **Chaos Duration:** 30 seconds
- **Use Case:** Performance validation, stress testing
- **Expected:** System stability under load, proper queue management

```bash
./scripts/test-chaos.sh --difficulty 3
```

### Level 4 - Expert (Pre-Production)
**Scenario:** Multiple concurrent failures
- **Task Load:** 200 tasks with 0.05s delay
- **Chaos Duration:** 45 seconds
- **Use Case:** Production readiness validation
- **Expected:** Fault isolation, graceful recovery

```bash
./scripts/test-chaos.sh --difficulty 4
```

### Level 5 - Extreme (Production Validation)
**Scenario:** Sustained chaos with recovery testing
- **Task Load:** 500 tasks with 0.01s delay
- **Chaos Duration:** 60 seconds
- **Use Case:** Production chaos engineering, disaster recovery testing
- **Expected:** Full resilience, minimal data loss

```bash
./scripts/test-chaos.sh --difficulty 5
```

## 🧪 Available Scenarios

### `baseline`
**Purpose:** Validate normal system functionality
**Duration:** ~30 seconds
**What it tests:**
- All API endpoints respond correctly
- Authentication flow works
- Task creation and processing
- Dead letter queue functionality

```bash
./scripts/test-chaos.sh --scenarios baseline
```

### `db-failure`
**Purpose:** Test database connection failure resilience
**Duration:** ~60 seconds
**What it tests:**
- Database connection loss detection
- Health check accuracy during outage
- Service recovery after database restart
- Connection pool recovery

```bash
./scripts/test-chaos.sh --scenarios db-failure
```

### `server-restart`
**Purpose:** Test HTTP server restart resilience
**Duration:** ~45 seconds
**What it tests:**
- Graceful shutdown handling
- Process restart reliability
- Service discovery recovery
- State reconstruction

```bash
./scripts/test-chaos.sh --scenarios server-restart
```

### `worker-restart`
**Purpose:** Test background worker restart resilience
**Duration:** ~60 seconds
**What it tests:**
- Task processing interruption
- Worker restart reliability
- Task queue persistence
- Processing resumption

```bash
./scripts/test-chaos.sh --scenarios worker-restart
```

### `task-flood`
**Purpose:** Test high task load handling
**Duration:** ~90 seconds (varies by difficulty)
**What it tests:**
- System performance under load
- Queue management efficiency
- Memory and resource usage
- Task processing throughput

```bash
./scripts/test-chaos.sh --scenarios task-flood
```

### `circuit-breaker`
**Purpose:** Test circuit breaker activation and recovery
**Duration:** ~75 seconds
**What it tests:**
- Circuit breaker triggering on failures
- Fast failure during outage
- Recovery detection and reopening
- Fault isolation between task types

```bash
./scripts/test-chaos.sh --scenarios circuit-breaker
```

### `mixed-chaos`
**Purpose:** Test multiple simultaneous failures
**Duration:** ~120 seconds (varies by difficulty)
**What it tests:**
- Concurrent failure handling
- System stability under multiple stressors
- Recovery coordination
- Resource contention

```bash
./scripts/test-chaos.sh --scenarios mixed-chaos
```

### `recovery`
**Purpose:** Measure and validate recovery times
**Duration:** ~180 seconds
**What it tests:**
- Service restart time consistency
- Recovery time under load
- Time to first successful request
- Recovery time SLA validation

```bash
./scripts/test-chaos.sh --scenarios recovery
```

## 🛠️ Helper Scripts

The chaos testing framework includes modular helper scripts:

### `scripts/helpers/auth-helper.sh`
Creates test users and returns authentication tokens.

```bash
# Basic usage
./scripts/helpers/auth-helper.sh

# Custom configuration
./scripts/helpers/auth-helper.sh --prefix "loadtest" --url "http://localhost:8080"

# Output: {"token": "abc123...", "user_id": "uuid-here"}
```

### `scripts/helpers/task-flood.sh`
Creates floods of tasks for load testing.

```bash
# Basic task flood
./scripts/helpers/task-flood.sh --auth "$TOKEN" --count 100

# Failing tasks for circuit breaker testing
./scripts/helpers/task-flood.sh --auth "$TOKEN" --count 20 --fail

# Specific task types
./scripts/helpers/task-flood.sh --auth "$TOKEN" --type webhook --count 50
```

### `scripts/helpers/service-chaos.sh`
Simulates various service failures.

```bash
# Kill and restart server
./scripts/helpers/service-chaos.sh restart --service server --port 3000

# Stop database for 30 seconds
./scripts/helpers/service-chaos.sh db-restart --delay 30

# Kill worker process
./scripts/helpers/service-chaos.sh kill --service worker
```

## 📈 Progressive Testing Strategy

### Phase 1: Development Validation
```bash
# Daily developer testing
./scripts/test-chaos.sh --difficulty 1 --scenarios "baseline,db-failure"

# Should complete in ~2 minutes
# Expected: 100% pass rate
```

### Phase 2: Integration Testing
```bash
# Pre-commit testing
./scripts/test-chaos.sh --difficulty 2 --scenarios "baseline,server-restart,worker-restart"

# Should complete in ~5 minutes
# Expected: 100% pass rate with graceful degradation
```

### Phase 3: Load Testing
```bash
# Performance validation
./scripts/test-chaos.sh --difficulty 3 --scenarios "task-flood,circuit-breaker"

# Should complete in ~8 minutes
# Expected: 90%+ pass rate, circuit breaker activation
```

### Phase 4: Resilience Testing
```bash
# Pre-production validation
./scripts/test-chaos.sh --difficulty 4 --scenarios "mixed-chaos,recovery"

# Should complete in ~12 minutes
# Expected: 85%+ pass rate, recovery under 15s
```

### Phase 5: Production Readiness
```bash
# Full chaos testing
./scripts/test-chaos.sh --difficulty 5 --verbose

# Should complete in ~20 minutes
# Expected: 80%+ pass rate, comprehensive resilience
```

## 📊 Interpreting Results

### Success Criteria by Difficulty

| Level | Min Success Rate | Max Recovery Time | Max Task Failures |
|-------|------------------|-------------------|-------------------|
| 1     | 100%            | 10s               | 0%                |
| 2     | 95%             | 15s               | 5%                |
| 3     | 90%             | 20s               | 10%               |
| 4     | 85%             | 25s               | 15%               |
| 5     | 80%             | 30s               | 20%               |

### Key Metrics

1. **API Success Rate:** Percentage of API calls that succeed
2. **Recovery Time:** Time from failure to first successful API call
3. **Task Processing:** Tasks completed vs. tasks created
4. **Circuit Breaker:** Proper activation and recovery
5. **Database Resilience:** Recovery after connection loss

### Warning Signs

- 🚨 **Recovery time > 30s:** Infrastructure or configuration issues
- 🚨 **Success rate < 70%:** Fundamental reliability problems
- 🚨 **Memory leaks:** Resource usage grows during testing
- 🚨 **Database locks:** Tasks stuck in processing state
- 🚨 **Circuit breaker stuck:** Not recovering after service restoration

## 🔧 Customization

### Environment Variables

```bash
# Custom configuration
export PORT=8080
export BASE_URL="https://staging.example.com"
export OUTPUT_DIR="chaos-results"
export VERBOSE=true

./scripts/test-chaos.sh
```

### Custom Scenarios

Create your own scenario by modifying `test-chaos.sh`:

```bash
# Add to the case statement in run_scenario()
my-custom-test)
    log "INFO" "Running my custom test"
    
    # Your custom chaos logic here
    # Use helper scripts for common operations
    
    if run_api_test "Custom Test"; then
        log "SUCCESS" "Custom scenario passed"
        PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
        TEST_RESULTS+=("✅ my-custom-test: PASS")
    else
        log "ERROR" "Custom scenario failed"
        TEST_RESULTS+=("❌ my-custom-test: FAIL")
    fi
    ;;
```

### Task Type Testing

Test specific task types by customizing the flood helper:

```bash
# Test webhook resilience
./scripts/helpers/task-flood.sh --type webhook --count 100 --auth "$TOKEN"

# Test file cleanup under load
./scripts/helpers/task-flood.sh --type file_cleanup --count 50 --auth "$TOKEN"

# Test report generation circuit breaker
./scripts/helpers/task-flood.sh --type nonexistent_type --count 20 --auth "$TOKEN"
```

## 🚀 CI/CD Integration

### GitHub Actions Example

```yaml
name: Chaos Testing
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  chaos-test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Setup environment
      run: ./scripts/dev-server.sh 3000
    - name: Run chaos tests
      run: ./scripts/test-chaos.sh --difficulty 2
    - name: Upload results
      uses: actions/upload-artifact@v4
      with:
        name: chaos-test-results
        path: tmp/
```

### Pre-deployment Validation

```bash
#!/bin/bash
# deploy-validation.sh

echo "🧪 Running pre-deployment chaos testing..."

# Start services
./scripts/dev-server.sh 3000

# Run progressive testing
for level in 1 2 3; do
    echo "Testing difficulty level $level..."
    if ! ./scripts/test-chaos.sh --difficulty $level; then
        echo "❌ Chaos testing failed at level $level"
        exit 1
    fi
done

echo "✅ All chaos tests passed - ready for deployment"
```

## 📋 Troubleshooting

### Common Issues

**Test timeouts:**
- Increase timeout values in helper scripts
- Check system resources (CPU, memory)
- Verify network connectivity

**High failure rates:**
- Check service logs: `tail -f /tmp/starter-*.log`
- Verify database connectivity
- Review circuit breaker thresholds

**Worker not processing tasks:**
- Check worker process: `ps aux | grep starter`
- Review worker logs for errors
- Verify database task queue

**Database connection issues:**
- Check PostgreSQL container: `docker-compose ps`
- Verify connection string configuration
- Check connection pool settings

### Debug Mode

```bash
# Enable verbose logging
./scripts/test-chaos.sh --verbose --difficulty 1

# Check individual components
./scripts/status.sh
./scripts/test-server.sh 3000

# Manual testing
curl -X GET http://localhost:3000/health
curl -X GET http://localhost:3000/tasks/stats
```

## 🎯 Best Practices

1. **Start Small:** Begin with difficulty 1 and gradually increase
2. **Automate:** Integrate chaos testing into CI/CD pipelines
3. **Monitor:** Watch system metrics during testing
4. **Document:** Record failure modes and recovery patterns
5. **Iterate:** Improve resilience based on test results
6. **Schedule:** Run regular chaos tests in staging environments
7. **Alert:** Set up monitoring for chaos test failures

## 📚 Further Reading

- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Netflix Chaos Engineering](https://netflix.github.io/chaosmonkey/)
- [Rust Reliability Patterns](../docs/reliability.md)
- [Background Tasks Guide](../docs/guides/04-background-tasks.md)
- [Circuit Breaker Pattern](../docs/guides/03-patterns.md)

---

**Happy Chaos Testing! 🔥**

Remember: The goal isn't to break your system, but to understand how it breaks and improve its resilience.