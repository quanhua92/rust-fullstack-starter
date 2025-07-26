#!/bin/bash

echo "📊 Starter Services Status"
echo "=========================="

# Check server processes
echo "🖥️ Server Processes:"
SERVER_PIDS=$(pgrep -f "starter server" 2>/dev/null || true)
if [ -n "$SERVER_PIDS" ]; then
    echo "$SERVER_PIDS" | while read pid; do
        if [ -n "$pid" ]; then
            cmd=$(ps -p "$pid" -o args= 2>/dev/null || echo "Unknown")
            echo "   PID $pid: $cmd"
        fi
    done
else
    echo "   No server processes running"
fi

# Check worker processes
echo ""
echo "⚙️ Worker Processes:"
WORKER_PIDS=$(pgrep -f "starter worker" 2>/dev/null || true)
if [ -n "$WORKER_PIDS" ]; then
    echo "$WORKER_PIDS" | while read pid; do
        if [ -n "$pid" ]; then
            cmd=$(ps -p "$pid" -o args= 2>/dev/null || echo "Unknown")
            echo "   PID $pid: $cmd"
        fi
    done
else
    echo "   No worker processes running"
fi

# Check port usage
echo ""
echo "🔌 Port Usage:"
for PORT in 3000 8080; do
    PIDS=$(lsof -ti:$PORT 2>/dev/null || true)
    if [ -n "$PIDS" ]; then
        PROCESS=$(lsof -i:$PORT 2>/dev/null | grep LISTEN | awk '{print $1}' | head -1)
        echo "   Port $PORT: $PROCESS (PIDs: $PIDS)"
    fi
done

# Check PID files
echo ""
echo "📄 PID Files:"
for pidfile in /tmp/starter-*.pid; do
    if [ -f "$pidfile" ]; then
        pid=$(cat "$pidfile" 2>/dev/null || echo "empty")
        filename=$(basename "$pidfile")
        if [ -n "$pid" ] && [ "$pid" != "empty" ] && kill -0 "$pid" 2>/dev/null; then
            echo "   $filename: PID $pid (ACTIVE)"
        else
            echo "   $filename: PID $pid (STALE)"
        fi
    fi
done

# Check log files
echo ""
echo "📋 Recent Log Files:"
for logfile in /tmp/starter-*.log; do
    if [ -f "$logfile" ]; then
        filename=$(basename "$logfile")
        size=$(stat -f%z "$logfile" 2>/dev/null || echo "unknown")
        size_mb=$(echo "$size" | awk '{print int($1/1024/1024)}')
        echo "   $filename: ${size_mb}MB"
    fi
done

# Test connectivity if server is running
echo ""
echo "🔗 Connectivity Tests:"
for PORT in 3000 8080; do
    if lsof -ti:$PORT > /dev/null 2>&1; then
        if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
            echo "   Port $PORT: ✅ Health endpoint responding"
        else
            echo "   Port $PORT: ❌ Port open but health endpoint not responding"
        fi
    fi
done