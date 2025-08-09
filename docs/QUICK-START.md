# Quick Start Guide

*Get a complete full-stack application with React frontend, Rust API, and PostgreSQL running in 2 minutes. Perfect for POCs, learning, urgent projects, and interview demos.*

## ⚡ 2-Minute Setup

```bash
# 1. Clone and enter directory
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter

# 2. Start everything (database + unified server with React frontend)
./scripts/dev-server.sh 3000

# 3. Open your browser
open http://localhost:3000
```

**That's it!** 🎉 You now have a complete working application:

- ✅ **React 18 Frontend** with authentication, admin dashboard, type-safe API integration
- ✅ **REST API** (37 endpoints) with OpenAPI documentation at `/api-docs`
- ✅ **PostgreSQL Database** with migrations and sample data
- ✅ **Background Task System** with retry logic and monitoring
- ✅ **Unified Static Serving** - Single server for both API and frontend

## 🧪 Test It Works

### Register & Login
```bash
# Register a new user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com", 
    "password": "SecurePass123!"
  }'

# Login to get a token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"SecurePass123!"}' | \
  jq -r '.data.session_token')
```

### Create Background Task
```bash
# Create an async task
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {"to": "user@example.com", "subject": "Hello World"}
  }'

# Check task status
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/tasks
```

### Explore API Documentation
Visit **http://localhost:3000/api-docs** for interactive Swagger UI with all 37 endpoints.

## Prerequisites

- **Rust 1.75+** - Install via [rustup.rs](https://rustup.rs/)
- **Node.js 18+** and **pnpm** (`npm install -g pnpm`)
- **Docker 20.10+** and **Docker Compose 2.0+**

> **Performance Note**: Setup takes 2-3 seconds, 184 tests run in ~21 seconds

## 🔧 Development Commands

### Start Services
```bash
# Complete environment (recommended)
./scripts/dev-server.sh 3000                # Database + Web + API

# Individual services
./scripts/server.sh 3000                    # API server only
./scripts/worker.sh -f                      # Background worker with logs

# Frontend development
cd web && pnpm dev                          # React dev server (port 5173)
```

### Quality & Testing
```bash
# Quality checks (run before commits)
./scripts/check.sh                          # Backend: format, lint, test
web/scripts/check-web.sh                   # Frontend: 10-step validation

# Testing
cargo nextest run                           # 184 integration tests
./scripts/test-with-curl.sh                 # Test all 37 API endpoints
./scripts/test-chaos.sh                     # Resilience testing
```

### Database Operations
```bash
# Migrations
cd starter && sqlx migrate run              # Apply migrations
cd starter && sqlx migrate add my_feature   # Create new migration

# Direct access
psql postgres://starter_user:starter_pass@localhost:5432/starter_db
```

### Admin Operations
```bash
# Admin CLI (direct database access)
cargo run -- admin task-stats               # System statistics
cargo run -- admin list-tasks --limit 10   # Recent tasks
cargo run -- admin clear-completed          # Maintenance cleanup
```

## 🎯 Working Directory Guide

- **Scripts**: Run from project root (`./scripts/dev-server.sh`)
- **Cargo commands**: Run from project root (`cargo check`, `cargo run`)
- **SQLx commands**: Auto-handled by scripts (or `cd starter && sqlx migrate run`)
- **Tests**: Run from project root (`cargo nextest run`)

## 🛠️ Common Development Tasks

### Add New API Endpoint
```bash
# 1. Add handler in starter/src/
# 2. Update OpenAPI and regenerate frontend types
./scripts/prepare-openapi.sh

# 3. Use in React with type safety
import { useApiQueries } from '@/hooks/useApiQueries';
const { data } = useApiQueries.myNewEndpoint();
```

### Customize Database Schema
```bash
# 1. Create migration
cd starter && sqlx migrate add add_my_table

# 2. Edit migration file in starter/migrations/
# 3. Apply changes
sqlx migrate run
```

### Deploy to Production
```bash
# Docker deployment (single artifact)
cp .env.example .env.prod
# Edit .env.prod with your production secrets
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

## 🆘 Troubleshooting

### Quick Fixes

**Server won't start**
```bash
# Kill conflicting processes
lsof -ti:3000 | xargs kill -9
./scripts/dev-server.sh 3000
```

**Database issues**
```bash
# Complete reset
docker-compose down -v
docker-compose up -d postgres && sleep 5
cd starter && sqlx migrate run
```

**Frontend build fails**
```bash
cd web
rm -rf node_modules dist
pnpm install && pnpm run generate-api
```

**Permission errors**
```bash
chmod +x scripts/*.sh
```

### Common Issues

- **Port 3000 in use**: Use `./scripts/dev-server.sh 8080`
- **Docker not running**: Start Docker Desktop and wait for full startup
- **Migration errors**: Ensure you're in the `starter/` directory: `cd starter && sqlx migrate run`
- **TypeScript errors**: Regenerate types: `cd web && pnpm run generate-api`

### Development Workflow Issues

**Tests failing**: Database isolation requires clean state
```bash
cargo nextest run                           # Uses isolated test databases
```

**Hot reload not working**: Use separate development servers
```bash
# Terminal 1: API server
./scripts/server.sh 3000

# Terminal 2: React dev server
cd web && pnpm dev
```

## 🎯 Perfect For

- **Learning full-stack development** - Modern Rust + React patterns
- **Rapid prototyping** - Everything works out-of-the-box
- **Starting serious projects** - Production-ready foundation
- **Interview preparation** - Demonstrate full-stack skills
- **Understanding modern architectures** - Type-safe API integration

## 🚀 Next Steps

### **📖 Understand the System**
- Architecture principles and design decisions
- Authentication & authorization patterns  
- Background task processing system

### **🔧 Build Features**
- User management and RBAC implementation
- Monitoring and observability integration
- Testing strategies and patterns

### **🚢 Go to Production**
- Docker deployment and configuration
- Chaos testing and resilience patterns
- Performance optimization and monitoring

---

*This starter gets you productive immediately while providing patterns for serious applications. The unified server approach eliminates deployment complexity while maintaining development flexibility.*