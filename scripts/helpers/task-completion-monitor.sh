#!/bin/bash

# Task Completion Monitor
# Monitors task completion and verifies all tasks are processed within deadlines

set -e

# Default values
BASE_URL="${BASE_URL:-http://localhost:8888}"
TASK_PREFIX="${TASK_PREFIX:-chaos}"
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
    echo "Monitor task completion for chaos testing scenarios"
    echo ""
    echo "Options:"
    echo "  -u, --url URL             API base URL (default: $BASE_URL)"
    echo "  -p, --prefix PREFIX       Task prefix to monitor (default: $TASK_PREFIX)"
    echo "  -d, --deadline SECONDS    Task deadline in seconds (default: $DEADLINE_SECONDS)"
    echo "  -i, --interval SECONDS    Check interval (default: $CHECK_INTERVAL)"
    echo "  -b, --buffer SECONDS      Timeout buffer beyond deadline (default: $TIMEOUT_BUFFER)"
    echo "  -a, --auth TOKEN          Authentication token (required)"
    echo "  -v, --verbose             Verbose output"
    echo "  -h, --help                Show this help"
    echo ""
    echo "The monitor will:"
    echo "  - Track all tasks with the specified prefix"
    echo "  - Monitor completion status within deadline"
    echo "  - Report on retry behavior and worker resilience"
    echo "  - Provide detailed statistics on task processing"
}

# Parse arguments
AUTH_TOKEN=""
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -p|--prefix)
            TASK_PREFIX="$2"
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
        -a|--auth)
            if [ "$VERBOSE" = true ]; then
                echo "DEBUG: Setting AUTH_TOKEN from parameter: '${2:0:20}...'"
            fi
            AUTH_TOKEN="$2"
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

if [ -z "$AUTH_TOKEN" ]; then
    echo "Error: Authentication token required (-a/--auth)" >&2
    usage >&2
    exit 1
fi

# Show debug info if verbose
if [ "$VERBOSE" = true ]; then
    echo "DEBUG: Monitoring tasks with prefix: '$TASK_PREFIX'"
    echo "DEBUG: Using AUTH_TOKEN: '${AUTH_TOKEN:0:20}...'"
    echo "DEBUG: Base URL: $BASE_URL"
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

# Get all tasks with the specified prefix
get_tasks_by_prefix() {
    local response=$(curl -s "$BASE_URL/tasks?limit=1000" \
        -H "Authorization: Bearer $AUTH_TOKEN" 2>/dev/null || echo "")
    
    if [ -z "$response" ]; then
        echo "[]"
        return 1
    fi
    
    # Filter tasks by prefix in metadata or task payload with detailed analysis
    echo "$response" | python3 -c "
import json, sys
from collections import Counter
verbose = '${VERBOSE}' == 'true'
prefix = '${TASK_PREFIX}'

try:
    data = json.load(sys.stdin)
    tasks = data.get('data', []) if isinstance(data, dict) else data
    filtered_tasks = []
    
    # Analyze all tasks first
    task_types = Counter()
    status_counts = Counter() 
    worker_counts = Counter()
    status_by_type = {}  # {task_type: {status: count}}
    
    for task in tasks:
        task_type = task.get('task_type', 'unknown')
        status = task.get('status', 'unknown')
        metadata = task.get('metadata', {})
        
        task_types[task_type] += 1
        status_counts[status] += 1
        
        # Track status distribution per task type
        if task_type not in status_by_type:
            status_by_type[task_type] = Counter()
        status_by_type[task_type][status] += 1
        
        # Count workers if present in metadata
        if isinstance(metadata, dict):
            worker_id = metadata.get('worker_id', metadata.get('processed_by', 'none'))
            if worker_id != 'none':
                worker_counts[worker_id] += 1
    
    if verbose:
        print(f'DEBUG: === PRE-FILTER ANALYSIS ===', file=sys.stderr)
        print(f'DEBUG: Total tasks in system: {len(tasks)}', file=sys.stderr)
        print(f'DEBUG: Task types: {dict(task_types)}', file=sys.stderr)
        print(f'DEBUG: Overall status distribution: {dict(status_counts)}', file=sys.stderr)
        print(f'DEBUG: Status distribution per task type:', file=sys.stderr)
        for task_type, statuses in sorted(status_by_type.items()):
            print(f'DEBUG:   {task_type}: {dict(statuses)}', file=sys.stderr)
        if worker_counts:
            print(f'DEBUG: Worker distribution: {dict(worker_counts)}', file=sys.stderr)
        else:
            print(f'DEBUG: No worker information found in task metadata', file=sys.stderr)
        print(f'DEBUG: Looking for prefix: \"{prefix}\"', file=sys.stderr)
        print(f'DEBUG: === FILTERING PROCESS ===', file=sys.stderr)
    
    # Now filter tasks
    matching_task_types = Counter()
    matching_status_counts = Counter()
    matching_worker_counts = Counter()
    matching_status_by_type = {}  # {task_type: {status: count}}
    
    for task in tasks:
        task_data = task.get('payload', {})
        metadata = task.get('metadata', {})
        task_id = task.get('id', '')[:8]
        task_type = task.get('task_type', 'unknown')
        status = task.get('status', 'unknown')
        
        # Check if it's our test task - look in metadata for task_prefix
        matches_prefix = (isinstance(metadata, dict) and 
                         metadata.get('task_prefix') == prefix)
        matches_task_id = (isinstance(metadata, dict) and 
                          isinstance(metadata.get('task_id', ''), str) and
                          metadata.get('task_id', '').startswith(prefix))
        
        if matches_prefix or matches_task_id:
            if verbose:
                worker_info = metadata.get('worker_id', metadata.get('processed_by', 'none'))
                print(f'DEBUG: ✓ Match {task_id}: type={task_type}, status={status}, worker={worker_info}, prefix={metadata.get(\"task_prefix\", \"none\")}', file=sys.stderr)
            
            filtered_tasks.append(task)
            matching_task_types[task_type] += 1
            matching_status_counts[status] += 1
            
            # Track status distribution per task type for matching tasks
            if task_type not in matching_status_by_type:
                matching_status_by_type[task_type] = Counter()
            matching_status_by_type[task_type][status] += 1
            
            # Count workers in matching tasks
            if isinstance(metadata, dict):
                worker_id = metadata.get('worker_id', metadata.get('processed_by', 'none'))
                if worker_id != 'none':
                    matching_worker_counts[worker_id] += 1
                    
        elif verbose and len(filtered_tasks) == 0 and len([t for t in tasks if t.get('id') == task.get('id')]) <= 3:
            print(f'DEBUG: ✗ No match {task_id}: type={task_type}, prefix={metadata.get(\"task_prefix\", \"none\")}, task_id={metadata.get(\"task_id\", \"none\")}', file=sys.stderr)
    
    if verbose:
        print(f'DEBUG: === POST-FILTER ANALYSIS ===', file=sys.stderr)
        print(f'DEBUG: Filtered to {len(filtered_tasks)} matching tasks', file=sys.stderr)
        if matching_task_types:
            print(f'DEBUG: Matching task types: {dict(matching_task_types)}', file=sys.stderr)
        if matching_status_counts:
            print(f'DEBUG: Matching overall status distribution: {dict(matching_status_counts)}', file=sys.stderr)
            print(f'DEBUG: Matching status distribution per task type:', file=sys.stderr)
            for task_type, statuses in sorted(matching_status_by_type.items()):
                print(f'DEBUG:   {task_type}: {dict(statuses)}', file=sys.stderr)
        if matching_worker_counts:
            print(f'DEBUG: Matching worker distribution: {dict(matching_worker_counts)}', file=sys.stderr)
        else:
            print(f'DEBUG: No worker information in matching tasks', file=sys.stderr)
        print(f'DEBUG: === END ANALYSIS ===', file=sys.stderr)
    
    print(json.dumps(filtered_tasks))
except Exception as e:
    if verbose:
        print(f'DEBUG: Error filtering tasks: {e}', file=sys.stderr)
    print('[]')
"
}

# Get task statistics
get_task_stats() {
    local tasks_json="$1"
    
    echo "$tasks_json" | python3 -c "
import json, sys
from datetime import datetime, timezone
import time

try:
    tasks = json.load(sys.stdin)
    stats = {
        'total': len(tasks),
        'pending': 0,
        'running': 0,
        'completed': 0,
        'failed': 0,
        'retrying': 0,
        'cancelled': 0,
        'within_deadline': 0,
        'exceeded_deadline': 0,
        'retry_attempts': 0,
        'avg_completion_time': 0
    }
    
    completion_times = []
    current_time = time.time()
    
    for task in tasks:
        status = task.get('status', '').lower()
        stats[status] = stats.get(status, 0) + 1
        
        # Count retry attempts
        current_attempt = task.get('current_attempt', 1)
        if current_attempt > 1:
            stats['retry_attempts'] += (current_attempt - 1)
        
        # Check deadline compliance for completed tasks
        if status == 'completed':
            created_at = task.get('created_at')
            completed_at = task.get('completed_at')
            
            if created_at and completed_at:
                try:
                    created_time = datetime.fromisoformat(created_at.replace('Z', '+00:00')).timestamp()
                    completed_time = datetime.fromisoformat(completed_at.replace('Z', '+00:00')).timestamp()
                    completion_duration = completed_time - created_time
                    completion_times.append(completion_duration)
                    
                    if completion_duration <= $DEADLINE_SECONDS:
                        stats['within_deadline'] += 1
                    else:
                        stats['exceeded_deadline'] += 1
                except:
                    pass
    
    if completion_times:
        stats['avg_completion_time'] = sum(completion_times) / len(completion_times)
    
    print(json.dumps(stats, indent=2))
except Exception as e:
    print('{\"error\": \"' + str(e) + '\"}')
"
}

# Monitor task progress
monitor_tasks() {
    local start_time=$(date +%s)
    local end_time=$((start_time + DEADLINE_SECONDS + TIMEOUT_BUFFER))
    local last_stats=""
    
    log "INFO" "Starting task completion monitoring..."
    log "INFO" "Deadline: ${DEADLINE_SECONDS}s, Buffer: ${TIMEOUT_BUFFER}s, Check interval: ${CHECK_INTERVAL}s"
    log "INFO" "Monitoring tasks with prefix: $TASK_PREFIX"
    
    echo -e "${BLUE}📊 Task Progress Monitoring${NC}"
    echo "=================================="
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time=$(date +%s)
        local elapsed_time=$((current_time - start_time))
        local remaining_time=$((end_time - current_time))
        
        # Get current task status
        local tasks_json=$(get_tasks_by_prefix)
        local stats_json=$(get_task_stats "$tasks_json")
        
        # Parse stats
        local total=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))")
        local completed=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))")
        local failed=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('failed', 0))")
        local running=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('running', 0))")
        local pending=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('pending', 0))")
        local retrying=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('retrying', 0))")
        local within_deadline=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('within_deadline', 0))")
        local exceeded_deadline=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('exceeded_deadline', 0))")
        local retry_attempts=$(echo "$stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('retry_attempts', 0))")
        
        # Display progress
        local progress=""
        if [ "$total" -gt 0 ]; then
            local completion_rate=$(echo "scale=1; ($completed * 100) / $total" | bc -l 2>/dev/null || echo "0")
            progress=" (${completion_rate}%)"
        fi
        
        echo -e "⏱️  ${elapsed_time}s elapsed | ${remaining_time}s remaining"
        echo -e "📋 Tasks: ${total} total | ${GREEN}${completed}${NC} completed${progress} | ${RED}${failed}${NC} failed"
        echo -e "🔄 Active: ${YELLOW}${running}${NC} running | ${BLUE}${pending}${NC} pending | ${PURPLE}${retrying}${NC} retrying"
        
        if [ "$completed" -gt 0 ]; then
            echo -e "⏰ Deadline: ${GREEN}${within_deadline}${NC} within | ${RED}${exceeded_deadline}${NC} exceeded"
        fi
        
        if [ "$retry_attempts" -gt 0 ]; then
            echo -e "🔁 Retry attempts: $retry_attempts"
        fi
        
        # Check if all tasks are complete
        local terminal_tasks=$((completed + failed))
        if [ "$total" -gt 0 ] && [ "$terminal_tasks" -eq "$total" ]; then
            log "SUCCESS" "All tasks completed!"
            break
        fi
        
        # Check if we're past the deadline
        if [ $elapsed_time -gt $DEADLINE_SECONDS ]; then
            local active_tasks=$((running + pending + retrying))
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
    echo -e "${PURPLE}📋 Final Monitoring Report${NC}"
    echo "=================================="
    
    # Get final stats
    local final_tasks_json=$(get_tasks_by_prefix)
    local final_stats_json=$(get_task_stats "$final_tasks_json")
    
    echo "Monitoring duration: ${total_duration}s"
    echo "Task deadline: ${DEADLINE_SECONDS}s"
    echo ""
    
    # Parse final stats
    local final_total=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('total', 0))")
    local final_completed=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('completed', 0))")
    local final_failed=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('failed', 0))")
    local final_within_deadline=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('within_deadline', 0))")
    local final_exceeded_deadline=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('exceeded_deadline', 0))")
    local final_retry_attempts=$(echo "$final_stats_json" | python3 -c "import json,sys; print(json.load(sys.stdin).get('retry_attempts', 0))")
    local avg_completion_time=$(echo "$final_stats_json" | python3 -c "import json,sys; print(round(json.load(sys.stdin).get('avg_completion_time', 0), 2))")
    
    echo "Task Summary:"
    echo "  Total tasks: $final_total"
    echo "  Completed: $final_completed"
    echo "  Failed: $final_failed"
    echo "  Success rate: $(echo "scale=1; ($final_completed * 100) / $final_total" | bc -l 2>/dev/null || echo "0")%"
    echo ""
    
    echo "Deadline Compliance:"
    echo "  Within deadline: $final_within_deadline"
    echo "  Exceeded deadline: $final_exceeded_deadline"
    if [ "$final_completed" -gt 0 ]; then
        echo "  Deadline compliance: $(echo "scale=1; ($final_within_deadline * 100) / $final_completed" | bc -l 2>/dev/null || echo "0")%"
    fi
    echo ""
    
    echo "Resilience Metrics:"
    echo "  Total retry attempts: $final_retry_attempts"
    echo "  Average completion time: ${avg_completion_time}s"
    
    # Determine overall result
    local all_completed=$((final_completed + final_failed == final_total))
    local deadline_met=$((total_duration <= DEADLINE_SECONDS))
    local good_completion_rate=$(echo "($final_completed * 100) / $final_total >= 80" | bc -l 2>/dev/null || echo "0")
    
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
    
    # Output JSON summary
    echo ""
    echo "{\"total\": $final_total, \"completed\": $final_completed, \"failed\": $final_failed, \"duration\": $total_duration, \"deadline\": $DEADLINE_SECONDS, \"deadline_met\": $([ $deadline_met -eq 1 ] && echo "true" || echo "false"), \"retry_attempts\": $final_retry_attempts, \"avg_completion_time\": $avg_completion_time, \"within_deadline\": $final_within_deadline, \"exceeded_deadline\": $final_exceeded_deadline}"
}

# Main execution
echo -e "${PURPLE}🔍 Task Completion Monitor${NC}"
echo "Target: $BASE_URL"
echo "Prefix: $TASK_PREFIX"
echo "Deadline: ${DEADLINE_SECONDS}s"
echo ""

monitor_tasks