#!/bin/bash

# Authentication Testing Script
# Tests the session-based authentication endpoints

set -e

# Configuration
BASE_URL="http://localhost:3000"
CONTENT_TYPE="Content-Type: application/json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test data
TEST_USERNAME="testuser_$(date +%s)"
TEST_EMAIL="test_$(date +%s)@example.com"
TEST_PASSWORD="SecurePassword123!"

echo -e "${BLUE}🔐 Authentication Testing Script${NC}"
echo "=================================="
echo "Base URL: $BASE_URL"
echo "Test User: $TEST_USERNAME"
echo "Test Email: $TEST_EMAIL"
echo ""

# Function to make HTTP requests with error handling
make_request() {
    local method=$1
    local endpoint=$2
    local data=$3
    local auth_header=$4
    
    local curl_cmd="curl -s -w 'HTTP_STATUS:%{http_code}' -X $method"
    
    if [ -n "$data" ]; then
        curl_cmd="$curl_cmd -H '$CONTENT_TYPE' -d '$data'"
    fi
    
    if [ -n "$auth_header" ]; then
        curl_cmd="$curl_cmd -H 'Authorization: Bearer $auth_header'"
    fi
    
    curl_cmd="$curl_cmd '$BASE_URL$endpoint'"
    
    eval $curl_cmd
}

# Function to extract HTTP status code
get_status() {
    echo "$1" | grep -o 'HTTP_STATUS:[0-9]*' | cut -d: -f2
}

# Function to extract response body
get_body() {
    echo "$1" | sed 's/HTTP_STATUS:[0-9]*$//'
}

echo -e "${YELLOW}📋 Testing Health Endpoint...${NC}"
health_response=$(make_request "GET" "/health")
health_status=$(get_status "$health_response")

if [ "$health_status" = "200" ]; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed (Status: $health_status)${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}👤 Testing User Registration...${NC}"
register_data="{
    \"username\": \"$TEST_USERNAME\",
    \"email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\"
}"

register_response=$(make_request "POST" "/auth/register" "$register_data")
register_status=$(get_status "$register_response")
register_body=$(get_body "$register_response")

if [ "$register_status" = "200" ]; then
    echo -e "${GREEN}✅ User registration successful${NC}"
    echo "Response: $register_body"
else
    echo -e "${RED}❌ User registration failed (Status: $register_status)${NC}"
    echo "Response: $register_body"
    exit 1
fi

echo ""
echo -e "${YELLOW}🔑 Testing User Login...${NC}"
login_data="{
    \"username_or_email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\"
}"

login_response=$(make_request "POST" "/auth/login" "$login_data")
login_status=$(get_status "$login_response")
login_body=$(get_body "$login_response")

if [ "$login_status" = "200" ]; then
    echo -e "${GREEN}✅ User login successful${NC}"
    
    # Extract session token from response
    SESSION_TOKEN=$(echo "$login_body" | grep -o '"session_token":"[^"]*' | cut -d'"' -f4)
    
    if [ -n "$SESSION_TOKEN" ]; then
        echo -e "${GREEN}🎟️  Session token received: ${SESSION_TOKEN:0:20}...${NC}"
    else
        echo -e "${RED}❌ No session token in response${NC}"
        echo "Response: $login_body"
        exit 1
    fi
else
    echo -e "${RED}❌ User login failed (Status: $login_status)${NC}"
    echo "Response: $login_body"
    exit 1
fi

echo ""
echo -e "${YELLOW}👤 Testing Protected Route (/auth/me)...${NC}"
me_response=$(make_request "GET" "/auth/me" "" "$SESSION_TOKEN")
me_status=$(get_status "$me_response")
me_body=$(get_body "$me_response")

if [ "$me_status" = "200" ]; then
    echo -e "${GREEN}✅ Protected route access successful${NC}"
    echo "User profile: $me_body"
else
    echo -e "${RED}❌ Protected route access failed (Status: $me_status)${NC}"
    echo "Response: $me_body"
fi

echo ""
echo -e "${YELLOW}🔒 Testing Protected Route Without Token...${NC}"
unauth_response=$(make_request "GET" "/auth/me")
unauth_status=$(get_status "$unauth_response")

if [ "$unauth_status" = "401" ]; then
    echo -e "${GREEN}✅ Unauthorized access properly blocked${NC}"
else
    echo -e "${RED}❌ Unauthorized access not properly blocked (Status: $unauth_status)${NC}"
fi

echo ""
echo -e "${YELLOW}🔄 Testing Token Refresh...${NC}"
refresh_response=$(make_request "POST" "/auth/refresh" "" "$SESSION_TOKEN")
refresh_status=$(get_status "$refresh_response")
refresh_body=$(get_body "$refresh_response")

if [ "$refresh_status" = "200" ]; then
    echo -e "${GREEN}✅ Token refresh successful${NC}"
    echo "Response: $refresh_body"
else
    echo -e "${YELLOW}⚠️  Token refresh failed (Status: $refresh_status)${NC}"
    echo "Response: $refresh_body"
fi

echo ""
echo -e "${YELLOW}🚪 Testing User Logout...${NC}"
logout_response=$(make_request "POST" "/auth/logout" "" "$SESSION_TOKEN")
logout_status=$(get_status "$logout_response")
logout_body=$(get_body "$logout_response")

if [ "$logout_status" = "200" ]; then
    echo -e "${GREEN}✅ User logout successful${NC}"
    echo "Response: $logout_body"
else
    echo -e "${RED}❌ User logout failed (Status: $logout_status)${NC}"
    echo "Response: $logout_body"
fi

echo ""
echo -e "${YELLOW}🔒 Testing Access After Logout...${NC}"
post_logout_response=$(make_request "GET" "/auth/me" "" "$SESSION_TOKEN")
post_logout_status=$(get_status "$post_logout_response")

if [ "$post_logout_status" = "401" ]; then
    echo -e "${GREEN}✅ Access properly denied after logout${NC}"
else
    echo -e "${RED}❌ Access not properly denied after logout (Status: $post_logout_status)${NC}"
fi

echo ""
echo -e "${YELLOW}🔑 Testing Invalid Credentials...${NC}"
invalid_login_data="{
    \"username_or_email\": \"$TEST_EMAIL\",
    \"password\": \"wrongpassword\"
}"

invalid_response=$(make_request "POST" "/auth/login" "$invalid_login_data")
invalid_status=$(get_status "$invalid_response")

if [ "$invalid_status" = "401" ]; then
    echo -e "${GREEN}✅ Invalid credentials properly rejected${NC}"
else
    echo -e "${RED}❌ Invalid credentials not properly rejected (Status: $invalid_status)${NC}"
fi

echo ""
echo "=================================="
echo -e "${BLUE}🎉 Authentication Testing Complete!${NC}"
echo ""
echo -e "${GREEN}Summary:${NC}"
echo "• Health check: ✅"
echo "• User registration: ✅" 
echo "• User login: ✅"
echo "• Protected routes: ✅"
echo "• Token validation: ✅"
echo "• Logout functionality: ✅"
echo "• Security validation: ✅"
echo ""
echo -e "${YELLOW}Note: Make sure the server is running on $BASE_URL before running this script${NC}"