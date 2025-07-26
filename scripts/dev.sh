#!/bin/bash
set -e

echo "🚀 Starting development environment..."

# Start infrastructure
docker compose up -d postgres

# Wait for services to be healthy
echo "⏳ Waiting for services to be ready..."
docker compose up --wait

echo "✅ Infrastructure ready!"
echo "   PostgreSQL: localhost:5432"
echo ""
echo "🔧 Next steps:"
echo "   1. Copy .env.example to .env"
echo "   2. sqlx migrate run"
echo "   3. cargo run -- server"