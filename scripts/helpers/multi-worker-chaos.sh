#!/bin/bash

# Multi-Worker Chaos Helper
# Uses Docker Compose scaling to manage multiple workers

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

# Docker compose file for chaos testing
CHAOS_COMPOSE_FILE="$PROJECT_ROOT/docker-compose.chaos.yaml"

# Simple state tracking
CHAOS_LOG_DIR="/tmp/multi-worker-chaos"
CHAOS_EVENT_COUNT=0

usage() {
    echo "Usage: $0 [ACTION] [OPTIONS]"
    echo ""
    echo "Manage multiple workers using Docker Compose scaling"
    echo ""
    echo "Actions:"
    echo "  start-multi    Scale workers to specified count"
    echo "  stop-all       Scale workers to 0"
    echo "  chaos-run      Run chaos scenario with worker failures"
    echo "  status         Show worker status"
    echo "  cleanup        Clean up worker files"
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
    echo "  $0 start-multi -w 4                    # Scale to 4 workers"
    echo "  $0 chaos-run -w 3 -d 120 -s 15 -S 30  # 3 workers, 2min chaos, 15-30s stops"
    echo "  $0 stop-all                            # Scale to 0 workers"
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
        mkdir -p "$CHAOS_LOG_DIR"
        echo "[$timestamp] $level: $message" >> "$CHAOS_LOG_DIR/multi-worker.log"
    fi
}

# Setup chaos environment
setup_chaos_environment() {
    mkdir -p "$CHAOS_LOG_DIR"
    CHAOS_EVENT_COUNT=0
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Multi-worker chaos started with $WORKER_COUNT workers" > "$CHAOS_LOG_DIR/events.log"
}

# Scale workers using Docker Compose
scale_workers() {
    local target_count="$1"
    
    log "INFO" "Scaling workers to $target_count..."
    
    cd "$PROJECT_ROOT"
    
    # Use docker-compose scale command
    docker-compose -f "$CHAOS_COMPOSE_FILE" up -d --scale workers="$target_count"
    
    # Wait a moment for containers to stabilize
    sleep 3
    
    # Verify scaling using Docker ps
    local actual_count=$(docker ps --filter "name=rust-fullstack-starter-workers" --format "{{.Names}}" | wc -l)
    
    if [ "$actual_count" -eq "$target_count" ]; then
        log "SUCCESS" "Successfully scaled to $target_count workers"
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Scaled workers to $target_count (actual: $actual_count)" >> "$CHAOS_LOG_DIR/events.log"
        return 0
    else
        log "ERROR" "Scaling failed. Expected $target_count, got $actual_count workers"
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Scale FAILED: expected $target_count, got $actual_count" >> "$CHAOS_LOG_DIR/events.log"
        return 1
    fi
}

# Get list of running worker container names
get_running_workers() {
    docker ps --filter "name=rust-fullstack-starter-workers" --format "{{.Names}}"
}

# Get random worker container name
get_random_worker() {
    local workers_list=$(get_running_workers)
    
    if [ -z "$workers_list" ]; then
        echo ""
        return 1
    fi
    
    # Convert to array and select random worker
    local workers=($workers_list)
    local worker_count=${#workers[@]}
    
    if [ $worker_count -eq 0 ]; then
        echo ""
        return 1
    fi
    
    local random_index=$((RANDOM % worker_count))
    echo "${workers[$random_index]}"
    return 0
}

# Kill a specific worker container
kill_worker() {
    local worker_name="$1"
    
    if [ -z "$worker_name" ]; then
        log "ERROR" "No worker name provided"
        return 1
    fi
    
    log "WARN" "🔥 CHAOS EVENT: Killing worker container $worker_name"
    
    # Kill the container
    docker kill "$worker_name" >/dev/null 2>&1 || docker stop "$worker_name" >/dev/null 2>&1 || true
    
    # Record chaos event
    CHAOS_EVENT_COUNT=$((CHAOS_EVENT_COUNT + 1))
    echo "$(date '+%Y-%m-%d %H:%M:%S') - CHAOS EVENT $CHAOS_EVENT_COUNT: Killed worker $worker_name" >> "$CHAOS_LOG_DIR/events.log"
    
    log "SUCCESS" "Worker $worker_name killed"
    return 0
}

# Restart dead worker containers (Docker Compose will automatically recreate them)
restart_workers() {
    log "INFO" "Restarting any dead worker containers..."
    
    cd "$PROJECT_ROOT"
    
    # Docker Compose will automatically recreate killed containers when we run up again
    docker-compose -f "$CHAOS_COMPOSE_FILE" up -d workers >/dev/null 2>&1
    
    # Wait for containers to stabilize
    sleep 3
    
    local worker_count=$(docker ps --filter "name=rust-fullstack-starter-workers" --format "{{.Names}}" | wc -l)
    log "SUCCESS" "Worker restart completed. Currently $worker_count workers running"
    
    # Record restart event
    echo "$(date '+%Y-%m-%d %H:%M:%S') - RESTART: Now have $worker_count workers running" >> "$CHAOS_LOG_DIR/events.log"
}

# Random chaos events
run_chaos_scenario() {
    local start_time=$(date +%s)
    local end_time=$((start_time + CHAOS_DURATION))
    local next_chaos_time=$((start_time + $(shuf -i $MIN_STOP_INTERVAL-$MAX_STOP_INTERVAL -n 1)))
    
    log "INFO" "Starting chaos scenario for ${CHAOS_DURATION}s..."
    log "INFO" "Kill intervals: ${MIN_STOP_INTERVAL}-${MAX_STOP_INTERVAL}s, restart delay: ${RESTART_DELAY}s"
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time=$(date +%s)
        
        if [ $current_time -ge $next_chaos_time ]; then
            # Time for chaos event
            local victim_worker=$(get_random_worker)
            
            if [ -n "$victim_worker" ]; then
                kill_worker "$victim_worker"
                
                # For extreme chaos (when min/max intervals are very short), 
                # sometimes don't restart workers to create catastrophic failure
                local should_restart=true
                if [ $MIN_STOP_INTERVAL -le 3 ] && [ $MAX_STOP_INTERVAL -le 6 ]; then
                    # 30% chance of not restarting in extreme chaos mode
                    if [ $((RANDOM % 10)) -lt 3 ]; then
                        should_restart=false
                        log "WARN" "🚨 EXTREME CHAOS: Worker will NOT be restarted (catastrophic failure mode)"
                        echo "$(date '+%Y-%m-%d %H:%M:%S') - EXTREME CHAOS: $victim_worker NOT restarted" >> "$CHAOS_LOG_DIR/events.log"
                    fi
                fi
                
                if [ "$should_restart" = true ]; then
                    # Wait before restarting
                    if [ $RESTART_DELAY -gt 0 ]; then
                        log "INFO" "Waiting ${RESTART_DELAY}s before restart..."
                        sleep $RESTART_DELAY
                    fi
                    
                    # Restart workers (Docker Compose will recreate killed containers)
                    restart_workers
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
    echo "$(date '+%Y-%m-%d %H:%M:%S') - Chaos scenario completed. Total events: $CHAOS_EVENT_COUNT" >> "$CHAOS_LOG_DIR/events.log"
}

# Show status of all workers
show_worker_status() {
    echo -e "${BLUE}🔧 Multi-Worker Status${NC}"
    echo "=================================="
    
    local workers_list=$(get_running_workers)
    local running_count=0
    
    if [ -z "$workers_list" ]; then
        echo -e "${RED}No workers currently running${NC}"
    else
        local workers=($workers_list)
        running_count=${#workers[@]}
        echo -e "${GREEN}$running_count workers running:${NC}"
        
        for worker in "${workers[@]}"; do
            local container_id=$(docker ps -q -f name="$worker" 2>/dev/null || echo "unknown")
            local started_at=$(docker inspect "$worker" --format='{{.State.StartedAt}}' 2>/dev/null || echo "unknown")
            if [ "$started_at" != "unknown" ]; then
                local start_timestamp=$(date -d "$started_at" +%s 2>/dev/null || echo "0")
                local current_time=$(date +%s)
                local age=$((current_time - start_timestamp))
                echo "  - $worker (ID: ${container_id:0:12}, Age: ${age}s)"
            else
                echo "  - $worker (ID: ${container_id:0:12}, Age: unknown)"
            fi
        done
    fi
    
    echo ""
    echo "Target worker count: $WORKER_COUNT"
    echo "Actual worker count: $running_count"
    
    # Show chaos events if available
    if [ -f "$CHAOS_LOG_DIR/events.log" ]; then
        echo "Chaos events recorded: $CHAOS_EVENT_COUNT"
    fi
}

# Clean up all workers and files
cleanup_workers() {
    log "INFO" "Cleaning up all workers and files..."
    
    cd "$PROJECT_ROOT"
    
    # Scale workers to 0
    docker-compose -f "$CHAOS_COMPOSE_FILE" up -d --scale workers=0 >/dev/null 2>&1
    
    # Remove any remaining worker containers
    docker ps -a --filter "name=rust-fullstack-starter-workers" --format "{{.Names}}" | \
        xargs -I {} docker rm -f {} >/dev/null 2>&1 || true
    
    # Clean up files
    rm -rf "$CHAOS_LOG_DIR"
    
    log "SUCCESS" "Cleanup completed"
}

# Execute action
case "$ACTION" in
    start-multi)
        setup_chaos_environment
        scale_workers "$WORKER_COUNT"
        show_worker_status
        ;;
        
    stop-all)
        log "INFO" "Scaling workers to 0..."
        scale_workers 0
        ;;
        
    chaos-run)
        setup_chaos_environment
        
        # Scale workers to target count
        scale_workers "$WORKER_COUNT"
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