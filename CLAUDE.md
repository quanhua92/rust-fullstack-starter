# CLAUDE.md

This file provides guidance to Claude Code when working with this Rust fullstack starter project.

## Project Constraints

- **Starter Project**: Tone down language - never say "production" or "enterprise ready"
- **Quality First**: Always run `./scripts/check.sh` before every commit

## Architecture Notes

### Database Connection Type
- **DbConn**: `sqlx::PgConnection` (not `PoolConnection`) to support both pool connections and transactions
- **Connection patterns**: Use `conn.as_mut()` for pool connections, `tx.as_mut()` for transactions
- **SQLx queries**: Use `&mut *conn` for all `.fetch_*()` and `.execute()` calls

## Essential Commands

### Development Workflow
```bash
# Quick start (recommended)
./scripts/dev-server.sh              # Complete environment: DB + web + API + worker
./scripts/check.sh                   # Quality checks (MANDATORY before commit)
./scripts/test-with-curl.sh          # 83 API endpoint tests
./scripts/reset-all.sh --reset-database  # Clean reset

# Testing
cargo nextest run                    # 136 integration tests (~17s)
./scripts/test-chaos.sh             # Docker-based resilience testing
cd web && ./scripts/check-web.sh    # Frontend quality checks
```

### Key Scripts
- `check.sh` - **Comprehensive quality validation (9 steps, ~40s)**
- `dev-server.sh` - Complete development environment
- `server.sh` / `worker.sh` - Individual services
- `test-with-curl.sh` - API testing (83 endpoints)
- `test-chaos.sh` - Resilience testing
- `test-template-with-curl.sh` - Generated module API testing
- `test-generate.sh` - Module generator system validation

## Code Patterns

### Module Generation
```bash
# Generate modules
cargo run -- generate module books --template basic
cargo run -- generate module products --template production --force
cargo run -- generate module orders --dry-run

# Safety revert
cargo run -- revert module books --dry-run  # Preview first
cargo run -- revert module books --yes      # Skip prompts (DANGEROUS)

# Manual integration (3 steps)
# 1. Add to src/lib.rs: pub mod books;
# 2. Add to src/server.rs: use crate::books::api::books_routes;
# 3. Add to src/openapi.rs: use crate::books::models::*;
```

### Task Handlers
```rust
use crate::{extract_fields, require_field, require_typed_field};

// Clean field extraction
let (to, subject, body) = extract_fields!(context.payload, "to", "subject", "body")?;
let file_path = require_field!(context.payload, "file_path")?;
let count = require_typed_field!(context.payload, "count", as_i64)?;

// Better error messages
TaskError::missing_field("field")
TaskError::invalid_field_type("field", "string")
```

### RBAC Usage
```rust
use crate::rbac::services as rbac_services;

// Ownership-based access control (for individual operations)
rbac_services::can_access_own_resource(&auth_user, resource.created_by)?;

// Task-specific access control (legacy - use ownership pattern for new code)
rbac_services::can_access_task(&auth_user, task.created_by)?;

// Role-based access control (for bulk operations and admin features)
rbac_services::require_moderator_or_higher(&auth_user)?;
rbac_services::check_permission(&auth_user, Resource::Tasks, Permission::Write)?;
```

### Ownership Pattern (Recommended)
```rust
// For individual CRUD operations - users can access their own, admins can access all
let mut tx = pool.begin().await?;
let existing_item = get_item_service(tx.as_mut(), id).await?;
rbac_services::can_access_own_resource(&auth_user, existing_item.created_by)?;
let updated_item = update_item_service(tx.as_mut(), id, request).await?;
tx.commit().await?;

// For bulk operations - require moderator permissions
rbac_services::require_moderator_or_higher(&auth_user)?;
bulk_create_items_service(conn.as_mut(), request).await?;
```

### Monitoring Integration
```rust
// Event logging
services::create_event(&mut conn, CreateEventRequest {
    event_type: "log".to_string(),
    source: "service-name".to_string(),
    message: Some("Action completed".to_string()),
    level: Some("info".to_string()),
    tags: HashMap::from([("user_id".to_string(), json!(user.id))]),
    payload: HashMap::new(),
    recorded_at: None,
}).await?;
```

### Module Generator Usage
```bash
# Generate modules with templates
cargo run -- generate module books --template basic      # Simple CRUD
cargo run -- generate module products --template production  # Advanced features

# Post-generation workflow
cd starter && sqlx migrate run
cd .. && ./scripts/prepare-sqlx.sh
./scripts/test-template-with-curl.sh products            # Test generated API

# Clean up
cargo run -- revert module products --yes
```

## Architecture Overview

### Core Systems
- **Authentication**: Session-based with 3-tier RBAC (User/Moderator/Admin)
- **Background Tasks**: Async processing with retry strategies and circuit breakers
- **User Management**: 12 endpoints for profile/admin operations
- **Monitoring**: 14 endpoints for events/metrics/alerts/incidents
- **Module Generator**: Template-based code generation with testing validation
- **Testing**: 136 integration tests with database isolation

### Module Structure
```
starter/src/
├── auth/          # Authentication & sessions
├── users/         # User management (12 endpoints)
├── tasks/         # Background task system
├── monitoring/    # Observability (14 endpoints)
├── rbac/          # Role-based access control
└── cli/           # Admin CLI commands
```

### Key Features
- **Task Type Registration**: Workers register types before creating tasks
- **Health Endpoints**: `/api/v1/health/*` for monitoring
- **OpenAPI Docs**: `/api-docs` with Swagger UI
- **Admin CLI**: Direct DB access for debugging (`cargo run -- admin task-stats`)
- **Code Generation**: Templates with compile-time SQLx validation and route patterns

## Development Notes

### Quality Requirements
1. **Pre-commit**: Always run `./scripts/check.sh`
2. **Testing**: 136 integration tests must pass
3. **SQLx**: Use `./scripts/prepare-sqlx.sh` for query cache updates
4. **Frontend**: Run `cd web && ./scripts/check-web.sh` for React validation

### Common Tasks
- **Start workers before creating tasks** (registration requirement)
- **Use `recorded_at: None` for monitoring structs** (not `timestamp`)
- **Admin account**: Set `STARTER__INITIAL_ADMIN_PASSWORD` in `.env`
- **Chaos testing**: Docker-based with 6 difficulty levels
- **Template testing**: Use `./scripts/test-template-with-curl.sh products` for API validation
- **Route registration**: Manually add `use crate::products::api::products_routes;` to `server.rs`

### Script Utilities
```bash
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
print_status "step|success|error|warning|info" "message"
run_cmd "Description" command args
validate_project_root
```

## Important Endpoints

### Authentication & Users
- `POST /auth/register` - User registration
- `POST /auth/login` - Session login
- `GET /api/v1/users` - List users (Moderator+)
- `PUT /api/v1/users/me/profile` - Update own profile

### Tasks & Monitoring
- `POST /api/v1/tasks` - Create background task
- `GET /api/v1/tasks/types` - List registered task types
- `POST /api/v1/monitoring/events` - Log events
- `GET /api/v1/monitoring/metrics/prometheus` - Prometheus metrics

### Health & Admin
- `GET /api/v1/health` - Basic health check
- `GET /api/v1/admin/users/stats` - User analytics (Admin)

This starter provides a solid foundation for learning Rust web development with modern patterns for authentication, task processing, monitoring, testing, and rapid module scaffolding.