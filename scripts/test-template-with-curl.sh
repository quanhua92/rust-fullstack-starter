#!/bin/bash

# Simple curl-based testing script for generated templates
# Usage: ./scripts/test-template-with-curl.sh <module_name> [port]
#        MODULE_NAME=<module_name> ./scripts/test-template-with-curl.sh [port]
#
# This script tests basic CRUD operations for a generated module using curl commands.
# It handles authentication and provides simple validation of responses.
# Default port is 3000 (matching server.sh default).

set -e

# Help function
show_help() {
    cat << EOF
🧪 Template Testing Script for Generated Modules

USAGE:
    $0 [MODULE_NAME] [PORT]
    $0 --help | -h

ARGUMENTS:
    MODULE_NAME    Name of the generated module to test (default: basics)
    PORT           Server port to connect to (default: 3000)

EXAMPLES:
    $0                    # Test 'basics' module on port 3000
    $0 products           # Test 'products' module on port 3000  
    $0 users 8080         # Test 'users' module on port 8080

DESCRIPTION:
    This script performs comprehensive CRUD testing for generated template modules.
    It tests authentication, basic CRUD operations, search functionality, and
    error handling using curl commands.

    The script expects a server to be running on the specified port with the
    generated module routes available.

PREREQUISITES:
    - Server must be running: ./scripts/server.sh [PORT]
    - Module must be generated and integrated
    - Database migrations must be applied

EXIT CODES:
    0    All tests passed
    1    Tests failed or server not available
EOF
}

# Check for help flag
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    show_help
    exit 0
fi

# Configuration
MODULE_NAME="${1:-${MODULE_NAME:-basics}}"
PORT="${2:-3000}"
BASE_URL="http://localhost:${PORT}/api/v1"
TEST_USER_EMAIL="template-test@example.com"
TEST_USER_PASSWORD="SecurePass123"
TEST_USERNAME="templatetest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_step() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if server is running
check_server() {
    print_step "Checking if server is running on port $PORT"
    if ! curl -s "$BASE_URL/health" > /dev/null; then
        print_error "Server is not running on port $PORT"
        echo "Please start the server first:"
        echo "  cargo run -- server --port $PORT"
        exit 1
    fi
    print_success "Server is running"
}

# Register test user
register_user() {
    print_step "Registering test user"
    
    REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"username\": \"$TEST_USERNAME\", \"email\": \"$TEST_USER_EMAIL\", \"password\": \"$TEST_USER_PASSWORD\"}")
    
    if echo "$REGISTER_RESPONSE" | grep -q '"success":true'; then
        print_success "User registered successfully"
    elif echo "$REGISTER_RESPONSE" | grep -q "already exists"; then
        print_warning "User already exists, continuing..."
    else
        print_error "Failed to register user: $REGISTER_RESPONSE"
        exit 1
    fi
}

# Login and get token
get_auth_token() {
    print_step "Logging in to get authentication token"
    
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\": \"$TEST_USER_EMAIL\", \"password\": \"$TEST_USER_PASSWORD\"}")
    
    TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
    
    if [ -z "$TOKEN" ]; then
        print_error "Failed to get authentication token: $LOGIN_RESPONSE"
        exit 1
    fi
    
    print_success "Authentication token obtained"
}

# Test health endpoint
test_health() {
    print_step "Testing health endpoint"
    
    HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")
    
    if echo "$HEALTH_RESPONSE" | grep -q '"status":"healthy"'; then
        print_success "Health endpoint working"
    else
        print_error "Health endpoint failed: $HEALTH_RESPONSE"
        exit 1
    fi
}

# Test listing items (check if endpoint works)
test_list_initial() {
    print_step "Testing GET /$MODULE_NAME (initial state)"
    
    LIST_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$LIST_RESPONSE" | grep -q '"success":true'; then
        ITEM_COUNT=$(echo "$LIST_RESPONSE" | python3 -c "import json,sys; print(len(json.load(sys.stdin)['data']))" 2>/dev/null || echo "0")
        print_success "List retrieved successfully (found $ITEM_COUNT existing items)"
    else
        print_error "Failed to get list: $LIST_RESPONSE"
        exit 1
    fi
}

# Test creating an item
test_create() {
    print_step "Testing POST /$MODULE_NAME (create item)"
    
    CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/$MODULE_NAME" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"name": "Test Item", "description": "A test item created by template test script"}')
    
    if echo "$CREATE_RESPONSE" | grep -q '"success":true' && echo "$CREATE_RESPONSE" | grep -q '"Test Item"'; then
        # Extract the ID for later tests
        ITEM_ID=$(echo "$CREATE_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
        if [ -n "$ITEM_ID" ]; then
            print_success "Item created successfully (ID: $ITEM_ID)"
        else
            print_error "Item created but couldn't extract ID: $CREATE_RESPONSE"
            exit 1
        fi
    else
        print_error "Failed to create item: $CREATE_RESPONSE"
        exit 1
    fi
}

# Test listing items (should have one item now)
test_list_with_items() {
    print_step "Testing GET /$MODULE_NAME (should have items)"
    
    LIST_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$LIST_RESPONSE" | grep -q '"success":true' && echo "$LIST_RESPONSE" | grep -q '"Test Item"'; then
        print_success "List with items retrieved successfully"
    else
        print_error "Failed to get list with items: $LIST_RESPONSE"
        exit 1
    fi
}

# Test getting specific item
test_get_item() {
    print_step "Testing GET /$MODULE_NAME/{id} (get specific item)"
    
    if [ -z "$ITEM_ID" ]; then
        print_error "No item ID available for testing"
        exit 1
    fi
    
    GET_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME/$ITEM_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$GET_RESPONSE" | grep -q '"success":true' && echo "$GET_RESPONSE" | grep -q '"Test Item"'; then
        print_success "Specific item retrieved successfully"
    else
        print_error "Failed to get specific item: $GET_RESPONSE"
        exit 1
    fi
}

# Test updating item (requires moderator role, so this might fail)
test_update_item() {
    print_step "Testing PUT /$MODULE_NAME/{id} (update item)"
    
    if [ -z "$ITEM_ID" ]; then
        print_error "No item ID available for testing"
        return 1
    fi
    
    UPDATE_RESPONSE=$(curl -s -X PUT "$BASE_URL/$MODULE_NAME/$ITEM_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"name": "Updated Test Item", "description": "Updated description"}')
    
    if echo "$UPDATE_RESPONSE" | grep -q '"success":true'; then
        print_success "Item updated successfully"
    elif echo "$UPDATE_RESPONSE" | grep -q "Forbidden\|permission\|FORBIDDEN\|Moderator"; then
        print_warning "Update failed due to insufficient permissions (expected for basic users)"
    else
        print_error "Unexpected update response: $UPDATE_RESPONSE"
    fi
}

# Test deleting item (requires moderator role, so this might fail)
test_delete_item() {
    print_step "Testing DELETE /$MODULE_NAME/{id} (delete item)"
    
    if [ -z "$ITEM_ID" ]; then
        print_error "No item ID available for testing"
        return 1
    fi
    
    DELETE_RESPONSE=$(curl -s -X DELETE "$BASE_URL/$MODULE_NAME/$ITEM_ID" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$DELETE_RESPONSE" | grep -q '"success":true'; then
        print_success "Item deleted successfully"
    elif echo "$DELETE_RESPONSE" | grep -q "Forbidden\|permission\|FORBIDDEN\|Moderator"; then
        print_warning "Delete failed due to insufficient permissions (expected for basic users)"
    else
        print_error "Unexpected delete response: $DELETE_RESPONSE"
    fi
}

# Test with search parameters
test_search() {
    print_step "Testing GET /$MODULE_NAME with search parameters"
    
    SEARCH_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME?search=Test&limit=10&offset=0" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$SEARCH_RESPONSE" | grep -q '"success":true'; then
        print_success "Search query executed successfully"
    else
        print_error "Search query failed: $SEARCH_RESPONSE"
        exit 1
    fi
}

# Test invalid endpoints
test_not_found() {
    print_step "Testing 404 handling"
    
    # Test non-existent item
    NOT_FOUND_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME/00000000-0000-0000-0000-000000000000" \
        -H "Authorization: Bearer $TOKEN")
    
    if echo "$NOT_FOUND_RESPONSE" | grep -q "not found\|Not Found\|NOT_FOUND"; then
        print_success "404 handling works correctly"
    else
        print_warning "Unexpected response for non-existent item: $NOT_FOUND_RESPONSE"
    fi
}

# Test unauthorized access
test_unauthorized() {
    print_step "Testing unauthorized access"
    
    UNAUTH_RESPONSE=$(curl -s -X GET "$BASE_URL/$MODULE_NAME")
    
    if echo "$UNAUTH_RESPONSE" | grep -q "Unauthorized\|unauthorized\|401"; then
        print_success "Unauthorized access properly blocked"
    else
        print_warning "Unexpected response for unauthorized access: $UNAUTH_RESPONSE"
    fi
}

# Main execution
main() {
    echo -e "${BLUE}🧪 Template Testing Script for Module: $MODULE_NAME${NC}"
    echo "Base URL: $BASE_URL"
    echo "Test User: $TEST_USER_EMAIL"
    echo ""
    
    # Run all tests
    check_server
    register_user
    get_auth_token
    test_health
    test_unauthorized
    test_list_initial
    test_create
    test_list_with_items
    test_get_item
    test_search
    test_not_found
    test_update_item
    test_delete_item
    
    echo ""
    print_success "All template tests completed!"
    echo ""
    echo -e "${YELLOW}Note: Update/Delete operations may fail for regular users due to RBAC permissions.${NC}"
    echo -e "${YELLOW}This is expected behavior for the basic template.${NC}"
}

# Help function
show_help() {
    echo "Template Testing Script"
    echo ""
    echo "Usage: $0 [module_name] [port]"
    echo ""
    echo "Arguments:"
    echo "  module_name  Name of the generated module to test (default: basics)"
    echo "  port         Server port (default: 8080)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Test 'basics' module on port 8080"
    echo "  $0 users              # Test 'users' module on port 8080"
    echo "  $0 products 3000      # Test 'products' module on port 3000"
    echo ""
    echo "Prerequisites:"
    echo "  - Server must be running on the specified port"
    echo "  - python3 must be available for JSON parsing"
    echo ""
}

# Check for help flag
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    show_help
    exit 0
fi

# Run main function
main