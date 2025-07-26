# Architecture Overview

This document provides a high-level overview of the Rust Full-Stack Starter project architecture, design decisions, and patterns.

## System Overview

The Rust Full-Stack Starter is designed as a production-ready foundation for building full-stack web applications. It follows clean architecture principles with clear separation of concerns.

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Client    │    │   CLI Client    │    │  External APIs  │
│   (Future)      │    │                 │    │   (Future)      │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────▼───────────────┐
                    │     Rust Application       │
                    │  ┌─────────┐ ┌─────────┐   │
                    │  │ Server  │ │ Worker  │   │
                    │  │  Mode   │ │  Mode   │   │
                    │  └─────────┘ └─────────┘   │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │     PostgreSQL Database     │
                    │   ┌─────────────────────┐   │
                    │   │  users, sessions,   │   │
                    │   │  api_keys, tasks    │   │
                    │   └─────────────────────┘   │
                    └─────────────────────────────┘
```

## Core Components

### 1. Application Binary (`starter`)

A single binary that operates in multiple modes:

- **Server Mode**: HTTP API server with authentication and business logic
- **Worker Mode**: Background job processor for async tasks
- **CLI Interface**: Command-line interface for operational tasks

**Key Features:**
- Single binary deployment
- Mode selection via CLI arguments
- Shared configuration and database layer
- Graceful shutdown handling

### 2. Configuration System

Hierarchical configuration with environment variable support:

```
Default Values → Environment Variables → CLI Arguments → Runtime Config
```

**Features:**
- Type-safe configuration structs
- Environment variable parsing with nested support (`STARTER__DATABASE__HOST`)
- Comprehensive validation at startup
- Separate concerns (app config vs. tooling config)

### 3. Database Layer

Production-ready PostgreSQL integration with:

- **Connection Pooling**: Configurable min/max connections with timeouts
- **Migrations**: Version-controlled schema changes with sqlx
- **Health Checks**: Database connectivity monitoring
- **Type Safety**: Compile-time SQL validation where possible

### 4. Error Handling

Comprehensive error handling system:

- **Custom Error Types**: Domain-specific error variants
- **HTTP Error Mapping**: Automatic conversion to appropriate HTTP status codes
- **Structured Logging**: Contextual error information
- **Validation Errors**: Field-level validation with helpful messages

## Design Patterns

### 1. Clean Architecture

The application follows clean architecture principles:

```
┌─────────────────────────────────────────────────────────┐
│                    Presentation Layer                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ HTTP Routes │  │ CLI Handler │  │ RPC Endpoints│     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                  Business Logic Layer                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Services   │  │ Validation  │  │ Authentication│    │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────┬───────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│                   Data Access Layer                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Repository  │  │   Models    │  │  Database   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

### 2. Domain-Driven Design (DDD)

Core business entities and their relationships:

- **User**: Authentication and authorization
- **Session**: User sessions and access control  
- **ApiKey**: Machine-to-machine authentication
- **Task**: Background job processing

### 3. Dependency Injection

Configuration and database connections are injected through application state:

```rust
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: AppConfig,
}
```

## Database Schema

### Core Tables

```sql
users           -- User accounts and authentication
├── id (UUID, PK)
├── username (unique)
├── email (unique)
├── password_hash
├── role (admin/user)
├── is_active
├── email_verified
└── timestamps

sessions        -- User session management
├── id (UUID, PK)
├── user_id (FK → users.id)
├── token (unique)
├── expires_at
├── user_agent
├── ip_address
└── timestamps

api_keys        -- Machine authentication
├── id (UUID, PK)
├── name
├── key_hash (unique)
├── key_prefix
├── created_by (FK → users.id)
├── permissions (JSONB)
└── timestamps

tasks           -- Background job queue
├── id (UUID, PK)
├── task_type
├── payload (JSONB)
├── status
├── priority
├── retry_count
└── timestamps
```

### Key Design Decisions

1. **UUIDs**: All primary keys use UUIDs for distributed system compatibility
2. **JSONB**: Flexible data storage for permissions and task payloads
3. **Timestamps**: Automatic created_at/updated_at with triggers
4. **Indexes**: Performance-optimized indexes on frequently queried columns
5. **Constraints**: Data integrity enforced at database level

## Security Architecture

### Authentication Flow

```
1. User Login → Password Verification → Session Creation
2. Request with Session Token → Session Validation → User Context
3. Session Expiry → Cleanup Process → Token Invalidation
```

### Security Features

- **Password Hashing**: Argon2 with secure defaults
- **Session Management**: Secure token generation and validation
- **API Key Authentication**: Machine-to-machine access
- **Role-Based Access**: Admin/user role separation
- **SQL Injection Prevention**: Parameterized queries only

## Performance Considerations

### Database Performance

- **Connection Pooling**: Prevents connection exhaustion
- **Query Optimization**: Indexed columns for common queries
- **Migration Strategy**: Non-blocking schema changes
- **Health Monitoring**: Database connectivity checks

### Application Performance

- **Async Runtime**: Tokio for high-concurrency workloads
- **Memory Management**: Zero-copy operations where possible
- **Error Handling**: Fast-path for common error cases
- **Configuration**: Validated once at startup

## Scalability Design

### Horizontal Scaling

- **Stateless Design**: No server-side state (except database)
- **Session Storage**: Database-backed sessions for multi-instance deployment
- **Background Jobs**: Distributed task processing
- **Database Pooling**: Configurable connection limits

### Operational Scaling

- **Single Binary**: Easy deployment and management
- **Health Endpoints**: Load balancer integration
- **Graceful Shutdown**: Clean connection handling
- **Configuration**: Environment-based configuration

## Development Patterns

### Code Organization

```
src/
├── main.rs         -- CLI entry point and mode selection
├── lib.rs          -- Library exports and module structure
├── config.rs       -- Configuration management
├── database.rs     -- Database connection and migrations
├── error.rs        -- Error types and HTTP conversion
├── types.rs        -- Common type definitions
├── models.rs       -- Domain models and validation
├── api/            -- HTTP handlers and routing (future)
├── auth/           -- Authentication and authorization (future)
└── worker/         -- Background job processing (future)
```

### Testing Strategy

- **Unit Tests**: Individual function and module testing
- **Integration Tests**: Database and API endpoint testing
- **Property Tests**: Data validation and business logic
- **End-to-End Tests**: Full application workflow testing

### Error Handling Strategy

1. **Domain Errors**: Business logic validation errors
2. **Infrastructure Errors**: Database and external service errors
3. **HTTP Errors**: Request validation and authentication errors
4. **System Errors**: Configuration and runtime errors

## Future Architecture

### Planned Extensions

1. **HTTP API Layer**: RESTful API with Axum
2. **Authentication Middleware**: Session and API key validation
3. **Business Logic Services**: User management, data processing
4. **Background Workers**: Task processing and scheduling
5. **Frontend Integration**: React or similar SPA framework

### Technology Choices

- **Web Framework**: Axum (fast, type-safe, async)
- **Database**: PostgreSQL (ACID compliance, JSON support)
- **Authentication**: Session-based (scalable, secure)
- **Background Jobs**: Database-backed queue (simple, reliable)
- **Deployment**: Docker containers (portable, consistent)

## Operational Considerations

### Monitoring

- **Health Endpoints**: Application and database health
- **Metrics Collection**: Performance and usage metrics
- **Logging**: Structured logging with correlation IDs
- **Error Tracking**: Comprehensive error reporting

### Deployment

- **Container Strategy**: Single container, multiple processes
- **Environment Management**: Environment-specific configuration
- **Database Migrations**: Automated schema management
- **Secret Management**: External secret injection

### Maintenance

- **Dependency Updates**: Regular security and feature updates
- **Database Maintenance**: Index optimization and cleanup
- **Log Rotation**: Disk space management
- **Backup Strategy**: Data protection and recovery