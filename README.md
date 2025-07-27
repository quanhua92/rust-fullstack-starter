# Rust Full-Stack Starter

A modern Rust web application starter template with authentication, background tasks, and comprehensive API documentation. Built with Axum, SQLx, and PostgreSQL for learning and rapid prototyping.

## Features

- **Authentication System** - User registration, login, and session management
- **Background Tasks** - Async job processing with retry logic and circuit breakers
- **Database Integration** - PostgreSQL with migrations and connection pooling
- **API Documentation** - Interactive OpenAPI/Swagger documentation
- **Testing Framework** - Comprehensive integration tests with isolated databases
- **Development Tools** - Docker Compose, health checks, and development scripts
- **Docker Support** - Development and production container configurations

## Quick Start

### Prerequisites

- Rust 1.75+ ([rustup.rs](https://rustup.rs/))
- Docker and Docker Compose
- PostgreSQL client tools (optional)

### Setup

```bash
# Clone and enter directory
git clone https://github.com/quanhua92/rust-fullstack-starter.git
cd rust-fullstack-starter

# Start development environment
./scripts/dev-server.sh 3000

# Or step by step
./scripts/dev.sh                    # Start database
./scripts/server.sh 3000            # Start server
./scripts/test-server.sh 3000       # Verify setup
```

### Verify Installation

```bash
# Check health
curl http://localhost:3000/health

# View API documentation
open http://localhost:3000/api-docs
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

## Development Workflow

### Running Tests

```bash
# Install test runner (recommended)
cargo install cargo-nextest

# Run integration tests (40 tests, ~10 seconds)
cargo nextest run

# Test API endpoints (29 endpoint tests)
./scripts/test-with-curl.sh

# Combined workflow
cargo nextest run && ./scripts/test-with-curl.sh
```

### API Development

The starter includes interactive API documentation:

- **Documentation**: http://localhost:3000/api-docs
- **OpenAPI Schema**: http://localhost:3000/api-docs/openapi.json
- **Health Check**: http://localhost:3000/health

Key endpoints:
- `POST /auth/register` - User registration
- `POST /auth/login` - User authentication
- `GET /users/{id}` - User profile
- `POST /tasks` - Create background task
- `GET /tasks` - List tasks

### Background Tasks

Create and process async jobs:

```bash
# Start worker process
./scripts/worker.sh

# Create task via API
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"task_type": "email", "payload": {"to": "user@example.com"}}'
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
- **Task Processing Tests** - Background job lifecycle
- **Health Check Tests** - Application monitoring

## Documentation

Comprehensive guides available in `docs/`:

- **[Getting Started](docs/getting-started.md)** - Setup and first steps
- **[Development Guide](docs/development.md)** - Daily development workflow
- **[API Reference](docs/api-reference.md)** - Complete endpoint documentation
- **[Configuration](docs/configuration.md)** - Environment variables
- **[Production Deployment](docs/production-deployment.md)** - Docker and deployment
- **[Architecture Guides](docs/guides/)** - In-depth system documentation

## Learning Resources

This starter is designed for learning modern Rust web development:

### Key Learning Areas

- **Async Rust** - Tokio, async/await patterns
- **Web Development** - Axum framework, HTTP handling
- **Database Integration** - SQLx, migrations, connection pooling
- **Testing Strategies** - Integration testing, test isolation
- **Error Handling** - Result types, custom error types
- **Security** - Authentication, session management

### Code Examples

The codebase includes examples for:
- User authentication and authorization
- Background job processing with retries
- Database transactions and error handling
- API documentation with OpenAPI
- Docker containerization
- Comprehensive testing patterns

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
