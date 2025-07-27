#!/bin/bash

# Service Chaos Helper
# Simulates various service failures and recoveries

set -e

# Default values
ACTION="${ACTION:-kill}"
SERVICE="${SERVICE:-server}"
PORT="${PORT:-3000}"
DELAY="${DELAY:-5}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

usage() {
    echo "Usage: $0 [ACTION] [OPTIONS]"
    echo ""
    echo "Simulate service failures for chaos testing"
    echo ""
    echo "Actions:"
    echo "  kill           Kill service process (default)"
    echo "  restart        Kill and restart service"
    echo "  db-stop        Stop database"
    echo "  db-restart     Stop and restart database"
    echo "  network        Simulate network issues (not implemented)"
    echo ""
    echo "Options:"
    echo "  -s, --service TYPE     Service type: server|worker (default: $SERVICE)"
    echo "  -p, --port PORT        Server port (default: $PORT)"
    echo "  -d, --delay SECONDS    Delay before restart (default: $DELAY)"
    echo "  -v, --verbose          Verbose output"
    echo "  -h, --help             Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 kill --service server --port 3000"
    echo "  $0 restart --service worker --delay 10"
    echo "  $0 db-stop"
    echo "  $0 db-restart --delay 30"
}

# Parse arguments
VERBOSE=false

# Get action from first argument if provided
if [[ $# -gt 0 ]] && [[ ! "$1" =~ ^- ]]; then
    ACTION="$1"
    shift
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--service)
            SERVICE="$2"
            shift 2
            ;;
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -d|--delay)
            DELAY="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
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

log() {
    if [ "$VERBOSE" = true ]; then
        echo -e "$1"
    fi
}

# Get project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

kill_service() {
    local service_type="$1"
    local port="$2"
    
    case "$service_type" in
        server)
            echo -e "${RED}💀 Killing server on port $port...${NC}"
            log "   Using stop-server.sh script"
            "$PROJECT_ROOT/scripts/stop-server.sh" "$port" || true
            ;;
        worker)
            echo -e "${RED}💀 Killing worker process...${NC}"
            log "   Using stop-worker.sh script"
            "$PROJECT_ROOT/scripts/stop-worker.sh" || true
            ;;
        *)
            echo "Unknown service type: $service_type" >&2
            exit 1
            ;;
    esac
}

start_service() {
    local service_type="$1"
    local port="$2"
    
    case "$service_type" in
        server)
            echo -e "${GREEN}🚀 Starting server on port $port...${NC}"
            log "   Using server.sh script"
            "$PROJECT_ROOT/scripts/server.sh" "$port"
            sleep 2
            echo -e "${BLUE}🔍 Testing server health...${NC}"
            "$PROJECT_ROOT/scripts/test-server.sh" "$port" || echo "   Server health check failed"
            ;;
        worker)
            echo -e "${GREEN}🔄 Starting worker process...${NC}"
            log "   Using worker.sh script"
            "$PROJECT_ROOT/scripts/worker.sh"
            sleep 2
            echo -e "${BLUE}🔍 Checking worker process...${NC}"
            if [ -f "/tmp/starter-worker.pid" ] && kill -0 "$(cat /tmp/starter-worker.pid)" 2>/dev/null; then
                echo "   Worker is running"
            else
                echo "   Worker health check failed"
            fi
            ;;
        *)
            echo "Unknown service type: $service_type" >&2
            exit 1
            ;;
    esac
}

stop_database() {
    echo -e "${RED}🛑 Stopping database...${NC}"
    log "   Using docker-compose stop"
    cd "$PROJECT_ROOT"
    docker-compose stop postgres || echo "   Database may already be stopped"
}

start_database() {
    echo -e "${GREEN}🗄️ Starting database...${NC}"
    log "   Using docker-compose start"
    cd "$PROJECT_ROOT"
    docker-compose start postgres
    
    echo -e "${YELLOW}⏳ Waiting for database to be ready...${NC}"
    timeout=30
    while [ $timeout -gt 0 ]; do
        if docker-compose exec -T postgres pg_isready -U starter_user -d starter_db > /dev/null 2>&1; then
            echo "   Database is ready"
            break
        fi
        sleep 1
        timeout=$((timeout - 1))
    done
    
    if [ $timeout -eq 0 ]; then
        echo "   Warning: Database readiness timeout"
    fi
}

# Execute action
case "$ACTION" in
    kill)
        kill_service "$SERVICE" "$PORT"
        ;;
    restart)
        kill_service "$SERVICE" "$PORT"
        if [ "$DELAY" -gt 0 ]; then
            echo -e "${YELLOW}⏳ Waiting ${DELAY}s before restart...${NC}"
            sleep "$DELAY"
        fi
        start_service "$SERVICE" "$PORT"
        ;;
    db-stop)
        stop_database
        ;;
    db-restart)
        stop_database
        if [ "$DELAY" -gt 0 ]; then
            echo -e "${YELLOW}⏳ Waiting ${DELAY}s before restart...${NC}"
            sleep "$DELAY"
        fi
        start_database
        ;;
    *)
        echo "Unknown action: $ACTION" >&2
        usage >&2
        exit 1
        ;;
esac

echo -e "${GREEN}✅ Chaos action completed: $ACTION${NC}"