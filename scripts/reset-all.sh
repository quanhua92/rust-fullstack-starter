#!/bin/bash

# Parse command line arguments
RESET_DATABASE=false
for arg in "$@"; do
    case $arg in
        --reset-database)
            RESET_DATABASE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--reset-database]"
            echo ""
            echo "Reset all starter processes and optionally database:"
            echo "  --reset-database    Also reset the database (DANGEROUS!)"
            echo "  --help, -h          Show this help message"
            echo ""
            echo "By default, only stops processes and cleans up files."
            echo "Database reset requires explicit flag for safety."
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Validate we're in the right directory
if [ ! -f "docker-compose.yaml" ] || [ ! -d "starter" ]; then
    echo "❌ Please run this script from the project root directory"
    echo "   Current directory: $(pwd)"
    echo "   Expected files: docker-compose.yaml, starter/"
    exit 1
fi

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

# Database reset (only if explicitly requested)
if [ "$RESET_DATABASE" = true ]; then
    echo "🗄️ Resetting database..."
    cd starter
    sqlx database reset -y || {
        echo "⚠️ Database reset failed, continuing anyway..."
    }
    cd ..
    DATABASE_STATUS="and database reset"
else
    echo "📊 Database preserved (use --reset-database to reset database)"
    echo "   💡 To reset database manually:"
    echo "      cd starter && sqlx database reset -y"
    DATABASE_STATUS="(database preserved)"
fi

# Wait for cleanup
sleep 2

echo "✅ Reset complete! All processes stopped $DATABASE_STATUS."
echo ""
echo "🚀 Next steps:"
echo "   Start server: ./scripts/server.sh [port]"
echo "   Start worker: ./scripts/worker.sh"
echo "   Run tests: ./scripts/test_tasks_integration.sh"
if [ "$RESET_DATABASE" = false ]; then
    echo ""
    echo "💡 To also reset database next time:"
    echo "   ./scripts/reset-all.sh --reset-database"
fi