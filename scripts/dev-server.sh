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

# Start server in foreground
echo "🖥️  Starting server in foreground..."
echo "   Server: http://localhost:$PORT"
echo "   Health: http://localhost:$PORT/api/v1/health"
echo "   API Docs: http://localhost:$PORT/api-docs"
echo ""
echo "🛑 To stop: Ctrl+C"
echo "📚 Next: Check docs/guides/ for learning materials"
echo ""

# Kill any existing server on the port first
echo "🛑 Stopping any existing server on port $PORT..."
lsof -ti:$PORT | xargs kill -9 2>/dev/null || true
sleep 1

# Start server in foreground
cd starter
exec cargo run -- server --port $PORT