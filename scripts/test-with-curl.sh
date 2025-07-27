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
test_api "GET /health" "GET" "/health" "200"
test_api "GET /health/detailed" "GET" "/health/detailed" "200"

echo ""
echo -e "${YELLOW}🔐 Authentication Flow${NC}"

# Register user with unique name
TIMESTAMP=$(date +%s)
USER_DATA="{\"username\": \"testuser_$TIMESTAMP\", \"email\": \"test_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
test_api "POST /auth/register" "POST" "/auth/register" "200" "" "$USER_DATA"

# Login user and extract token
echo "🔑 Logging in to get session token..."
LOGIN_DATA="{\"username_or_email\": \"test_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" -H "Content-Type: application/json" -d "$LOGIN_DATA")
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
    test_api "GET /auth/me" "GET" "/auth/me" "200" "$USER_TOKEN"
    test_api "POST /auth/refresh" "POST" "/auth/refresh" "200" "$USER_TOKEN"
    
    echo ""
    echo -e "${YELLOW}👤 User Management${NC}"
    
    # Test user by ID
    if [ -n "$USER_ID" ]; then
        test_api "GET /users/{id}" "GET" "/users/$USER_ID" "200" "$USER_TOKEN"
    fi
    
    # Test nonexistent user
    test_api "GET /users/nonexistent" "GET" "/users/00000000-0000-0000-0000-000000000000" "404" "$USER_TOKEN"
    
    echo ""
    echo -e "${YELLOW}📋 Task Management${NC}"
    
    # Get initial task stats
    test_api "GET /tasks/stats" "GET" "/tasks/stats" "200" "$USER_TOKEN"
    
    # Create email task
    EMAIL_TASK='{"task_type": "email", "payload": {"to": "test@example.com", "subject": "Test Email", "body": "Hello from API test"}, "priority": "normal"}'
    TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/tasks" -H "Content-Type: application/json" -H "Authorization: Bearer $USER_TOKEN" -d "$EMAIL_TASK")
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
    test_api "POST /tasks (data processing)" "POST" "/tasks" "200" "$USER_TOKEN" "$DATA_TASK"
    
    # List tasks
    test_api "GET /tasks" "GET" "/tasks" "200" "$USER_TOKEN"
    
    # Get specific task
    if [ -n "$TASK_ID" ]; then
        test_api "GET /tasks/{id}" "GET" "/tasks/$TASK_ID" "200" "$USER_TOKEN"
    fi
    
    # Get updated task stats
    test_api "GET /tasks/stats (updated)" "GET" "/tasks/stats" "200" "$USER_TOKEN"
    
    # Test task cancellation (might fail if task already processed)
    if [ -n "$TASK_ID" ]; then
        # This might return 400 if task is already completed, which is expected behavior
        CANCEL_RESPONSE=$(curl -s -w 'HTTP_STATUS:%{http_code}' -X POST "$BASE_URL/tasks/$TASK_ID/cancel" -H "Authorization: Bearer $USER_TOKEN")
        CANCEL_STATUS=$(echo "$CANCEL_RESPONSE" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2)
        if [ "$CANCEL_STATUS" = "200" ] || [ "$CANCEL_STATUS" = "400" ]; then
            echo -e "${GREEN}✅ PASS${NC} POST /tasks/{id}/cancel (Status: $CANCEL_STATUS - expected 200 or 400)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAIL${NC} POST /tasks/{id}/cancel (Expected: 200 or 400, Got: $CANCEL_STATUS)"
        fi
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
    fi
    
    # Test logout (single session) - test this last since it invalidates the token
    test_api "POST /auth/logout" "POST" "/auth/logout" "200" "$USER_TOKEN"
fi

echo ""
echo -e "${YELLOW}❌ Error Response Testing${NC}"

# Test unauthorized access
test_api "GET /auth/me (no auth)" "GET" "/auth/me" "401"

# Test invalid login
INVALID_LOGIN='{"username_or_email": "wrong", "password": "wrong"}'
test_api "POST /auth/login (invalid)" "POST" "/auth/login" "401" "" "$INVALID_LOGIN"

# Test validation error
INVALID_REGISTER='{"username": "", "email": "invalid", "password": "weak"}'
test_api "POST /auth/register (validation)" "POST" "/auth/register" "400" "" "$INVALID_REGISTER"

# Test 404
test_api "GET /nonexistent" "GET" "/nonexistent" "404"

echo ""
echo -e "${YELLOW}👑 Admin Endpoints${NC}"

# Test admin endpoint without auth
test_api "GET /admin/health (no auth)" "GET" "/admin/health" "401"


# Note: Admin endpoint with proper admin credentials would require setting up 
# an admin user, which is not configured in this test environment

echo ""
echo -e "${YELLOW}🧪 Additional API Tests${NC}"

# Test task types that should work
if [ -n "$USER_TOKEN" ]; then
    # Create a new user for clean testing
    NEW_USER_DATA="{\"username\": \"testuser2_$TIMESTAMP\", \"email\": \"test2_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
    curl -s -X POST "$BASE_URL/auth/register" -H "Content-Type: application/json" -d "$NEW_USER_DATA" > /dev/null
    
    NEW_LOGIN_DATA="{\"username_or_email\": \"test2_$TIMESTAMP@example.com\", \"password\": \"SecurePass123\"}"
    NEW_LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" -H "Content-Type: application/json" -d "$NEW_LOGIN_DATA")
    NEW_TOKEN=$(echo "$NEW_LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
    
    if [ -n "$NEW_TOKEN" ]; then
        # Test all supported task types
        WEBHOOK_TASK='{"task_type": "webhook", "payload": {"url": "https://httpbin.org/post", "method": "POST", "payload": {"test": "data"}}, "priority": "normal"}'
        test_api "POST /tasks (webhook)" "POST" "/tasks" "200" "$NEW_TOKEN" "$WEBHOOK_TASK"
        
        FILE_CLEANUP_TASK='{"task_type": "file_cleanup", "payload": {"file_path": "/tmp/test", "max_age_hours": 24}, "priority": "low"}'
        test_api "POST /tasks (file_cleanup)" "POST" "/tasks" "200" "$NEW_TOKEN" "$FILE_CLEANUP_TASK"
        
        REPORT_TASK='{"task_type": "report_generation", "payload": {"report_type": "sales", "start_date": "2024-01-01", "end_date": "2024-01-31", "format": "pdf"}, "priority": "normal"}'
        test_api "POST /tasks (report_generation)" "POST" "/tasks" "200" "$NEW_TOKEN" "$REPORT_TASK"
        
        # Test unknown task type (API accepts it, worker will reject during processing)
        UNKNOWN_TASK='{"task_type": "nonexistent", "payload": {"test": "data"}, "priority": "normal"}'
        test_api "POST /tasks (unknown type)" "POST" "/tasks" "200" "$NEW_TOKEN" "$UNKNOWN_TASK"
        
        # Test admin endpoint with regular user (should get 401)
        test_api "GET /admin/health (non-admin)" "GET" "/admin/health" "401" "$NEW_TOKEN"
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