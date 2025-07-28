#!/bin/bash

# Admin CLI Helper - Centralized docker exec wrapper for admin CLI commands
# Provides consistent admin CLI access across all chaos testing scripts

set -e

# Default container name
CONTAINER_NAME="${CONTAINER_NAME:-chaos-starter-server}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD_RED='\033[1;31m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        "ERROR")
            echo -e "${BOLD_RED}[${timestamp}] ERROR:${NC} $message" >&2
            ;;
        "WARN")
            echo -e "${YELLOW}[${timestamp}] WARN:${NC} $message" >&2
            ;;
        *)
            echo -e "[${timestamp}] ${level}: $message"
            ;;
    esac
}

# Show usage
show_usage() {
    echo "Admin CLI Helper - Centralized docker exec wrapper"
    echo ""
    echo "Usage: $0 [options] <command> [command-args...]"
    echo ""
    echo "Options:"
    echo "  -c, --container NAME   Container name (default: $CONTAINER_NAME)"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Commands:"
    echo "  task-stats [--tag TAG]           Get task statistics"
    echo "  list-tasks [--limit N] [--verbose] List tasks"
    echo "  health-check                     Check system health"
    echo ""
    echo "Examples:"
    echo "  $0 task-stats --tag baseline"
    echo "  $0 list-tasks --limit 10 --verbose"
    echo "  $0 -c my-server task-stats"
    echo ""
    echo "Environment Variables:"
    echo "  CONTAINER_NAME  - Default container name"
}

# Execute admin CLI command via docker exec
execute_admin_cli() {
    local admin_args=("$@")
    local cmd="docker exec \"$CONTAINER_NAME\" /app/starter admin ${admin_args[*]}"
    
    if [ "$VERBOSE" = true ]; then
        log "INFO" "Executing: $cmd"
    fi
    
    # Execute the command and capture both output and exit code
    local output
    local exit_code
    
    output=$(docker exec "$CONTAINER_NAME" /app/starter admin "${admin_args[@]}" 2>&1)
    exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log "ERROR" "Admin CLI command failed (exit code: $exit_code)"
        log "ERROR" "Command: ${admin_args[*]}"
        log "ERROR" "Output: $output"
        return $exit_code
    fi
    
    echo "$output"
    return 0
}

# Get task statistics by tag
get_task_stats() {
    local tag="$1"
    local args=("task-stats")
    
    if [ -n "$tag" ]; then
        args+=("--tag" "$tag")
    fi
    
    execute_admin_cli "${args[@]}"
}

# Parse task stats output to extract specific metrics
parse_task_stats() {
    local stats_output="$1"
    local metric="$2"  # completed, failed, total, etc.
    
    case "$metric" in
        "completed")
            echo "$stats_output" | grep "completed:" | awk '{print $2}' || echo "0"
            ;;
        "failed")
            echo "$stats_output" | grep "failed:" | awk '{print $2}' || echo "0"
            ;;
        "total")
            echo "$stats_output" | grep "Total:" | awk '{print $2}' || echo "0"
            ;;
        "pending")
            echo "$stats_output" | grep "pending:" | awk '{print $2}' || echo "0"
            ;;
        "running")
            echo "$stats_output" | grep "running:" | awk '{print $2}' || echo "0"
            ;;
        *)
            log "ERROR" "Unknown metric: $metric"
            return 1
            ;;
    esac
}

# List tasks
list_tasks() {
    local limit="$1"
    local verbose="$2"
    local args=("list-tasks")
    
    if [ -n "$limit" ]; then
        args+=("--limit" "$limit")
    fi
    
    if [ "$verbose" = true ]; then
        args+=("--verbose")
    fi
    
    execute_admin_cli "${args[@]}"
}

# Health check
health_check() {
    execute_admin_cli "health-check" || return 1
}

# Main execution
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--container)
            CONTAINER_NAME="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        task-stats)
            shift
            # Parse task-stats specific arguments
            TAG=""
            while [[ $# -gt 0 ]] && [[ $1 == --* ]]; do
                case $1 in
                    --tag)
                        TAG="$2"
                        shift 2
                        ;;
                    *)
                        log "ERROR" "Unknown task-stats option: $1"
                        exit 1
                        ;;
                esac
            done
            get_task_stats "$TAG"
            exit $?
            ;;
        list-tasks)
            shift
            # Parse list-tasks specific arguments
            LIMIT=""
            TASK_VERBOSE=false
            while [[ $# -gt 0 ]] && [[ $1 == --* ]]; do
                case $1 in
                    --limit)
                        LIMIT="$2"
                        shift 2
                        ;;
                    --verbose)
                        TASK_VERBOSE=true
                        shift
                        ;;
                    *)
                        log "ERROR" "Unknown list-tasks option: $1"
                        exit 1
                        ;;
                esac
            done
            list_tasks "$LIMIT" "$TASK_VERBOSE"
            exit $?
            ;;
        health-check)
            shift
            health_check
            exit $?
            ;;
        *)
            log "ERROR" "Unknown command: $1"
            show_usage
            exit 1
            ;;
    esac
done

# If no command provided, show usage
show_usage
exit 1