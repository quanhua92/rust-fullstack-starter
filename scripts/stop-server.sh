#!/bin/bash

PORT=${1:-3000}
PID_FILE="/tmp/starter-server-${PORT}.pid"

echo "🛑 Stopping server on port $PORT..."

# Kill process from PID file first
if [ -f "$PID_FILE" ]; then
    server_pid=$(cat "$PID_FILE" 2>/dev/null || echo "")
    if [ -n "$server_pid" ] && kill -0 "$server_pid" 2>/dev/null; then
        echo "   Killing server process: $server_pid"
        kill -TERM "$server_pid" 2>/dev/null || kill -9 "$server_pid" 2>/dev/null || true
        # Wait for graceful shutdown
        for i in {1..5}; do
            if ! kill -0 "$server_pid" 2>/dev/null; then
                break
            fi
            sleep 1
        done
        # Force kill if still running
        if kill -0 "$server_pid" 2>/dev/null; then
            kill -9 "$server_pid" 2>/dev/null || true
        fi
    fi
    rm -f "$PID_FILE"
    echo "   Removed PID file: $PID_FILE"
else
    echo "   No PID file found: $PID_FILE"
fi

# Kill processes using the port
PIDS=$(lsof -ti:$PORT 2>/dev/null || true)
if [ -n "$PIDS" ]; then
    echo "   Killing processes on port $PORT: $PIDS"
    echo "$PIDS" | xargs kill -9 2>/dev/null || true
else
    echo "   No process found on port $PORT"
fi

# Kill any remaining starter server processes
STARTER_PIDS=$(pgrep -f "starter server" 2>/dev/null || true)
if [ -n "$STARTER_PIDS" ]; then
    echo "   Killing remaining starter processes: $STARTER_PIDS"
    echo "$STARTER_PIDS" | xargs kill -9 2>/dev/null || true
else
    echo "   No remaining starter server processes found"
fi

echo "✅ Server stopped"