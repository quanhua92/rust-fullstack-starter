#!/bin/bash

# Full-stack development server script
# Builds the web frontend and starts the Rust server with static file serving

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
PORT=3000
FOREGROUND=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -f|--foreground)
            FOREGROUND=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  -p, --port PORT     Set server port (default: 3000)"
            echo "  -f, --foreground    Run server in foreground mode"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use -h or --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}🚀 Starting full-stack development environment...${NC}"
echo -e "${BLUE}================================${NC}"

# Get project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}📁 Project root: $PROJECT_ROOT${NC}"
echo -e "${BLUE}🌐 Server port: $PORT${NC}"

cd "$PROJECT_ROOT"

# Step 1: Build web frontend
echo -e "\n${BLUE}🏗️  Step 1: Building web frontend...${NC}"
if ! ./scripts/build-web.sh; then
    echo -e "${RED}❌ Web frontend build failed!${NC}"
    exit 1
fi

# Step 2: Start database if not running
echo -e "\n${BLUE}🗄️  Step 2: Checking database...${NC}"
if command -v docker-compose >/dev/null 2>&1; then
    if ! docker-compose ps postgres | grep -q "Up"; then
        echo -e "${YELLOW}⚠️  Database not running, starting...${NC}"
        docker-compose up -d postgres
        echo -e "${GREEN}✅ Database started${NC}"
    else
        echo -e "${GREEN}✅ Database already running${NC}"
    fi
elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    if ! docker compose ps postgres | grep -q "Up"; then
        echo -e "${YELLOW}⚠️  Database not running, starting...${NC}"
        docker compose up -d postgres
        echo -e "${GREEN}✅ Database started${NC}"
    else
        echo -e "${GREEN}✅ Database already running${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Docker not available, assuming database is available${NC}"
fi

# Step 3: Start the Rust server
echo -e "\n${BLUE}🦀 Step 3: Starting Rust server...${NC}"

# Set absolute path for web build directory
export STARTER__SERVER__WEB_BUILD_PATH="$PROJECT_ROOT/web/dist"

if [ "$FOREGROUND" = true ]; then
    echo -e "${CYAN}🔥 Starting server in foreground mode on port $PORT...${NC}"
    echo -e "${CYAN}💡 Press Ctrl+C to stop the server${NC}"
    echo -e "${BLUE}================================${NC}"
    ./scripts/server.sh "$PORT" -f
else
    echo -e "${CYAN}🔥 Starting server in background mode on port $PORT...${NC}"
    ./scripts/server.sh "$PORT"
    
    # Wait a moment for server to start
    sleep 2
    
    echo -e "\n${BLUE}================================${NC}"
    echo -e "${GREEN}🎉 Full-stack development environment ready!${NC}"
    echo -e "\n${CYAN}📋 Available endpoints:${NC}"
    echo -e "   🌐 Frontend: ${BLUE}http://localhost:$PORT${NC}"
    echo -e "   🔌 API: ${BLUE}http://localhost:$PORT/api/v1${NC}"
    echo -e "   📚 API Docs: ${BLUE}http://localhost:$PORT/api-docs${NC}"
    echo -e "   ❤️  Health: ${BLUE}http://localhost:$PORT/api/v1/health${NC}"
    
    echo -e "\n${CYAN}🛠️  Development commands:${NC}"
    echo -e "   Stop server: ${BLUE}./scripts/stop-server.sh $PORT${NC}"
    echo -e "   Start worker: ${BLUE}./scripts/worker.sh -f${NC}"
    echo -e "   Rebuild web: ${BLUE}./scripts/build-web.sh${NC}"
    echo -e "   Full restart: ${BLUE}./scripts/dev-full-stack.sh -p $PORT${NC}"
    
    echo -e "\n${CYAN}🧪 Testing commands:${NC}"
    echo -e "   API tests: ${BLUE}./scripts/test-with-curl.sh localhost $PORT${NC}"
    echo -e "   Quality checks: ${BLUE}./scripts/check.sh${NC}"
fi