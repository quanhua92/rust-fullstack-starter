# Rust Full-Stack Starter Documentation

A modern Rust web application starter template with authentication, background tasks, chaos testing, and comprehensive API documentation. Built with Axum, SQLx, and PostgreSQL for learning and rapid prototyping.

## Features

- **Authentication System** - User registration, login, and session management
- **Background Tasks** - Async job processing with retry logic, dead letter queue, and circuit breakers
- **Task Type Registry** - API validation ensures only workers can handle registered task types
- **Database Integration** - PostgreSQL with migrations and connection pooling
- **API Documentation** - Interactive OpenAPI/Swagger documentation
- **Testing Framework** - Comprehensive integration tests with isolated databases
- **Chaos Testing** - Docker-based resilience testing with container isolation and failure simulation
- **Development Tools** - Docker Compose, health checks, and development scripts
- **Docker Support** - Development and production container configurations

## System Overview

```mermaid
graph TB
    subgraph "🚀 Rust Full-Stack Starter"
        subgraph "🌐 HTTP Layer"
            API[REST API Server<br/>📊 OpenAPI Docs<br/>🔒 Authentication]
        end
        
        subgraph "💼 Business Logic"
            AUTH[🔐 Auth Module<br/>Sessions & Users]
            USERS[👥 User Management<br/>Profiles & Permissions]
            TASKS[⚙️ Task System<br/>Background Jobs]
        end
        
        subgraph "💾 Data Layer"
            DB[(🗄️ PostgreSQL<br/>Users, Sessions, Tasks)]
            QUEUE[📋 Task Queue<br/>Async Processing]
        end
        
        subgraph "🧪 Quality Assurance"
            TESTS[✅ 53 Integration Tests<br/>🌐 41 API Tests<br/>🔥 Chaos Testing]
        end
    end
    
    subgraph "🛠️ Development Tools"
        DOCKER[🐳 Docker Compose<br/>Dev & Prod]
        SCRIPTS[📜 Automation Scripts<br/>Testing & Deployment]
        DOCS[📚 Comprehensive Docs<br/>Learning Guides]
    end
    
    API --> AUTH
    API --> USERS
    API --> TASKS
    AUTH --> DB
    USERS --> DB
    TASKS --> QUEUE
    QUEUE --> DB
    
    TESTS --> API
    DOCKER --> API
    SCRIPTS --> TESTS
```

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
./scripts/dev-server.sh 3000

# Verify
curl http://localhost:3000/api/v1/health
open http://localhost:3000/api-docs
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
- **[05 - Task Types](guides/05-task-types.md)** - Creating custom task handlers
- **[06 - Task Registry](guides/06-task-registry.md)** - Organizing and managing tasks
- **[07 - Testing](guides/07-testing.md)** - Comprehensive testing framework
- **[08 - Chaos Testing](guides/08-chaos-testing.md)** - Resilience testing and failure simulation

### Reference Documentation
- **[Task Handlers](reference/task-handlers.md)** - Built-in task type reference
- **[Project Customization](project-customization.md)** - Adapting the starter for your needs
- **[Docker Hub Setup](docker-hub-setup.md)** - Container registry configuration

## 📚 Learning Paths: First Principles Approach

This starter is designed as the **best educational resource** for full-stack development. We teach **understanding over memorization** through first principles thinking.

> **[📖 Read Our Learning Philosophy](learning-philosophy.md)** - Why we prioritize "why" before "how"

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

4. **[🌐 Full-Stack Integration](guides/09-web-frontend-integration.md)** *(🌐 Connection)*
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

6. **[📋 Task Registry](guides/06-task-registry.md)** *(📋 Organization)*
   - **Why**: Organization and scalability patterns
   - **Mental Model**: Domain-driven task organization
   - **Practice**: Implement domain-specific task registry

7. **[✅ Testing Strategy](guides/07-testing.md)** *(✅ Quality)*
   - **Why**: Integration tests over unit tests for this architecture
   - **Mental Model**: TestApp pattern and isolation strategies
   - **Practice**: Write tests for your custom features

8. **[🔧 Debugging & Troubleshooting](guides/10-debugging-and-troubleshooting.md)** *(🔧 Essential Skill)*
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

10. **[🌪️ Chaos Engineering](guides/08-chaos-testing.md)** *(🌪️ Resilience)*
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

### 📊 Quick Reference: Learning Progression

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

| Background | Recommended Path | Skills to Build |
|------------|------------------|-----------------|
| **New to Full-Stack** | Start with Beginner Path | Foundations → Implementation → Production |
| **Frontend Developer** | Start with Architecture (Step 2) | Backend patterns → Full-stack integration |
| **Backend Developer** | Start with Integration (Step 4) | Frontend patterns → Type-safe APIs |
| **Experienced Developer** | Jump to Intermediate Path | Advanced patterns → Production deployment |

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