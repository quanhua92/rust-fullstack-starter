#!/bin/bash

# Comprehensive Chaos Testing Script
# Tests system resilience under various failure scenarios

set -e

# ===== CONFIGURATION VARIABLES =====
# Default values  
PORT="${PORT:-8888}"
BASE_URL="http://localhost:$PORT"
DIFFICULTY="${DIFFICULTY:-1}"
SCENARIOS="${SCENARIOS:-all}"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp}"
VERBOSE="${VERBOSE:-false}"
RESET_DATABASE="${RESET_DATABASE:-false}"
NO_CLEANUP="${NO_CLEANUP:-false}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
BOLD='\033[1m'
BOLD_RED='\033[1;31m'
BOLD_YELLOW='\033[1;33m'
NC='\033[0m'

# Path configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HELPERS_DIR="$SCRIPT_DIR/helpers"
CHAOS_COMPOSE_FILE="$PROJECT_ROOT/docker-compose.chaos.yaml"

# Process/monitoring variables
STATUS_MONITOR_PID=""
TASK_FLOOD_PID=""
CHAOS_PID=""
MONITOR_PID=""
PHASE1_PID=""
PHASE2_PID=""

# Test results tracking
TEST_RESULTS=()
TOTAL_SCENARIOS=0
PASSED_SCENARIOS=0
START_TIME=$(date +%s)
END_TIME=""
TOTAL_DURATION=""

# Container names for chaos testing
MAIN_CONTAINER_NAME="chaos-starter-server"
WORKER_CONTAINER_NAME=""  # Will be dynamically determined
POSTGRES_CONTAINER_NAME="chaos-starter-postgres"

# Database configuration for chaos testing
DB_SERVICE_NAME="postgres"  # Service name in docker-compose.chaos.yaml
DB_USER="starter_user"
DB_NAME="starter_db"
DB_TABLES_TO_TRUNCATE=("sessions" "tasks" "task_types" "users")
DB_SEQUENCES_TO_RESET=("users_id_seq" "sessions_id_seq")

# Health check variables
health_attempts=20
health_attempt=0
api_attempts=15
api_attempt=0
task_type_attempts=10
task_type_attempt=0

# Scenario-specific variables
server_health=""
worker_health=""
postgres_health=""
admin_stats=""
auth_result=""
auth_token=""
token=""
monitor_result=""

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Comprehensive Docker-based chaos testing for the Rust starter application"
    echo "Note: This script automatically builds Docker images with the latest code before testing."
    echo ""
    echo "Options:"
    echo "  -p, --port PORT        Server port (default: $PORT)"
    echo "  -d, --difficulty LEVEL Difficulty level 1-6 (default: $DIFFICULTY)"
    echo "  -s, --scenarios LIST   Scenarios to run (default: all)"
    echo "  -o, --output DIR       Output directory (default: /tmp)"
    echo "  -r, --reset-database   Reset database before testing (clean slate)"
    echo "  -n, --no-cleanup       Keep containers running after test (for debugging)"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Difficulty Levels (Redesigned):"
    echo "  1 - Basic Resilience: Baseline functionality (2 workers, 10 tasks, ≥90% completion)"
    echo "  2 - Light Disruption: Introduction of failures (2 workers, 15 tasks, ≥85% completion)" 
    echo "  3 - Load Testing: Increased task volume (3 workers, 25 tasks, ≥80% completion)"
    echo "  4 - Resource Pressure: Challenging workload (3 workers, 35 tasks, ≥75% completion)"
    echo "  5 - Extreme Chaos: High-pressure scenarios (4 workers, 30 tasks, ≥60% completion)"
    echo "  6 - Catastrophic Load: Stress test limits (2 workers, 40 tasks, 20-50% completion)"
    echo ""
    echo "Available Scenarios:"
    echo "  baseline         - Baseline functionality test"
    echo "  db-failure       - Database connection failures"
    echo "  server-restart   - Server process restarts"
    echo "  worker-restart   - Worker process restarts"
    echo "  task-flood       - High task load testing"
    echo "  circuit-breaker  - Circuit breaker activation"
    echo "  mixed-chaos      - Multiple simultaneous failures"
    echo "  recovery         - Recovery time testing"
    echo "  multi-worker-chaos - Multiple workers with random failures and delay tasks"
    echo "  dynamic-scaling  - Dynamic worker scaling with 4 phases: optimal→reduced→gradual scale-up→completion"
    echo "  all              - Run all scenarios (default)"
    echo ""
    echo "Examples:"
    echo "  $0                                     # Basic chaos testing"
    echo "  $0 --difficulty 3 --port 8080         # Advanced testing on port 8080"
    echo "  $0 --scenarios \"db-failure,task-flood\" # Specific scenarios only"
    echo "  $0 --difficulty 5 --verbose           # Extreme testing with logs"
    echo "  $0 --reset-database --scenarios baseline # Clean database baseline test"
    echo "  $0 --scenarios baseline --no-cleanup    # Keep containers for debugging"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            BASE_URL="http://localhost:$PORT"
            shift 2
            ;;
        -d|--difficulty)
            DIFFICULTY="$2"
            shift 2
            ;;
        -s|--scenarios)
            SCENARIOS="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -r|--reset-database)
            RESET_DATABASE=true
            shift
            ;;
        -n|--no-cleanup)
            NO_CLEANUP=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

# Validate difficulty level
if [[ ! "$DIFFICULTY" =~ ^[1-6]$ ]]; then
    echo "Error: Difficulty must be 1-6" >&2
    exit 1
fi

# Project paths already defined at top of file

# Check if Docker compose file exists
if [ ! -f "$CHAOS_COMPOSE_FILE" ]; then
    echo "❌ Docker compose file not found: $CHAOS_COMPOSE_FILE" >&2
    echo "   Please ensure docker-compose.chaos.yaml exists" >&2
    exit 1
fi

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Test results tracking variables already defined at top of file

log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        INFO) echo -e "${BLUE}[$timestamp] INFO:${NC} $message" ;;
        WARN) echo -e "${YELLOW}[$timestamp] WARN:${NC} $message" ;;
        ERROR) echo -e "${RED}[$timestamp] ERROR:${NC} $message" ;;
        SUCCESS) echo -e "${GREEN}[$timestamp] SUCCESS:${NC} $message" ;;
        CRITICAL) echo -e "${BOLD_RED}[$timestamp] CRITICAL:${NC} $message" ;;
        BOLD_WARN) echo -e "${BOLD_YELLOW}[$timestamp] WARNING:${NC} $message" ;;
        *) echo "[$timestamp] $level: $message" ;;
    esac
    
    if [ "$VERBOSE" = true ]; then
        echo "[$timestamp] $level: $message" >> "$OUTPUT_DIR/chaos-test.log"
    fi
}

# Critical error handler with cleanup and exit
critical_error() {
    local message="$1"
    local cleanup="${2:-true}"
    
    echo ""
    log "CRITICAL" "🚨 FATAL ERROR: $message"
    log "CRITICAL" "🛑 Chaos testing cannot continue - terminating immediately"
    
    if [ "$cleanup" = "true" ]; then
        log "INFO" "🧹 Cleaning up Docker environment before exit..."
        cd "$PROJECT_ROOT"
        docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans 2>/dev/null || true
    fi
    
    echo ""
    echo -e "${BOLD_RED}❌ CHAOS TESTING FAILED${NC}"
    echo -e "${BOLD_RED}   Reason: $message${NC}"
    echo ""
    exit 1
}

# Check for authentication/authorization errors
check_auth_response() {
    local response="$1"
    local context="$2"
    
    if echo "$response" | grep -qi "unauthorized\|forbidden\|401\|403"; then
        critical_error "Authentication/Authorization failed in $context - check API credentials and permissions"
    fi
    
    if echo "$response" | grep -qi "internal server error\|500\|502\|503\|504"; then
        log "BOLD_WARN" "⚠️  Server error detected in $context: $response"
        return 1
    fi
    
    return 0
}

# Docker-aware status check
check_docker_status() {
    local output_file="$1"
    
    echo "=== $(date '+%Y-%m-%d %H:%M:%S') - Docker Chaos Status ===" >> "$output_file"
    echo "🐳 Docker Containers:" >> "$output_file"
    
    cd "$PROJECT_ROOT"
    if docker-compose -f "$CHAOS_COMPOSE_FILE" ps >> "$output_file" 2>&1; then
        echo "" >> "$output_file"
        echo "📊 Container Resource Usage:" >> "$output_file"
        local container_ids=$(docker-compose -f "$CHAOS_COMPOSE_FILE" ps -q 2>/dev/null)
        if [ -n "$container_ids" ]; then
            docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}" $container_ids >> "$output_file" 2>&1 || true
        fi
    else
        echo "   No chaos containers running" >> "$output_file"
    fi
    
    echo "" >> "$output_file"
    echo "🔌 Port Status:" >> "$output_file"
    for port in 3000 8080; do
        if lsof -ti:$port > /dev/null 2>&1; then
            local process=$(lsof -i:$port 2>/dev/null | grep LISTEN | awk '{print $1}' | head -1)
            echo "   Port $port: $process" >> "$output_file"
        fi
    done
    
    echo "" >> "$output_file"
}

# Start background status monitoring (Docker-aware)
start_status_monitor() {
    local scenario="$1"
    local output_file="$OUTPUT_DIR/status-monitor-$scenario.log"
    
    {
        while true; do
            check_docker_status "$output_file"
            sleep 10
        done
    } &
    
    STATUS_MONITOR_PID=$!
    echo "$STATUS_MONITOR_PID" > "$OUTPUT_DIR/status-monitor.pid"
    
    if [ "$VERBOSE" = true ]; then
        log "INFO" "Started Docker status monitor (PID: $STATUS_MONITOR_PID) -> $output_file"
    fi
}

# Stop background status monitoring
stop_status_monitor() {
    if [ -f "$OUTPUT_DIR/status-monitor.pid" ]; then
        local monitor_pid=$(cat "$OUTPUT_DIR/status-monitor.pid" 2>/dev/null)
        if [ -n "$monitor_pid" ] && kill -0 "$monitor_pid" 2>/dev/null; then
            kill "$monitor_pid" 2>/dev/null || true
            if [ "$VERBOSE" = true ]; then
                log "INFO" "Stopped status monitor (PID: $monitor_pid)"
            fi
        fi
        rm -f "$OUTPUT_DIR/status-monitor.pid"
    fi
}

run_api_test() {
    local test_name="$1"
    local expect_success="${2:-true}"
    
    log "INFO" "Running API test: $test_name"
    
    local result_file="$OUTPUT_DIR/api-test-$(echo "$test_name" | tr ' ' '-' | tr '[:upper:]' '[:lower:]').txt"
    
    if timeout 60 "$PROJECT_ROOT/scripts/test-with-curl.sh" localhost "$PORT" > "$result_file" 2>&1; then
        local success_rate=$(grep "Success rate:" "$result_file" | awk '{print $3}' | tr -d '%')
        if [ -n "$success_rate" ] && [ "$success_rate" -ge 80 ]; then
            log "SUCCESS" "API test passed: $test_name (Success rate: $success_rate%)"
            return 0
        else
            log "WARN" "API test degraded: $test_name (Success rate: $success_rate%)"
            return 1
        fi
    else
        if [ "$expect_success" = false ]; then
            log "SUCCESS" "API test failed as expected: $test_name"
            return 0
        else
            log "ERROR" "API test failed: $test_name"
            return 1
        fi
    fi
}

get_difficulty_params() {
    local level="$1"
    
    case "$level" in
        1) echo "task_count=20 delay=0.5 chaos_duration=10" ;;
        2) echo "task_count=50 delay=0.2 chaos_duration=20" ;;
        3) echo "task_count=100 delay=0.1 chaos_duration=30" ;;
        4) echo "task_count=200 delay=0.05 chaos_duration=45" ;;
        5) echo "task_count=500 delay=0.01 chaos_duration=60" ;;
        6) echo "task_count=1000 delay=0.005 chaos_duration=90" ;;
    esac
}

run_scenario() {
    local scenario="$1"
    
    TOTAL_SCENARIOS=$((TOTAL_SCENARIOS + 1))
    
    log "INFO" "=========================================="
    log "INFO" "Starting scenario: $scenario (Difficulty: $DIFFICULTY)"
    log "INFO" "=========================================="
    
    # Fail fast: Check total runtime and skip remaining scenarios if taking too long
    local current_time=$(date +%s)
    local total_runtime=$((current_time - START_TIME))
    local max_runtime=1800  # 30 minutes max for all scenarios
    
    if [ $total_runtime -gt $max_runtime ]; then
        log "ERROR" "⚡ FAIL FAST: Total runtime exceeded ${max_runtime}s - skipping remaining scenarios"
        TEST_RESULTS+=("❌ $scenario: SKIP (Timeout)")
        continue
    fi
    
    # Start status monitoring for this scenario
    start_status_monitor "$scenario"
    
    local scenario_start=$(date +%s)
    local params=$(get_difficulty_params "$DIFFICULTY")
    eval "$params"
    
    case "$scenario" in
        baseline)
            log "INFO" "Running comprehensive baseline test with all task types and 100% success validation"
            
            # First run basic API test
            if ! run_api_test "Baseline Test"; then
                log "ERROR" "Baseline API test failed"
                TEST_RESULTS+=("❌ baseline: FAIL (API)")
                continue
            fi
            
            # Create authentication token using proven auth helper
            log "INFO" "Creating authentication for comprehensive baseline testing"
            local auth_result
            auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "baseline_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -z "$token" ]; then
                log "ERROR" "Failed to get authentication token for baseline test"
                TEST_RESULTS+=("❌ baseline: FAIL (Auth)")
                continue
            fi
            
            # Use proven delay-task-flood.sh for reliable task creation and monitoring
            local total_tasks=12
            local task_delay=0.5  # Fast tasks for quick baseline test
            local deadline=30     # 30 seconds should be plenty for 12 x 0.5s tasks
            
            log "INFO" "Creating baseline test suite with $total_tasks delay tasks using proven flood script"
            
            # Run the proven delay-task-flood script with timeout
            if timeout 60 "$HELPERS_DIR/delay-task-flood.sh" \
                --count "$total_tasks" \
                --delay "$task_delay" \
                --deadline "$deadline" \
                --auth "$token" \
                --tag "baseline" \
                --verbose; then
                
                log "INFO" "Baseline task flood completed successfully"
                
                # Get final stats using the fixed admin CLI parsing
                local admin_stats
                admin_stats=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats --tag "baseline" 2>&1)
                
                # Parse with the fixed ANSI code removal
                local clean_output=$(echo "$admin_stats" | sed 's/\x1b\[[0-9;]*m//g')
                local completed_count=$(echo "$clean_output" | grep -E "^\s*completed:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
                local failed_count=$(echo "$clean_output" | grep -E "^\s*failed:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0") 
                local total_count=$(echo "$clean_output" | grep -E "^\s*Total:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
                
                # Clean up numbers
                completed_count=$(echo "$completed_count" | tr -cd '0-9' || echo "0")
                failed_count=$(echo "$failed_count" | tr -cd '0-9' || echo "0")
                total_count=$(echo "$total_count" | tr -cd '0-9' || echo "0")
                
                # Use total_count if available, otherwise use completed + failed
                local final_total=${total_count:-$((completed_count + failed_count))}
                
            else
                log "ERROR" "Baseline delay-task-flood script failed or timed out"
                TEST_RESULTS+=("❌ baseline: FAIL (Task Flood Timeout)")
                continue
            fi
            
            local success_rate=0
            if [ $completed_count -gt 0 ] && [ $total_tasks -gt 0 ]; then
                success_rate=$(echo "scale=1; ($completed_count * 100) / $total_tasks" | bc -l 2>/dev/null || echo "0")
            fi
            
            log "INFO" "Task completion results: $completed_count completed, $failed_count failed"
            log "INFO" "Success rate: ${success_rate}% ($completed_count/$total_tasks tasks)"
            
            # Validate 100% success rate
            if [ "$completed_count" -eq "$total_tasks" ] && [ "$failed_count" -eq 0 ]; then
                log "SUCCESS" "🎯 BASELINE PERFECT: 100% success rate achieved!"
                log "SUCCESS" "✅ All $total_tasks delay tasks completed successfully"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ baseline: PASS (100%: $completed_count/$total_tasks)")
            elif [ "$completed_count" -gt 0 ] && [ "$(echo "$success_rate >= 95" | bc -l)" -eq 1 ]; then
                log "SUCCESS" "Baseline very good: ${success_rate}% success rate"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ baseline: PASS (${success_rate}%: $completed_count/$total_tasks)")
            else
                log "ERROR" "Baseline failed: Only ${success_rate}% success rate ($completed_count/$total_tasks)"
                log "ERROR" "Failed tasks: $failed_count"
                TEST_RESULTS+=("❌ baseline: FAIL (${success_rate}%: $completed_count/$total_tasks)")
            fi
            ;;
            
        db-failure)
            log "INFO" "Testing database failure resilience"
            
            # Stop database
            "$HELPERS_DIR/service-chaos.sh" db-stop
            sleep 2
            
            # Test API during failure
            run_api_test "DB Failure Test" false
            
            # Restart database
            "$HELPERS_DIR/service-chaos.sh" db-restart --delay 5
            
            # Test recovery
            if run_api_test "DB Recovery Test"; then
                log "SUCCESS" "Database failure scenario passed"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ db-failure: PASS")
            else
                log "ERROR" "Database failure scenario failed"
                TEST_RESULTS+=("❌ db-failure: FAIL")
            fi
            ;;
            
        server-restart)
            log "INFO" "Testing server restart resilience"
            
            # Kill and restart server
            "$HELPERS_DIR/service-chaos.sh" restart --service server --port "$PORT" --delay "$chaos_duration"
            
            # Test recovery
            if run_api_test "Server Restart Test"; then
                log "SUCCESS" "Server restart scenario passed"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ server-restart: PASS")
            else
                log "ERROR" "Server restart scenario failed"
                TEST_RESULTS+=("❌ server-restart: FAIL")
            fi
            ;;
            
        worker-restart)
            log "INFO" "Testing worker restart resilience"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "worker_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -n "$token" ]; then
                # Create some delay tasks before killing worker
                "$HELPERS_DIR/delay-task-flood.sh" --count 10 --delay 0.1 --deadline 15 --auth "$token" --tag "db_test"
                
                # Kill and restart worker
                "$HELPERS_DIR/service-chaos.sh" restart --service worker --delay "$chaos_duration"
                
                # Test that API still works
                if run_api_test "Worker Restart Test"; then
                    log "SUCCESS" "Worker restart scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ worker-restart: PASS")
                else
                    log "ERROR" "Worker restart scenario failed"
                    TEST_RESULTS+=("❌ worker-restart: FAIL")
                fi
            else
                log "ERROR" "Failed to get auth token for worker restart test"
                TEST_RESULTS+=("❌ worker-restart: FAIL")
            fi
            ;;
            
        task-flood)
            log "INFO" "Testing high task load (Count: $task_count, Delay: ${delay}s)"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "flood_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -z "$token" ]; then
                log "ERROR" "⚡ FAIL FAST: No auth token - skipping task flood test"
                TEST_RESULTS+=("❌ task-flood: FAIL (Auth)")
                continue
            fi
            
            # Quick API health check before flooding
            if ! curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
                log "ERROR" "⚡ FAIL FAST: API unhealthy before task flood - skipping"
                TEST_RESULTS+=("❌ task-flood: FAIL (API)")
                continue
            fi
            
            # Create delay task flood with timeout (faster execution)
            if timeout 60 "$HELPERS_DIR/delay-task-flood.sh" --count "$task_count" --delay 0.5 --deadline 45 --auth "$token" --tag "flood_test" --verbose; then
                # Test system stability under load
                if run_api_test "Task Flood Test"; then
                    log "SUCCESS" "Task flood scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ task-flood: PASS")
                else
                    log "ERROR" "Task flood scenario failed"
                    TEST_RESULTS+=("❌ task-flood: FAIL")
                fi
            else
                log "ERROR" "⚡ FAIL FAST: Task flood helper timed out or failed"
                TEST_RESULTS+=("❌ task-flood: FAIL (Timeout)")
            fi
            ;;
            
        circuit-breaker)
            log "INFO" "Testing circuit breaker activation"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "cb_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -z "$token" ]; then
                log "ERROR" "⚡ FAIL FAST: No auth token - skipping circuit breaker test"
                TEST_RESULTS+=("❌ circuit-breaker: FAIL (Auth)")
                continue
            fi
            
            # Quick API health check
            if ! curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
                log "ERROR" "⚡ FAIL FAST: API unhealthy - skipping circuit breaker test"
                TEST_RESULTS+=("❌ circuit-breaker: FAIL (API)")
                continue
            fi
            
            # Create failing delay tasks to trigger circuit breaker with timeout  
            if timeout 30 "$HELPERS_DIR/delay-task-flood.sh" --count 20 --delay 0.1 --deadline 20 --auth "$token" --tag "cb_test" --verbose; then
                # Test system stability with circuit breaker
                if run_api_test "Circuit Breaker Test"; then
                    log "SUCCESS" "Circuit breaker scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ circuit-breaker: PASS")
                else
                    log "ERROR" "Circuit breaker scenario failed"
                    TEST_RESULTS+=("❌ circuit-breaker: FAIL")
                fi
            else
                log "ERROR" "⚡ FAIL FAST: Circuit breaker setup timed out"
                TEST_RESULTS+=("❌ circuit-breaker: FAIL (Timeout)")
            fi
            ;;
            
        mixed-chaos)
            log "INFO" "Testing multiple simultaneous failures"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "mixed_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -n "$token" ]; then
                # Start delay task flood in background
                "$HELPERS_DIR/delay-task-flood.sh" --count "$task_count" --delay "$delay" --deadline 60 --auth "$token" --tag "mixed_test" &
                FLOOD_PID=$!
                
                sleep 5
                
                # Kill worker during flood
                "$HELPERS_DIR/service-chaos.sh" kill --service worker
                
                sleep "$chaos_duration"
                
                # Restart worker
                "$HELPERS_DIR/service-chaos.sh" restart --service worker
                
                # Wait for flood to complete
                wait $FLOOD_PID || true
                
                # Test recovery
                if run_api_test "Mixed Chaos Test"; then
                    log "SUCCESS" "Mixed chaos scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ mixed-chaos: PASS")
                else
                    log "ERROR" "Mixed chaos scenario failed"
                    TEST_RESULTS+=("❌ mixed-chaos: FAIL")
                fi
            else
                log "ERROR" "Failed to get auth token for mixed chaos test"
                TEST_RESULTS+=("❌ mixed-chaos: FAIL")
            fi
            ;;
            
        recovery)
            log "INFO" "Testing system recovery times"
            
            local recovery_times=()
            
            for i in {1..3}; do
                log "INFO" "Recovery test iteration $i/3"
                
                # Kill server and measure recovery time
                local kill_time=$(date +%s)
                "$HELPERS_DIR/service-chaos.sh" kill --service server --port "$PORT"
                
                # Start server
                "$HELPERS_DIR/service-chaos.sh" restart --service server --port "$PORT"
                
                # Measure time to first successful API call
                local recovered=false
                local timeout_count=0
                while [ $timeout_count -lt 30 ]; do
                    if curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
                        local recovery_time=$(($(date +%s) - kill_time))
                        recovery_times+=("$recovery_time")
                        log "SUCCESS" "Recovery time iteration $i: ${recovery_time}s"
                        recovered=true
                        break
                    fi
                    sleep 1
                    timeout_count=$((timeout_count + 1))
                done
                
                if [ "$recovered" = false ]; then
                    log "ERROR" "Recovery test iteration $i failed (timeout)"
                    recovery_times+=("30")
                fi
            done
            
            # Calculate average recovery time
            local total=0
            for time in "${recovery_times[@]}"; do
                total=$((total + time))
            done
            local avg_recovery=$((total / ${#recovery_times[@]}))
            
            log "INFO" "Average recovery time: ${avg_recovery}s"
            
            # Docker containers take longer to restart than binary processes
            # Adjusted threshold to 20s for Docker-based deployments
            if [ "$avg_recovery" -le 20 ]; then
                log "SUCCESS" "Recovery scenario passed"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ recovery: PASS (${avg_recovery}s avg)")
            else
                log "ERROR" "Recovery scenario failed"
                TEST_RESULTS+=("❌ recovery: FAIL (${avg_recovery}s avg)")
            fi
            ;;
            
        multi-worker-chaos)
            log "INFO" "Testing multi-worker chaos with delay tasks and deadlines"
            
            # Calculate scenario parameters based on difficulty
            local worker_count=3
            local delay_per_task=5
            local task_deadline=60
            local min_stop_interval=10
            local max_stop_interval=20
            
            # Redesigned levels with logical progression
            case "$DIFFICULTY" in
                1) # Basic Resilience - Baseline functionality
                   worker_count=2; task_count=10; delay_per_task=2; task_deadline=30; chaos_duration=8
                   min_stop_interval=20; max_stop_interval=30  # Minimal disruption
                   ;;
                2) # Light Disruption - Introduction of failures  
                   worker_count=2; task_count=15; delay_per_task=3; task_deadline=45; chaos_duration=12
                   min_stop_interval=15; max_stop_interval=25  # Moderate disruption
                   ;;
                3) # Load Testing - Increased task volume
                   worker_count=3; task_count=25; delay_per_task=3; task_deadline=60; chaos_duration=15
                   min_stop_interval=10; max_stop_interval=15  # Regular failures
                   ;;
                4) # Resource Pressure - Challenging workload
                   worker_count=3; task_count=35; delay_per_task=4; task_deadline=90; chaos_duration=20
                   min_stop_interval=5; max_stop_interval=10   # Aggressive cycling
                   ;;
                5) # Extreme Chaos - High-pressure scenarios
                   worker_count=4; task_count=30; delay_per_task=5; task_deadline=80; chaos_duration=25
                   min_stop_interval=3; max_stop_interval=7    # Continuous failures
                   ;;
                6) # Catastrophic Load - Stress test limits
                   worker_count=2; task_count=40; delay_per_task=6; task_deadline=60; chaos_duration=30
                   min_stop_interval=2; max_stop_interval=5    # Constant failures + impossible load
                   ;;
            esac
            
            if [ "$DIFFICULTY" -eq 6 ]; then
                log "WARN" "⚠️  CATASTROPHIC MODE: Overwhelming workload designed for partial completion!"
                log "INFO" "Level 6 creates impossible workload: $task_count tasks × ${delay_per_task}s = $((task_count * delay_per_task))s needed, only ${task_deadline}s allowed"
                log "INFO" "Theoretical capacity: $worker_count workers can handle ~$((task_deadline * worker_count / delay_per_task)) tasks in ${task_deadline}s"
            fi
            
            log "INFO" "Multi-worker scenario: $worker_count workers, ${delay_per_task}s delays, ${task_deadline}s deadline"
            
            # Create auth token for tasks
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "multiworker_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -z "$token" ]; then
                log "ERROR" "Failed to get auth token for multi-worker chaos test"
                TEST_RESULTS+=("❌ multi-worker-chaos: FAIL (auth)")
                return
            fi
            
            # Clean up any existing workers
            "$HELPERS_DIR/multi-worker-chaos.sh" cleanup > /dev/null 2>&1 || true
            sleep 2
            
            # Start multiple workers
            log "INFO" "Starting $worker_count workers for chaos scenario..."
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers "$worker_count" --verbose
            
            # Wait for workers to be ready
            sleep 5
            
            # Create delay tasks that will test the worker resilience
            log "INFO" "Creating delay tasks (count: $task_count, delay: ${delay_per_task}s each)"
            "$HELPERS_DIR/delay-task-flood.sh" \
                --count "$task_count" \
                --delay "$delay_per_task" \
                --deadline "$task_deadline" \
                --auth "$token" \
                --tag "multiworker" \
                --verbose &
            TASK_FLOOD_PID=$!
            
            # Give tasks a moment to be created
            sleep 3
            
            # Start worker chaos in background
            log "INFO" "Starting worker chaos (duration: ${chaos_duration}s, stop intervals: ${min_stop_interval}-${max_stop_interval}s)"
            timeout "$chaos_duration" "$HELPERS_DIR/multi-worker-chaos.sh" chaos-run \
                --workers "$worker_count" \
                --duration "$chaos_duration" \
                --min-stop "$min_stop_interval" \
                --max-stop "$max_stop_interval" \
                --restart-delay 3 \
                --verbose > /dev/null 2>&1 &
            CHAOS_PID=$!
            
            # Monitor task completion using admin CLI
            log "INFO" "Monitoring task completion with deadline enforcement..."
            "$HELPERS_DIR/task-completion-monitor.sh" \
                --container "$MAIN_CONTAINER_NAME" \
                --tag "multiworker" \
                --deadline "$task_deadline" \
                --verbose > "$OUTPUT_DIR/multi-worker-monitor.log" 2>&1 &
            MONITOR_PID=$!
            
            # Wait for task flood to complete
            wait $TASK_FLOOD_PID || true
            
            # Wait for chaos scenario to complete
            wait $CHAOS_PID || true
            
            # Wait for monitoring to complete
            wait $MONITOR_PID || true
            
            # Get final results from monitor log
            local monitor_result=""
            if [ -f "$OUTPUT_DIR/multi-worker-monitor.log" ]; then
                monitor_result=$(tail -1 "$OUTPUT_DIR/multi-worker-monitor.log" | grep -o '{.*}' || echo "{}")
            fi
            
            # Parse results
            local completed_tasks=0
            local total_tasks=0
            local deadline_met=false
            local retry_attempts=0
            
            if [ -n "$monitor_result" ] && [ "$monitor_result" != "{}" ]; then
                completed_tasks=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))" 2>/dev/null || echo "0")
                total_tasks=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))" 2>/dev/null || echo "0")
                deadline_met=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('deadline_met', False))" 2>/dev/null || echo "false")
                retry_attempts=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('retry_attempts', 0))" 2>/dev/null || echo "0")
            fi
            
            # Clean up workers
            "$HELPERS_DIR/multi-worker-chaos.sh" stop-all > /dev/null 2>&1 || true
            
            # Evaluate results
            local success_rate=0
            if [ "$total_tasks" -gt 0 ]; then
                success_rate=$(echo "scale=1; ($completed_tasks * 100) / $total_tasks" | bc -l 2>/dev/null || echo "0")
            fi
            
            # Pass criteria depend on difficulty level - redesigned for logical progression
            case "$DIFFICULTY" in
                1) # Level 1: Basic Resilience - ≥90% completion, deadline met
                   local min_success=$(echo "$success_rate >= 90" | bc -l 2>/dev/null || echo "0")
                   local level_name="Basic Resilience"
                   local require_deadline=true
                   ;;
                2) # Level 2: Light Disruption - ≥85% completion, deadline met  
                   local min_success=$(echo "$success_rate >= 85" | bc -l 2>/dev/null || echo "0")
                   local level_name="Light Disruption"
                   local require_deadline=true
                   ;;
                3) # Level 3: Load Testing - ≥80% completion, deadline met
                   local min_success=$(echo "$success_rate >= 80" | bc -l 2>/dev/null || echo "0")
                   local level_name="Load Testing" 
                   local require_deadline=true
                   ;;
                4) # Level 4: Resource Pressure - ≥75% completion, deadline met
                   local min_success=$(echo "$success_rate >= 75" | bc -l 2>/dev/null || echo "0")
                   local level_name="Resource Pressure"
                   local require_deadline=true
                   ;;
                5) # Level 5: Extreme Chaos - ≥60% completion (deadline may be missed)
                   local min_success=$(echo "$success_rate >= 60" | bc -l 2>/dev/null || echo "0")
                   local level_name="Extreme Chaos"
                   local require_deadline=false
                   ;;
                6) # Level 6: Catastrophic Load - 20-50% completion (impossible workload)
                   local min_success=$(echo "$success_rate >= 20" | bc -l 2>/dev/null || echo "0")
                   local max_success=$(echo "$success_rate <= 50" | bc -l 2>/dev/null || echo "0")
                   local level_name="Catastrophic Load"
                   local require_deadline=false
                   ;;
            esac
            
            log "INFO" "Level $DIFFICULTY ($level_name): Evaluating results..."
            log "INFO" "Multi-worker chaos results: $completed_tasks/$total_tasks tasks completed (${success_rate}%)"
            log "INFO" "Retry attempts: $retry_attempts, Deadline met: $deadline_met"
            
            if [ "$DIFFICULTY" -eq 6 ]; then
                # Special logic for Level 6 - partial completion range
                if [ "$min_success" -eq 1 ] && [ "$max_success" -eq 1 ]; then
                    log "SUCCESS" "$level_name scenario passed - partial completion within expected range"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ multi-worker-chaos: PASS (${success_rate}%, $retry_attempts retries)")
                else
                    log "ERROR" "$level_name scenario failed - expected 20-50% completion, got ${success_rate}%"
                    TEST_RESULTS+=("❌ multi-worker-chaos: FAIL (${success_rate}%, $retry_attempts retries)")
                fi
            else
                # Standard logic for Levels 1-5
                local deadline_ok=0
                if [ "$require_deadline" = "true" ]; then
                    if [ "$deadline_met" = "true" ] || [ "$deadline_met" = "True" ]; then
                        deadline_ok=1
                    fi
                else
                    deadline_ok=1  # Don't require deadline for levels that allow it to be missed
                fi
                
                if [ "$min_success" -eq 1 ] && [ "$deadline_ok" -eq 1 ]; then
                    log "SUCCESS" "$level_name scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ multi-worker-chaos: PASS (${success_rate}%, $retry_attempts retries)")
                else
                    if [ "$min_success" -eq 0 ]; then
                        log "ERROR" "$level_name scenario failed - insufficient completion rate (${success_rate}%)"
                    fi
                    if [ "$deadline_ok" -eq 0 ]; then
                        log "ERROR" "$level_name scenario failed - deadline not met"
                    fi
                    TEST_RESULTS+=("❌ multi-worker-chaos: FAIL (${success_rate}%, $retry_attempts retries)")
                fi
            fi
            
            # Final API test to ensure system is still healthy
            run_api_test "Multi-Worker Post-Chaos Test" || true
            ;;
            
        dynamic-scaling)
            log "INFO" "Testing dynamic worker scaling with 4 phases: optimal→reduced→gradual scale-up→completion"
            
            # Calculate scenario parameters based on difficulty - OPTIMIZED for speed
            local initial_workers=5
            local reduced_workers=2
            local task_delay=0.5  # Much faster task execution
            local phase1_tasks=20  # Reduced task count for faster testing
            local phase2_tasks=10  # Smaller workload
            local total_deadline=60  # 1 minute total - much faster
            
            # Adjust parameters based on difficulty level - all optimized for speed
            case "$DIFFICULTY" in
                1) # Basic - Fast testing
                   phase1_tasks=15; phase2_tasks=8; task_delay=0.5; total_deadline=60
                   ;;
                2) # Light - Slightly more load but still fast
                   phase1_tasks=20; phase2_tasks=10; task_delay=0.5; total_deadline=75
                   ;;
                3) # Load testing - Moderate load with speed
                   phase1_tasks=30; phase2_tasks=15; task_delay=1.0; total_deadline=90
                   ;;
                4) # Resource pressure - Higher load but faster than before
                   phase1_tasks=40; phase2_tasks=20; task_delay=1.0; total_deadline=120
                   ;;
                5) # Extreme - High load but still reasonable timing
                   phase1_tasks=50; phase2_tasks=25; task_delay=1.5; total_deadline=150
                   ;;
                6) # Catastrophic - Maximum load but not excessive waiting
                   phase1_tasks=60; phase2_tasks=30; task_delay=2.0; total_deadline=180
                   ;;
            esac
            
            log "INFO" "Dynamic scaling parameters: ${initial_workers}→${reduced_workers} workers, delays: ${task_delay}s, deadline: ${total_deadline}s"
            
            # Create auth token for tasks
            log "INFO" "Creating authentication token for dynamic scaling test..."
            local auth_result
            auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "dynamic_scaling_test" 2>&1)
            local auth_exit_code=$?
            
            if [ $auth_exit_code -ne 0 ]; then
                log "BOLD_WARN" "⚠️  Auth helper script failed with exit code $auth_exit_code"
                log "BOLD_WARN" "Auth output: $auth_result"
                critical_error "Authentication setup failed - cannot proceed with dynamic scaling test"
            fi
            
            # Check for auth errors in response
            check_auth_response "$auth_result" "authentication setup"
            
            local token
            token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -z "$token" ]; then
                log "BOLD_WARN" "⚠️  Failed to extract token from auth response"
                log "BOLD_WARN" "Auth response: $auth_result"
                critical_error "Authentication token extraction failed - invalid response format"
            fi
            
            log "SUCCESS" "Authentication token obtained successfully"
            
            # Clean up any existing workers
            "$HELPERS_DIR/multi-worker-chaos.sh" cleanup > /dev/null 2>&1 || true
            sleep 2
            
            local scenario_start=$(date +%s)
            
            # Pre-Phase Health Check
            log "INFO" "⚡ FAIL FAST: Checking system health before dynamic scaling phases..."
            if ! curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
                log "ERROR" "⚡ FAIL FAST: API unhealthy before dynamic scaling - aborting"
                TEST_RESULTS+=("❌ dynamic-scaling: FAIL (Pre-check API)")
                continue
            fi
            
            # Phase 1: Optimal Capacity (0-60s)
            log "INFO" "🚀 Phase 1: Starting with $initial_workers workers for optimal capacity"
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers "$initial_workers" --verbose
            sleep 5  # Wait for workers to be ready
            
            log "INFO" "Creating Phase 1 tasks: $phase1_tasks tasks with ${task_delay}s delay each"
            "$HELPERS_DIR/delay-task-flood.sh" \
                --count "$phase1_tasks" \
                --delay "$task_delay" \
                --deadline 20 \
                --auth "$token" \
                --tag "dynamic_phase" \
                --verbose &
            PHASE1_PID=$!
            
            # Wait for Phase 1 duration (20 seconds)
            log "INFO" "Phase 1: Waiting 20s for optimal capacity processing..."
            sleep 20
            wait $PHASE1_PID || true
            
            local phase1_end=$(date +%s)
            local phase1_duration=$((phase1_end - scenario_start))
            log "INFO" "Phase 1 completed in ${phase1_duration}s"
            
            # Check Phase 1 completion rate - must be 100% to continue
            log "INFO" "Checking Phase 1 completion rate..."
            local phase1_stats
            phase1_stats=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats --tag "dynamic_phase" 2>&1)
            local cli_exit_code=$?
            
            if [ $cli_exit_code -ne 0 ]; then
                log "BOLD_WARN" "⚠️  Admin CLI failed during Phase 1 check (exit code: $cli_exit_code)"
                log "BOLD_WARN" "CLI output: $phase1_stats"
                critical_error "Admin CLI failure during Phase 1 verification - cannot determine task completion status"
            fi
            
            local phase1_completed=0
            local phase1_total=0
            
            if echo "$phase1_stats" | grep -q "completed.*[0-9]"; then
                phase1_completed=$(echo "$phase1_stats" | grep "completed:" | awk '{print $2}')
            fi
            if echo "$phase1_stats" | grep -q "Total.*[0-9]"; then
                phase1_total=$(echo "$phase1_stats" | grep "Total:" | awk '{print $2}')
            fi
            
            log "INFO" "Phase 1 results: $phase1_completed/$phase1_total tasks completed"
            
            # Debug: Show all task stats if Phase 1 shows no tasks
            if [ "$phase1_total" -eq 0 ]; then
                log "BOLD_WARN" "⚠️  No Phase 1 tasks found! Checking overall task status for debugging..."
                local all_stats
                all_stats=$(docker exec "$MAIN_CONTAINER_NAME" /app/starter admin task-stats 2>&1)
                log "BOLD_WARN" "Overall task stats: $all_stats"
                
                # Check if workers are running
                local worker_count
                worker_count=$(docker-compose -f "$CHAOS_COMPOSE_FILE" ps -q workers | wc -l)
                log "BOLD_WARN" "Active worker containers: $worker_count"
                
                critical_error "No Phase 1 tasks found - possible task creation failure or wrong prefix filter"
            fi
            
            # Early exit if Phase 1 didn't achieve 100% completion
            if [ "$phase1_completed" -ne "$phase1_total" ]; then
                log "ERROR" "Phase 1 failed - not all tasks completed ($phase1_completed/$phase1_total)"
                log "ERROR" "Cannot proceed to Phase 2 without 100% Phase 1 completion"
                
                # Show failed/pending tasks for debugging
                local failed_stats
                failed_stats=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats --tag "dynamic_phase1" 2>&1)
                log "BOLD_WARN" "⚠️  Phase 1 detailed stats: $failed_stats"
                
                "$HELPERS_DIR/multi-worker-chaos.sh" stop-all > /dev/null 2>&1 || true
                TEST_RESULTS+=("❌ dynamic-scaling: FAIL (Phase 1: $phase1_completed/$phase1_total)")
                return
            fi
            
            log "SUCCESS" "Phase 1 achieved 100% completion - proceeding to Phase 2"
            
            # Phase 2: Capacity Reduction (60-120s)
            log "INFO" "⬇️  Phase 2: Scaling down to $reduced_workers workers to create capacity pressure"
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers "$reduced_workers" --verbose
            sleep 3
            
            log "INFO" "Creating Phase 2 tasks: $phase2_tasks tasks with ${task_delay}s delay each"
            "$HELPERS_DIR/delay-task-flood.sh" \
                --count "$phase2_tasks" \
                --delay "$task_delay" \
                --deadline 20 \
                --auth "$token" \
                --tag "dynamic_phase" \
                --verbose &
            PHASE2_PID=$!
            
            # Wait for Phase 2 duration (20 seconds)
            log "INFO" "Phase 2: Waiting 20s with reduced capacity..."
            sleep 20
            wait $PHASE2_PID || true
            
            local phase2_end=$(date +%s)
            local phase2_duration=$((phase2_end - phase1_end))
            log "INFO" "Phase 2 completed in ${phase2_duration}s"
            
            # Check Phase 2 completion rate - should handle reduced capacity gracefully
            log "INFO" "Checking Phase 2 completion rate..."
            local phase2_stats
            phase2_stats=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats --tag "dynamic_phase" 2>&1)
            local phase2_completed=0
            local phase2_total=0
            
            if echo "$phase2_stats" | grep -q "completed.*[0-9]"; then
                phase2_completed=$(echo "$phase2_stats" | grep "completed:" | awk '{print $2}')
            fi
            if echo "$phase2_stats" | grep -q "Total.*[0-9]"; then
                phase2_total=$(echo "$phase2_stats" | grep "Total:" | awk '{print $2}')
            fi
            
            log "INFO" "Phase 2 results: $phase2_completed/$phase2_total tasks completed"
            
            # Allow some tolerance in Phase 2 due to reduced capacity, but require substantial progress
            local phase2_success_rate=0
            if [ "$phase2_total" -gt 0 ]; then
                phase2_success_rate=$(echo "scale=0; ($phase2_completed * 100) / $phase2_total" | bc -l 2>/dev/null || echo "0")
            fi
            
            if [ "$phase2_total" -eq 0 ] || [ "$phase2_success_rate" -lt 80 ]; then
                log "ERROR" "Phase 2 failed - insufficient completion rate (${phase2_success_rate}%)"
                log "ERROR" "Cannot proceed to Phase 3 without substantial Phase 2 progress"
                "$HELPERS_DIR/multi-worker-chaos.sh" stop-all > /dev/null 2>&1 || true
                TEST_RESULTS+=("❌ dynamic-scaling: FAIL (Phase 2: ${phase2_success_rate}%)")
                return
            fi
            
            log "SUCCESS" "Phase 2 achieved ${phase2_success_rate}% completion - proceeding to Phase 3"
            
            # Phase 3: Gradual Scale-Up (40-55s)
            log "INFO" "⬆️  Phase 3: Gradually scaling up workers (+1 every 3s)"
            
            # Stop creating new tasks - focus on processing backlog
            log "INFO" "No new tasks in Phase 3 - processing existing queue"
            
            # Scale up gradually: 3 workers
            log "INFO" "Scaling to 3 workers..."
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers 3 --verbose
            sleep 3
            
            # 4 workers
            log "INFO" "Scaling to 4 workers..."
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers 4 --verbose
            sleep 3
            
            # 5 workers
            log "INFO" "Scaling back to $initial_workers workers..."
            "$HELPERS_DIR/multi-worker-chaos.sh" start-multi --workers "$initial_workers" --verbose
            sleep 3
            
            local phase3_end=$(date +%s)
            local phase3_duration=$((phase3_end - phase2_end))
            log "INFO" "Phase 3 completed in ${phase3_duration}s"
            
            # Phase 4: Completion Phase
            log "INFO" "🎯 Phase 4: All workers active - monitoring completion"
            
            # Calculate remaining time for completion
            local elapsed_time=$((phase3_end - scenario_start))
            local remaining_time=$((total_deadline - elapsed_time))
            
            if [ $remaining_time -lt 10 ]; then
                remaining_time=10  # Minimum 10s for completion monitoring
            fi
            
            log "INFO" "Monitoring task completion for up to ${remaining_time}s..."
            
            # Start comprehensive monitoring using admin CLI
            timeout $remaining_time "$HELPERS_DIR/task-completion-monitor.sh" \
                --container "$MAIN_CONTAINER_NAME" \
                --tag "dynamic_phase" \
                --deadline "$total_deadline" \
                --verbose > "$OUTPUT_DIR/dynamic-scaling-monitor.log" 2>&1 &
            MONITOR_PID=$!
            
            # Wait for monitoring to complete or timeout
            wait $MONITOR_PID || true
            
            local final_end=$(date +%s)
            local total_duration=$((final_end - scenario_start))
            
            # Clean up workers
            "$HELPERS_DIR/multi-worker-chaos.sh" stop-all > /dev/null 2>&1 || true
            
            # Parse monitoring results
            local monitor_result=""
            if [ -f "$OUTPUT_DIR/dynamic-scaling-monitor.log" ]; then
                monitor_result=$(tail -1 "$OUTPUT_DIR/dynamic-scaling-monitor.log" | grep -o '{.*}' || echo "{}")
            fi
            
            # Extract results
            local completed_tasks=0
            local total_tasks=0
            local deadline_met=false
            local retry_attempts=0
            
            if [ -n "$monitor_result" ] && [ "$monitor_result" != "{}" ]; then
                completed_tasks=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))" 2>/dev/null || echo "0")
                total_tasks=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))" 2>/dev/null || echo "0")
                deadline_met=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('deadline_met', False))" 2>/dev/null || echo "false")
                retry_attempts=$(echo "$monitor_result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('retry_attempts', 0))" 2>/dev/null || echo "0")
            fi
            
            # Calculate success rate
            local success_rate=0
            if [ "$total_tasks" -gt 0 ]; then
                success_rate=$(echo "scale=1; ($completed_tasks * 100) / $total_tasks" | bc -l 2>/dev/null || echo "0")
            fi
            
            log "INFO" "Dynamic scaling results: $completed_tasks/$total_tasks tasks completed (${success_rate}%)"
            log "INFO" "Total duration: ${total_duration}s (deadline: ${total_deadline}s)"
            log "INFO" "Retry attempts: $retry_attempts, Deadline met: $deadline_met"
            
            # Determine success criteria
            local deadline_ok=$((total_duration <= total_deadline))
            local completion_ok=$(echo "$success_rate >= 100.0" | bc -l 2>/dev/null || echo "0")
            
            # Success criteria: 100% completion within 4 minutes
            if [ "$completion_ok" -eq 1 ] && [ "$deadline_ok" -eq 1 ]; then
                log "SUCCESS" "Dynamic scaling scenario passed - 100% completion within deadline"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ dynamic-scaling: PASS (${success_rate}%, ${total_duration}s, $retry_attempts retries)")
            elif [ "$completion_ok" -eq 1 ]; then
                log "SUCCESS" "Dynamic scaling scenario partially passed - 100% completion but exceeded deadline"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ dynamic-scaling: PASS* (${success_rate}%, ${total_duration}s, $retry_attempts retries)")
            else
                log "ERROR" "Dynamic scaling scenario failed - incomplete tasks or deadline exceeded"
                TEST_RESULTS+=("❌ dynamic-scaling: FAIL (${success_rate}%, ${total_duration}s, $retry_attempts retries)")
            fi
            
            # Final API test to ensure system is still healthy
            run_api_test "Dynamic Scaling Post-Test" || true
            ;;
            
        *)
            log "ERROR" "Unknown scenario: $scenario"
            TEST_RESULTS+=("❌ $scenario: UNKNOWN")
            ;;
    esac
    
    # Stop status monitoring for this scenario
    stop_status_monitor
    
    local scenario_end=$(date +%s)
    local scenario_duration=$((scenario_end - scenario_start))
    log "INFO" "Scenario '$scenario' completed in ${scenario_duration}s"
    log "INFO" ""
}

# Main execution
echo -e "${PURPLE}🔥 Rust Starter Chaos Testing Framework${NC}"
echo "================================================="
echo "Port: $PORT"
echo "Difficulty: $DIFFICULTY"
echo "Scenarios: $SCENARIOS"
echo "Output: $OUTPUT_DIR"
echo ""

# Clean up any existing chaos environment
cleanup_existing_environment() {
    log "INFO" "Cleaning up any existing chaos environment..."
    cd "$PROJECT_ROOT"
    
    # Stop any existing chaos containers
    docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans 2>/dev/null || true
    
    # Clean up any leftover containers with chaos-related names
    docker ps -aq --filter "name=starter-" | xargs docker rm -f 2>/dev/null || true
    
    # Kill any processes using the target port
    local target_port="8888"  # Fixed port for chaos testing
    lsof -ti:$target_port | xargs kill -9 2>/dev/null || true
    
    log "INFO" "Environment cleanup completed"
}

# Reset database if requested
reset_database_if_requested() {
    if [ "$RESET_DATABASE" = true ]; then
        log "INFO" "🗑️  Resetting database for clean testing environment..."
        
        # Start containers first if not running (needed for database reset)
        if ! docker ps --format '{{.Names}}' | grep -q "$POSTGRES_CONTAINER_NAME"; then
            log "INFO" "Starting containers for database reset..."
            docker-compose -f "$CHAOS_COMPOSE_FILE" up -d "$DB_SERVICE_NAME"
            
            # Wait for PostgreSQL to be ready
            local max_wait=30
            local wait_count=0
            while [ $wait_count -lt $max_wait ]; do
                if docker exec "$POSTGRES_CONTAINER_NAME" pg_isready -U "$DB_USER" >/dev/null 2>&1; then
                    log "SUCCESS" "PostgreSQL container is ready"
                    break
                fi
                sleep 1
                wait_count=$((wait_count + 1))
            done
            
            if [ $wait_count -eq $max_wait ]; then
                log "ERROR" "PostgreSQL container failed to start within ${max_wait}s"
                exit 1
            fi
        fi
        
        # Reset database using docker exec psql commands
        log "INFO" "Truncating all tables in chaos database..."
        
        # Truncate each table using configured list
        for table in "${DB_TABLES_TO_TRUNCATE[@]}"; do
            if docker exec "$POSTGRES_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -c "TRUNCATE TABLE $table RESTART IDENTITY CASCADE;" >/dev/null 2>&1; then
                log "INFO" "✅ Truncated table: $table"
            else
                log "WARN" "⚠️  Failed to truncate table: $table (may not exist yet)"
            fi
        done
        
        # Reset sequences using configured list
        for seq in "${DB_SEQUENCES_TO_RESET[@]}"; do
            if docker exec "$POSTGRES_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -c "ALTER SEQUENCE $seq RESTART WITH 1;" >/dev/null 2>&1; then
                log "INFO" "✅ Reset sequence: $seq"
            else
                log "WARN" "⚠️  Failed to reset sequence: $seq (may not exist yet)"
            fi
        done
        
        log "SUCCESS" "Database reset completed successfully"
    fi
}

# Check for unfinished tasks and warn
check_unfinished_tasks() {
    if [ "$RESET_DATABASE" != true ]; then
        log "INFO" "🔍 Checking for unfinished tasks from previous runs..."
        
        # Try to get task stats using admin CLI helper (if containers are running)
        local existing_tasks
        existing_tasks=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats 2>/dev/null || echo "")
        
        if [ -n "$existing_tasks" ]; then
            local pending_count=$(echo "$existing_tasks" | grep "pending:" | awk '{print $2}' 2>/dev/null || echo "0")
            local running_count=$(echo "$existing_tasks" | grep "running:" | awk '{print $2}' 2>/dev/null || echo "0")
            local total_unfinished=$((pending_count + running_count))
            
            if [ "$total_unfinished" -gt 0 ]; then
                log "WARN" "⚠️  Found $total_unfinished unfinished tasks from previous runs"
                log "WARN" "This may cause inaccurate results. Consider using --reset-database for clean testing"
                log "WARN" "Pending: $pending_count, Running: $running_count"
            else
                log "INFO" "✅ No unfinished tasks detected"
            fi
        fi
    fi
}

# Validate environment
log "INFO" "Validating environment..."

# Clean up first to ensure clean state
cleanup_existing_environment

# Reset database if requested
reset_database_if_requested

# Check if Docker services are running for chaos testing
log "INFO" "Starting Docker-based chaos testing environment..."
cd "$PROJECT_ROOT"

# Using .env.example directly via docker-compose.chaos.yaml - no need to create .env.chaos

# Build and start chaos environment
log "INFO" "Building Docker images with latest code..."
docker-compose -f "$CHAOS_COMPOSE_FILE" build

log "INFO" "Starting Docker chaos environment..."
docker-compose -f "$CHAOS_COMPOSE_FILE" up -d

# Wait for all containers to be healthy
log "INFO" "Waiting for all containers to be healthy..."
# health check variables already defined at top of file
health_attempt=0
while [ $health_attempt -lt $health_attempts ]; do
    # Check container health using container names
    server_health=$(docker inspect "$MAIN_CONTAINER_NAME" --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    worker_health=$(docker-compose -f "$CHAOS_COMPOSE_FILE" ps -q workers | head -1 | xargs docker inspect --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    postgres_health=$(docker inspect "$POSTGRES_CONTAINER_NAME" --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    
    if [ "$server_health" = "healthy" ] && [ "$worker_health" = "healthy" ] && [ "$postgres_health" = "healthy" ]; then
        log "SUCCESS" "All containers are healthy"
        
        # Check for unfinished tasks now that containers are healthy
        check_unfinished_tasks
        
        break
    else
        log "INFO" "Container health status - server: $server_health, worker: $worker_health, postgres: $postgres_health"
    fi
    
    health_attempt=$((health_attempt + 1))
    sleep 2
done

if [ $health_attempt -eq $health_attempts ]; then
    log "WARNING" "Not all containers reached healthy status within timeout, checking API availability..."
fi

# Validate that the API service is responding  
log "INFO" "Validating API service availability..."
# api check variables already defined at top of file
api_attempt=0
while [ $api_attempt -lt $api_attempts ]; do
    if curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
        log "SUCCESS" "API service is responding"
        break
    fi
    api_attempt=$((api_attempt + 1))
    sleep 2
done

# Wait for workers to complete task type registration
log "INFO" "Waiting for workers to register task types..."
# task type check variables already defined at top of file
task_type_attempt=0
while [ $task_type_attempt -lt $task_type_attempts ]; do
    # Check if task types are registered by checking the /tasks/types endpoint
    task_types_response=$(curl -s "$BASE_URL/tasks/types" 2>/dev/null || echo "")
    if echo "$task_types_response" | grep -q '"task_type".*"email"' && echo "$task_types_response" | grep -q '"task_type".*"webhook"'; then
        log "SUCCESS" "Workers have registered task types"
        break
    fi
    task_type_attempt=$((task_type_attempt + 1))
    sleep 2
done

if [ $task_type_attempt -eq $task_type_attempts ]; then
    log "WARNING" "Workers may not have fully registered all task types yet, continuing anyway"
fi

# Foundation validation function for admin CLI functionality
# This ensures the CLI works reliably before any chaos scenarios run
validate_admin_cli() {
    log "INFO" "Testing admin CLI database connection and query functionality..."
    
    # Test 1: Basic admin CLI connection and database query
    local cli_test_result
    cli_test_result=$(docker exec "$MAIN_CONTAINER_NAME" /app/starter admin task-stats 2>&1)
    local cli_exit_code=$?
    
    if [ $cli_exit_code -ne 0 ]; then
        log "ERROR" "🚫 FOUNDATION CHECK FAILED: Admin CLI container exec error"
        log "ERROR" "CLI error output: $cli_test_result"
        log "ERROR" "🚫 BLOCKING ALL SCENARIOS: Cannot proceed without working admin CLI"
        docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true
        exit 1
    fi
    
    # Test 2: Verify CLI returns expected statistics format
    if ! echo "$cli_test_result" | grep -q "Task Statistics"; then
        log "ERROR" "🚫 FOUNDATION CHECK FAILED: Admin CLI unexpected output format"
        log "ERROR" "Expected 'Task Statistics' in output, got: $cli_test_result"
        log "ERROR" "🚫 BLOCKING ALL SCENARIOS: Cannot proceed without working admin CLI"
        docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true
        exit 1
    fi
    
    # Test 3: Verify admin CLI with prefix filter (critical for scenario monitoring)
    log "INFO" "Testing admin CLI prefix filtering functionality..."
    local prefix_test_result
    prefix_test_result=$("$HELPERS_DIR/admin-cli-helper.sh" -c "$MAIN_CONTAINER_NAME" task-stats --tag "foundation_test" 2>&1)
    local prefix_exit_code=$?
    
    if [ $prefix_exit_code -ne 0 ]; then
        log "ERROR" "🚫 FOUNDATION CHECK FAILED: Admin CLI prefix filter error"
        log "ERROR" "Prefix test output: $prefix_test_result"
        log "ERROR" "🚫 BLOCKING ALL SCENARIOS: Prefix filtering is critical for chaos monitoring"
        docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true
        exit 1
    fi
    
    # Test 4: Test CLI with list-tasks command (used in diagnostics)
    log "INFO" "Testing admin CLI list-tasks functionality..."
    local list_test_result
    list_test_result=$(docker exec "$MAIN_CONTAINER_NAME" /app/starter admin list-tasks --limit 5 2>&1)
    local list_exit_code=$?
    
    if [ $list_exit_code -ne 0 ]; then
        log "ERROR" "🚫 FOUNDATION CHECK FAILED: Admin CLI list-tasks error"
        log "ERROR" "List-tasks output: $list_test_result"
        log "ERROR" "🚫 BLOCKING ALL SCENARIOS: List-tasks is critical for task monitoring"
        docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true
        exit 1
    fi
    
    log "SUCCESS" "✅ All admin CLI tests passed - foundation is solid"
}

# Check if API service validation succeeded before proceeding
if [ $api_attempt -eq $api_attempts ]; then
    log "ERROR" "API service not responding on $BASE_URL after ${api_attempts} attempts"
    log "INFO" "Check Docker containers: docker-compose -f docker-compose.chaos.yaml ps"
    exit 1
fi

# ===================================================================
# FOUNDATION CHECK: Admin CLI Validation 
# ===================================================================
# This pre-check ensures the admin CLI works before running ANY scenario.
# If this fails, ALL chaos testing is blocked - preventing false positives
# and ensuring reliable monitoring across all scenarios.
# ===================================================================
log "INFO" "🔍 FOUNDATION CHECK: Validating admin CLI for all scenarios..."
validate_admin_cli
log "SUCCESS" "🛡️  FOUNDATION CHECK PASSED: Admin CLI ready for all chaos scenarios"

# Parse scenarios to run
if [ "$SCENARIOS" = "all" ]; then
    SCENARIO_LIST="baseline db-failure server-restart worker-restart task-flood circuit-breaker mixed-chaos recovery multi-worker-chaos dynamic-scaling"
else
    SCENARIO_LIST=$(echo "$SCENARIOS" | tr ',' ' ')
fi

log "INFO" "Will run scenarios: $SCENARIO_LIST"

# Make helper scripts executable
chmod +x "$HELPERS_DIR"/*.sh

# Run scenarios
for scenario in $SCENARIO_LIST; do
    run_scenario "$scenario"
done

# Generate final report
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))

echo ""
echo "================================================="
echo -e "${PURPLE}🔥 Chaos Testing Results${NC}"
echo "================================================="
echo "Total scenarios: $TOTAL_SCENARIOS"
echo "Passed: $PASSED_SCENARIOS"
echo "Failed: $((TOTAL_SCENARIOS - PASSED_SCENARIOS))"
echo "Success rate: $(( PASSED_SCENARIOS * 100 / TOTAL_SCENARIOS ))%"
echo "Total duration: ${TOTAL_DURATION}s"
echo ""

echo "Scenario Results:"
for result in "${TEST_RESULTS[@]}"; do
    echo "  $result"
done

echo ""

# Write detailed report
REPORT_FILE="$OUTPUT_DIR/chaos-test-report.md"
cat > "$REPORT_FILE" << EOF
# Chaos Testing Report

**Date:** $(date)
**Difficulty Level:** $DIFFICULTY
**Port:** $PORT
**Total Duration:** ${TOTAL_DURATION}s

## Summary

- **Total Scenarios:** $TOTAL_SCENARIOS
- **Passed:** $PASSED_SCENARIOS
- **Failed:** $((TOTAL_SCENARIOS - PASSED_SCENARIOS))
- **Success Rate:** $(( PASSED_SCENARIOS * 100 / TOTAL_SCENARIOS ))%

## Scenario Results

$(for result in "${TEST_RESULTS[@]}"; do echo "- $result"; done)

## Test Configuration

- **Difficulty:** $DIFFICULTY
- **Scenarios:** $SCENARIOS
- **Parameters:** $(get_difficulty_params "$DIFFICULTY")

## Files Generated

- \`chaos-test.log\` - Detailed execution log
- \`api-test-*.txt\` - Individual API test results
- \`chaos-test-report.md\` - This report

## Recommendations

$(if [ $PASSED_SCENARIOS -eq $TOTAL_SCENARIOS ]; then
    echo "✅ All scenarios passed. System shows good resilience."
else
    echo "⚠️ Some scenarios failed. Review logs for improvements."
fi)

EOF

# Cleanup any leftover status monitors
stop_status_monitor

# Cleanup Docker containers after chaos testing (unless --no-cleanup)
if [ "$NO_CLEANUP" = true ]; then
    log "INFO" "🔍 Skipping cleanup - containers left running for debugging"
    log "INFO" "To examine logs: docker logs chaos-starter-server"
    log "INFO" "To examine logs: docker logs rust-fullstack-starter-workers-1"
    log "INFO" "To clean up later: docker-compose -f docker-compose.chaos.yaml down"
else
    log "INFO" "Cleaning up Docker chaos environment..."
    cd "$PROJECT_ROOT"
    docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true
fi

log "SUCCESS" "Chaos testing completed!"
log "INFO" "Report written to: $REPORT_FILE"

if [ $PASSED_SCENARIOS -eq $TOTAL_SCENARIOS ]; then
    echo -e "${GREEN}🎉 All chaos scenarios passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️ Some scenarios failed. Check logs for details.${NC}"
    exit 1
fi