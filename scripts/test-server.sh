#!/bin/bash
set -e

PORT=${1:-3000}
BASE_URL="http://localhost:$PORT"

echo "🧪 Testing server on port $PORT..."

# Wait for server to start
echo "⏳ Waiting for server to be ready..."
for i in {1..30}; do
    if curl -s "$BASE_URL/api/v1/health" > /dev/null 2>&1; then
        echo "✅ Server is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Server failed to start after 30 seconds"
        exit 1
    fi
    sleep 1
done

echo ""
echo "🔍 Testing endpoints:"

# Test basic health
echo -n "  Basic health: "
RESPONSE=$(curl -s "$BASE_URL/api/v1/health")
if echo "$RESPONSE" | grep -q '"success":true'; then
    echo "✅ PASS"
else
    echo "❌ FAIL - $RESPONSE"
fi

# Test detailed health
echo -n "  Detailed health: "
RESPONSE=$(curl -s "$BASE_URL/api/v1/health/detailed")
if echo "$RESPONSE" | grep -q '"status":"healthy"'; then
    echo "✅ PASS"
else
    echo "❌ FAIL - $RESPONSE"
fi

echo ""
echo "📊 Server info:"
echo "   URL: $BASE_URL"
echo "   Health: $BASE_URL/api/v1/health"
echo "   Detailed: $BASE_URL/api/v1/health/detailed"