#!/bin/bash
set -e

# Default values
PORT=3000
FOREGROUND=false
BUILD_WEB=true
START_WORKER=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -f|--foreground)
            FOREGROUND=true
            shift
            ;;
        --api-only)
            BUILD_WEB=false
            shift
            ;;
        -w|--with-worker)
            START_WORKER=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  -p, --port PORT     Set server port (default: 3000)"
            echo "  -f, --foreground    Run server in foreground mode"
            echo "  -w, --with-worker   Also start background worker (ID 0)"
            echo "  --api-only          Skip web frontend build (API only)"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        [0-9]*)
            PORT=$1
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

echo "🚀 Starting complete full-stack development environment on port $PORT..."

# Validate we're in the right directory
if [ ! -f "docker-compose.yaml" ]; then
    echo "❌ docker-compose.yaml not found"
    echo "   Please run this script from the project root directory"
    echo "   Current directory: $(pwd)"
    exit 1
fi

if [ ! -d "starter/migrations" ]; then
    echo "❌ starter/migrations directory not found"
    echo "   Please run this script from the project root directory"
    exit 1
fi

# Start infrastructure
echo "📦 Starting PostgreSQL..."
docker compose up -d postgres

# Wait for services to be healthy
echo "⏳ Waiting for services to be ready..."
docker compose up --wait

# Copy .env if it doesn't exist
if [ ! -f ".env" ]; then
    echo "📝 Copying .env.example to .env..."
    cp .env.example .env
    echo "   ✅ .env created with development defaults"
fi

# Build web frontend if requested and available
if [ "$BUILD_WEB" = true ] && [ -d "web" ]; then
    echo "🌐 Building web frontend..."
    if ! ./scripts/build-web.sh; then
        echo "❌ Web frontend build failed!"
        echo "   Run './scripts/build-web.sh' for details"
        echo "   Continuing with API-only server..."
        BUILD_WEB=false
    else
        echo "   ✅ Web frontend built successfully"
    fi
elif [ "$BUILD_WEB" = true ]; then
    echo "⚠️  Web directory not found, starting API-only server"
    BUILD_WEB=false
fi

# Set web build path for server
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
export STARTER__SERVER__WEB_BUILD_PATH="$PROJECT_ROOT/web/dist"

# Check sqlx-cli
if ! command -v sqlx &> /dev/null; then
    echo "⚠️  sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
    echo "   ✅ sqlx-cli installed"
fi

# Run migrations
echo "🔄 Running database migrations..."
cd starter
sqlx migrate run || {
    echo "❌ Migration failed. Common solutions:"
    echo "   1. Check database is running: docker compose ps"
    echo "   2. Verify DATABASE_URL in .env file"
    echo "   3. Reset database: docker compose down -v && docker compose up -d"
    echo "   4. Wait for database to be ready and try again"
    exit 1
}
cd ..

# Display available endpoints
echo "🖥️  Starting development server..."
echo ""
echo "📍 Available endpoints:"
if [ "$BUILD_WEB" = true ]; then
    echo "   🌐 Frontend: http://localhost:$PORT (React SPA)"
fi
echo "   🔌 API: http://localhost:$PORT/api/v1"
echo "   ❤️  Health: http://localhost:$PORT/api/v1/health"
echo "   📚 API Docs: http://localhost:$PORT/api-docs"
echo ""
if [ "$FOREGROUND" = true ]; then
    echo "🛑 To stop: Ctrl+C"
else
    echo "🛑 To stop: ./scripts/stop-server.sh $PORT"
fi
echo "📚 Next: Check docs/guides/ for learning materials"
echo ""

# Start worker if requested (background mode only)
if [ "$START_WORKER" = true ] && [ "$FOREGROUND" = false ]; then
    echo "⚙️  Starting background worker..."
    ./scripts/worker.sh
    echo "   ✅ Worker started (ID 0)"
fi

# Start server using the enhanced server.sh script
if [ "$FOREGROUND" = true ]; then
    echo "🚀 Starting server in foreground mode..."
    if [ "$START_WORKER" = true ]; then
        echo "💡 Note: Worker not started in foreground mode. Start separately:"
        echo "   ./scripts/worker.sh -f  # In another terminal"
        echo ""
    fi
    exec ./scripts/server.sh "$PORT" -f
else
    echo "🚀 Starting server in background mode..."
    ./scripts/server.sh "$PORT"
    
    # Show status
    echo ""
    echo "✅ Development environment ready!"
    echo "📋 Server logs: tail -f /tmp/starter-server-$PORT.log"
    if [ "$START_WORKER" = true ]; then
        echo "📋 Worker logs: tail -f /tmp/starter-worker-0.log"
    else
        echo "💡 Start worker: ./scripts/worker.sh"
    fi
    echo "📊 Check status: ./scripts/status.sh"
fi