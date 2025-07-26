#!/bin/bash

PID_FILE="/tmp/starter-worker.pid"

echo "🛑 Stopping background worker..."

# Kill process from PID file first
if [ -f "$PID_FILE" ]; then
    worker_pid=$(cat "$PID_FILE" 2>/dev/null || echo "")
    if [ -n "$worker_pid" ] && kill -0 "$worker_pid" 2>/dev/null; then
        echo "   Killing worker process: $worker_pid"
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
    rm -f "$PID_FILE"
    echo "   Removed PID file: $PID_FILE"
else
    echo "   No PID file found: $PID_FILE"
fi

# Kill any remaining starter worker processes
WORKER_PIDS=$(pgrep -f "starter worker" 2>/dev/null || true)
if [ -n "$WORKER_PIDS" ]; then
    echo "   Killing remaining worker processes: $WORKER_PIDS"
    echo "$WORKER_PIDS" | xargs kill -9 2>/dev/null || true
else
    echo "   No remaining worker processes found"
fi

echo "✅ Worker stopped"