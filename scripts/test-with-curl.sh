#!/bin/bash

# Comprehensive API Testing Script
# Tests all endpoints with curl to verify documentation accuracy
#
# Usage:
#   ./test-with-curl.sh                    # Test localhost:3000 (default)
#   ./test-with-curl.sh localhost 8080    # Test localhost:8080
#   ./test-with-curl.sh example.com 443   # Test https://example.com:443

set -e

# Parse command line arguments
HOST=${1:-localhost}
PORT=${2:-3000}

# Determine protocol based on port
if [ "$PORT" = "443" ]; then
    PROTOCOL="https"
else
    PROTOCOL="http"
fi

BASE_URL="${PROTOCOL}://${HOST}:${PORT}"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Comprehensive API Testing with curl${NC}"
echo "=================================================="
echo "Testing: ${HOST}:${PORT} (${PROTOCOL})"
echo "Base URL: $BASE_URL"
echo "Date: $(date)"
echo ""

# Test counter
TOTAL_TESTS=0
PASSED_TESTS=0

# Test function
test_api() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="$4"
    local auth_token="$5"
    local data="$6"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Build curl command
    local curl_cmd="curl -s -w 'HTTP_STATUS:%{http_code}' -X $method"
    
    if [ -n "$data" ]; then
        curl_cmd="$curl_cmd -H 'Content-Type: application/json' -d '$data'"
    fi
    
    if [ -n "$auth_token" ]; then
        curl_cmd="$curl_cmd -H 'Authorization: Bearer $auth_token'"
    fi
    
    curl_cmd="$curl_cmd '$BASE_URL$endpoint'"
    
    # Execute and parse response
    local response=$(eval $curl_cmd 2>/dev/null)
    local status=$(echo "$response" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    local body=$(echo "$response" | sed 's/HTTP_STATUS:[0-9]*$//')
    
    # Check result
    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✅ PASS${NC} $name (Status: $status)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} $name (Expected: $expected_status, Got: $status)"
        if [ -n "$body" ] && [ "$body" != "" ]; then
            echo "    Response: $body"
        fi
        return 1
    fi
}

# Store tokens and IDs globally
USER_TOKEN=""
USER_ID=""
TASK_ID=""

echo -e "${YELLOW}📊 Health Endpoints${NC}"
test_api "GET /api/v1/health" "GET" "/api/v1/health" "200"
test_api "GET /api/v1/health/detailed" "GET" "/api/v1/health/detailed" "200"
test_api "GET /api/v1/health/live" "GET" "/api/v1/health/live" "200"
test_api "GET /api/v1/health/ready" "GET" "/api/v1/health/ready" "200"
test_api "GET /api/v1/health/startup" "GET" "/api/v1/health/startup" "200"

echo ""
echo -e "${YELLOW}🔐 Authentication Flow${NC}"

# Register user with unique name
TIMESTAMP=$(date +%s)
USER_DATA="{\"username\": \"testuser_$TIMESTAMP\", \"email\": \"test_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
test_api "POST /api/v1/auth/register" "POST" "/api/v1/auth/register" "200" "" "$USER_DATA"

# Login user and extract token
echo "🔑 Logging in to get session token..."
LOGIN_DATA="{\"email\": \"test_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" -H "Content-Type: application/json" -d "$LOGIN_DATA")
if echo "$LOGIN_RESPONSE" | grep -q '"success":true'; then
    USER_TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
    USER_ID=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['user']['id'])" 2>/dev/null || echo "")
    if [ -n "$USER_TOKEN" ]; then
        echo -e "${GREEN}✅ PASS${NC} POST /auth/login (Token obtained: ${USER_TOKEN:0:20}...)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC} POST /auth/login (No token in response)"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
else
    echo -e "${RED}❌ FAIL${NC} POST /auth/login (Login failed)"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
fi

# Test protected endpoints with auth
if [ -n "$USER_TOKEN" ]; then
    test_api "GET /api/v1/auth/me" "GET" "/api/v1/auth/me" "200" "$USER_TOKEN"
    test_api "POST /api/v1/auth/refresh" "POST" "/api/v1/auth/refresh" "200" "$USER_TOKEN"
    
    # Test refresh rate limiting (should fail on immediate second request)
    sleep 1 # Brief pause to ensure we're testing rate limiting
    REFRESH_RATE_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X POST "$BASE_URL/api/v1/auth/refresh" -H "Authorization: Bearer $USER_TOKEN")
    REFRESH_RATE_STATUS=$(echo "$REFRESH_RATE_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    if [ "$REFRESH_RATE_STATUS" = "409" ]; then
        echo -e "${GREEN}✅ PASS${NC} POST /api/v1/auth/refresh (rate limited) (Status: $REFRESH_RATE_STATUS - expected 409 CONFLICT)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC} POST /api/v1/auth/refresh (rate limited) (Expected: 409, Got: $REFRESH_RATE_STATUS)"
        REFRESH_RATE_BODY=$(echo "$REFRESH_RATE_RESPONSE" | sed 's/HTTP_STATUS:[0-9]*$//')
        if [ -n "$REFRESH_RATE_BODY" ]; then
            echo "    Response: $REFRESH_RATE_BODY"
        fi
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo ""
    echo -e "${YELLOW}👤 User Management${NC}"
    
    # Test user by ID
    if [ -n "$USER_ID" ]; then
        test_api "GET /api/v1/users/{id}" "GET" "/api/v1/users/$USER_ID" "200" "$USER_TOKEN"
    fi
    
    # Test nonexistent user
    test_api "GET /api/v1/users/nonexistent" "GET" "/api/v1/users/00000000-0000-0000-0000-000000000000" "404" "$USER_TOKEN"
    
    echo ""
    echo -e "${YELLOW}📋 Task Management${NC}"
    
    # Get initial task stats
    test_api "GET /api/v1/tasks/stats" "GET" "/api/v1/tasks/stats" "200" "$USER_TOKEN"
    
    # Test task type management
    echo ""
    echo "🔧 Testing Task Type Management..."
    
    # Wait for workers to register task types (especially important for Docker deployments)
    echo "⏳ Waiting for workers to register task types..."
    wait_attempts=5
    wait_attempt=0
    while [ $wait_attempt -lt $wait_attempts ]; do
        task_types_response=$(curl -s "$BASE_URL/api/v1/tasks/types" 2>/dev/null || echo "")
        if echo "$task_types_response" | grep -q '"task_type".*"email"' && echo "$task_types_response" | grep -q '"task_type".*"webhook"'; then
            echo "✅ Workers have registered expected task types"
            break
        fi
        wait_attempt=$((wait_attempt + 1))
        sleep 1
    done
    
    if [ $wait_attempt -eq $wait_attempts ]; then
        echo "⚠️ Workers may not have fully registered all task types, registering manually for compatibility..."
        # Fallback: register all task types that workers would normally auto-register
        EMAIL_TASK_TYPE='{"task_type": "email", "description": "Email sending task"}'
        test_api "POST /api/v1/tasks/types (email)" "POST" "/api/v1/tasks/types" "200" "" "$EMAIL_TASK_TYPE"
        
        DATA_TASK_TYPE='{"task_type": "data_processing", "description": "Data processing task"}'
        test_api "POST /api/v1/tasks/types (data_processing)" "POST" "/api/v1/tasks/types" "200" "" "$DATA_TASK_TYPE"
        
        WEBHOOK_TASK_TYPE='{"task_type": "webhook", "description": "Webhook notification tasks"}'
        test_api "POST /api/v1/tasks/types (webhook)" "POST" "/api/v1/tasks/types" "200" "" "$WEBHOOK_TASK_TYPE"
        
        FILE_CLEANUP_TASK_TYPE='{"task_type": "file_cleanup", "description": "File system cleanup tasks"}'
        test_api "POST /api/v1/tasks/types (file_cleanup)" "POST" "/api/v1/tasks/types" "200" "" "$FILE_CLEANUP_TASK_TYPE"
        
        REPORT_TASK_TYPE='{"task_type": "report_generation", "description": "Report generation tasks"}'
        test_api "POST /api/v1/tasks/types (report_generation)" "POST" "/api/v1/tasks/types" "200" "" "$REPORT_TASK_TYPE"
        
        DELAY_TASK_TYPE='{"task_type": "delay_task", "description": "Delay/sleep tasks for testing and chaos scenarios"}'
        test_api "POST /api/v1/tasks/types (delay_task)" "POST" "/api/v1/tasks/types" "200" "" "$DELAY_TASK_TYPE"
    fi
    
    # List registered task types  
    test_api "GET /api/v1/tasks/types" "GET" "/api/v1/tasks/types" "200" ""
    
    # Test task creation with valid types
    echo ""
    echo "🔧 Testing Task Creation..."
    
    # Test creating task with unregistered type (should fail with 400)
    UNREGISTERED_TASK='{"task_type": "absolutely_unknown_type_9999", "payload": {"test": "data"}, "priority": "normal"}'
    test_api "POST /api/v1/tasks (unregistered type)" "POST" "/api/v1/tasks" "400" "$USER_TOKEN" "$UNREGISTERED_TASK"
    
    echo ""
    echo "📋 Testing Task Creation..."
    
    # Create email task
    EMAIL_TASK='{"task_type": "email", "payload": {"to": "test@example.com", "subject": "Test Email", "body": "Hello from API test"}, "priority": "normal"}'
    TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/tasks" -H "Content-Type: application/json" -H "Authorization: Bearer $USER_TOKEN" -d "$EMAIL_TASK")
    if echo "$TASK_RESPONSE" | grep -q '"success":true'; then
        TASK_ID=$(echo "$TASK_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
        echo -e "${GREEN}✅ PASS${NC} POST /tasks (Email task created: ${TASK_ID:0:8}...)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC} POST /tasks (Task creation failed)"
        echo "    Response: $TASK_RESPONSE"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Create data processing task
    DATA_TASK='{"task_type": "data_processing", "payload": {"operation": "sum", "data": [1, 2, 3, 4, 5]}, "priority": "high"}'
    test_api "POST /api/v1/tasks (data processing)" "POST" "/api/v1/tasks" "200" "$USER_TOKEN" "$DATA_TASK"
    
    # List tasks
    test_api "GET /api/v1/tasks" "GET" "/api/v1/tasks" "200" "$USER_TOKEN"
    
    # Get specific task
    if [ -n "$TASK_ID" ]; then
        test_api "GET /api/v1/tasks/{id}" "GET" "/api/v1/tasks/$TASK_ID" "200" "$USER_TOKEN"
    fi
    
    # Get updated task stats
    test_api "GET /api/v1/tasks/stats (updated)" "GET" "/api/v1/tasks/stats" "200" "$USER_TOKEN"
    
    # Test task cancellation (might fail if task already processed)
    if [ -n "$TASK_ID" ]; then
        # This might return 400 if task is already completed, which is expected behavior
        CANCEL_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X POST "$BASE_URL/api/v1/tasks/$TASK_ID/cancel" -H "Authorization: Bearer $USER_TOKEN")
        CANCEL_STATUS=$(echo "$CANCEL_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
        if [ "$CANCEL_STATUS" = "200" ] || [ "$CANCEL_STATUS" = "400" ]; then
            echo -e "${GREEN}✅ PASS${NC} POST /tasks/{id}/cancel (Status: $CANCEL_STATUS - expected 200 or 400)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAIL${NC} POST /tasks/{id}/cancel (Expected: 200 or 400, Got: $CANCEL_STATUS)"
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    echo ""
    echo -e "${YELLOW}🗃️ Dead Letter Queue Management${NC}"
    
    # Test filtering tasks by status
    test_api "GET /api/v1/tasks?status=pending" "GET" "/api/v1/tasks?status=pending" "200" "$USER_TOKEN"
    test_api "GET /api/v1/tasks?status=failed" "GET" "/api/v1/tasks?status=failed" "200" "$USER_TOKEN"
    
    # Test dead letter queue endpoint
    test_api "GET /api/v1/tasks/dead-letter" "GET" "/api/v1/tasks/dead-letter" "200" "$USER_TOKEN"
    test_api "GET /api/v1/tasks/dead-letter (paginated)" "GET" "/api/v1/tasks/dead-letter?limit=5&offset=0" "200" "$USER_TOKEN"
    
    # Create a task that we can mark as failed for testing
    FAILED_TASK='{"task_type": "email", "payload": {"to": "test@example.com", "subject": "Test Failed", "body": "fail"}, "priority": "normal"}'
    FAILED_TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/tasks" -H "Content-Type: application/json" -H "Authorization: Bearer $USER_TOKEN" -d "$FAILED_TASK")
    if echo "$FAILED_TASK_RESPONSE" | grep -q '"success":true'; then
        FAILED_TASK_ID=$(echo "$FAILED_TASK_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
        echo -e "${GREEN}✅ PASS${NC} POST /tasks (Failed task created: ${FAILED_TASK_ID:0:8}...)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC} POST /tasks (Failed task creation failed)"
        echo "    Response: $FAILED_TASK_RESPONSE"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Test retry endpoint (on non-failed task - should fail)
    if [ -n "$TASK_ID" ]; then
        RETRY_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X POST "$BASE_URL/api/v1/tasks/$TASK_ID/retry" -H "Authorization: Bearer $USER_TOKEN")
        RETRY_STATUS=$(echo "$RETRY_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
        if [ "$RETRY_STATUS" = "404" ]; then
            echo -e "${GREEN}✅ PASS${NC} POST /tasks/{id}/retry (pending task) (Status: $RETRY_STATUS - expected 404)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAIL${NC} POST /tasks/{id}/retry (pending task) (Expected: 404, Got: $RETRY_STATUS)"
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    # Test delete endpoint (task may be completed by worker, so expect 200 or 404)
    if [ -n "$TASK_ID" ]; then
        DELETE_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X DELETE "$BASE_URL/api/v1/tasks/$TASK_ID" -H "Authorization: Bearer $USER_TOKEN")
        DELETE_STATUS=$(echo "$DELETE_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
        if [ "$DELETE_STATUS" = "200" ] || [ "$DELETE_STATUS" = "404" ]; then
            echo -e "${GREEN}✅ PASS${NC} DELETE /tasks/{id} (task) (Status: $DELETE_STATUS - expected 200 or 404)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAIL${NC} DELETE /tasks/{id} (task) (Expected: 200 or 404, Got: $DELETE_STATUS)"
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    # Test retry on nonexistent task
    FAKE_TASK_ID="00000000-0000-0000-0000-000000000000"
    test_api "POST /api/v1/tasks/{id}/retry (nonexistent)" "POST" "/api/v1/tasks/$FAKE_TASK_ID/retry" "404" "$USER_TOKEN"
    
    # Test delete on nonexistent task
    test_api "DELETE /api/v1/tasks/{id} (nonexistent)" "DELETE" "/api/v1/tasks/$FAKE_TASK_ID" "404" "$USER_TOKEN"
    
    # Test logout-all endpoint before regular logout
    echo ""
    echo "🔐 Testing Multi-Session Logout..."
    # Create a second session for testing logout-all
    SECOND_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" -H "Content-Type: application/json" -d "$LOGIN_DATA")
    SECOND_TOKEN=$(echo "$SECOND_LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
    
    if [ -n "$SECOND_TOKEN" ]; then
        # Test logout-all with the first token (should invalidate both sessions)
        test_api "POST /api/v1/auth/logout-all" "POST" "/api/v1/auth/logout-all" "200" "$USER_TOKEN"
        
        # Verify both tokens are invalidated by testing /auth/me with the second token
        ME_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X GET "$BASE_URL/api/v1/auth/me" -H "Authorization: Bearer $SECOND_TOKEN")
        ME_STATUS=$(echo "$ME_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
        if [ "$ME_STATUS" = "401" ]; then
            echo -e "${GREEN}✅ PASS${NC} All sessions invalidated after logout-all"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAIL${NC} Second token should be invalidated after logout-all (Got: $ME_STATUS)"
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    else
        # Fallback if second login fails - just test logout-all endpoint
        test_api "POST /api/v1/auth/logout-all" "POST" "/api/v1/auth/logout-all" "200" "$USER_TOKEN"
    fi
    
    # Test logout (single session) - test this last since it invalidates the token
    # Note: This might return 401 if logout-all already invalidated the token, which is expected
    LOGOUT_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X POST "$BASE_URL/api/v1/auth/logout" -H "Authorization: Bearer $USER_TOKEN")
    LOGOUT_STATUS=$(echo "$LOGOUT_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
    if [ "$LOGOUT_STATUS" = "200" ] || [ "$LOGOUT_STATUS" = "401" ]; then
        echo -e "${GREEN}✅ PASS${NC} POST /auth/logout (Status: $LOGOUT_STATUS - expected 200 or 401)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC} POST /auth/logout (Expected: 200 or 401, Got: $LOGOUT_STATUS)"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
fi

echo ""
echo -e "${YELLOW}❌ Error Response Testing${NC}"

# Test unauthorized access
test_api "GET /api/v1/auth/me (no auth)" "GET" "/api/v1/auth/me" "401"

# Test invalid login
INVALID_LOGIN='{"username": "wrong", "password": "wrong"}'
test_api "POST /api/v1/auth/login (invalid)" "POST" "/api/v1/auth/login" "401" "" "$INVALID_LOGIN"

# Test validation error
INVALID_REGISTER='{"username": "", "email": "invalid", "password": "weak"}'
test_api "POST /api/v1/auth/register (validation)" "POST" "/api/v1/auth/register" "400" "" "$INVALID_REGISTER"

# Test 404
test_api "GET /nonexistent" "GET" "/nonexistent" "404"

echo ""
echo -e "${YELLOW}📚 Documentation Endpoints${NC}"

# Test documentation endpoints (public access)
test_api "GET /api-docs" "GET" "/api-docs" "200"
test_api "GET /api-docs/openapi.json" "GET" "/api-docs/openapi.json" "200"

echo ""
echo -e "${YELLOW}👑 Admin Endpoints${NC}"

# Test admin endpoint without auth
test_api "GET /api/v1/admin/health (no auth)" "GET" "/api/v1/admin/health" "401"


# Note: Admin endpoint with proper admin credentials would require setting up 
# an admin user, which is not configured in this test environment

echo ""
echo -e "${YELLOW}🧪 Additional API Tests${NC}"

# Test task types that should work
if [ -n "$USER_TOKEN" ]; then
    # Create a new user for clean testing
    NEW_USER_DATA="{\"username\": \"testuser2_$TIMESTAMP\", \"email\": \"test2_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
    curl -s -X POST "$BASE_URL/api/v1/auth/register" -H "Content-Type: application/json" -d "$NEW_USER_DATA" > /dev/null
    
    NEW_LOGIN_DATA="{\"email\": \"test2_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
    NEW_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" -H "Content-Type: application/json" -d "$NEW_LOGIN_DATA")
    NEW_TOKEN=$(echo "$NEW_LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
    
    if [ -n "$NEW_TOKEN" ]; then
        # Test all supported task types
        WEBHOOK_TASK='{"task_type": "webhook", "payload": {"url": "https://httpbin.org/post", "method": "POST", "payload": {"test": "data"}}, "priority": "normal"}'
        test_api "POST /api/v1/tasks (webhook)" "POST" "/api/v1/tasks" "200" "$NEW_TOKEN" "$WEBHOOK_TASK"
        
        FILE_CLEANUP_TASK='{"task_type": "file_cleanup", "payload": {"file_path": "/tmp/test", "max_age_hours": 24}, "priority": "low"}'
        test_api "POST /api/v1/tasks (file_cleanup)" "POST" "/api/v1/tasks" "200" "$NEW_TOKEN" "$FILE_CLEANUP_TASK"
        
        REPORT_TASK='{"task_type": "report_generation", "payload": {"report_type": "sales", "start_date": "2024-01-01", "end_date": "2024-01-31", "format": "pdf"}, "priority": "normal"}'
        test_api "POST /api/v1/tasks (report_generation)" "POST" "/api/v1/tasks" "200" "$NEW_TOKEN" "$REPORT_TASK"
        
        # Test unknown task type (should now be rejected by API)
        UNKNOWN_TASK='{"task_type": "truly_unknown_type_12345", "payload": {"test": "data"}, "priority": "normal"}'
        test_api "POST /api/v1/tasks (unknown type)" "POST" "/api/v1/tasks" "400" "$NEW_TOKEN" "$UNKNOWN_TASK"
        
        # Test admin endpoint with regular user (should get 401)
        test_api "GET /api/v1/admin/health (non-admin)" "GET" "/api/v1/admin/health" "401" "$NEW_TOKEN"
    fi
fi

echo ""
echo "=================================================="
echo -e "${BLUE}📊 Test Results Summary${NC}"
echo "Total tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $((TOTAL_TESTS - PASSED_TESTS))"
echo "Success rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
echo ""

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "${GREEN}🎉 All API endpoints work as documented!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  $((TOTAL_TESTS - PASSED_TESTS)) test(s) failed${NC}"
    exit 1
fi