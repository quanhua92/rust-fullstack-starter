#!/bin/bash
set -e

PROJECT_NAME="starter"
WORKER_ID="0"
MAX_LOG_SIZE_MB=50
FOLLOW_LOGS=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--foreground)
            FOLLOW_LOGS=true
            shift
            ;;
        --id)
            WORKER_ID="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [-f|--foreground] [--id ID]"
            echo "  -f, --foreground    Run in foreground mode (Ctrl+C to stop)"
            echo "  --id ID             Worker ID for concurrent workers (default: 0)"
            exit 1
            ;;
    esac
done

# Set up dynamic file paths based on WORKER_ID
LOG_FILE="/tmp/starter-worker-${WORKER_ID}.log"
PID_FILE="/tmp/starter-worker-${WORKER_ID}.pid"

# Validate we're in the right directory
if [ ! -f "docker-compose.yaml" ] || [ ! -d "starter" ]; then
    echo "❌ Please run this script from the project root directory"
    echo "   Current directory: $(pwd)"
    echo "   Expected files: docker-compose.yaml, starter/"
    exit 1
fi

if [ "$FOLLOW_LOGS" = true ]; then
    echo "🔄 Starting $PROJECT_NAME worker (ID: $WORKER_ID) in foreground mode..."
else
    echo "🔄 Starting $PROJECT_NAME background worker (ID: $WORKER_ID)..."
fi

# Function to rotate log if it's too large
rotate_log_if_needed() {
    if [ -f "$LOG_FILE" ]; then
        local size_mb=$(stat -f%z "$LOG_FILE" 2>/dev/null | awk '{print int($1/1024/1024)}')
        if [ "$size_mb" -gt "$MAX_LOG_SIZE_MB" ]; then
            echo "📁 Rotating log file (${size_mb}MB > ${MAX_LOG_SIZE_MB}MB)"
            mv "$LOG_FILE" "${LOG_FILE}.old"
            echo "$(date): Log rotated due to size (${size_mb}MB)" > "$LOG_FILE"
        fi
    fi
}

# Kill any existing worker processes with the same ID and clean up PID file
echo "🛑 Stopping any existing workers with ID: $WORKER_ID..."
if [ -f "$PID_FILE" ]; then
    old_pid=$(cat "$PID_FILE" 2>/dev/null || echo "")
    if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
        echo "🛑 Killing existing worker process: $old_pid"
        kill -TERM "$old_pid" 2>/dev/null || kill -9 "$old_pid" 2>/dev/null || true
        # Wait for graceful shutdown
        for i in {1..5}; do
            if ! kill -0 "$old_pid" 2>/dev/null; then
                break
            fi
            sleep 1
        done
    fi
    rm -f "$PID_FILE"
fi

# Kill workers with the same WORKER_ID (environment variable visible in process list)
pkill -f "WORKER_ID=${WORKER_ID}.*starter worker" 2>/dev/null || true

# Give it a moment to clean up
sleep 1

# Rotate log if needed
rotate_log_if_needed

# Check if running in foreground mode
if [ "$FOLLOW_LOGS" = true ]; then
    echo "🚀 Starting worker (ID: $WORKER_ID) in foreground mode (Ctrl+C to exit)..."
    echo "📋 Running directly with exec (no PID file or logs)"
    echo "=================================="
    cd starter
    exec env WORKER_ID="$WORKER_ID" cargo run -- worker
fi

# Background mode - start with PID tracking and logging
echo "🚀 Starting new background worker..."
echo "📝 Log file: $LOG_FILE"
echo "📄 PID file: $PID_FILE"

# Use absolute path and proper backgrounding
SCRIPT_DIR=$(pwd)
bash -c "cd '$SCRIPT_DIR/starter' && exec env WORKER_ID='$WORKER_ID' cargo run -- worker" > "$LOG_FILE" 2>&1 &
WORKER_PID=$!

# Save PID immediately
echo $WORKER_PID > "$PID_FILE"

echo "✅ Worker (ID: $WORKER_ID) started with PID: $WORKER_PID"
echo "🛑 To stop: ./scripts/stop-worker.sh --id $WORKER_ID"
echo "📋 View logs: tail -f $LOG_FILE"

# Quick validation without blocking
sleep 2
if ! kill -0 $WORKER_PID 2>/dev/null; then
    echo "❌ Worker failed to start. Check logs: cat $LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
else
    echo "🟢 Worker (ID: $WORKER_ID) running successfully"
fi