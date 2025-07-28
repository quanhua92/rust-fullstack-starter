#!/bin/bash

# Comprehensive Chaos Testing Script
# Tests system resilience under various failure scenarios

set -e

# Default values  
PORT="${PORT:-8888}"
BASE_URL="http://localhost:$PORT"
DIFFICULTY="${DIFFICULTY:-1}"
SCENARIOS="${SCENARIOS:-all}"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp}"
VERBOSE="${VERBOSE:-false}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

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
    echo "  all              - Run all scenarios (default)"
    echo ""
    echo "Examples:"
    echo "  $0                                     # Basic chaos testing"
    echo "  $0 --difficulty 3 --port 8080         # Advanced testing on port 8080"
    echo "  $0 --scenarios \"db-failure,task-flood\" # Specific scenarios only"
    echo "  $0 --difficulty 5 --verbose           # Extreme testing with logs"
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

# Get project root and setup paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HELPERS_DIR="$SCRIPT_DIR/helpers"

# Docker compose file for chaos testing
CHAOS_COMPOSE_FILE="$PROJECT_ROOT/docker-compose.chaos.yaml"

# Check if Docker compose file exists
if [ ! -f "$CHAOS_COMPOSE_FILE" ]; then
    echo "❌ Docker compose file not found: $CHAOS_COMPOSE_FILE" >&2
    echo "   Please ensure docker-compose.chaos.yaml exists" >&2
    exit 1
fi

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Test results tracking
TEST_RESULTS=()
TOTAL_SCENARIOS=0
PASSED_SCENARIOS=0
START_TIME=$(date +%s)

log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        INFO) echo -e "${BLUE}[$timestamp] INFO:${NC} $message" ;;
        WARN) echo -e "${YELLOW}[$timestamp] WARN:${NC} $message" ;;
        ERROR) echo -e "${RED}[$timestamp] ERROR:${NC} $message" ;;
        SUCCESS) echo -e "${GREEN}[$timestamp] SUCCESS:${NC} $message" ;;
        *) echo "[$timestamp] $level: $message" ;;
    esac
    
    if [ "$VERBOSE" = true ]; then
        echo "[$timestamp] $level: $message" >> "$OUTPUT_DIR/chaos-test.log"
    fi
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
    
    # Start status monitoring for this scenario
    start_status_monitor "$scenario"
    
    local scenario_start=$(date +%s)
    local params=$(get_difficulty_params "$DIFFICULTY")
    eval "$params"
    
    case "$scenario" in
        baseline)
            log "INFO" "Running baseline functionality test"
            if run_api_test "Baseline Test"; then
                log "SUCCESS" "Baseline test passed"
                PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                TEST_RESULTS+=("✅ baseline: PASS")
            else
                log "ERROR" "Baseline test failed"
                TEST_RESULTS+=("❌ baseline: FAIL")
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
                # Create some tasks before killing worker
                "$HELPERS_DIR/task-flood.sh" --count 10 --auth "$token" --delay 0.1
                
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
            
            if [ -n "$token" ]; then
                # Create task flood
                "$HELPERS_DIR/task-flood.sh" --count "$task_count" --delay "$delay" --auth "$token" --verbose
                
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
                log "ERROR" "Failed to get auth token for task flood test"
                TEST_RESULTS+=("❌ task-flood: FAIL")
            fi
            ;;
            
        circuit-breaker)
            log "INFO" "Testing circuit breaker activation"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "cb_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -n "$token" ]; then
                # Create failing tasks to trigger circuit breaker
                "$HELPERS_DIR/task-flood.sh" --count 20 --auth "$token" --fail --delay 0.1 --verbose
                
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
                log "ERROR" "Failed to get auth token for circuit breaker test"
                TEST_RESULTS+=("❌ circuit-breaker: FAIL")
            fi
            ;;
            
        mixed-chaos)
            log "INFO" "Testing multiple simultaneous failures"
            
            # Create auth token
            local auth_result=$(BASE_URL="$BASE_URL" timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "mixed_test")
            local token=$(echo "$auth_result" | python3 -c "import json,sys; print(json.load(sys.stdin)['token'])" 2>/dev/null || echo "")
            
            if [ -n "$token" ]; then
                # Start task flood in background
                "$HELPERS_DIR/task-flood.sh" --count "$task_count" --delay "$delay" --auth "$token" &
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
                --prefix "multiworker" \
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
            
            # Monitor task completion
            log "INFO" "Monitoring task completion with deadline enforcement..."
            "$HELPERS_DIR/task-completion-monitor.sh" \
                --prefix "multiworker" \
                --deadline "$task_deadline" \
                --auth "$token" \
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
    local target_port=$(grep "CHAOS_APP_PORT" .env.chaos.example 2>/dev/null | cut -d'=' -f2 || echo "3000")
    lsof -ti:$target_port | xargs kill -9 2>/dev/null || true
    
    log "INFO" "Environment cleanup completed"
}

# Validate environment
log "INFO" "Validating environment..."

# Clean up first to ensure clean state
cleanup_existing_environment

# Check if Docker services are running for chaos testing
log "INFO" "Starting Docker-based chaos testing environment..."
cd "$PROJECT_ROOT"

# Create environment file if it doesn't exist
if [ ! -f .env.chaos ]; then
    log "INFO" "Creating .env.chaos from example..."
    cp .env.chaos.example .env.chaos
fi

# Build and start chaos environment
log "INFO" "Building Docker images with latest code..."
docker-compose -f "$CHAOS_COMPOSE_FILE" build

log "INFO" "Starting Docker chaos environment..."
docker-compose -f "$CHAOS_COMPOSE_FILE" up -d

# Wait for all containers to be healthy
log "INFO" "Waiting for all containers to be healthy..."
health_attempts=20  # 40 seconds total wait time (optimized)
health_attempt=0
while [ $health_attempt -lt $health_attempts ]; do
    # Check container health using container names
    server_health=$(docker inspect chaos-starter-server --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    worker_health=$(docker inspect chaos-starter-worker --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    postgres_health=$(docker inspect chaos-starter-postgres --format='{{.State.Health.Status}}' 2>/dev/null || echo "")
    
    if [ "$server_health" = "healthy" ] && [ "$worker_health" = "healthy" ] && [ "$postgres_health" = "healthy" ]; then
        log "SUCCESS" "All containers are healthy"
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
api_attempts=15  # 30 seconds max
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
task_type_attempts=10  # 20 seconds max for worker registration 
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

if [ $api_attempt -eq $api_attempts ]; then
    log "ERROR" "API service not responding on $BASE_URL after ${api_attempts} attempts"
    log "INFO" "Check Docker containers: docker-compose -f docker-compose.chaos.yaml ps"
    exit 1
fi

# Parse scenarios to run
if [ "$SCENARIOS" = "all" ]; then
    SCENARIO_LIST="baseline db-failure server-restart worker-restart task-flood circuit-breaker mixed-chaos recovery multi-worker-chaos"
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

# Cleanup Docker containers after chaos testing
log "INFO" "Cleaning up Docker chaos environment..."
cd "$PROJECT_ROOT"
docker-compose -f "$CHAOS_COMPOSE_FILE" down --remove-orphans || true

log "SUCCESS" "Chaos testing completed!"
log "INFO" "Report written to: $REPORT_FILE"

if [ $PASSED_SCENARIOS -eq $TOTAL_SCENARIOS ]; then
    echo -e "${GREEN}🎉 All chaos scenarios passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️ Some scenarios failed. Check logs for details.${NC}"
    exit 1
fi