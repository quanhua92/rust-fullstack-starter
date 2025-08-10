#!/bin/bash
# Production deployment script

set -e

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Configuration
ENV_FILE=".env.prod"
COMPOSE_FILE="docker-compose.prod.yaml"
BACKUP_DIR="./backups"

print_status "step" "Production Deployment Script"
echo "=================================="

# Check if running as root (not recommended)
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Warning: Running as root is not recommended for production${NC}"
    read -p "Continue anyway? (y/N): " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled."
        exit 1
    fi
fi

# Check for required files
echo -e "${BLUE}📋 Checking prerequisites...${NC}"

if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}❌ Error: $ENV_FILE not found${NC}"
    echo ""
    echo "Please create production environment file:"
    echo "1. Copy template: cp .env.prod.example .env.prod"
    echo "2. Edit settings: nano .env.prod"
    echo "3. Set strong passwords and secrets"
    exit 1
fi

if [ ! -f "$COMPOSE_FILE" ]; then
    echo -e "${RED}❌ Error: $COMPOSE_FILE not found${NC}"
    exit 1
fi

# Validate environment variables
echo -e "${BLUE}🔍 Validating environment configuration...${NC}"

# Source the env file to check variables
set -a
source "$ENV_FILE"
set +a

# Check required variables
REQUIRED_VARS=(
    "POSTGRES_PASSWORD"
    "SESSION_SECRET"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        echo -e "${RED}❌ Error: Required variable $var is not set in $ENV_FILE${NC}"
        exit 1
    fi
done

# Check if passwords are changed from defaults
if [[ "$POSTGRES_PASSWORD" == "CHANGE_ME_STRONG_PASSWORD_HERE" ]]; then
    echo -e "${RED}❌ Error: POSTGRES_PASSWORD must be changed from default${NC}"
    exit 1
fi

if [[ "$SESSION_SECRET" == "CHANGE_ME_64_CHARACTER_SECRET_FOR_SESSION_SIGNING_12345678" ]]; then
    echo -e "${RED}❌ Error: SESSION_SECRET must be changed from default${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Environment validation passed${NC}"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Check if this is an update (existing containers)
if docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps -q postgres 2>/dev/null | grep -q .; then
    echo -e "${YELLOW}📦 Existing deployment detected${NC}"
    
    read -p "Create database backup before update? (Y/n): " backup_confirm
    if [[ ! $backup_confirm =~ ^[Nn]$ ]]; then
        echo -e "${BLUE}💾 Creating database backup...${NC}"
        docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" exec postgres /usr/local/bin/backup-db.sh || {
            echo -e "${RED}❌ Backup failed${NC}"
            exit 1
        }
        echo -e "${GREEN}✅ Backup completed${NC}"
    fi
fi

# Build and deploy
echo -e "${BLUE}🔨 Building application...${NC}"
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" build --no-cache

echo -e "${BLUE}🚀 Starting services...${NC}"
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d

# Wait for services to be healthy
echo -e "${BLUE}⏳ Waiting for services to be healthy...${NC}"
timeout=60
counter=0

while [ $counter -lt $timeout ]; do
    if docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps --format json | jq -r '.[].Health' | grep -q "unhealthy"; then
        echo -e "${RED}❌ Some services are unhealthy${NC}"
        docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps
        exit 1
    fi
    
    # Check if all services are healthy
    if docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps --format json | jq -r '.[].Health' | grep -v "healthy" | grep -q "starting\|none"; then
        echo -n "."
        sleep 2
        counter=$((counter + 2))
    else
        break
    fi
done

if [ $counter -ge $timeout ]; then
    echo -e "${RED}❌ Timeout waiting for services to be healthy${NC}"
    docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps
    exit 1
fi

echo ""
echo -e "${GREEN}✅ All services are healthy${NC}"

# Show status
echo ""
echo -e "${BLUE}📊 Deployment Status${NC}"
docker compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" ps

# Test deployment
echo ""
echo -e "${BLUE}🧪 Testing deployment...${NC}"

# Get the app port
APP_PORT=$(grep "^APP_PORT=" "$ENV_FILE" | cut -d'=' -f2 || echo "8080")

# Test health endpoint
if curl -f -s "http://localhost:$APP_PORT/api/v1/health" > /dev/null; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    echo "Check logs: docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs app"
    exit 1
fi

# Success message
echo ""
echo -e "${GREEN}🎉 Production deployment completed successfully!${NC}"
echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. Configure your domain/DNS to point to this server"
echo "2. Set up SSL certificates (if using nginx profile)"
echo "3. Configure monitoring and alerting"
echo "4. Set up log aggregation"
echo "5. Schedule regular backups"
echo ""
echo -e "${YELLOW}🔧 Useful commands:${NC}"
echo "• View logs: docker compose -f $COMPOSE_FILE --env-file $ENV_FILE logs -f"
echo "• Scale worker: docker compose -f $COMPOSE_FILE --env-file $ENV_FILE up -d --scale worker=3"
echo "• Backup DB: docker compose -f $COMPOSE_FILE --env-file $ENV_FILE exec postgres /usr/local/bin/backup-db.sh"
echo "• Stop: docker compose -f $COMPOSE_FILE --env-file $ENV_FILE down"
echo ""
echo -e "${BLUE}🌐 Your application is now running at:${NC}"
echo "• HTTP: http://localhost:$APP_PORT"
echo "• Health: http://localhost:$APP_PORT/api/v1/health"

# Show environment-specific info
if grep -q "NGINX_PORT=" "$ENV_FILE"; then
    NGINX_PORT=$(grep "^NGINX_PORT=" "$ENV_FILE" | cut -d'=' -f2 || echo "80")
    echo "• Nginx: http://localhost:$NGINX_PORT (if using nginx profile)"
fi