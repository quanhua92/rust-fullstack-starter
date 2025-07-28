#!/bin/bash
set -e

PORT=""
FOLLOW_LOGS=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--follow)
            FOLLOW_LOGS=true
            shift
            ;;
        [0-9]*)
            PORT=$1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [port] [-f|--follow]"
            echo "  port            Port number (default: 3000)"
            echo "  -f, --follow    Follow logs after starting server"
            exit 1
            ;;
    esac
done

# Set default port if not provided
PORT=${PORT:-3000}

PROJECT_NAME="starter"
LOG_FILE="/tmp/starter-server-${PORT}.log"
PID_FILE="/tmp/starter-server-${PORT}.pid"
MAX_LOG_SIZE_MB=50

# Validate we're in the right directory
if [ ! -f "docker-compose.yaml" ] || [ ! -d "starter" ]; then
    echo "❌ Please run this script from the project root directory"
    echo "   Current directory: $(pwd)"
    echo "   Expected files: docker-compose.yaml, starter/"
    exit 1
fi

echo "🔄 Starting $PROJECT_NAME server on port $PORT..."

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

# Kill any existing server on the port
echo "🛑 Stopping any existing server on port $PORT..."
lsof -ti:$PORT | xargs kill -9 2>/dev/null || true

# Kill any existing starter processes and clean up PID file
if [ -f "$PID_FILE" ]; then
    old_pid=$(cat "$PID_FILE" 2>/dev/null || echo "")
    if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
        echo "🛑 Killing existing server process: $old_pid"
        kill -9 "$old_pid" 2>/dev/null || true
    fi
    rm -f "$PID_FILE"
fi

pkill -f "starter server" 2>/dev/null || true

# Give it a moment to clean up
sleep 1

# Rotate log if needed
rotate_log_if_needed

# Start the new server
echo "🚀 Starting new server..."
echo "📝 Log file: $LOG_FILE"
echo "📄 PID file: $PID_FILE"

# Use absolute path and proper backgrounding
SCRIPT_DIR=$(pwd)
bash -c "cd '$SCRIPT_DIR/starter' && exec cargo run -- server --port $PORT" > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

# Save PID immediately
echo $SERVER_PID > "$PID_FILE"

echo "✅ Server started with PID: $SERVER_PID"
echo "🔍 To test: ./scripts/test-server.sh $PORT"
echo "🛑 To stop: ./scripts/stop-server.sh $PORT"
echo "📋 View logs: tail -f $LOG_FILE"

# Quick validation without blocking
sleep 1
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start. Check logs: cat $LOG_FILE"
    rm -f "$PID_FILE"
    exit 1
else
    echo "🟢 Server running successfully"
fi

# Follow logs if requested
if [ "$FOLLOW_LOGS" = true ]; then
    echo ""
    echo "📋 Following server logs (Ctrl+C to exit)..."
    echo "=================================="
    tail -f "$LOG_FILE"
fi