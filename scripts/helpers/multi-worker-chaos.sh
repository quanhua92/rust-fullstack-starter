#!/bin/bash

# Multi-Worker Chaos Helper
# Manages multiple workers and simulates random worker failures

set -e

# Default values
WORKER_COUNT="${WORKER_COUNT:-3}"
CHAOS_DURATION="${CHAOS_DURATION:-60}"
MIN_STOP_INTERVAL="${MIN_STOP_INTERVAL:-10}"
MAX_STOP_INTERVAL="${MAX_STOP_INTERVAL:-20}"
RESTART_DELAY="${RESTART_DELAY:-5}"
ACTION="${ACTION:-start}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Get project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Worker tracking
WORKER_PIDS=()
WORKER_LOG_DIR="/tmp/multi-worker-chaos"
WORKER_STATE_FILE="/tmp/multi-worker-state.json"

usage() {
    echo "Usage: $0 [ACTION] [OPTIONS]"
    echo ""
    echo "Manage multiple workers for chaos testing"
    echo ""
    echo "Actions:"
    echo "  start-multi    Start multiple workers"
    echo "  stop-all       Stop all managed workers"
    echo "  chaos-run      Start workers and run chaos scenario"
    echo "  status         Show worker status"
    echo "  cleanup        Clean up worker files and processes"
    echo ""
    echo "Options:"
    echo "  -w, --workers COUNT       Number of workers (default: $WORKER_COUNT)"
    echo "  -d, --duration SECONDS    Chaos test duration (default: $CHAOS_DURATION)"
    echo "  -s, --min-stop SECONDS    Min interval between stops (default: $MIN_STOP_INTERVAL)"
    echo "  -S, --max-stop SECONDS    Max interval between stops (default: $MAX_STOP_INTERVAL)"
    echo "  -r, --restart-delay SEC   Delay before restarting worker (default: $RESTART_DELAY)"
    echo "  -v, --verbose             Verbose output"
    echo "  -h, --help                Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 start-multi -w 4                    # Start 4 workers"
    echo "  $0 chaos-run -w 3 -d 120 -s 15 -S 30  # 3 workers, 2min chaos, 15-30s stops"
    echo "  $0 stop-all                            # Stop all workers"
    echo "  $0 status                              # Check worker status"
}

# Parse arguments
VERBOSE=false

# DEBUG: Show all received arguments
echo "DEBUG: multi-worker-chaos.sh received $# arguments: $@"

# Get action from first argument if provided
if [[ $# -gt 0 ]] && [[ ! "$1" =~ ^- ]]; then
    ACTION="$1"
    echo "DEBUG: Setting ACTION from parameter: '$ACTION'"
    shift
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        -w|--workers)
            WORKER_COUNT="$2"
            shift 2
            ;;
        -d|--duration)
            CHAOS_DURATION="$2"
            shift 2
            ;;
        -s|--min-stop)
            MIN_STOP_INTERVAL="$2"
            shift 2
            ;;
        -S|--max-stop)
            MAX_STOP_INTERVAL="$2"
            shift 2
            ;;
        -r|--restart-delay)
            RESTART_DELAY="$2"
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
        echo "[$timestamp] $level: $message" >> "$WORKER_LOG_DIR/multi-worker.log"
    fi
}

# Setup worker directories
setup_worker_environment() {
    mkdir -p "$WORKER_LOG_DIR"
    rm -f "$WORKER_STATE_FILE"
    
    # Initialize worker state
    echo "{\"workers\": [], \"start_time\": $(date +%s), \"chaos_events\": []}" > "$WORKER_STATE_FILE"
}

# Start a single worker with unique ID
start_worker() {
    local worker_id="$1"
    local worker_pid_file="/tmp/starter-worker-${worker_id}.pid"
    local worker_log_file="$WORKER_LOG_DIR/worker-${worker_id}.log"
    
    log "INFO" "Starting worker $worker_id..."
    
    # Stop any existing worker with this ID
    if [ -f "$worker_pid_file" ]; then
        local old_pid=$(cat "$worker_pid_file" 2>/dev/null || echo "")
        if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
            log "WARN" "Killing existing worker $worker_id (PID: $old_pid)"
            kill -TERM "$old_pid" 2>/dev/null || kill -9 "$old_pid" 2>/dev/null || true
            sleep 1
        fi
        rm -f "$worker_pid_file"
    fi
    
    # Start new worker
    bash -c "cd '$PROJECT_ROOT/starter' && exec cargo run -- worker" > "$worker_log_file" 2>&1 &
    local worker_pid=$!
    
    # Save PID
    echo $worker_pid > "$worker_pid_file"
    WORKER_PIDS+=("$worker_pid")
    
    # Update state file
    local worker_info="{\"id\": $worker_id, \"pid\": $worker_pid, \"pid_file\": \"$worker_pid_file\", \"log_file\": \"$worker_log_file\", \"started_at\": $(date +%s), \"status\": \"running\"}"
    python3 -c "
import json, sys
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
state['workers'].append($worker_info)
with open('$WORKER_STATE_FILE', 'w') as f:
    json.dump(state, f, indent=2)
"
    
    # Quick validation
    sleep 2
    if kill -0 $worker_pid 2>/dev/null; then
        log "SUCCESS" "Worker $worker_id started (PID: $worker_pid)"
        return 0
    else
        log "ERROR" "Worker $worker_id failed to start"
        rm -f "$worker_pid_file"
        return 1
    fi
}

# Stop a specific worker
stop_worker() {
    local worker_id="$1"
    local worker_pid_file="/tmp/starter-worker-${worker_id}.pid"
    
    if [ ! -f "$worker_pid_file" ]; then
        log "WARN" "Worker $worker_id PID file not found"
        return 1
    fi
    
    local worker_pid=$(cat "$worker_pid_file" 2>/dev/null || echo "")
    if [ -z "$worker_pid" ]; then
        log "WARN" "Worker $worker_id PID file empty"
        return 1
    fi
    
    if ! kill -0 "$worker_pid" 2>/dev/null; then
        log "WARN" "Worker $worker_id (PID: $worker_pid) not running"
        rm -f "$worker_pid_file"
        return 1
    fi
    
    log "INFO" "Stopping worker $worker_id (PID: $worker_pid)..."
    
    # Graceful shutdown
    kill -TERM "$worker_pid" 2>/dev/null || true
    
    # Wait for graceful shutdown
    for i in {1..10}; do
        if ! kill -0 "$worker_pid" 2>/dev/null; then
            break
        fi
        sleep 1
    done
    
    # Force kill if still running
    if kill -0 "$worker_pid" 2>/dev/null; then
        log "WARN" "Force killing worker $worker_id"
        kill -9 "$worker_pid" 2>/dev/null || true
    fi
    
    rm -f "$worker_pid_file"
    
    # Update state file
    python3 -c "
import json, sys
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
for worker in state['workers']:
    if worker['id'] == $worker_id:
        worker['status'] = 'stopped'
        worker['stopped_at'] = $(date +%s)
        break
with open('$WORKER_STATE_FILE', 'w') as f:
    json.dump(state, f, indent=2)
"
    
    log "SUCCESS" "Worker $worker_id stopped"
    return 0
}

# Get random worker ID from running workers
get_random_running_worker() {
    local running_workers=()
    
    for worker_id in $(seq 1 $WORKER_COUNT); do
        local worker_pid_file="/tmp/starter-worker-${worker_id}.pid"
        if [ -f "$worker_pid_file" ]; then
            local worker_pid=$(cat "$worker_pid_file" 2>/dev/null || echo "")
            if [ -n "$worker_pid" ] && kill -0 "$worker_pid" 2>/dev/null; then
                running_workers+=("$worker_id")
            fi
        fi
    done
    
    if [ ${#running_workers[@]} -eq 0 ]; then
        echo ""
        return 1
    fi
    
    # Select random worker
    local random_index=$((RANDOM % ${#running_workers[@]}))
    echo "${running_workers[$random_index]}"
    return 0
}

# Random chaos events
run_chaos_scenario() {
    local start_time=$(date +%s)
    local end_time=$((start_time + CHAOS_DURATION))
    local next_chaos_time=$((start_time + $(shuf -i $MIN_STOP_INTERVAL-$MAX_STOP_INTERVAL -n 1)))
    
    log "INFO" "Starting chaos scenario for ${CHAOS_DURATION}s..."
    log "INFO" "Stop intervals: ${MIN_STOP_INTERVAL}-${MAX_STOP_INTERVAL}s, restart delay: ${RESTART_DELAY}s"
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time=$(date +%s)
        
        if [ $current_time -ge $next_chaos_time ]; then
            # Time for chaos event
            local victim_worker=$(get_random_running_worker)
            
            if [ -n "$victim_worker" ]; then
                log "WARN" "🔥 CHAOS EVENT: Stopping worker $victim_worker"
                
                # Record chaos event
                local chaos_event="{\"type\": \"worker_stop\", \"worker_id\": $victim_worker, \"timestamp\": $current_time}"
                python3 -c "
import json, sys
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
state['chaos_events'].append($chaos_event)
with open('$WORKER_STATE_FILE', 'w') as f:
    json.dump(state, f, indent=2)
"
                
                stop_worker "$victim_worker"
                
                # For extreme chaos (when min/max intervals are very short), 
                # sometimes don't restart workers to create catastrophic failure
                local should_restart=true
                if [ $MIN_STOP_INTERVAL -le 3 ] && [ $MAX_STOP_INTERVAL -le 6 ]; then
                    # 50% chance of not restarting in extreme chaos mode
                    if [ $((RANDOM % 10)) -lt 5 ]; then
                        should_restart=false
                        log "WARN" "🚨 EXTREME CHAOS: Worker $victim_worker will NOT be restarted (catastrophic failure mode)"
                    fi
                fi
                
                if [ "$should_restart" = true ]; then
                    # Wait before restarting
                    if [ $RESTART_DELAY -gt 0 ]; then
                        log "INFO" "Waiting ${RESTART_DELAY}s before restart..."
                        sleep $RESTART_DELAY
                    fi
                    
                    # Restart worker
                    start_worker "$victim_worker"
                    
                    # Record restart event
                    local restart_event="{\"type\": \"worker_restart\", \"worker_id\": $victim_worker, \"timestamp\": $(date +%s)}"
                    python3 -c "
import json, sys
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
state['chaos_events'].append($restart_event)
with open('$WORKER_STATE_FILE', 'w') as f:
    json.dump(state, f, indent=2)
"
                else
                    # Record permanent stop event
                    local perm_stop_event="{\"type\": \"worker_permanent_stop\", \"worker_id\": $victim_worker, \"timestamp\": $(date +%s)}"
                    python3 -c "
import json, sys
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
state['chaos_events'].append($perm_stop_event)
with open('$WORKER_STATE_FILE', 'w') as f:
    json.dump(state, f, indent=2)
"
                fi
            else
                log "WARN" "No running workers found for chaos event"
            fi
            
            # Schedule next chaos event
            next_chaos_time=$((current_time + $(shuf -i $MIN_STOP_INTERVAL-$MAX_STOP_INTERVAL -n 1)))
        fi
        
        sleep 2
    done
    
    log "SUCCESS" "Chaos scenario completed"
}

# Show status of all workers
show_worker_status() {
    echo -e "${BLUE}🔧 Multi-Worker Status${NC}"
    echo "=================================="
    
    if [ ! -f "$WORKER_STATE_FILE" ]; then
        echo "No worker state file found. Run 'start-multi' first."
        return 1
    fi
    
    local running_count=0
    local total_count=0
    
    for worker_id in $(seq 1 $WORKER_COUNT); do
        local worker_pid_file="/tmp/starter-worker-${worker_id}.pid"
        total_count=$((total_count + 1))
        
        if [ -f "$worker_pid_file" ]; then
            local worker_pid=$(cat "$worker_pid_file" 2>/dev/null || echo "")
            if [ -n "$worker_pid" ] && kill -0 "$worker_pid" 2>/dev/null; then
                echo -e "Worker $worker_id: ${GREEN}RUNNING${NC} (PID: $worker_pid)"
                running_count=$((running_count + 1))
            else
                echo -e "Worker $worker_id: ${RED}STOPPED${NC} (stale PID file)"
                rm -f "$worker_pid_file"
            fi
        else
            echo -e "Worker $worker_id: ${YELLOW}NOT STARTED${NC}"
        fi
    done
    
    echo ""
    echo "Summary: $running_count/$total_count workers running"
    
    # Show chaos events if available
    if [ -f "$WORKER_STATE_FILE" ]; then
        local event_count=$(python3 -c "
import json
with open('$WORKER_STATE_FILE', 'r') as f:
    state = json.load(f)
print(len(state.get('chaos_events', [])))
" 2>/dev/null || echo "0")
        echo "Chaos events recorded: $event_count"
    fi
}

# Clean up all workers and files
cleanup_workers() {
    log "INFO" "Cleaning up all workers and files..."
    
    # Stop all workers
    for worker_id in $(seq 1 10); do  # Check up to 10 workers
        stop_worker "$worker_id" 2>/dev/null || true
    done
    
    # Clean up remaining processes
    pkill -f "starter worker" 2>/dev/null || true
    
    # Clean up files
    rm -f /tmp/starter-worker-*.pid
    rm -f "$WORKER_STATE_FILE"
    rm -rf "$WORKER_LOG_DIR"
    
    log "SUCCESS" "Cleanup completed"
}

# Execute action
case "$ACTION" in
    start-multi)
        setup_worker_environment
        log "INFO" "Starting $WORKER_COUNT workers..."
        
        for worker_id in $(seq 1 $WORKER_COUNT); do
            start_worker "$worker_id"
            sleep 1  # Stagger starts slightly
        done
        
        show_worker_status
        ;;
        
    stop-all)
        log "INFO" "Stopping all workers..."
        for worker_id in $(seq 1 $WORKER_COUNT); do
            stop_worker "$worker_id" 2>/dev/null || true
        done
        ;;
        
    chaos-run)
        setup_worker_environment
        
        # Start workers
        log "INFO" "Starting $WORKER_COUNT workers for chaos scenario..."
        for worker_id in $(seq 1 $WORKER_COUNT); do
            start_worker "$worker_id"
            sleep 1
        done
        
        show_worker_status
        
        # Run chaos scenario
        run_chaos_scenario
        
        show_worker_status
        ;;
        
    status)
        show_worker_status
        ;;
        
    cleanup)
        cleanup_workers
        ;;
        
    *)
        echo "Unknown action: $ACTION" >&2
        usage >&2
        exit 1
        ;;
esac

log "SUCCESS" "Multi-worker chaos action completed: $ACTION"