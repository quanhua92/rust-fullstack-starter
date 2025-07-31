# Quick Start Guide

*Get a working full-stack application running in 2 minutes. Perfect for POCs, learning experiments, or when you just need something that works.*

## ⚡ 2-Minute Setup

```bash
# 1. Clone and enter directory
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter

# 2. Start everything (database + server + worker + frontend)
./scripts/dev-server.sh 3000

# 3. Open your browser
open http://localhost:3000
```

**That's it!** 🎉 You now have:
- ✅ **REST API** running on port 3000
- ✅ **React Frontend** with authentication
- ✅ **PostgreSQL Database** with sample data
- ✅ **Background Task Worker** processing async jobs
- ✅ **Interactive API Documentation** at `/api-docs`

## 🧪 Test It Works

### Register a User
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com", 
    "password": "SecurePass123!"
  }'
```

### Create a Background Task
```bash
# First login to get a token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"SecurePass123!"}' | \
  jq -r '.data.session_token')

# Create a task
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "task_type": "email",
    "payload": {"to": "user@example.com", "subject": "Hello World"}
  }'
```

### Check API Documentation
Visit http://localhost:3000/api-docs for interactive Swagger UI with all endpoints.

## 🎯 What You Get

This starter includes everything needed for modern full-stack development:

### **Backend (Rust)**
- **Authentication** - Session-based with secure password hashing
- **REST API** - 18+ endpoints with OpenAPI documentation  
- **Background Jobs** - Async task processing with retry logic
- **Database** - PostgreSQL with migrations and connection pooling
- **Health Checks** - Kubernetes-ready monitoring endpoints

### **Frontend (React + TypeScript)**
- **Modern React 18** - With TanStack Router and Query
- **Type Safety** - Auto-generated types from OpenAPI schema
- **Authentication UI** - Login, register, and session management
- **Admin Dashboard** - Task management and system monitoring
- **Responsive Design** - Mobile-friendly with Tailwind CSS

### **Development Experience**
- **Hot Reloading** - Frontend and backend restart on changes
- **Type Safety** - End-to-end TypeScript from API to UI
- **Testing** - 53 integration tests with isolated databases
- **Quality Checks** - Automated linting, formatting, and validation
- **Docker Support** - Development and production containers

## 🚀 Common Next Steps

### Add Your First Feature
```bash
# 1. Add a new API endpoint in starter/src/
# 2. Regenerate frontend types
cd web && pnpm run generate-api

# 3. Use the new endpoint in React
import { myNewApi } from '@/lib/api/client';
const data = await myNewApi.getData();
```

### Customize the Database
```bash
# 1. Create a new migration
cd starter && sqlx migrate add my_new_table

# 2. Edit the migration file in migrations/
# 3. Apply it
sqlx migrate run
```

### Deploy to Production
```bash
# Using Docker Compose
cp .env.prod.example .env.prod
# Edit .env.prod with your secrets
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

## 🛠️ Development Commands

```bash
# Backend only
cargo run server                    # Start API server
cargo run worker                    # Start background worker
cargo run -- admin task-stats       # Check system status

# Frontend only  
cd web && pnpm dev                   # Start React dev server
cd web && pnpm run generate-api      # Regenerate API types
cd web && ./scripts/check-web.sh     # Run quality checks

# Testing
cargo nextest run                    # Run 53 integration tests
./scripts/test-with-curl.sh          # Test all API endpoints
./scripts/test-chaos.sh              # Resilience testing

# Database
sqlx migrate run                     # Apply migrations
sqlx migrate revert                  # Rollback last migration
```

## 🆘 Troubleshooting

### "Permission denied" on scripts
```bash
chmod +x scripts/*.sh
```

### Database connection failed
```bash
# Reset everything
docker-compose down -v
docker-compose up -d postgres
sleep 5
sqlx migrate run
```

### Port already in use
```bash
# Use different port
./scripts/dev-server.sh 8080
```

### Frontend build fails
```bash
cd web
rm -rf node_modules .next
pnpm install
pnpm run generate-api
```

## 📚 Want to Learn More?

This quick start gets you running, but there's much more to explore:

### **📖 Understanding the Code**
- **[Learning Philosophy](learning-philosophy.md)** - Why this architecture exists
- **[Architecture Overview](guides/01-architecture.md)** - How the pieces fit together
- **[Web Integration Guide](guides/09-web-frontend-integration.md)** - Frontend ↔ Backend patterns

### **🔧 Adding Features**
- **[Authentication Patterns](guides/02-authentication.md)** - Secure user management
- **[Background Tasks](guides/04-background-tasks.md)** - Async job processing
- **[Testing Strategies](guides/07-testing.md)** - Comprehensive testing approaches

### **🚢 Going to Production**
- **[Production Deployment](production-deployment.md)** - Docker, Kubernetes, monitoring
- **[Chaos Testing](guides/08-chaos-testing.md)** - Building resilient systems
- **[Performance](reliability.md)** - Optimization and monitoring

### **🐛 When Things Break**
- **[Debugging Guide](guides/10-debugging-and-troubleshooting.md)** - Systematic problem solving
- **[Security](security.md)** - Security considerations and best practices

---

## 🎯 Perfect For

- **Learning full-stack development** - Real-world patterns
- **Prototyping new ideas** - Everything works out-of-the-box  
- **Starting serious projects** - Production-ready foundation
- **Understanding modern architectures** - Rust + React + PostgreSQL
- **Interview preparation** - Demonstrate full-stack skills

---

*This starter is designed to get you productive immediately while providing a foundation for serious applications. Start here, then dive deeper into the concepts that interest you most.*