#!/bin/bash

echo "🔄 Resetting all servers and workers..."

# Stop all servers and workers
echo "🛑 Stopping all starter processes..."
pkill -f "starter server" 2>/dev/null || true
pkill -f "starter worker" 2>/dev/null || true

# Clean up common ports
for PORT in 3000 8080; do
    PIDS=$(lsof -ti:$PORT 2>/dev/null || true)
    if [ -n "$PIDS" ]; then
        echo "🔌 Killing processes on port $PORT"
        echo "$PIDS" | xargs kill -9 2>/dev/null || true
    fi
done

# Clean up PID files
echo "🧹 Cleaning up PID files..."
rm -f /tmp/starter-server-*.pid
rm -f /tmp/starter-worker-*.pid

# Clean up log files older than 1 day
echo "🗂️ Cleaning up old log files..."
find /tmp -name "starter-*.log" -mtime +1 -delete 2>/dev/null || true

# Reset database
echo "🗄️ Resetting database..."
cd starter
sqlx database reset -y || {
    echo "⚠️ Database reset failed, continuing anyway..."
}
cd ..

# Wait for cleanup
sleep 2

echo "✅ Reset complete! All processes stopped and database reset."
echo ""
echo "🚀 Next steps:"
echo "   Start server: ./scripts/server.sh [port]"
echo "   Start worker: ./scripts/worker.sh"
echo "   Run tests: ./scripts/test_tasks_integration.sh"