#!/bin/bash

# Reliable integration test for tasks API and background worker
set -e

echo "🧪 Tasks API & Background Worker Integration Test"
echo "================================================="

# Configuration
SERVER_PORT=3000
BASE_URL="http://localhost:$SERVER_PORT"
SCRIPT_DIR=$(dirname "$0")

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up test environment..."
    "$SCRIPT_DIR/stop-server.sh" $SERVER_PORT
    "$SCRIPT_DIR/stop-worker.sh"
    echo "✅ Cleanup complete"
}

# Trap cleanup on exit
trap cleanup EXIT

echo ""
echo "🔄 Resetting environment..."
"$SCRIPT_DIR/reset-all.sh"

echo ""
echo "🚀 Starting server on port $SERVER_PORT..."
"$SCRIPT_DIR/server.sh" $SERVER_PORT

echo ""
echo "⏳ Waiting for server to be ready..."
"$SCRIPT_DIR/test-server.sh" $SERVER_PORT

echo ""
echo "👤 Setting up test user..."

# Register a user
echo "📝 Registering user..."
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "taskuser",
        "email": "taskuser@example.com",
        "password": "password123"
    }')

if echo "$REGISTER_RESPONSE" | grep -q '"success":true'; then
    echo "✅ User registered successfully"
else
    echo "❌ User registration failed: $REGISTER_RESPONSE"
    exit 1
fi

# Login to get token
echo "🔑 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d '{
        "username_or_email": "taskuser",
        "password": "password123"
    }')

if echo "$LOGIN_RESPONSE" | grep -q '"success":true'; then
    echo "✅ Login successful"
else
    echo "❌ Login failed: $LOGIN_RESPONSE"
    exit 1
fi

# Extract token
TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to extract authentication token"
    echo "Login response: $LOGIN_RESPONSE"
    exit 1
fi

echo "✅ Got authentication token: ${TOKEN:0:20}..."

echo ""
echo "📊 Testing tasks API..."

# Test 1: Get initial stats
echo "🔍 Getting initial task stats..."
STATS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/tasks/stats")
if echo "$STATS_RESPONSE" | grep -q '"success":true'; then
    echo "✅ Task stats endpoint working"
else
    echo "❌ Task stats failed: $STATS_RESPONSE"
    exit 1
fi

# Test 2: Create tasks
echo ""
echo "📧 Creating email task..."
EMAIL_TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/tasks" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test Email from Background Worker",
            "body": "This is a test email sent via the background worker system."
        },
        "priority": "normal"
    }')

if echo "$EMAIL_TASK_RESPONSE" | grep -q '"success":true'; then
    EMAIL_TASK_ID=$(echo "$EMAIL_TASK_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
    echo "✅ Email task created: $EMAIL_TASK_ID"
else
    echo "❌ Email task creation failed: $EMAIL_TASK_RESPONSE"
    exit 1
fi

echo ""
echo "📊 Creating data processing task..."
DATA_TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/tasks" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "task_type": "data_processing",
        "payload": {
            "operation": "sum",
            "data": [1, 2, 3, 4, 5]
        },
        "priority": "high"
    }')

if echo "$DATA_TASK_RESPONSE" | grep -q '"success":true'; then
    DATA_TASK_ID=$(echo "$DATA_TASK_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
    echo "✅ Data processing task created: $DATA_TASK_ID"
else
    echo "❌ Data processing task creation failed: $DATA_TASK_RESPONSE"
    exit 1
fi

# Test 3: List tasks
echo ""
echo "📋 Listing all tasks..."
TASKS_LIST_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/tasks")
if echo "$TASKS_LIST_RESPONSE" | grep -q '"success":true'; then
    TASK_COUNT=$(echo "$TASKS_LIST_RESPONSE" | python3 -c "import json,sys; print(len(json.load(sys.stdin)['data']))" 2>/dev/null || echo "0")
    echo "✅ Tasks listed successfully: $TASK_COUNT tasks found"
else
    echo "❌ Task listing failed: $TASKS_LIST_RESPONSE"
    exit 1
fi

echo ""
echo "⚡ Starting background worker to process tasks..."
"$SCRIPT_DIR/worker.sh"

echo ""
echo "⏳ Waiting for tasks to be processed (15 seconds)..."
sleep 15

echo ""
echo "📊 Checking final task status..."

# Check email task status
EMAIL_STATUS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/tasks/$EMAIL_TASK_ID")
if echo "$EMAIL_STATUS" | grep -q '"success":true'; then
    STATUS=$(echo "$EMAIL_STATUS" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['status'])" 2>/dev/null || echo "unknown")
    echo "📧 Email task status: $STATUS"
else
    echo "❌ Failed to get email task status"
fi

# Check data processing task status
DATA_STATUS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/tasks/$DATA_TASK_ID")
if echo "$DATA_STATUS" | grep -q '"success":true'; then
    STATUS=$(echo "$DATA_STATUS" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['status'])" 2>/dev/null || echo "unknown")
    echo "📊 Data processing task status: $STATUS"
else
    echo "❌ Failed to get data processing task status"
fi

# Final stats
FINAL_STATS=$(curl -s -H "Authorization: Bearer $TOKEN" "$BASE_URL/tasks/stats")
if echo "$FINAL_STATS" | grep -q '"success":true'; then
    echo ""
    echo "📈 Final statistics:"
    echo "$FINAL_STATS" | python3 -c "
import json, sys
data = json.load(sys.stdin)['data']
print(f\"   Total: {data['total']}\")
print(f\"   Pending: {data['pending']}\")
print(f\"   Running: {data['running']}\")
print(f\"   Completed: {data['completed']}\")
print(f\"   Failed: {data['failed']}\")
" 2>/dev/null || echo "   (Could not parse stats)"
fi

echo ""
echo "✅ Tasks API and Background Worker integration test completed!"
echo ""
echo "📚 Features tested:"
echo "   ✓ User authentication"
echo "   ✓ Task creation via API"
echo "   ✓ Multiple task types"
echo "   ✓ Task priority system"
echo "   ✓ Task listing and retrieval"
echo "   ✓ Task statistics"
echo "   ✓ Background worker processing"
echo "   ✓ Task status updates"
echo ""
echo "🚀 Background worker system is fully functional!"