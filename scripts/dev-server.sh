#!/bin/bash
set -e

PORT=${1:-3000}
echo "🚀 Starting complete development environment on port $PORT..."

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
./scripts/dev.sh

# Copy .env if it doesn't exist
if [ ! -f ".env" ]; then
    echo "📝 Copying .env.example to .env..."
    cp .env.example .env
    echo "   ✅ .env created with development defaults"
fi

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

# Start server
echo "🖥️  Starting server..."
./scripts/server.sh $PORT

# Test server
echo "🧪 Testing server..."
./scripts/test-server.sh $PORT

echo ""
echo "✅ Development environment ready!"
echo "   Server: http://localhost:$PORT"
echo "   Health: http://localhost:$PORT/health"
echo "   API Docs: http://localhost:$PORT/health/detailed"
echo "   Logs: tail -f /tmp/starter-server-$PORT.log"
echo ""
echo "🛑 To stop: ./scripts/stop-server.sh $PORT"
echo "📚 Next: Check docs/guides/ for learning materials"