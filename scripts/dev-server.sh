#!/bin/bash
set -e

PORT=${1:-3000}

echo "🛠️  Starting development server with auto-restart..."

# Start infrastructure first
./scripts/dev.sh

echo ""
echo "🔄 Starting server with restart capability..."

# Function to handle cleanup
cleanup() {
    echo ""
    echo "🛑 Shutting down..."
    ./scripts/stop-server.sh $PORT
    exit 0
}

# Set trap to catch Ctrl+C
trap cleanup SIGINT SIGTERM

# Start the server and test it
./scripts/server.sh $PORT &
SERVER_PID=$!

# Wait a bit and test
sleep 3
./scripts/test-server.sh $PORT

echo ""
echo "🎯 Server running! Press Ctrl+C to stop"
echo "   To restart: ./scripts/server.sh $PORT"
echo "   To test: ./scripts/test-server.sh $PORT"
echo "   To stop: ./scripts/stop-server.sh $PORT"

# Wait for the server process
wait $SERVER_PID