#!/bin/bash

# Task Flood Helper for Chaos Testing
# Creates a flood of tasks to test system performance

set -e

# Default values
BASE_URL="${BASE_URL:-http://localhost:3000}"
TASK_COUNT="${TASK_COUNT:-50}"
TASK_TYPE="${TASK_TYPE:-email}"
DELAY="${DELAY:-0.1}"
PRIORITY="${PRIORITY:-normal}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Create a flood of tasks for load testing"
    echo ""
    echo "Options:"
    echo "  -u, --url URL          API base URL (default: $BASE_URL)"
    echo "  -c, --count COUNT      Number of tasks to create (default: $TASK_COUNT)"
    echo "  -t, --type TYPE        Task type (default: $TASK_TYPE)"
    echo "  -d, --delay SECONDS    Delay between tasks (default: $DELAY)"
    echo "  -p, --priority PRIO    Task priority (default: $PRIORITY)"
    echo "  -a, --auth TOKEN       Authentication token (required)"
    echo "  -f, --fail             Create failing tasks instead"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Task Types:"
    echo "  email                  Email tasks (default)"
    echo "  webhook                Webhook calls"
    echo "  data_processing        Data processing tasks"
    echo "  file_cleanup           File cleanup tasks"
    echo "  report_generation      Report generation tasks"
    echo "  failing_task           Invalid task type (for testing failures)"
}

# Parse arguments
AUTH_TOKEN=""
CREATE_FAILING=false
VERBOSE=false

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
        -t|--type)
            TASK_TYPE="$2"
            shift 2
            ;;
        -d|--delay)
            DELAY="$2"
            shift 2
            ;;
        -p|--priority)
            PRIORITY="$2"
            shift 2
            ;;
        -a|--auth)
            AUTH_TOKEN="$2"
            shift 2
            ;;
        -f|--fail)
            CREATE_FAILING=true
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

if [ -z "$AUTH_TOKEN" ]; then
    echo "Error: Authentication token required (-a/--auth)" >&2
    usage >&2
    exit 1
fi

# Task payload templates
get_task_payload() {
    local task_type="$1"
    local index="$2"
    
    case "$task_type" in
        email)
            echo "{\"task_type\": \"email\", \"payload\": {\"to\": \"flood$index@example.com\", \"subject\": \"Flood Test $index\", \"body\": \"Load test email $index\"}, \"priority\": \"$PRIORITY\"}"
            ;;
        webhook)
            echo "{\"task_type\": \"webhook\", \"payload\": {\"url\": \"https://httpbin.org/post\", \"method\": \"POST\", \"payload\": {\"test\": \"flood$index\"}}, \"priority\": \"$PRIORITY\"}"
            ;;
        data_processing)
            echo "{\"task_type\": \"data_processing\", \"payload\": {\"operation\": \"sum\", \"data\": [1, 2, 3, $index]}, \"priority\": \"$PRIORITY\"}"
            ;;
        file_cleanup)
            echo "{\"task_type\": \"file_cleanup\", \"payload\": {\"file_path\": \"/tmp/flood_test_$index\", \"max_age_hours\": 1}, \"priority\": \"$PRIORITY\"}"
            ;;
        report_generation)
            echo "{\"task_type\": \"report_generation\", \"payload\": {\"report_type\": \"load_test\", \"index\": $index}, \"priority\": \"$PRIORITY\"}"
            ;;
        failing_task)
            echo "{\"task_type\": \"failing_task_type_$index\", \"payload\": {\"fail\": true}, \"priority\": \"$PRIORITY\"}"
            ;;
        *)
            echo "{\"task_type\": \"$task_type\", \"payload\": {\"index\": $index}, \"priority\": \"$PRIORITY\"}"
            ;;
    esac
}

# Override task type if creating failing tasks
if [ "$CREATE_FAILING" = true ]; then
    TASK_TYPE="failing_task"
fi

echo -e "${YELLOW}🌊 Starting task flood...${NC}"
echo "   Target: $BASE_URL"
echo "   Tasks: $TASK_COUNT"
echo "   Type: $TASK_TYPE"
echo "   Priority: $PRIORITY"
echo "   Delay: ${DELAY}s"
echo ""

# Track statistics
CREATED=0
FAILED=0
START_TIME=$(date +%s)

for i in $(seq 1 "$TASK_COUNT"); do
    TASK_DATA=$(get_task_payload "$TASK_TYPE" "$i")
    
    RESPONSE=$(curl -s -w "HTTP_STATUS:%{http_code}" -X POST "$BASE_URL/tasks" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d "$TASK_DATA")
    
    STATUS=$(echo "$RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    BODY=$(echo "$RESPONSE" | sed 's/HTTP_STATUS:[0-9]*$//')
    
    if [ "$STATUS" = "200" ]; then
        CREATED=$((CREATED + 1))
        if [ "$VERBOSE" = true ] || [ $((i % 10)) -eq 0 ]; then
            echo -e "   ${GREEN}✅${NC} Created task $i/$TASK_COUNT"
        fi
    else
        FAILED=$((FAILED + 1))
        if [ "$VERBOSE" = true ]; then
            echo -e "   ${RED}❌${NC} Failed task $i (Status: $STATUS)"
        fi
    fi
    
    # Add delay between requests
    if [ "$DELAY" != "0" ] && [ "$DELAY" != "0.0" ]; then
        sleep "$DELAY"
    fi
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "${GREEN}✅ Task flood completed${NC}"
echo "   Created: $CREATED/$TASK_COUNT"
echo "   Failed: $FAILED"
echo "   Duration: ${DURATION}s"
echo "   Rate: $(echo "scale=2; $TASK_COUNT / $DURATION" | bc 2>/dev/null || echo "N/A") tasks/sec"

# Output JSON summary
echo ""
echo "{\"created\": $CREATED, \"failed\": $FAILED, \"total\": $TASK_COUNT, \"duration\": $DURATION, \"task_type\": \"$TASK_TYPE\"}"