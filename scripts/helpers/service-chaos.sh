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

# Docker compose file for chaos testing
CHAOS_COMPOSE_FILE="$PROJECT_ROOT/docker-compose.chaos.yaml"

kill_service() {
    local service_type="$1"
    local port="$2"
    
    cd "$PROJECT_ROOT"
    
    case "$service_type" in
        server)
            echo -e "${RED}💀 Killing Docker server container...${NC}"
            log "   Stopping server container"
            docker-compose -f "$CHAOS_COMPOSE_FILE" stop server || true
            ;;
        worker)
            echo -e "${RED}💀 Killing Docker worker container...${NC}"
            log "   Stopping worker container"
            docker-compose -f "$CHAOS_COMPOSE_FILE" stop worker || true
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
    
    cd "$PROJECT_ROOT"
    
    case "$service_type" in
        server)
            echo -e "${GREEN}🚀 Starting Docker server container...${NC}"
            log "   Starting server container"
            docker-compose -f "$CHAOS_COMPOSE_FILE" up -d server
            
            echo -e "${YELLOW}⏳ Waiting for server to be ready...${NC}"
            local max_attempts=30
            local attempt=0
            while [ $attempt -lt $max_attempts ]; do
                if curl -s -f "http://localhost:$port/health" > /dev/null 2>&1; then
                    echo -e "${GREEN}✅ Server is ready${NC}"
                    break
                fi
                attempt=$((attempt + 1))
                sleep 2
            done
            
            if [ $attempt -eq $max_attempts ]; then
                echo -e "${RED}❌ Server readiness timeout${NC}"
            fi
            ;;
        worker)
            echo -e "${GREEN}🔄 Starting Docker worker container...${NC}"
            log "   Starting worker container"
            docker-compose -f "$CHAOS_COMPOSE_FILE" up -d worker
            
            echo -e "${YELLOW}⏳ Waiting for worker to be ready...${NC}"
            sleep 5
            
            echo -e "${BLUE}🔍 Checking worker container...${NC}"
            if docker-compose -f "$CHAOS_COMPOSE_FILE" ps worker | grep -q "Up"; then
                echo -e "${GREEN}✅ Worker is running${NC}"
            else
                echo -e "${RED}❌ Worker health check failed${NC}"
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
    log "   Using chaos docker-compose stop"
    cd "$PROJECT_ROOT"
    docker-compose -f "$CHAOS_COMPOSE_FILE" stop postgres || echo "   Database may already be stopped"
}

start_database() {
    echo -e "${GREEN}🗄️ Starting database...${NC}"
    log "   Using chaos docker-compose start"
    cd "$PROJECT_ROOT"
    docker-compose -f "$CHAOS_COMPOSE_FILE" start postgres
    
    echo -e "${YELLOW}⏳ Waiting for database to be ready...${NC}"
    timeout=30
    while [ $timeout -gt 0 ]; do
        if docker-compose -f "$CHAOS_COMPOSE_FILE" exec -T postgres pg_isready -U starter_user -d starter_db > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Database is ready${NC}"
            break
        fi
        sleep 1
        timeout=$((timeout - 1))
    done
    
    if [ $timeout -eq 0 ]; then
        echo -e "${YELLOW}⚠️ Warning: Database readiness timeout${NC}"
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