#!/bin/bash

WORKER_ID="0"
STOP_ALL=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --id)
            WORKER_ID="$2"
            shift 2
            ;;
        --all)
            STOP_ALL=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--id ID] [--all]"
            echo "  --id ID    Stop worker with specific ID (default: 0)"
            echo "  --all      Stop all workers (finds all PID files)"
            exit 1
            ;;
    esac
done

if [ "$STOP_ALL" = true ]; then
    echo "🛑 Stopping all background workers..."
else
    echo "🛑 Stopping background worker (ID: $WORKER_ID)..."
fi

# Function to kill worker by PID file
kill_worker_by_pid_file() {
    local pid_file="$1"
    local worker_id="$2"
    
    if [ -f "$pid_file" ]; then
        worker_pid=$(cat "$pid_file" 2>/dev/null || echo "")
        if [ -n "$worker_pid" ] && kill -0 "$worker_pid" 2>/dev/null; then
            echo "   Killing worker process (ID: $worker_id): $worker_pid"
            kill -TERM "$worker_pid" 2>/dev/null || kill -9 "$worker_pid" 2>/dev/null || true
            # Wait for graceful shutdown
            for i in {1..10}; do
                if ! kill -0 "$worker_pid" 2>/dev/null; then
                    break
                fi
                sleep 1
            done
            # Force kill if still running
            if kill -0 "$worker_pid" 2>/dev/null; then
                echo "   Force killing worker process..."
                kill -9 "$worker_pid" 2>/dev/null || true
            fi
        fi
        rm -f "$pid_file"
        echo "   Removed PID file: $pid_file"
    else
        echo "   No PID file found: $pid_file"
    fi
}

if [ "$STOP_ALL" = true ]; then
    # Find all worker PID files
    found_any=false
    for pid_file in /tmp/starter-worker-*.pid; do
        if [ -f "$pid_file" ]; then
            found_any=true
            # Extract worker ID from filename
            worker_id=$(basename "$pid_file" | sed 's/starter-worker-\(.*\)\.pid/\1/')
            kill_worker_by_pid_file "$pid_file" "$worker_id"
        fi
    done
    
    if [ "$found_any" = false ]; then
        echo "   No worker PID files found in /tmp/"
    fi
else
    # Kill specific worker
    PID_FILE="/tmp/starter-worker-${WORKER_ID}.pid"
    kill_worker_by_pid_file "$PID_FILE" "$WORKER_ID"
fi

# Kill any remaining starter worker processes
if [ "$STOP_ALL" = true ]; then
    WORKER_PIDS=$(pgrep -f "starter worker" 2>/dev/null || true)
    if [ -n "$WORKER_PIDS" ]; then
        echo "   Killing remaining worker processes: $WORKER_PIDS"
        echo "$WORKER_PIDS" | xargs kill -9 2>/dev/null || true
    else
        echo "   No remaining worker processes found"
    fi
    echo "✅ All workers stopped"
else
    # Kill workers with specific WORKER_ID
    WORKER_PIDS=$(pgrep -f "WORKER_ID=${WORKER_ID}.*starter worker" 2>/dev/null || true)
    if [ -n "$WORKER_PIDS" ]; then
        echo "   Killing remaining worker processes (ID: $WORKER_ID): $WORKER_PIDS"
        echo "$WORKER_PIDS" | xargs kill -9 2>/dev/null || true
    else
        echo "   No remaining worker processes found for ID: $WORKER_ID"
    fi
    echo "✅ Worker (ID: $WORKER_ID) stopped"
fi