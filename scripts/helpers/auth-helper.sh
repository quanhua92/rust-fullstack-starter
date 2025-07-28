#!/bin/bash

# Authentication Helper for Chaos Testing
# Creates test users and returns authentication tokens

set -e

# Default values
BASE_URL="${BASE_URL:-http://localhost:8888}"
USERNAME_PREFIX="${USERNAME_PREFIX:-testuser}"
EMAIL_DOMAIN="${EMAIL_DOMAIN:-example.com}"
PASSWORD="${PASSWORD:-SecurePass123}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Create test user and return authentication token"
    echo ""
    echo "Options:"
    echo "  -u, --url URL          API base URL (default: $BASE_URL)"
    echo "  -p, --prefix PREFIX    Username prefix (default: $USERNAME_PREFIX)"
    echo "  -d, --domain DOMAIN    Email domain (default: $EMAIL_DOMAIN)"
    echo "  -w, --password PASS    Password (default: $PASSWORD)"
    echo "  -t, --timestamp        Use timestamp suffix (default: yes)"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Output: JSON with token and user_id"
    echo "Example: {\"token\": \"abc123...\", \"user_id\": \"uuid-here\"}"
}

# Parse arguments
USE_TIMESTAMP=true
while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -p|--prefix)
            USERNAME_PREFIX="$2"
            shift 2
            ;;
        -d|--domain)
            EMAIL_DOMAIN="$2"
            shift 2
            ;;
        -w|--password)
            PASSWORD="$2"
            shift 2
            ;;
        -t|--timestamp)
            USE_TIMESTAMP=true
            shift
            ;;
        --no-timestamp)
            USE_TIMESTAMP=false
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

# Generate unique identifiers
if [ "$USE_TIMESTAMP" = true ]; then
    TIMESTAMP=$(date +%s%N | cut -b1-13)  # milliseconds
    USERNAME="${USERNAME_PREFIX}_${TIMESTAMP}"
    EMAIL="${USERNAME_PREFIX}_${TIMESTAMP}@${EMAIL_DOMAIN}"
else
    USERNAME="${USERNAME_PREFIX}"
    EMAIL="${USERNAME_PREFIX}@${EMAIL_DOMAIN}"
fi

# Create user
USER_DATA="{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}"
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
    -H "Content-Type: application/json" \
    -d "$USER_DATA")

if ! echo "$REGISTER_RESPONSE" | grep -q '"success":true'; then
    echo "{\"error\": \"Registration failed\", \"response\": $REGISTER_RESPONSE}" >&2
    exit 1
fi

# Login and get token
LOGIN_DATA="{\"username_or_email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "$LOGIN_DATA")

if ! echo "$LOGIN_RESPONSE" | grep -q '"success":true'; then
    echo "{\"error\": \"Login failed\", \"response\": $LOGIN_RESPONSE}" >&2
    exit 1
fi

# Extract token and user ID
TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['session_token'])" 2>/dev/null || echo "")
USER_ID=$(echo "$LOGIN_RESPONSE" | python3 -c "import json,sys; print(json.load(sys.stdin)['data']['user']['id'])" 2>/dev/null || echo "")

if [ -z "$TOKEN" ] || [ -z "$USER_ID" ]; then
    echo "{\"error\": \"Failed to extract token or user_id\", \"response\": $LOGIN_RESPONSE}" >&2
    exit 1
fi

# Output result
echo "{\"token\": \"$TOKEN\", \"user_id\": \"$USER_ID\", \"username\": \"$USERNAME\", \"email\": \"$EMAIL\"}"