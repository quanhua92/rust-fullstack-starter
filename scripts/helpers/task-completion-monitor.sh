#!/bin/bash

# Task Completion Monitor (Admin CLI Version)
# Monitors task completion using admin CLI commands instead of API authentication
# Uses docker exec to bypass authentication issues during chaos testing

set -e

# Default values
CONTAINER_NAME="${CONTAINER_NAME:-chaos-starter-server}"
TASK_TAG="${TASK_TAG:-chaos}"
DEADLINE_SECONDS="${DEADLINE_SECONDS:-60}"
CHECK_INTERVAL="${CHECK_INTERVAL:-5}"
TIMEOUT_BUFFER="${TIMEOUT_BUFFER:-10}"

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
    echo "Monitor task completion for chaos testing scenarios using admin CLI"
    echo ""
    echo "Options:"
    echo "  -c, --container NAME      Container name (default: $CONTAINER_NAME)"
    echo "  -t, --tag TAG             Task tag to monitor (default: $TASK_TAG)"
    echo "  -d, --deadline SECONDS    Task deadline in seconds (default: $DEADLINE_SECONDS)"
    echo "  -i, --interval SECONDS    Check interval (default: $CHECK_INTERVAL)"
    echo "  -b, --buffer SECONDS      Timeout buffer beyond deadline (default: $TIMEOUT_BUFFER)"
    echo "  -v, --verbose             Verbose output"
    echo "  -h, --help                Show this help"
    echo ""
    echo "The monitor will:"
    echo "  - Track all tasks with the specified tag using admin CLI"
    echo "  - Monitor completion status within deadline"
    echo "  - Report on retry behavior and worker resilience"
    echo "  - Provide detailed statistics on task processing"
    echo "  - Bypass API authentication by using docker exec"
}

# Parse arguments
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--container)
            CONTAINER_NAME="$2"
            shift 2
            ;;
        -t|--tag)
            TASK_TAG="$2"
            shift 2
            ;;
        -d|--deadline)
            DEADLINE_SECONDS="$2"
            shift 2
            ;;
        -i|--interval)
            CHECK_INTERVAL="$2"
            shift 2
            ;;
        -b|--buffer)
            TIMEOUT_BUFFER="$2"
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

# Show debug info if verbose
if [ "$VERBOSE" = true ]; then
    echo "DEBUG: Monitoring tasks with tag: '$TASK_TAG'"
    echo "DEBUG: Using container: $CONTAINER_NAME"
    echo "DEBUG: Check interval: ${CHECK_INTERVAL}s"
    echo "DEBUG: Deadline: ${DEADLINE_SECONDS}s"
fi

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
}

# Check if container exists and is running
check_container() {
    if ! docker ps --format "table {{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
        log "ERROR" "Container '$CONTAINER_NAME' not found or not running"
        log "INFO" "Available containers:"
        docker ps --format "table {{.Names}}\t{{.Status}}"
        exit 1
    fi
}

# Get task statistics using admin CLI
get_task_stats() {
    local tag="$1"
    
    if [ "$VERBOSE" = true ]; then
        log "INFO" "Getting task stats for tag '$tag' from container '$CONTAINER_NAME'"
    fi
    
    # Use admin CLI helper to get task statistics with tag filter
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local stats_output
    
    if [ "$VERBOSE" = true ]; then
        echo "DEBUG: Executing: $script_dir/admin-cli-helper.sh task-stats --tag '$tag'"
    fi
    
    stats_output=$("$script_dir/admin-cli-helper.sh" task-stats --tag "$tag" 2>&1 || echo "")
    
    if [ -z "$stats_output" ] || echo "$stats_output" | grep -q "error\|Error\|ERROR"; then
        if [ "$VERBOSE" = true ]; then
            echo "DEBUG: Prefix-specific query failed or errored, trying general stats"
            echo "DEBUG: Error output: $stats_output"
        fi
        # Fallback to general stats if tag-specific fails
        stats_output=$("$script_dir/admin-cli-helper.sh" task-stats 2>&1 || echo "")
    fi
    
    if [ "$VERBOSE" = true ]; then
        echo "DEBUG: Raw stats output (full):"
        echo "$stats_output"
        echo "DEBUG: ====== END RAW OUTPUT ======"
    fi
    
    # Parse the stats output to extract numbers (clean ANSI codes and debug logs)
    local clean_output=$(echo "$stats_output" | sed 's/\x1b\[[0-9;]*m//g')  # Remove ANSI color codes
    
    local pending_count=$(echo "$clean_output" | grep -E "^\s*pending:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
    local running_count=$(echo "$clean_output" | grep -E "^\s*running:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
    local completed_count=$(echo "$clean_output" | grep -E "^\s*completed:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
    local failed_count=$(echo "$clean_output" | grep -E "^\s*failed:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
    local total_count=$(echo "$clean_output" | grep -E "^\s*Total:" | sed 's/.*: *\([0-9]*\).*/\1/' | head -1 || echo "0")
    
    if [ "$VERBOSE" = true ]; then
        echo "DEBUG: Clean output sample:"
        echo "$clean_output" | head -20
        echo "DEBUG: Completed line: $(echo "$clean_output" | grep -E "^\s*completed:")"
        echo "DEBUG: Total line: $(echo "$clean_output" | grep -E "^\s*Total:")"
        echo "DEBUG: Extracted counts - pending: '$pending_count', running: '$running_count', completed: '$completed_count', failed: '$failed_count', total: '$total_count'"
    fi
    
    # Clean up the numbers (remove any non-numeric characters)
    pending_count=$(echo "$pending_count" | tr -cd '0-9' || echo "0")
    running_count=$(echo "$running_count" | tr -cd '0-9' || echo "0")
    completed_count=$(echo "$completed_count" | tr -cd '0-9' || echo "0")
    failed_count=$(echo "$failed_count" | tr -cd '0-9' || echo "0")
    total_count=$(echo "$total_count" | tr -cd '0-9' || echo "0")
    
    # Fallback values
    [ -z "$pending_count" ] && pending_count=0
    [ -z "$running_count" ] && running_count=0
    [ -z "$completed_count" ] && completed_count=0
    [ -z "$failed_count" ] && failed_count=0
    [ -z "$total_count" ] && total_count=0
    
    if [ "$VERBOSE" = true ]; then
        echo "DEBUG: Parsed stats - Total: $total_count, Completed: $completed_count, Failed: $failed_count, Running: $running_count, Pending: $pending_count"
    fi
    
    # Output JSON
    echo "{\"total\": $total_count, \"completed\": $completed_count, \"failed\": $failed_count, \"running\": $running_count, \"pending\": $pending_count}"
}

# Get recent tasks for more detailed analysis
get_recent_tasks() {
    local limit="${1:-20}"
    
    if [ "$VERBOSE" = true ]; then
        log "INFO" "Getting recent $limit tasks from container '$CONTAINER_NAME'"
    fi
    
    # Use admin CLI to list recent tasks
    docker exec "$CONTAINER_NAME" /app/starter admin list-tasks --limit "$limit" --verbose 2>&1 || echo "CLI_ERROR"
}

# Monitor task progress
monitor_tasks() {
    local start_time=$(date +%s)
    local end_time=$((start_time + DEADLINE_SECONDS + TIMEOUT_BUFFER))
    local last_stats=""
    
    log "INFO" "Starting task completion monitoring using admin CLI..."
    log "INFO" "Container: $CONTAINER_NAME"
    log "INFO" "Deadline: ${DEADLINE_SECONDS}s, Buffer: ${TIMEOUT_BUFFER}s, Check interval: ${CHECK_INTERVAL}s"
    log "INFO" "Monitoring tasks with prefix: $TASK_TAG"
    
    echo -e "${BLUE}📊 Task Progress Monitoring (Admin CLI)${NC}"
    echo "=================================="
    
    # Initial stats to establish baseline
    local initial_stats_json=$(get_task_stats "$TASK_TAG")
    local initial_total=$(echo "$initial_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))" 2>/dev/null || echo "0")
    
    if [ "$initial_total" -eq 0 ]; then
        log "WARN" "No tasks found with prefix '$TASK_TAG', monitoring all tasks"
        TASK_TAG=""
    fi
    
    local retry_attempts=0
    local last_completed=0
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))
        local remaining_time=$((end_time - current_time))
        
        # Get current task status using admin CLI
        local stats_json=$(get_task_stats "$TASK_TAG")
        
        # Parse stats
        local total=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))" 2>/dev/null || echo "0")
        local completed=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))" 2>/dev/null || echo "0")
        local failed=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('failed', 0))" 2>/dev/null || echo "0")
        local running=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('running', 0))" 2>/dev/null || echo "0")
        local pending=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('pending', 0))" 2>/dev/null || echo "0")
        
        # Estimate retry attempts by tracking completion count changes
        if [ "$completed" -lt "$last_completed" ]; then
            retry_attempts=$((retry_attempts + 1))
        fi
        last_completed=$completed
        
        # Display progress
        local progress=""
        if [ "$total" -gt 0 ]; then
            local completion_rate=$(echo "scale=1; ($completed * 100) / $total" | bc -l 2>/dev/null || echo "0")
            progress=" (${completion_rate}%)"
        fi
        
        echo -e "⏱️  ${elapsed_time}s elapsed | ${remaining_time}s remaining"
        echo -e "📋 Tasks: ${total} total | ${GREEN}${completed}${NC} completed${progress} | ${RED}${failed}${NC} failed"
        echo -e "🔄 Active: ${YELLOW}${running}${NC} running | ${BLUE}${pending}${NC} pending"
        
        if [ "$retry_attempts" -gt 0 ]; then
            echo -e "🔁 Estimated retry events: $retry_attempts"
        fi
        
        # Check if all tasks are complete
        local terminal_tasks=$((completed + failed))
        if [ "$total" -gt 0 ] && [ "$terminal_tasks" -eq "$total" ]; then
            log "SUCCESS" "All tasks completed!"
            break
        fi
        
        # Check if we're past the deadline
        if [ $elapsed_time -gt $DEADLINE_SECONDS ]; then
            local active_tasks=$((running + pending))
            if [ "$active_tasks" -gt 0 ]; then
                log "WARN" "Deadline exceeded with $active_tasks active tasks"
            fi
        fi
        
        echo "=================================="
        sleep $CHECK_INTERVAL
    done
    
    # Final report
    local final_time=$(date +%s)
    local total_duration=$((final_time - start_time))
    
    echo ""
    echo -e "${PURPLE}📋 Final Monitoring Report (Admin CLI)${NC}"
    echo "=================================="
    
    # Get final stats
    local final_stats_json=$(get_task_stats "$TASK_TAG")
    
    echo "Monitoring duration: ${total_duration}s"
    echo "Task deadline: ${DEADLINE_SECONDS}s"
    echo ""
    
    # Parse final stats
    local final_total=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))" 2>/dev/null || echo "0")
    local final_completed=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))" 2>/dev/null || echo "0")
    local final_failed=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('failed', 0))" 2>/dev/null || echo "0")
    local final_running=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('running', 0))" 2>/dev/null || echo "0")
    local final_pending=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('pending', 0))" 2>/dev/null || echo "0")
    
    echo "Task Summary:"
    echo "  Total tasks: $final_total"
    echo "  Completed: $final_completed"
    echo "  Failed: $final_failed"
    echo "  Running: $final_running"
    echo "  Pending: $final_pending"
    if [ "$final_total" -gt 0 ]; then
        echo "  Success rate: $(echo "scale=1; ($final_completed * 100) / $final_total" | bc -l 2>/dev/null || echo "0")%"
    fi
    echo ""
    
    echo "Resilience Metrics:"
    echo "  Estimated retry events: $retry_attempts"
    echo "  Total duration: ${total_duration}s"
    
    # Determine overall result
    local all_completed=$((final_completed + final_failed == final_total))
    local deadline_met=$((total_duration <= DEADLINE_SECONDS))
    local good_completion_rate=0
    if [ "$final_total" -gt 0 ]; then
        good_completion_rate=$(echo "($final_completed * 100) / $final_total >= 80" | bc -l 2>/dev/null || echo "0")
    fi
    
    echo ""
    if [ "$all_completed" -eq 1 ] && [ "$good_completion_rate" -eq 1 ]; then
        echo -e "${GREEN}✅ SCENARIO PASSED${NC}"
        echo "  - All tasks processed"
        echo "  - Good completion rate (≥80%)"
        if [ "$deadline_met" -eq 1 ]; then
            echo "  - Completed within deadline"
        fi
    else
        echo -e "${RED}❌ SCENARIO FAILED${NC}"
        [ "$all_completed" -eq 0 ] && echo "  - Not all tasks processed"
        [ "$good_completion_rate" -eq 0 ] && echo "  - Poor completion rate (<80%)"
    fi
    
    # Get recent task details for additional insights
    if [ "$VERBOSE" = true ]; then
        echo ""
        echo "Recent Task Details:"
        get_recent_tasks 10
    fi
    
    # Output JSON summary
    echo ""
    echo "{\"total\": $final_total, \"completed\": $final_completed, \"failed\": $final_failed, \"running\": $final_running, \"pending\": $final_pending, \"duration\": $total_duration, \"deadline\": $DEADLINE_SECONDS, \"deadline_met\": $([ $deadline_met -eq 1 ] && echo "true" || echo "false"), \"retry_attempts\": $retry_attempts}"
}

# Test admin CLI connectivity
test_admin_cli() {
    echo "🧪 Testing admin CLI connectivity..."
    
    # Test basic admin command
    echo "Testing basic task-stats command:"
    docker exec "$CONTAINER_NAME" /app/starter admin task-stats 2>&1 || echo "Failed to execute admin task-stats"
    
    echo ""
    echo "Testing list-tasks command:"
    docker exec "$CONTAINER_NAME" /app/starter admin list-tasks --limit 5 2>&1 || echo "Failed to execute admin list-tasks"
    
    echo ""
    echo "Testing binary path:"
    docker exec "$CONTAINER_NAME" ls -la /app/target/release/ 2>&1 || echo "Failed to list release directory"
    
    echo ""
    echo "Testing container file system:"
    docker exec "$CONTAINER_NAME" ls -la /app/ 2>&1 || echo "Failed to list app directory"
    
    echo ""
}

# Main execution
check_container

echo -e "${PURPLE}🔍 Task Completion Monitor (Admin CLI)${NC}"
echo "Container: $CONTAINER_NAME"
echo "Prefix: $TASK_TAG"
echo "Deadline: ${DEADLINE_SECONDS}s"
echo ""

if [ "$VERBOSE" = true ]; then
    test_admin_cli
fi

monitor_tasks