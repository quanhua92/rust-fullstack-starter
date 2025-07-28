#!/bin/bash

# Delay Task Flood Helper for Multi-Worker Chaos Testing
# Creates delay tasks with configurable deadlines for testing worker resilience

set -e

# Default values
BASE_URL="${BASE_URL:-http://localhost:8888}"
TASK_COUNT="${TASK_COUNT:-30}"
DELAY_DURATION="${DELAY_DURATION:-5}"     # How long each task should delay (seconds)
TASK_DEADLINE="${TASK_DEADLINE:-60}"      # Maximum time for all tasks to complete
PRIORITY="${PRIORITY:-normal}"
TASK_TAG="${TASK_TAG:-chaos}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
BOLD_RED='\033[1;31m'
NC='\033[0m'

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Create delay tasks with deadlines for multi-worker chaos testing"
    echo ""
    echo "Options:"
    echo "  -u, --url URL          API base URL (default: $BASE_URL)"
    echo "  -c, --count COUNT      Number of delay tasks to create (default: $TASK_COUNT)"
    echo "  -d, --delay SECONDS    Delay duration per task (default: $DELAY_DURATION)"
    echo "  -t, --deadline SECONDS Total deadline for all tasks (default: $TASK_DEADLINE)"
    echo "  -p, --priority PRIO    Task priority (default: $PRIORITY)"
    echo "  -a, --auth TOKEN       Authentication token (required)"
    echo "  -g, --tag TAG          Task identifier tag (default: $TASK_TAG)"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "The scenario creates tasks that:"
    echo "  - Each task delays for DELAY seconds before completing"
    echo "  - All tasks must complete within DEADLINE seconds"
    echo "  - If delays are too long vs deadline, some tasks will miss deadline"
    echo "  - Failed tasks should be retried by available workers"
    echo ""
    echo "Examples:"
    echo "  $0 -c 20 -d 3 -t 30 -a \$TOKEN    # 20 tasks, 3s delay each, 30s total deadline"
    echo "  $0 -c 50 -d 5 -t 60 -a \$TOKEN    # Stress test: might cause deadline misses"
}

# Parse arguments
AUTH_TOKEN=""
VERBOSE=false

# DEBUG: Show all received arguments
echo "DEBUG: delay-task-flood.sh received $# arguments: $@"

while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -c|--count)
            TASK_COUNT="$2"
            shift 2
            ;;
        -d|--delay)
            DELAY_DURATION="$2"
            shift 2
            ;;
        -t|--deadline)
            TASK_DEADLINE="$2"
            shift 2
            ;;
        -p|--priority)
            PRIORITY="$2"
            shift 2
            ;;
        -a|--auth)
            echo "DEBUG: Setting AUTH_TOKEN from parameter: '${2:0:20}...'"
            AUTH_TOKEN="$2"
            shift 2
            ;;
        -g|--tag)
            TASK_TAG="$2"
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

# DEBUG: Show final AUTH_TOKEN value
echo "DEBUG: Final AUTH_TOKEN length: ${#AUTH_TOKEN}, starts with: '${AUTH_TOKEN:0:20}...'"

if [ -z "$AUTH_TOKEN" ]; then
    echo "Error: Authentication token required (-a/--auth)" >&2
    echo "DEBUG: AUTH_TOKEN is empty after parsing!" >&2
    usage >&2
    exit 1
fi

# Validate parameters
if [ "$TASK_COUNT" -le 0 ]; then
    echo "Error: Task count must be positive" >&2
    exit 1
fi

if [ "$(echo "$DELAY_DURATION <= 0" | bc -l)" -eq 1 ]; then
    echo "Error: Delay duration must be positive" >&2
    exit 1
fi

if [ "$TASK_DEADLINE" -le 0 ]; then
    echo "Error: Task deadline must be positive" >&2
    exit 1
fi

# Calculate theoretical minimum time needed
MIN_TIME_NEEDED=$(echo "scale=2; $TASK_COUNT * $DELAY_DURATION" | bc -l)
WORKERS_NEEDED=$(echo "scale=0; ($MIN_TIME_NEEDED + $TASK_DEADLINE - 1) / $TASK_DEADLINE" | bc -l)

echo -e "${BLUE}🕐 Delay Task Flood Configuration${NC}"
echo "================================================="
echo "Target: $BASE_URL"
echo "Tasks: $TASK_COUNT"
echo "Delay per task: ${DELAY_DURATION}s"
echo "Total deadline: ${TASK_DEADLINE}s"
echo "Task tag: $TASK_TAG"
echo "Priority: $PRIORITY"
echo ""
echo -e "${YELLOW}📊 Theoretical Analysis:${NC}"
echo "Minimum time if sequential: ${MIN_TIME_NEEDED}s"
echo "Workers needed for deadline: $WORKERS_NEEDED"
if [ "$(echo "$MIN_TIME_NEEDED > $TASK_DEADLINE" | bc -l)" -eq 1 ]; then
    echo -e "${RED}⚠️  WARNING: Sequential execution would exceed deadline!${NC}"
    echo "   This will test worker failure and retry scenarios."
else
    echo -e "${GREEN}✅ Sequential execution within deadline.${NC}"
fi
echo ""

# Task payload template for delay tasks
get_delay_task_payload() {
    local index="$1"
    local task_id="${TASK_TAG}_delay_${index}"
    local deadline_timestamp=$(date -u -v+${TASK_DEADLINE}S +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "{
        \"task_type\": \"delay_task\",
        \"payload\": {
            \"delay_seconds\": $DELAY_DURATION,
            \"task_id\": \"$task_id\",
            \"deadline\": \"$deadline_timestamp\",
            \"test_scenario\": \"multi_worker_chaos\"
        },
        \"priority\": \"$PRIORITY\",
        \"metadata\": {
            \"chaos_test\": true,
            \"tag\": \"$TASK_TAG\",
            \"delay_duration\": $DELAY_DURATION,
            \"deadline_seconds\": $TASK_DEADLINE,
            \"created_for_worker_test\": true,
            \"task_id\": \"$task_id\"
        }
    }"
}

# Track statistics
CREATED=0
FAILED=0
START_TIME=$(date +%s)
TASK_IDS=()

echo -e "${YELLOW}🌊 Creating delay tasks...${NC}"

for i in $(seq 1 "$TASK_COUNT"); do
    TASK_DATA=$(get_delay_task_payload "$i")
    
    RESPONSE=$(curl -s -w "HTTP_STATUS:%{http_code}" -X POST "$BASE_URL/tasks" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d "$TASK_DATA")
    
    STATUS=$(echo "$RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    BODY=$(echo "$RESPONSE" | sed 's/HTTP_STATUS:[0-9]*$//')
    
    if [ "$STATUS" = "200" ] || [ "$STATUS" = "201" ]; then
        CREATED=$((CREATED + 1))
        # Extract task ID from response for tracking
        TASK_ID=$(echo "$BODY" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
        if [ -n "$TASK_ID" ]; then
            TASK_IDS+=("$TASK_ID")
        fi
        
        if [ "$VERBOSE" = true ] || [ $((i % 5)) -eq 0 ]; then
            echo -e "   ${GREEN}✅${NC} Created delay task $i/$TASK_COUNT (${DELAY_DURATION}s delay)"
        fi
    else
        FAILED=$((FAILED + 1))
        if [ "$VERBOSE" = true ]; then
            echo -e "   ${RED}❌${NC} Failed to create task $i (Status: $STATUS)"
            [ "$VERBOSE" = true ] && echo "   Response: $BODY"
        fi
    fi
done

CREATION_END_TIME=$(date +%s)
CREATION_DURATION=$((CREATION_END_TIME - START_TIME))

echo ""
echo -e "${GREEN}✅ Task creation completed${NC}"
echo "   Created: $CREATED/$TASK_COUNT"
echo "   Failed: $FAILED"
echo "   Creation time: ${CREATION_DURATION}s"
echo ""

# Wait for all tasks to complete or deadline to pass
echo -e "${BLUE}⏳ Monitoring task completion (deadline: ${TASK_DEADLINE}s)...${NC}"
DEADLINE_TIME=$((START_TIME + TASK_DEADLINE))
COMPLETED_TASKS=0
FAILED_TASKS=0
SAMPLE_SIZE=$CREATED

# Track completion statistics
while [ $COMPLETED_TASKS -lt $CREATED ] && [ $(date +%s) -lt $DEADLINE_TIME ]; do
    REMAINING_TIME=$((DEADLINE_TIME - $(date +%s)))
    
    # Use admin CLI helper to get task statistics
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    stats_output=$("$script_dir/admin-cli-helper.sh" task-stats --tag "$TASK_TAG" 2>/dev/null || echo "")
    
    if [ -n "$stats_output" ]; then
        SAMPLE_COMPLETED=$(echo "$stats_output" | grep "completed:" | awk '{print $2}' 2>/dev/null || echo "0")
        SAMPLE_FAILED=$(echo "$stats_output" | grep -E "failed:|cancelled:" | awk '{sum += $2} END {print sum}' 2>/dev/null || echo "0")
    else
        SAMPLE_COMPLETED=0
        SAMPLE_FAILED=0
    fi
    
    # Estimate total completion based on sample
    if [ "${SAMPLE_SIZE:-0}" -gt 0 ]; then
        COMPLETION_RATE=$(echo "scale=2; ($SAMPLE_COMPLETED * 100) / $SAMPLE_SIZE" | bc -l 2>/dev/null || echo "0")
        ESTIMATED_COMPLETED=$(echo "scale=0; ($COMPLETION_RATE * $CREATED) / 100" | bc -l 2>/dev/null || echo "0")
        echo -e "   ${YELLOW}📊${NC} Progress: ~${ESTIMATED_COMPLETED}/${CREATED} completed, ${REMAINING_TIME}s remaining"
        
        # Add diagnostic information if no progress after some time
        ELAPSED_TIME=$(($(date +%s) - START_TIME))
        
        # FAIL FAST: If no progress after 30 seconds, system is likely broken
        if [ $ELAPSED_TIME -gt 30 ] && [ $ESTIMATED_COMPLETED -eq 0 ]; then
            echo -e "   ${BOLD_RED}⚡ FAIL FAST:${NC} No task progress after 30s - system appears broken, aborting to save time"
            exit 1
        fi
        
        if [ $ELAPSED_TIME -gt 20 ] && [ $ESTIMATED_COMPLETED -eq 0 ]; then
            echo -e "   ${BOLD_RED}🔍 DIAGNOSTIC:${NC} No tasks completed after ${ELAPSED_TIME}s - checking task status..."
            
            # Check first few tasks for detailed status
            for diag_i in $(seq 1 3); do
                if [ $diag_i -le ${#TASK_IDS[@]} ]; then
                    DIAG_TASK_ID="${TASK_IDS[$((diag_i-1))]}"
                    DIAG_RESPONSE=$(curl -s "$BASE_URL/tasks/$DIAG_TASK_ID" \
                        -H "Authorization: Bearer $AUTH_TOKEN" 2>/dev/null || echo "")
                    
                    if [ -n "$DIAG_RESPONSE" ]; then
                        DIAG_STATUS=$(echo "$DIAG_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['status'])" 2>/dev/null || echo "unknown")
                        DIAG_TYPE=$(echo "$DIAG_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['task_type'])" 2>/dev/null || echo "unknown")
                        echo -e "   ${YELLOW}🔍${NC} Task $diag_i: ID=${DIAG_TASK_ID:0:8}..., Status=$DIAG_STATUS, Type=$DIAG_TYPE"
                        
                        # FAIL FAST: If multiple tasks show "unknown" status, this indicates API/system failure
                        if [ "$DIAG_STATUS" = "unknown" ] && [ "$DIAG_TYPE" = "unknown" ]; then
                            echo -e "   ${BOLD_RED}⚡ FAIL FAST:${NC} Task shows unknown status/type - API or system failure detected"
                            echo -e "   ${BOLD_RED}⚡ FAIL FAST:${NC} Aborting monitoring to save time - system appears broken"
                            exit 1
                        fi
                    else
                        echo -e "   ${RED}🔍${NC} Task $diag_i: Failed to fetch status (auth/network issue?)"
                        echo -e "   ${BOLD_RED}⚡ FAIL FAST:${NC} Cannot fetch task status - API communication failure"
                        exit 1
                    fi
                fi
            done
            
            # Check if workers are processing tasks at all
            echo -e "   ${YELLOW}🔍${NC} Checking overall task queue status via API..."
            QUEUE_RESPONSE=$(curl -s "$BASE_URL/tasks?limit=10" \
                -H "Authorization: Bearer $AUTH_TOKEN" 2>/dev/null || echo "")
            if [ -n "$QUEUE_RESPONSE" ]; then
                QUEUE_COUNT=$(echo "$QUEUE_RESPONSE" | python3 -c "import json,sys; print(len(json.load(sys.stdin)['data']))" 2>/dev/null || echo "0")
                echo -e "   ${YELLOW}🔍${NC} Recent tasks in queue: $QUEUE_COUNT"
            else
                echo -e "   ${RED}🔍${NC} Failed to check task queue (API issue?)"
            fi
        fi
    fi
    
    sleep 5
done

FINAL_TIME=$(date +%s)
TOTAL_DURATION=$((FINAL_TIME - START_TIME))

echo ""
echo -e "${PURPLE}📋 Final Results${NC}"
echo "================================================="
echo "Total duration: ${TOTAL_DURATION}s"
echo "Deadline: ${TASK_DEADLINE}s"

if [ $TOTAL_DURATION -le $TASK_DEADLINE ]; then
    echo -e "${GREEN}✅ Completed within deadline${NC}"
else
    echo -e "${RED}⏰ Exceeded deadline by $((TOTAL_DURATION - TASK_DEADLINE))s${NC}"
fi

# Output summary JSON
echo ""
echo "{\"created\": $CREATED, \"failed\": $FAILED, \"total\": $TASK_COUNT, \"duration\": $TOTAL_DURATION, \"deadline\": $TASK_DEADLINE, \"task_type\": \"delay_test\", \"delay_per_task\": $DELAY_DURATION, \"deadline_met\": $([ $TOTAL_DURATION -le $TASK_DEADLINE ] && echo "true" || echo "false")}"