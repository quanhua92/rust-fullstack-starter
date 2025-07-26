#!/bin/bash
set -e

PROJECT_NAME="starter"
LOG_FILE="/tmp/starter-worker.log"
PID_FILE="/tmp/starter-worker.pid"
MAX_LOG_SIZE_MB=50

echo "🔄 Starting $PROJECT_NAME background worker..."

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

# Kill any existing worker processes and clean up PID file
echo "🛑 Stopping any existing workers..."
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

pkill -f "starter worker" 2>/dev/null || true

# Give it a moment to clean up
sleep 1

# Rotate log if needed
rotate_log_if_needed

# Start the worker
echo "🚀 Starting new background worker..."
echo "📝 Log file: $LOG_FILE"
echo "📄 PID file: $PID_FILE"

# Use absolute path and proper backgrounding
SCRIPT_DIR=$(pwd)
bash -c "cd '$SCRIPT_DIR/starter' && exec cargo run -- worker" > "$LOG_FILE" 2>&1 &
WORKER_PID=$!

# Save PID immediately
echo $WORKER_PID > "$PID_FILE"

echo "✅ Worker started with PID: $WORKER_PID"
echo "🛑 To stop: ./scripts/stop-worker.sh"
echo "📋 View logs: tail -f $LOG_FILE"

# Quick validation without blocking
sleep 2
if ! kill -0 $WORKER_PID 2>/dev/null; then
    echo "❌ Worker failed to start. Check logs: cat $LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
else
    echo "🟢 Worker running successfully"
fi