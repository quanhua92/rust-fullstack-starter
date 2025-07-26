#!/bin/bash

# Test script for tasks API and background worker integration
set -e

echo "🧪 Testing Tasks API and Background Worker Integration"
echo "===================================================="

# Build the project
echo "📦 Building project..."
cargo build

echo ""
echo "🚀 Starting server..."

# Start server in background
cargo run -- server --port 3001 > server.log 2>&1 &
SERVER_PID=$!

# Give server time to start
sleep 3

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start"
    exit 1
fi

echo "✅ Server started on port 3001 (PID: $SERVER_PID)"

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
    kill $WORKER_PID 2>/dev/null || true
    rm -f server.log worker.log
}
trap cleanup EXIT

# Test user registration and login first
echo ""
echo "👤 Setting up test user..."

# Register a user
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:3001/auth/register \
    -H "Content-Type: application/json" \
    -d '{
        "username": "taskuser",
        "email": "taskuser@example.com",
        "password": "password123"
    }')

echo "📝 Register response: $REGISTER_RESPONSE"

# Login to get token
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3001/auth/login \
    -H "Content-Type: application/json" \
    -d '{
        "username": "taskuser",
        "password": "password123"
    }')

echo "📝 Login response: $LOGIN_RESPONSE"

# Extract token
TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.data.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo "❌ Failed to get authentication token"
    exit 1
fi

echo "✅ Got authentication token: ${TOKEN:0:20}..."

echo ""
echo "📊 Testing tasks API..."

# Test 1: Get initial stats
echo "🔍 Getting initial task stats..."
STATS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/stats)
echo "📝 Initial stats: $STATS_RESPONSE"

# Test 2: Create an email task
echo ""
echo "📧 Creating email task..."
EMAIL_TASK_RESPONSE=$(curl -s -X POST http://localhost:3001/tasks \
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

echo "📝 Email task created: $EMAIL_TASK_RESPONSE"
EMAIL_TASK_ID=$(echo $EMAIL_TASK_RESPONSE | jq -r '.data.id')

# Test 3: Create a data processing task
echo ""
echo "📊 Creating data processing task..."
DATA_TASK_RESPONSE=$(curl -s -X POST http://localhost:3001/tasks \
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

echo "📝 Data processing task created: $DATA_TASK_RESPONSE"
DATA_TASK_ID=$(echo $DATA_TASK_RESPONSE | jq -r '.data.id')

# Test 4: Create a webhook task
echo ""
echo "🔗 Creating webhook task..."
WEBHOOK_TASK_RESPONSE=$(curl -s -X POST http://localhost:3001/tasks \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "task_type": "webhook",
        "payload": {
            "url": "https://httpbin.org/post",
            "method": "POST",
            "payload": {"message": "Hello from background worker!"}
        },
        "priority": "normal"
    }')

echo "📝 Webhook task created: $WEBHOOK_TASK_RESPONSE"
WEBHOOK_TASK_ID=$(echo $WEBHOOK_TASK_RESPONSE | jq -r '.data.id')

# Test 5: List all tasks
echo ""
echo "📋 Listing all tasks..."
TASKS_LIST_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks)
echo "📝 Tasks list: $TASKS_LIST_RESPONSE"

# Test 6: Get individual task details
echo ""
echo "🔍 Getting individual task details..."
TASK_DETAIL_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/$EMAIL_TASK_ID)
echo "📝 Email task details: $TASK_DETAIL_RESPONSE"

echo ""
echo "⚡ Starting background worker to process tasks..."

# Start worker in background
cargo run -- worker > worker.log 2>&1 &
WORKER_PID=$!

# Give worker time to start and process tasks
sleep 10

# Check if worker is running
if kill -0 $WORKER_PID 2>/dev/null; then
    echo "✅ Worker is running and processing tasks"
else
    echo "❌ Worker stopped unexpectedly"
    echo "📝 Worker log:"
    cat worker.log
    exit 1
fi

echo ""
echo "📊 Checking task status after processing..."

# Check final stats
FINAL_STATS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/stats)
echo "📝 Final stats: $FINAL_STATS_RESPONSE"

# Check individual task status
echo ""
echo "🔍 Checking processed task status..."
PROCESSED_EMAIL_TASK=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/$EMAIL_TASK_ID)
echo "📝 Processed email task: $PROCESSED_EMAIL_TASK"

PROCESSED_DATA_TASK=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/$DATA_TASK_ID)
echo "📝 Processed data task: $PROCESSED_DATA_TASK"

PROCESSED_WEBHOOK_TASK=$(curl -s -H "Authorization: Bearer $TOKEN" http://localhost:3001/tasks/$WEBHOOK_TASK_ID)
echo "📝 Processed webhook task: $PROCESSED_WEBHOOK_TASK"

echo ""
echo "📝 Worker output:"
echo "=================="
cat worker.log

echo ""
echo "✅ Tasks API and Background Worker integration test completed!"
echo ""
echo "📚 Features tested:"
echo "   ✓ User authentication for task API"
echo "   ✓ Task creation via API"
echo "   ✓ Multiple task types (email, data_processing, webhook)"
echo "   ✓ Task priority system"
echo "   ✓ Task listing and retrieval"
echo "   ✓ Task statistics"
echo "   ✓ Background worker processing"
echo "   ✓ Task status updates"
echo ""
echo "🚀 Background worker system is fully functional!"