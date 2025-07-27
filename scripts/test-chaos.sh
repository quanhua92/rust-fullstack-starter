#!/bin/bash

# Comprehensive Chaos Testing Script
# Tests system resilience under various failure scenarios

set -e

# Default values
PORT="${PORT:-3000}"
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
    echo "Comprehensive chaos testing for the Rust starter application"
    echo ""
    echo "Options:"
    echo "  -p, --port PORT        Server port (default: $PORT)"
    echo "  -d, --difficulty LEVEL Difficulty level 1-5 (default: $DIFFICULTY)"
    echo "  -s, --scenarios LIST   Scenarios to run (default: all)"
    echo "  -o, --output DIR       Output directory (default: /tmp)"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Difficulty Levels:"
    echo "  1 - Basic: Simple restarts and database failures"
    echo "  2 - Moderate: Service interruptions with load"
    echo "  3 - Advanced: High load with circuit breaker triggers"
    echo "  4 - Expert: Multiple concurrent failures"
    echo "  5 - Extreme: Sustained chaos with recovery testing"
    echo "  6 - Catastrophic: Designed to fail - tests failure handling (multi-worker only)"
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
            local auth_result=$(timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "worker_test")
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
            local auth_result=$(timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "flood_test")
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
            local auth_result=$(timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "cb_test")
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
            local auth_result=$(timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "mixed_test")
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
            
            if [ "$avg_recovery" -le 15 ]; then
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
            
            case "$DIFFICULTY" in
                1) worker_count=2; delay_per_task=3; task_deadline=45; min_stop_interval=15; max_stop_interval=25 ;;
                2) worker_count=3; delay_per_task=4; task_deadline=50; min_stop_interval=12; max_stop_interval=20 ;;
                3) worker_count=3; delay_per_task=5; task_deadline=60; min_stop_interval=10; max_stop_interval=18 ;;
                4) worker_count=4; delay_per_task=6; task_deadline=70; min_stop_interval=8; max_stop_interval=15 ;;
                5) worker_count=5; delay_per_task=8; task_deadline=90; min_stop_interval=5; max_stop_interval=12 ;;
                6) worker_count=5; delay_per_task=15; task_deadline=45; min_stop_interval=2; max_stop_interval=5 ;;
            esac
            
            if [ "$DIFFICULTY" -eq 6 ]; then
                log "WARN" "⚠️  CATASTROPHIC MODE: This test is designed to fail!"
                log "INFO" "Level 6 creates impossible workload: 30 tasks × ${delay_per_task}s = $((30 * delay_per_task))s needed, only ${task_deadline}s allowed"
            fi
            
            log "INFO" "Multi-worker scenario: $worker_count workers, ${delay_per_task}s delays, ${task_deadline}s deadline"
            
            # Create auth token for tasks
            local auth_result=$(timeout 30 "$HELPERS_DIR/auth-helper.sh" --prefix "multiworker_test")
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
            # For Level 6, use fewer tasks but still impossible timing
            local actual_task_count=$task_count
            if [ "$DIFFICULTY" -eq 6 ]; then
                actual_task_count=30  # 30 tasks × 15s = 450s needed, only 45s allowed
                log "INFO" "Level 6: Reducing to $actual_task_count tasks for focused catastrophic failure"
            fi
            
            log "INFO" "Creating delay tasks (count: $actual_task_count, delay: ${delay_per_task}s each)"
            "$HELPERS_DIR/delay-task-flood.sh" \
                --count "$actual_task_count" \
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
            
            log "INFO" "Multi-worker chaos results: $completed_tasks/$total_tasks tasks completed (${success_rate}%)"
            log "INFO" "Retry attempts: $retry_attempts, Deadline met: $deadline_met"
            
            # Pass criteria depend on difficulty level
            if [ "$DIFFICULTY" -eq 6 ]; then
                # Level 6: Designed to fail - success criteria are inverted
                log "INFO" "Level 6 (Catastrophic): Testing designed failure scenario"
                # For level 6, success means the system properly detected it couldn't meet the impossible deadline
                # We expect 0% completion and deadline missed - this validates failure detection
                log "INFO" "Debug: deadline_met=$deadline_met, success_rate=$success_rate"
                local math_result=$(echo "$success_rate <= 10" | bc -l 2>/dev/null || echo "0")
                log "INFO" "Debug: math comparison result=$math_result"
                log "INFO" "Debug: deadline check: [$deadline_met = false] = $([ "$deadline_met" = "false" ] || [ "$deadline_met" = "False" ] && echo "true" || echo "false")"
                log "INFO" "Debug: math check: [$math_result -eq 1] = $([ "$math_result" -eq 1 ] && echo "true" || echo "false")"
                
                if ([ "$deadline_met" = "false" ] || [ "$deadline_met" = "False" ]) && [ "$math_result" -eq 1 ]; then
                    log "SUCCESS" "Multi-worker chaos scenario passed (designed failure validated)"
                    log "SUCCESS" "System properly detected impossible workload and failed gracefully"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ multi-worker-chaos: PASS (designed failure: ${success_rate}%, deadline missed as expected)")
                else
                    log "ERROR" "Multi-worker chaos scenario failed (system too resilient for level 6)"
                    log "ERROR" "Expected: deadline missed + <10% completion. Got: deadline_met=$deadline_met, completion=${success_rate}%"
                    TEST_RESULTS+=("❌ multi-worker-chaos: FAIL (system survived: ${success_rate}%, deadline: $deadline_met)")
                fi
            else
                # Levels 1-5: Normal pass criteria - >80% task completion, evidence of retries
                if [ "$total_tasks" -gt 0 ] && [ "$(echo "$success_rate >= 80" | bc -l)" -eq 1 ] && [ "$retry_attempts" -gt 0 ]; then
                    log "SUCCESS" "Multi-worker chaos scenario passed"
                    PASSED_SCENARIOS=$((PASSED_SCENARIOS + 1))
                    TEST_RESULTS+=("✅ multi-worker-chaos: PASS (${success_rate}%, $retry_attempts retries)")
                else
                    log "ERROR" "Multi-worker chaos scenario failed"
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

# Validate environment
log "INFO" "Validating environment..."

if ! curl -s -f "$BASE_URL/health" > /dev/null; then
    log "ERROR" "Server not responding on $BASE_URL"
    log "INFO" "Please ensure server is running: ./scripts/server.sh $PORT"
    exit 1
fi

log "SUCCESS" "Environment validation passed"

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

log "SUCCESS" "Chaos testing completed!"
log "INFO" "Report written to: $REPORT_FILE"

if [ $PASSED_SCENARIOS -eq $TOTAL_SCENARIOS ]; then
    echo -e "${GREEN}🎉 All chaos scenarios passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️ Some scenarios failed. Check logs for details.${NC}"
    exit 1
fi