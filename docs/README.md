# Rust Full-Stack Starter Documentation

**A complete full-stack application template with React frontend, Rust API backend, and PostgreSQL database. Get a working application in 2 minutes, then dive deep into modern development patterns.**

## 🚀 Quick Start (2 minutes)

```bash
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter
./scripts/dev-server.sh 3000
open http://localhost:3000
```

**Perfect for**: POCs, learning, urgent projects, interview demos

**[📖 Full Quick Start Guide →](quick-start.md)**

---

## 📚 Choose Your Learning Path

### ⚡ **Just Show Me Code** *(5-15 minutes)*
- **[Quick Start Guide](quick-start.md)** - Working app in 2 minutes
- **[API Examples](#api-examples)** - Copy-paste ready endpoints
- **[Common Recipes](#common-recipes)** - Add auth, tasks, deploy

### 🏗️ **Understand the Architecture** *(1-2 hours)*
- **[System Overview](#system-overview)** - How the pieces fit together
- **[Key Patterns](guides/03-patterns.md)** - Reliability and error handling
- **[Why This Approach?](learning-philosophy.md)** - Design decisions explained

### 🎓 **Master Full-Stack Development** *(Self-paced)*
- **[Complete Learning Paths](#learning-paths)** - Beginner → Intermediate → Advanced
- **[Production Deployment](#production-ready)** - Docker, Kubernetes, monitoring
- **[Advanced Topics](#advanced-topics)** - Chaos testing, performance, security

---

## What You Get

### **Full-Stack Application Ready to Use**
- ✅ **React 18 Frontend** - TypeScript, TanStack Router/Query, Tailwind CSS
- ✅ **Rust API Backend** - Axum, SQLx, PostgreSQL, background jobs
- ✅ **Authentication System** - Secure sessions, password hashing, role-based access
- ✅ **Interactive API Docs** - OpenAPI/Swagger UI with type generation
- ✅ **Production Ready** - Docker, health checks, comprehensive testing

## API Examples

### User Registration & Login
```bash
# Register new user
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","email":"user@example.com","password":"SecurePass123!"}'

# Login and get token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","password":"SecurePass123!"}' | jq -r '.data.session_token')

# Get current user info
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/auth/me
```

### Background Tasks
```bash
# Create a background task
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"email","payload":{"to":"user@example.com","subject":"Hello"}}'

# Check task status
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/tasks

# Get system statistics
curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/api/v1/tasks/stats
```

### Health Monitoring
```bash
# Basic health check
curl http://localhost:3000/api/v1/health

# Detailed health with database status
curl http://localhost:3000/api/v1/health/detailed

# Kubernetes-style probes
curl http://localhost:3000/api/v1/health/ready
```

## Common Recipes

### Add Authentication to Your Endpoint
```rust
// In your Rust handler
use crate::auth::middleware::require_auth;

pub async fn my_protected_endpoint(
    Extension(user): Extension<User>, // User extracted by middleware
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<MyData>>, Error> {
    // Your logic here - user is already authenticated
    Ok(Json(ApiResponse::success(my_data)))
}

// Add to router with auth middleware
Router::new()
    .route("/my-endpoint", get(my_protected_endpoint))
    .layer(middleware::from_fn_with_state(state.clone(), require_auth))
```

### Use the API in React
```typescript
// Auto-generated types from OpenAPI
import { authApi, tasksApi } from '@/lib/api/client';
import { useQuery, useMutation } from '@tanstack/react-query';

function MyComponent() {
  // Get current user
  const { data: user } = useQuery({
    queryKey: ['user'],
    queryFn: () => authApi.getCurrentUser()
  });

  // Create task mutation
  const createTask = useMutation({
    mutationFn: (taskData) => tasksApi.createTask(taskData),
    onSuccess: () => {
      // Refetch tasks list
      queryClient.invalidateQueries(['tasks']);
    }
  });

  return (
    <div>
      <p>Welcome, {user?.username}!</p>
      <button onClick={() => createTask.mutate({
        task_type: 'email',
        payload: { to: 'user@example.com' }
      })}>
        Create Task
      </button>
    </div>
  );
}
```

### Quick Production Deploy
```bash
# 1. Copy production config
cp .env.prod.example .env.prod

# 2. Edit secrets (REQUIRED - change default passwords!)
nano .env.prod

# 3. Deploy with Docker
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d

# 4. Verify deployment
curl https://yourdomain.com/api/v1/health
```

## System Overview

```mermaid
graph LR
    subgraph "🌐 Frontend"
        REACT[React 18<br/>TypeScript + Tailwind]
        ROUTER[TanStack Router<br/>File-based routing]
        STATE[TanStack Query<br/>Server state]
    end
    
    subgraph "🦀 Rust Backend"
        API[REST API<br/>Axum + SQLx]
        AUTH[Authentication<br/>Sessions + JWT]
        TASKS[Background Jobs<br/>Async processing]
    end
    
    subgraph "💾 Database"
        POSTGRES[(PostgreSQL<br/>Users + Tasks + Sessions)]
    end
    
    REACT --> API
    ROUTER --> API
    STATE --> API
    API --> AUTH
    API --> TASKS
    AUTH --> POSTGRES
    TASKS --> POSTGRES
    
    classDef frontend fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef backend fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef database fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    
    class REACT,ROUTER,STATE frontend
    class API,AUTH,TASKS backend
    class POSTGRES database
```

**Key Features**:
- **Type Safety** - OpenAPI schema generates TypeScript types automatically
- **Real-time Updates** - TanStack Query handles caching and synchronization
- **Background Processing** - Tasks run independently with retry logic
- **Production Ready** - Health checks, monitoring, Docker deployment

## Project Structure

```
rust-fullstack-starter/
├── Cargo.toml                 # Workspace configuration
├── docker-compose.yaml        # Development infrastructure
├── docker-compose.prod.yaml   # Production deployment
├── scripts/                   # Development automation
├── docs/                      # Comprehensive documentation
└── starter/                   # Main application
    ├── src/
    │   ├── auth/               # Authentication module
    │   ├── users/              # User management
    │   ├── tasks/              # Background job system
    │   ├── openapi.rs          # API documentation
    │   └── ...
    ├── migrations/             # Database schema
    └── tests/                  # Integration tests
```

## Getting Started

For complete setup instructions, see **[Getting Started Guide](./getting-started.md)**.

### Quick Setup

```bash
# Clone and start
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter

# Start database and HTTP server
./scripts/dev-server.sh

# Start background task worker with log following
./scripts/worker.sh -f

# Or multiple concurrent workers:
# ./scripts/worker.sh --id 1 -f
# ./scripts/worker.sh --id 2 -f

# Verify
curl http://localhost:3000/api/v1/health
open http://localhost:3000/api-docs

# Register a new user
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "password123"}'

# Login to get a session token
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "password123"}'
```

## Testing

### Running Tests

```bash
# Install test runner (recommended)
cargo install cargo-nextest

# Run integration tests (53 tests, ~12 seconds)
cargo nextest run

# Test API endpoints (41 endpoint tests)
./scripts/test-with-curl.sh

# Combined workflow
cargo nextest run && ./scripts/test-with-curl.sh

# Docker-based chaos testing for resilience validation
./scripts/test-chaos.sh
```

### API Development

The starter includes interactive API documentation:

- **Documentation**: http://localhost:3000/api-docs
- **OpenAPI Schema**: http://localhost:3000/api-docs/openapi.json
- **[Interactive Swagger UI](https://petstore.swagger.io/?url=https://raw.githubusercontent.com/quanhua92/rust-fullstack-starter/refs/heads/main/docs/openapi.json)**
- **Health Check**: http://localhost:3000/api/v1/health

Key endpoints:
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - User authentication
- `GET /api/v1/users/{id}` - User profile
- `POST /api/v1/tasks` - Create background task
- `GET /api/v1/tasks` - List tasks with filtering
- `GET /api/v1/tasks/dead-letter` - Dead letter queue (failed tasks)
- `POST /api/v1/tasks/{id}/retry` - Retry failed task
- `DELETE /api/v1/tasks/{id}` - Delete completed/failed task

### Background Tasks

Create and process async jobs with dead letter queue management:

```bash
# Start worker process
./scripts/worker.sh

# Or multiple concurrent workers:
# ./scripts/worker.sh --id 1
# ./scripts/worker.sh --id 2

# Create task via API
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"task_type": "email", "payload": {"to": "user@example.com"}}'

# Monitor failed tasks (dead letter queue)
curl http://localhost:3000/api/v1/tasks/dead-letter

# Retry failed task
curl -X POST http://localhost:3000/api/v1/tasks/{task_id}/retry

# Clean up completed/failed tasks
curl -X DELETE http://localhost:3000/api/v1/tasks/{task_id}
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
# Database
STARTER__DATABASE__USER=starter_user
STARTER__DATABASE__PASSWORD=starter_pass
STARTER__DATABASE__HOST=localhost
STARTER__DATABASE__DATABASE=starter_db

# Server
STARTER__SERVER__HOST=127.0.0.1
STARTER__SERVER__PORT=8080

# Initial admin user (remove after first startup)
STARTER__INITIAL_ADMIN_PASSWORD=SecurePassword123!
```

See `docs/configuration.md` for all options.

## Production Deployment

### Docker Deployment

```bash
# Copy production environment
cp .env.prod.example .env.prod

# Edit secrets and passwords
nano .env.prod

# Deploy with Docker Compose
docker-compose -f docker-compose.prod.yaml --env-file .env.prod up -d
```

### Manual Deployment

```bash
# Build optimized binary
cargo build --release

# Run migrations
sqlx migrate run

# Start services
./target/release/starter server --port 8080
./target/release/starter worker
```

## Architecture

### Core Components

- **Axum Web Framework** - HTTP server and routing
- **SQLx** - Database integration with compile-time checked queries
- **PostgreSQL** - Primary database with JSONB support
- **Tokio** - Async runtime for concurrent processing
- **utoipa** - OpenAPI documentation generation

### Design Patterns

- **Service Layer Pattern** - Function-based services for business logic and data access
- **Domain Models** - Clean separation between database entities and API responses
- **Background Jobs** - Async task processing with retry logic
- **Circuit Breaker** - Fault tolerance for external services
- **Health Checks** - Application and dependency monitoring

## Testing

The starter includes comprehensive testing patterns:

### Test Architecture

- **TestApp Pattern** - Spawns real server instances
- **Database Isolation** - Each test gets its own PostgreSQL database
- **Test Factories** - Consistent test data generation
- **Helper Utilities** - Common assertions and test setup

### Test Categories

- **Authentication Tests** - Registration, login, session management
- **API Standards Tests** - CORS, security headers, error handling
- **Task Processing Tests** - Background job lifecycle, task type validation, and dead letter queue management
- **Health Check Tests** - Application monitoring

## Documentation

### Getting Started & Operations
- **[Getting Started](getting-started.md)** - Setup and first steps
- **[Development Guide](development.md)** - Daily development workflow
- **[Configuration](configuration.md)** - Environment variables and settings
- **[Production Deployment](production-deployment.md)** - Docker and deployment strategies
- **[CI/CD Pipeline](cicd.md)** - GitHub Actions and automated testing

### API & Reference
- **[API Reference](api-reference.md)** - Complete endpoint documentation
- **[Security](security.md)** - Authentication and security patterns
- **[Reliability](reliability.md)** - Circuit breakers, retries, and resilience patterns
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

### Architecture Guides
Comprehensive guides in **[`guides/`](guides/)**:

- **[01 - Architecture](guides/01-architecture.md)** - System design and component overview
- **[02 - Authentication](guides/02-authentication.md)** - User management and security
- **[03 - Design Patterns](guides/03-patterns.md)** - Service layer and architectural patterns
- **[04 - Background Tasks](guides/04-background-tasks.md)** - Async job processing system
- **[05 - Task Handlers Reference](guides/05-task-handlers-reference.md)** - Built-in task type examples
- **[06 - Custom Task Types](guides/06-task-types.md)** - Creating custom task handlers
- **[07 - Task Registry](guides/07-task-registry.md)** - Organizing and managing tasks
- **[08 - Testing](guides/08-testing.md)** - Comprehensive testing framework
- **[09 - Chaos Testing](guides/09-chaos-testing.md)** - Resilience testing and failure simulation
- **[10 - Web Frontend Integration](guides/10-web-frontend-integration.md)** - React ↔ Rust patterns
- **[11 - Debugging & Troubleshooting](guides/11-debugging-and-troubleshooting.md)** - Systematic problem solving

### Reference Documentation
- **[Task Handlers](reference/task-handlers.md)** - Built-in task type reference
- **[Project Customization](project-customization.md)** - Adapting the starter for your needs
- **[Docker Hub Setup](docker-hub-setup.md)** - Container registry configuration

## Learning Paths

### ⚡ **Just Getting Started?**
- **[🚀 Quick Start Guide](quick-start.md)** - Working app in 2 minutes
- **[🔧 Common Recipes](#common-recipes)** - Add features, customize, deploy  
- **[📖 API Examples](#api-examples)** - Copy-paste ready code

### 🏗️ **Want to Understand How It Works?**
- **[System Overview](#system-overview)** - Architecture and component relationships
- **[Authentication Guide](guides/02-authentication.md)** - Secure user management patterns
- **[Background Tasks](guides/04-background-tasks.md)** - Async job processing system
- **[Web Integration](guides/10-web-frontend-integration.md)** - React ↔ Rust patterns

### 🚢 **Ready for Production?**
- **[Production Deployment](production-deployment.md)** - Docker, Kubernetes, security
- **[Testing Strategy](guides/08-testing.md)** - 53 integration tests + chaos testing
- **[Debugging Guide](guides/11-debugging-and-troubleshooting.md)** - Systematic problem solving
- **[Performance & Monitoring](reliability.md)** - Optimization and observability

### 🎓 **Master Full-Stack Development** *(Advanced)*

> **[📖 Learning Philosophy](learning-philosophy.md)** - First principles approach to understanding systems

This starter includes comprehensive educational content for deep learning:

**🎯 Beginner → Intermediate → Advanced progression** with:
- **Why before how** - Understand reasoning behind architectural choices
- **Mental models** - Visual diagrams for complex concepts  
- **Alternative approaches** - When to choose different patterns
- **Production patterns** - Real-world practices, not just tutorials

**[📚 Complete Learning Paths →](#comprehensive-learning-paths)** *(Scroll down for full curriculum)*

---

## Comprehensive Learning Paths

*This section provides structured, curriculum-style learning for those who want to master full-stack development from first principles.*

### 🎯 Beginner Path: Foundations
**Difficulty**: ⭐⭐☆☆☆ (Beginner)  
**Goal**: Build fundamental mental models for full-stack development

1. **[📖 Learning Philosophy](learning-philosophy.md)** *(⚡ Quick Read)*
   - First principles thinking for developers
   - Mental model building strategies
   - Why before how methodology

2. **[🏗️ Architecture Overview](guides/01-architecture.md)** *(🏗️ Foundation)*
   - **Why**: Single binary vs microservices tradeoffs
   - **Mental Model**: Layer-based system thinking
   - **Practice**: Trace a request through all layers

3. **[🔐 Authentication System](guides/02-authentication.md)** *(🔐 Core Concept)*
   - **Why**: Sessions vs JWT tradeoffs for this use case
   - **Mental Model**: Identity verification and state management
   - **Practice**: Implement user registration flow

4. **[🌐 Full-Stack Integration](guides/10-web-frontend-integration.md)** *(🌐 Connection)*
   - **Why**: OpenAPI-driven development approach
   - **Mental Model**: Type-safe contract between frontend and backend
   - **Practice**: Add a new API endpoint with frontend integration

**Prerequisites**: Basic programming knowledge, comfort with terminal  
**Success Criteria**: Can explain why each architectural choice was made

### 🚀 Intermediate Path: Implementation
**Difficulty**: ⭐⭐⭐☆☆ (Intermediate)  
**Goal**: Master implementation patterns and testing strategies

5. **[⚙️ Background Tasks](guides/04-background-tasks.md)** *(⚙️ Complex System)*
   - **Why**: Async processing necessity and patterns
   - **Mental Model**: Queue-based task processing
   - **Practice**: Create custom task handlers

6. **[📋 Task Registry](guides/07-task-registry.md)** *(📋 Organization)*
   - **Why**: Organization and scalability patterns
   - **Mental Model**: Domain-driven task organization
   - **Practice**: Implement domain-specific task registry

7. **[✅ Testing Strategy](guides/08-testing.md)** *(✅ Quality)*
   - **Why**: Integration tests over unit tests for this architecture
   - **Mental Model**: TestApp pattern and isolation strategies
   - **Practice**: Write tests for your custom features

8. **[🔧 Debugging & Troubleshooting](guides/11-debugging-and-troubleshooting.md)** *(🔧 Essential Skill)*
   - **Why**: Systematic debugging from first principles
   - **Mental Model**: Layer-based problem isolation
   - **Practice**: Debug real issues in the application

**Prerequisites**: Completed beginner path  
**Success Criteria**: Can implement and test new features independently

### 🔥 Advanced Path: Production
**Difficulty**: ⭐⭐⭐⭐☆ (Advanced)  
**Goal**: Production deployment and reliability engineering

9. **[🚢 Production Deployment](production-deployment.md)** *(🚢 Infrastructure)*
   - **Why**: Container orchestration and security considerations
   - **Mental Model**: Infrastructure as code and deployment pipelines
   - **Practice**: Deploy to staging environment

10. **[🌪️ Chaos Engineering](guides/09-chaos-testing.md)** *(🌪️ Resilience)*
    - **Why**: Building antifragile systems
    - **Mental Model**: Controlled failure experimentation
    - **Practice**: Design and run chaos experiments

11. **[⚡ Performance & Monitoring](reliability.md)** *(⚡ Optimization)*
    - **Why**: Observability and optimization strategies
    - **Mental Model**: Performance bottleneck identification
    - **Practice**: Implement monitoring and alerting

12. **[🎓 Graduation: Beyond the Starter](project-customization.md)** *(🎓 Mastery)*
    - **Why**: When and how to evolve beyond the starter patterns
    - **Mental Model**: Technology selection and scaling decisions
    - **Practice**: Plan your next architectural evolution

**Prerequisites**: Completed intermediate path  
**Success Criteria**: Ready to architect and deploy production systems

### 📊 Learning Progression Map

```mermaid
graph TD
    subgraph "🎯 Beginner: Mental Models"
        A1[📖 Learning Philosophy] --> A2[🏗️ Architecture]
        A2 --> A3[🔐 Authentication]
        A3 --> A4[🌐 Integration]
    end
    
    subgraph "🚀 Intermediate: Implementation"
        B1[⚙️ Background Tasks] --> B2[📋 Task Registry]
        B2 --> B3[✅ Testing]
        B3 --> B4[🔧 Debugging]
    end
    
    subgraph "🔥 Advanced: Production"
        C1[🚢 Deployment] --> C2[🌪️ Chaos Testing]
        C2 --> C3[⚡ Performance]
        C3 --> C4[🎓 Graduation]
    end
    
    A4 --> B1
    B4 --> C1
    
    classDef beginner fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef intermediate fill:#e3f2fd,stroke:#0277bd,stroke-width:2px
    classDef advanced fill:#ffebee,stroke:#c62828,stroke-width:2px
    
    class A1,A2,A3,A4 beginner
    class B1,B2,B3,B4 intermediate
    class C1,C2,C3,C4 advanced
```

### 🎯 Choose Your Starting Point

| Background                | Recommended Path                 | Skills to Build                           |
| ------------------------- | -------------------------------- | ----------------------------------------- |
| **New to Full-Stack**     | Start with Beginner Path         | Foundations → Implementation → Production |
| **Frontend Developer**    | Start with Architecture (Step 2) | Backend patterns → Full-stack integration |
| **Backend Developer**     | Start with Integration (Step 4)  | Frontend patterns → Type-safe APIs        |
| **Experienced Developer** | Jump to Intermediate Path        | Advanced patterns → Production deployment |

### Key Learning Areas Covered

- **🧠 First Principles Thinking** - Understanding why, not just how
- **⚛️ Modern React Patterns** - TanStack Router, Query, and type safety
- **🦀 Production Rust** - Axum, SQLx, and async patterns
- **🔒 Security Implementation** - Authentication, authorization, and session management
- **🔄 Async Processing** - Background tasks with retry logic and dead letter queues
- **✅ Testing Excellence** - Integration testing and chaos engineering
- **🚀 Production Deployment** - Docker, Kubernetes, and monitoring
- **🌐 Full-Stack Integration** - End-to-end type safety and error handling

### Real-World Applications

This starter demonstrates patterns used in production applications:
- **Authentication flows** similar to GitHub, GitLab
- **Background task processing** like Stripe webhooks, email queues
- **API documentation** standards used by Stripe, Twilio
- **Testing strategies** from Netflix, Spotify engineering teams
- **Chaos engineering** principles from Netflix's Chaos Monkey
- **Health monitoring** patterns from AWS, Google Cloud

### Learning Outcomes

After completing all paths, you will:

✅ **Understand Systems Thinking** - See how components connect and influence each other  
✅ **Debug from First Principles** - Systematically isolate and fix problems at any layer  
✅ **Make Informed Architecture Decisions** - Choose technologies based on requirements, not hype  
✅ **Build Production-Ready Applications** - Handle scale, failures, and security appropriately  
✅ **Teach Others** - Explain complex concepts clearly because you understand the fundamentals

## Contributing

This is a starter template for learning and development. When using this starter:

1. **Customize for your needs** - Remove unused features, add your own
2. **Update dependencies** - Keep dependencies current for your project
3. **Adapt patterns** - Modify architectural patterns to fit your use case
4. **Extend documentation** - Document your specific business logic

## License

MIT License - see LICENSE file for details.

## Support

- **Issues**: Report problems or ask questions via GitHub issues
- **Documentation**: Comprehensive guides in the `docs/` directory
- **Examples**: Study the test suite for usage patterns

---

*This starter template demonstrates modern Rust web development patterns and is intended for learning and rapid prototyping. Adapt and extend it based on your specific requirements.*