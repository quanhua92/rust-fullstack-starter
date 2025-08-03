# The Feynman Method: Mastering the Rust Fullstack Starter
## Teacher's Complete Guide to System Mastery

*"What I cannot create, I do not understand." - Richard P. Feynman*

---

## 🎯 Teaching Philosophy

### The Starter-Focused Approach

This course is NOT about general web development. It's about achieving complete mastery of THIS specific starter system. Every lesson, every exercise, every question focuses on understanding exactly how this codebase works and why it was built this way.

### The Feynman Learning Framework for THIS Starter

1. **Start with WHY** - Every feature in this starter exists for a specific reason
2. **Master Every Line** - No code remains mysterious or "magic"  
3. **Make it Concrete** - Connect every abstraction to real examples in the codebase
4. **Test Deep Understanding** - Can modify and extend any part of the system
5. **Own the Architecture** - Understand every design decision and trade-off

### Core Principles

- **No Magic Boxes**: Every abstraction in the starter will be opened and examined
- **Master THIS System**: Deep understanding of one real system beats shallow knowledge of many
- **Question Every Decision**: Why Rust? Why this file structure? Why these dependencies?
- **Debug the Actual Code**: Use real debugging tools on the actual system
- **Graduate to Ownership**: By the end, students can confidently modify any part

---

## 📚 Course Structure: 16 Lessons (3 Phases)

### 🦀 Phase 1: Backend Mastery (Lessons 1-9)
**Goal**: Complete understanding of every Rust file in `starter/src/`
**Duration**: 7-9 weeks  
**Philosophy**: Master the server completely before adding frontend complexity

### 🌐 Phase 2: Frontend Integration (Lessons 10-14)
**Goal**: Understand how the React app in `web/` connects to the backend
**Duration**: 4-5 weeks
**Philosophy**: Build on solid backend foundation with real monitoring data

### 🔧 Phase 3: Customization & Mastery (Lessons 15-16)
**Goal**: Use the rename script to create your own custom system
**Duration**: 2-3 weeks  
**Philosophy**: True mastery means ability to adapt and extend

---

## 🎯 PHASE 1: BACKEND MASTERY (Lessons 1-8)

### **Lesson 1: The Starter System Overview**
*"Before we dissect the frog, let's understand what makes it alive"*

**Learning Objectives:**
- Map the complete file structure of `starter/src/` (38 Rust files, 7,719 lines of code!)
- Understand what THIS starter provides vs a basic web server
- Trace the startup sequence from `main.rs` to running system
- Master the elegant simplicity of the architecture

**Starter-Specific Materials:**
- `starter/src/main.rs` - The actual entry point (only 6 lines!)
- `starter/src/lib.rs` - Library structure and module exports (13 modules)
- `starter/src/` - Complete module structure (8 core domains + 5 infrastructure)
- `starter/Cargo.toml` - The exact dependencies and features (24 production deps)
- `scripts/dev-server.sh` - Complete system bootstrap (172 lines of orchestration)
- `docs/getting-started.md` - This system's setup guide
- `docs/guides/01-architecture.md` - This system's design philosophy

**🎯 Key Insights for Teaching:**

**The 6-Line Miracle (`main.rs`):**
This starter demonstrates the power of proper architecture - the entire application entry point is just 6 lines! This is possible because all complexity is properly organized into modules. The `CliApp::run()` method handles both CLI commands and web server startup, showcasing elegant command pattern implementation.

**Module Architecture (13 total modules):**
- **Core Domains (8)**: `auth`, `rbac`, `tasks`, `users`, `cli`, `api`, `server`, `openapi`
- **Infrastructure (5)**: `config`, `database`, `error`, `models`, `types`

This separation demonstrates domain-driven design principles where business logic is separate from infrastructure concerns.

**Dependency Philosophy:**
The 24 production dependencies in `Cargo.toml` are carefully chosen for production readiness:
- **Axum**: Modern, type-safe web framework with excellent performance
- **SQLx**: Compile-time checked SQL queries with async support
- **Tokio**: Industry-standard async runtime
- **Utoipa**: Automatic OpenAPI documentation generation
- All dependencies use workspace inheritance for version consistency

**Bootstrap Orchestration (`dev-server.sh`):**
The 172-line bootstrap script demonstrates production-ready development practices:
1. **Infrastructure validation** (docker-compose.yaml, migrations directory)
2. **Service orchestration** (PostgreSQL startup with health checks)
3. **Configuration management** (.env setup from template)
4. **Frontend build integration** (optional web build with fallback)
5. **Database migration execution** (with comprehensive error handling)
6. **Service lifecycle management** (foreground/background modes)

**Deep Dive Questions:**
1. **Architecture Question**: What are the 8 core domain modules and 5 infrastructure modules? How does this separation demonstrate clean architecture?
2. **Performance Question**: Why does this starter use Axum over alternatives like Warp or Rocket? (Hint: examine the tower middleware ecosystem)
3. **DevOps Question**: What are the 6 distinct phases in `./scripts/dev-server.sh` and how do they handle failure scenarios?
4. **Scale Question**: We're mastering 7,719 lines of Rust code across 38 files - what's the average file size and what does this tell us about the code organization?
5. **Dependency Question**: This system depends on PostgreSQL and Docker - what would be required to swap PostgreSQL for MySQL?

**🔍 Teaching Experiments:**
1. **Minimal Change Test**: Modify the port in `main.rs` (trick question - it's not there!)
2. **Dependency Hunt**: Find where each of the 24 dependencies is actually used
3. **Bootstrap Failure**: What happens if you run `dev-server.sh` without Docker?
4. **Module Mapping**: Draw the dependency graph between the 13 modules

**💡 Architectural Insights:**
- **CLI-First Design**: The app can run as both web server and CLI tool
- **Workspace Architecture**: All dependencies inherit from root workspace
- **Graceful Degradation**: Web build can fail without breaking API functionality
- **Service Composition**: Database, worker, and server can be started independently
- **Production Patterns**: Health checks, proper error handling, structured logging

---

### **Lesson 2: Database Foundation (`starter/migrations/` & `database.rs`)**
*"Understanding the data is understanding the heart of the system"*

**Learning Objectives:**
- Master every table in the 5-table database schema (126 lines of SQL)
- Understand the migration system and dependency relationships
- See how `database.rs` manages connection pooling and admin user creation
- Grasp the production-ready database patterns used throughout

**Starter-Specific Materials:**
- `starter/src/database.rs` - Connection pool and transaction management (126 lines)
- `starter/migrations/001_users.up.sql` - Users table with RBAC (34 lines)
- `starter/migrations/002_sessions.up.sql` - Session management schema (24 lines)
- `starter/migrations/003_api_keys.up.sql` - API key authentication (26 lines)
- `starter/migrations/004_tasks.up.sql` - Background tasks with enums (42 lines)
- `starter/migrations/005_task_types.up.sql` - Task type registration (33 lines)
- `.env.example` - Database configuration template
- `docs/configuration.md` - Database configuration for this system

**🎯 Key Insights for Teaching:**

**The 5-Table Architecture:**
1. **`users`** (foundation) - RBAC with role constraints, comprehensive indexing
2. **`sessions`** (auth layer) - Token-based auth with activity tracking  
3. **`api_keys`** (machine auth) - M2M authentication with usage analytics
4. **`tasks`** (job queue) - Background processing with custom enums and retry logic
5. **`task_types`** (registry) - Dynamic task type validation system

**Production-Grade Schema Design:**

**Users Table Sophistication:**
- UUID primary keys with `gen_random_uuid()` for security
- Role constraint: `CHECK (role IN ('user', 'moderator', 'admin'))`
- 5 strategic indexes including composite `(role, is_active)` for performance
- Automatic `updated_at` triggers using PostgreSQL functions

**Task System Excellence:**
- Custom PostgreSQL enums: `task_status` (6 states) and `task_priority` (4 levels)
- JSONB payload for flexible task data with GIN indexing potential
- Composite index: `idx_tasks_ready_to_run` optimized for queue processing
- Soft foreign key to users (`ON DELETE SET NULL`) preserving task history

**Connection Pool Mastery (`database.rs`):**
The `Database::connect()` method demonstrates production-grade connection management:
- **Auto-database creation** if not exists (lines 15-23)
- **Configurable pool sizing** (min/max connections from config)
- **Comprehensive timeouts** (acquire, idle, max_lifetime)
- **Structured logging** with pool size reporting

**Admin User Bootstrap Pattern:**
The `ensure_initial_admin()` method (lines 56-115) showcases sophisticated initialization:
- **Conditional creation** - only if no admins exist
- **Secure password hashing** with Argon2 and salt generation
- **Comprehensive logging** with helpful setup instructions
- **Configuration-driven** via `STARTER__INITIAL_ADMIN_PASSWORD`

**Migration Dependency Chain:**
1. **001_users** → Foundation with trigger function
2. **002_sessions** → Depends on users (foreign key)
3. **003_api_keys** → Depends on users (created_by)
4. **004_tasks** → Depends on users (created_by), defines custom types
5. **005_task_types** → Standalone registry, backfills existing tasks

**Deep Dive Questions:**
1. **Schema Question**: What are the 5 tables and their 11 total indexes? How do the composite indexes optimize specific query patterns?
2. **Enum Question**: Why does this starter use PostgreSQL enums for `task_status` and `task_priority` instead of string constants?
3. **Pool Question**: How do the connection pool parameters (min: 5, max: 20, timeouts) prevent database overload under high concurrency?
4. **Migration Question**: What would happen if you ran migration 002 before 001? How does the foreign key cascade behavior protect data integrity?
5. **Bootstrap Question**: How does the admin user creation pattern balance security (Argon2 hashing) with development convenience?

**🔍 Teaching Experiments:**
1. **Index Performance**: Query `EXPLAIN ANALYZE` on tasks table with and without the composite index
2. **Pool Behavior**: Set max_connections to 2 and see how the system handles 10 concurrent requests
3. **Migration Rollback**: Run a down migration and observe the cascade effects
4. **Enum Validation**: Try inserting invalid task_status values and see PostgreSQL constraints in action
5. **Admin Creation**: Test the bootstrap process with various environment variable configurations

**💡 Database Insights:**
- **PostgreSQL-First**: Uses PG-specific features (enums, JSONB, generated UUIDs, triggers)
- **Performance-Conscious**: Strategic indexing for common query patterns
- **Security-Focused**: Foreign key constraints, role validation, secure password storage
- **Operations-Friendly**: Health checks, structured logging, graceful failure handling
- **Development-Optimized**: Auto-database creation, migration automation, helpful error messages

**🗃️ Schema Relationships:**
```
users (foundation)
├── sessions (1:many) → user_id
├── api_keys (1:many) → created_by  
└── tasks (1:many) → created_by [nullable]

task_types (registry)
└── tasks (1:many) → task_type [constraint to be added]
```

**Performance Characteristics:**
- **Total schema size**: 159 lines of SQL across 5 migrations
- **Index count**: 11 strategically placed indexes
- **Connection pool**: 5-20 connections with intelligent timeout management
- **Bootstrap time**: Sub-second admin user creation with Argon2 hashing

---

### **Lesson 3: Authentication System (`starter/src/auth/`)**
*"Every request must prove its identity"*

**Learning Objectives:**
- Master the complete auth module: 6 files, 844 lines of authentication code
- Understand session-based authentication with Bearer tokens
- See how 64-character session tokens and activity tracking work
- Grasp the sophisticated validation and refresh mechanisms

**Starter-Specific Materials:**
- `starter/src/auth/mod.rs` - Module structure and exports (11 lines)
- `starter/src/auth/models.rs` - Auth data structures and validation (126 lines)
- `starter/src/auth/services.rs` - Session management and crypto (317 lines)
- `starter/src/auth/api.rs` - 6 HTTP endpoints with OpenAPI docs (207 lines)
- `starter/src/auth/middleware.rs` - Route protection with 3 middleware types (127 lines)
- `starter/src/auth/cleanup.rs` - Hourly session cleanup background job (36 lines)
- `starter/tests/auth/` - Complete authentication test suite
- Database tables: `users` and `sessions` with activity tracking

**🎯 Key Insights for Teaching:**

**Session Model Sophistication (`models.rs`):**
The `Session` struct (lines 6-18) demonstrates enterprise-grade session management:
- **UUID primary keys** for security and uniqueness
- **Activity tracking** with `last_activity_at` and `last_refreshed_at`
- **User agent tracking** for security audit trails
- **Soft deletion** with `is_active` boolean (never actually delete sessions)
- **Built-in expiration logic** with `is_expired()` and `can_refresh()` methods

**Token Generation Security (`services.rs`):**
The `generate_session_token()` function (lines 11-21) showcases cryptographic best practices:
- **64-character tokens** using alphanumeric charset (62^64 combinations)
- **Cryptographically secure random** with `rand::rng()`
- **No predictable patterns** - purely random selection

**Authentication Flow Architecture:**
1. **Login** (lines 157-230): Transaction-based with password verification + session creation
2. **Session Validation** (lines 140-155): Token lookup + expiration check + activity update
3. **Logout** (lines 232-238): Soft deletion (sets `is_active = false`)
4. **Token Refresh** (lines 246-302): Rate-limited refresh with configurable intervals

**Middleware Layering (`middleware.rs`):**
Three distinct middleware types demonstrate flexible authentication:
1. **`auth_middleware`** (lines 33-81): Strict authentication required
2. **`optional_auth_middleware`** (lines 84-109): Optional auth for mixed endpoints
3. **`admin_middleware`** (lines 112-126): Role-based access control layer

**Request Validation Patterns (`models.rs`):**
The `LoginRequest::validate()` method (lines 59-92) shows sophisticated validation:
- **Exclusive validation**: Either username OR email, not both
- **Delegation to user validators**: Reuses username/email validation from users module
- **Clear error messages**: Specific field-level error reporting

**API Endpoint Design (`api.rs`):**
Six carefully designed endpoints with full OpenAPI documentation:
1. **`POST /auth/login`** - Username/email + password authentication
2. **`POST /auth/register`** - New user account creation
3. **`POST /auth/logout`** - Single session termination
4. **`POST /auth/logout-all`** - Multi-device session termination
5. **`GET /auth/me`** - Current user information
6. **`POST /auth/refresh`** - Token refresh with rate limiting

**Background Job Architecture (`cleanup.rs`):**
The cleanup job demonstrates production-ready maintenance:
- **Hourly execution** using `tokio::time::interval`
- **Graceful error handling** with structured logging
- **Soft session deletion** preserving audit trails
- **Non-blocking operation** using async/await

**Deep Dive Questions:**
1. **Security Question**: How does the 64-character token generation provide security, and what's the mathematical probability of collision?
2. **Architecture Question**: Why does login use a database transaction (lines 179-210) and what happens if it fails halfway?
3. **Middleware Question**: How do the 3 middleware types work together, and why is `admin_middleware` separate from `auth_middleware`?
4. **Session Question**: How does the refresh mechanism prevent abuse with `min_refresh_interval_minutes` and `can_refresh()` logic?
5. **Validation Question**: How does `LoginRequest::validate()` demonstrate the single responsibility principle by delegating to user validators?

**🔍 Teaching Experiments:**
1. **Token Analysis**: Generate 1000 session tokens and analyze their entropy and character distribution
2. **Middleware Stacking**: Create an endpoint with both auth and admin middleware and trace the request flow
3. **Session Lifecycle**: Create a session, use it, refresh it, then logout and see the database state changes
4. **Cleanup Testing**: Create expired sessions and run the cleanup job to see the soft deletion in action
5. **Validation Edge Cases**: Test login with various invalid combinations (both username+email, empty fields, etc.)

**💡 Authentication Insights:**
- **Session-Based Security**: Uses Bearer tokens instead of JWT for server-side session control
- **Activity Tracking**: Every API call updates `last_activity_at` for security monitoring
- **Soft Deletion Pattern**: Sessions are deactivated, not deleted, preserving audit trails
- **Rate-Limited Refresh**: Prevents token refresh abuse with configurable intervals
- **Flexible Middleware**: Three-tier approach (none/optional/required/admin) for different endpoints
- **Transaction Safety**: Critical operations like login use database transactions
- **Cryptographic Security**: 64-character tokens with 62^64 possible combinations

**🔐 Security Model:**
```
Request Flow:
1. Extract Bearer token from Authorization header
2. Look up session in database (with is_active=true filter)
3. Check expiration time against current timestamp
4. Update last_activity_at for tracking
5. Inject AuthUser into request extensions
6. Proceed to route handler or admin check
```

**Performance Characteristics:**
- **Session lookup**: Single indexed query by token
- **Activity update**: Single UPDATE per authenticated request  
- **Token generation**: O(1) with 64 random character selections
- **Cleanup frequency**: Hourly background job with batch updates

---

### **Lesson 4: RBAC System (`starter/src/rbac/`)**
*"Authentication says who you are, authorization says what you can do"*

**Learning Objectives:**
- Master the three-tier RBAC system: 4 files, 622 lines of authorization logic
- Understand the numerical hierarchy: User(1) < Moderator(2) < Admin(3)
- See how Permission + Resource combinations control access granularly
- Grasp the security-first design with enumeration prevention

**Starter-Specific Materials:**
- `starter/src/rbac/mod.rs` - RBAC module structure and exports (9 lines)
- `starter/src/rbac/models.rs` - UserRole enum with PostgreSQL integration (231 lines)
- `starter/src/rbac/services.rs` - Permission logic and ownership checks (163 lines)
- `starter/src/rbac/middleware.rs` - Route protection middleware (188 lines)
- `starter/src/users/` - User management with role elevation
- Examples throughout: `tasks/api.rs`, `users/api.rs`, `cli/api.rs`

**🎯 Key Insights for Teaching:**

**Three-Role Hierarchy (`models.rs`):**
The `UserRole` enum (lines 11-18) implements a numerical hierarchy system:
- **User = 1**: Can only access own resources (tasks, profile)
- **Moderator = 2**: Can manage all users' tasks and profiles (except admin users)
- **Admin = 3**: Full system access including admin endpoints

This uses Rust's `PartialOrd` derive to enable `>=` comparisons for role elevation checks.

**Fine-Grained Permission Matrix (`models.rs` lines 45-68):**
The `can_access()` method implements a comprehensive permission matrix:
```rust
// Moderators can read/write tasks and users, but can't delete users
(UserRole::Moderator, Resource::Users, Permission::Delete) => false,
// Users can manage own resources (ownership checked elsewhere)
(UserRole::User, Resource::Tasks, Permission::Read) => true, // Own only
```

**Security-First Access Control (`services.rs`):**
- **Anti-enumeration**: Returns "Task not found" instead of "Access denied" (lines 38, 42)
- **Ownership-based access**: `can_access_task()` checks `task_created_by` field (lines 28-46)
- **Role hierarchy protection**: Moderators can't see admin profiles (lines 59-65)
- **System task protection**: Tasks without `created_by` are admin-only (lines 40-43)

**PostgreSQL Integration (`models.rs` lines 105-126):**
Custom SQLx implementations for seamless database storage:
- **Zero-allocation encoding**: Uses `as_str()` instead of `to_string()` (line 118)
- **Fallback handling**: Invalid roles default to `User` for safety (line 100)
- **Case-insensitive parsing**: "USER", "Admin", "moderator" all work (line 81)

**Middleware Architecture (`middleware.rs`):**
Four middleware types for different protection levels:
1. **`require_role()`** - Higher-order function for specific role requirements
2. **`require_role_or_higher()`** - Hierarchical role checking  
3. **`require_permission()`** - Resource + permission-based access
4. **Convenience functions**: `require_admin_role()`, `require_moderator_role()`

**Access Patterns Demonstrated:**
- **Task Access**: Admin/Moderator see all, Users see own only
- **User Profile Access**: Admin sees all, Moderator sees non-admin, Users see own
- **Admin Endpoints**: Admin-only with explicit role checks
- **Cross-User Operations**: Role-based with ownership validation

**Deep Dive Questions:**
1. **Hierarchy Question**: How does the numerical enum values (1,2,3) enable the `>=` comparison in `has_role_or_higher()`?
2. **Security Question**: Why does `can_access_task()` return "Task not found" instead of "Access denied" for unauthorized access?
3. **Permission Question**: What's the difference between Resource-based permissions and ownership-based access in the permission matrix?
4. **Database Question**: How do the custom SQLx traits ensure zero-allocation role encoding/decoding?
5. **Middleware Question**: Why are there separate `require_role()` and `require_permission()` middleware instead of just one?

**🔍 Teaching Experiments:**
1. **Role Hierarchy**: Create users with different roles and test `>=` comparisons in the Rust REPL
2. **Permission Matrix**: Map out all 27 possible (Role, Resource, Permission) combinations and their outcomes
3. **Anti-Enumeration**: Try accessing non-existent vs unauthorized tasks and compare error messages
4. **Middleware Stacking**: Create routes with multiple middleware layers and trace the execution order
5. **Database Round-Trip**: Store and retrieve roles from database to see the SQLx encoding in action

**💡 RBAC Design Insights:**
- **Type-Safe Permissions**: Uses enums instead of strings to prevent runtime permission errors
- **Hierarchical Design**: Higher roles automatically include lower role permissions
- **Ownership Integration**: Combines role-based and ownership-based access control
- **Security-Conscious**: Prevents information leakage through error messages
- **Database-Optimized**: Custom SQLx traits minimize allocations and support fallbacks
- **Middleware Composition**: Supports both declarative (middleware) and imperative (service calls) patterns

**🛡️ Security Model:**
```
Access Decision Flow:
1. Check if user has required role level (numeric comparison)
2. If accessing specific resource, check permission matrix
3. If accessing owned resource, verify ownership match
4. If accessing user profile, check role hierarchy restrictions
5. Return appropriate error (403 Forbidden vs 404 Not Found)
```

**Role Capability Matrix:**
| Action | User | Moderator | Admin |
|--------|------|-----------|-------|
| Read own tasks | ✅ | ✅ | ✅ |
| Read all tasks | ❌ | ✅ | ✅ |
| Delete own tasks | ✅ | ✅ | ✅ |
| Delete user accounts | ❌ | ❌ | ✅ |
| Access admin endpoints | ❌ | ❌ | ✅ |
| View admin profiles | ❌ | ❌ | ✅ |

**Testing Coverage:**
- **Unit tests**: 30+ test cases covering role hierarchy, permissions, and edge cases
- **Integration tests**: Middleware testing with real HTTP requests
- **Property testing**: Role comparison and permission matrix validation

---

### **Lesson 5: Task System (`starter/src/tasks/`)**
*"The beating heart of background work"*

**Learning Objectives:**
- Master the complete task processing system: 7 files, 1,200+ lines of async job queue
- Understand the TaskProcessor with semaphore-based concurrency control
- See the sophisticated retry strategies (Exponential, Linear, Fixed) with circuit breakers
- Grasp the macro-based payload extraction system for type-safe task handling

**Starter-Specific Materials:**
- `starter/src/tasks/mod.rs` - Task system module structure (11 lines)
- `starter/src/tasks/types.rs` - Task data structures and enums (334 lines)
- `starter/src/tasks/processor.rs` - Background task execution engine (400+ lines)
- `starter/src/tasks/api.rs` - HTTP endpoints for task operations (200+ lines)
- `starter/src/tasks/handlers.rs` - Built-in task handlers with examples (150+ lines)
- `starter/src/tasks/retry.rs` - Retry strategies and circuit breaker (200+ lines)
- `starter/src/tasks/helpers.rs` - Convenience macros for payload extraction (152 lines)
- `docs/guides/04-background-tasks.md` - Task system design
- Database tables: `tasks`, `task_types` with PostgreSQL enums

**🎯 Key Insights for Teaching:**

**Task Status State Machine (`types.rs` lines 8-17):**
Six-state PostgreSQL enum system with precise transitions:
- **Pending** → **Running** → **Completed/Failed**
- **Failed** → **Retrying** → **Running** (retry loop)
- **Any State** → **Cancelled** (manual cancellation)

Each status maps to specific database constraints and business logic.

**Priority-Based Queue System (`types.rs` lines 19-26):**
Four-level priority system: `Critical > High > Normal > Low`
- Database index: `idx_tasks_ready_to_run ON tasks(priority DESC, created_at ASC)`
- Higher priority tasks always execute first
- FIFO ordering within same priority level

**Sophisticated Retry Strategies (`retry.rs`):**
Three mathematically-defined retry patterns:
```rust
// Exponential: delay = base_delay * multiplier^attempt
Exponential { base_delay: 1s, multiplier: 2.0, max_delay: 5min, max_attempts: 5 }
// Linear: delay = base_delay + (increment * attempt)  
Linear { base_delay: 1s, increment: 2s, max_delay: 1min, max_attempts: 10 }
// Fixed: same interval every time
Fixed { interval: 30s, max_attempts: 3 }
```

**TaskProcessor Concurrency Architecture (`processor.rs`):**
Production-grade async processing with:
- **Semaphore-based concurrency**: Limits simultaneous tasks (default: 10)
- **Circuit breaker per task type**: Prevents cascade failures
- **Configurable timeouts**: Task execution timeout (default: 5 minutes)
- **Batch processing**: Fetches multiple tasks per poll (default: 50)
- **Graceful shutdown**: Waits for running tasks to complete

**Type-Safe Payload Extraction (`helpers.rs`):**
Macro system for safe JSON payload handling:
```rust
// Extract multiple required fields at once
let (to, subject, body) = extract_fields!(context.payload, "to", "subject", "body")?;

// Extract typed fields with validation
let count = require_typed_field!(context.payload, "count", as_i64)?;
let enabled = require_typed_field!(context.payload, "enabled", as_bool)?;
```

**TaskHandler Trait System (`handlers.rs`):**
Async trait for task implementations:
```rust
#[async_trait]
pub trait TaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError>;
}
```

**Built-in Handler Examples:**
1. **EmailTaskHandler**: Simulates email sending with metadata tracking
2. **DataProcessingTaskHandler**: Array operations (count, sum) with error handling

**TaskContext Rich Data Structure (`types.rs` lines 192-201):**
Provides handlers with complete execution context:
- **Task metadata**: ID, type, attempt number, timestamps
- **Payload data**: JSON payload with helper extraction
- **User context**: `created_by` for ownership tracking
- **Execution tracking**: Attempt count, creation time

**Circuit Breaker Integration (`processor.rs`):**
Per-task-type circuit breakers prevent systemic failures:
- **Closed**: Normal operation
- **Open**: Failing fast after threshold breaches
- **Half-Open**: Testing recovery with limited requests

**API Endpoints Design (`api.rs`):**
RESTful task management with RBAC integration:
- **POST /tasks** - Create new task (validates task type registration)
- **GET /tasks** - List tasks (ownership-filtered for users)
- **GET /tasks/{id}** - Get specific task (ownership-checked)
- **DELETE /tasks/{id}** - Cancel task (ownership-validated)
- **GET /tasks/stats** - Task statistics (role-based access)
- **POST /tasks/types** - Register task type (public endpoint for workers)
- **GET /tasks/types** - List registered task types (public)

**Deep Dive Questions:**
1. **Concurrency Question**: How does the TaskProcessor use semaphores to limit concurrent task execution, and what happens when the limit is reached?
2. **Retry Question**: Compare the mathematical progression of Exponential vs Linear retry strategies - when would you use each?
3. **State Question**: What are the valid state transitions in the TaskStatus enum, and which transitions are forbidden?
4. **Handler Question**: How do the extraction macros (`extract_fields!`, `require_field!`) provide type safety for JSON payload handling?
5. **Architecture Question**: How does the circuit breaker pattern prevent cascade failures across different task types?

**🔍 Teaching Experiments:**
1. **Retry Mathematics**: Calculate actual delays for different retry strategies with various parameters
2. **Concurrency Testing**: Set max_concurrent_tasks to 2 and create 10 tasks to see semaphore blocking
3. **Circuit Breaker**: Create a failing task handler and watch the circuit breaker open/close cycle
4. **Priority Queue**: Create tasks with different priorities and observe execution order
5. **Payload Validation**: Test the extraction macros with various malformed JSON payloads

**💡 Task System Design Insights:**
- **Database-Driven Queue**: Uses PostgreSQL as the task queue with ACID guarantees
- **Async-First Architecture**: Built on Tokio with proper async/await throughout
- **Type Safety**: Compile-time payload validation through macro system
- **Resilience Patterns**: Circuit breakers, retry strategies, and timeout handling
- **Observability**: Structured logging with tracing throughout execution paths
- **RBAC Integration**: Task ownership and role-based access control
- **Production Ready**: Graceful shutdown, health checks, and metric collection

**🔄 Task Execution Flow:**
```
1. TaskProcessor polls database for ready tasks (status=pending|retrying, scheduled_at<=now)
2. Acquire semaphore permit (limits concurrency)
3. Check circuit breaker for task type
4. Set task status to 'running' and start_time
5. Execute handler with TaskContext
6. On success: set status to 'completed', completed_at
7. On failure: calculate retry delay or set to 'failed'
8. Release semaphore permit
9. Update circuit breaker state
10. Continue polling loop
```

**Performance Characteristics:**
- **Poll interval**: 5 seconds (configurable)
- **Task timeout**: 5 minutes (configurable)
- **Batch size**: 50 tasks per poll (configurable)
- **Concurrency**: 10 simultaneous tasks (configurable)
- **Database queries**: Indexed queries on (status, scheduled_at, priority)
- **Memory usage**: O(1) per task processor, O(n) per concurrent task

---

### **Lesson 6: API Layer (`starter/src/server.rs` & HTTP endpoints)**
*"How the outside world talks to our system"*

**Learning Objectives:**
- Master the complete HTTP API: 36 routes, 34 documented endpoints with OpenAPI
- Understand Axum's layered routing architecture with role-based middleware
- See comprehensive OpenAPI documentation generation with Bearer auth
- Grasp the four-tier security model (public, protected, moderator, admin)

**Starter-Specific Materials:**
- `starter/src/server.rs` - Complete router construction and middleware stack (265 lines)
- `starter/src/openapi.rs` - OpenAPI documentation generation (190 lines)
- `starter/src/api/health.rs` - Health check endpoints (5 Kubernetes-ready variants) (260 lines)
- `starter/src/error.rs` - Unified error handling and HTTP responses (150+ lines)
- `starter/src/auth/api.rs` - Authentication endpoints (6 endpoints)
- `starter/src/tasks/api.rs` - Task management endpoints (9 endpoints)
- `starter/src/users/api.rs` - User management endpoints (13 endpoints)
- `scripts/test-with-curl.sh` - Real API testing (44+ endpoint tests)

**🎯 Key Insights for Teaching:**

**Four-Tier Security Architecture (`server.rs` lines 109-175):**
The routing system demonstrates layered security with different access levels:

1. **Public Routes (8 endpoints)** - No authentication required:
   - Health checks: `/health`, `/health/detailed`, `/health/live`, `/health/ready`, `/health/startup`
   - Authentication: `/auth/login`, `/auth/register`
   - Task type registration: `/tasks/types` (POST/GET for workers)

2. **Protected Routes (17 endpoints)** - Authentication required:
   - Auth management: `/auth/logout`, `/auth/me`, `/auth/refresh`
   - Self-service: `/users/me/profile`, `/users/me/password`, `/users/me`
   - Task operations: `/tasks` (CRUD), `/tasks/{id}` (operations)

3. **Moderator Routes (3 endpoints)** - Moderator+ role required:
   - User oversight: `/users` (list), `/users/{id}/status`, `/users/{id}/reset-password`

4. **Admin Routes (8 endpoints)** - Admin role required:
   - System administration: `/admin/health`, `/admin/users/stats`
   - User management: `/users` (create), `/users/{id}` (profile/role/delete)

**OpenAPI Documentation System (`openapi.rs`):**
Comprehensive API documentation with 34 documented endpoints:
- **Auto-generated**: Uses `#[utoipa::path]` macros on handler functions
- **Type safety**: All request/response schemas derived from Rust structs
- **Security integration**: Bearer token authentication configured
- **Multiple formats**: JSON schema + interactive HTML documentation
- **Version controlled**: API version, contact info, and server endpoints

**Health Check Sophistication (`health.rs`):**
Five distinct health endpoints optimized for different monitoring needs:
1. **`/health`** - Basic health with version, uptime, documentation links
2. **`/health/detailed`** - Database connectivity + comprehensive dependency checks
3. **`/health/live`** - Kubernetes liveness probe (lightweight, always responds)
4. **`/health/ready`** - Kubernetes readiness probe (checks database + app readiness)
5. **`/health/startup`** - Kubernetes startup probe (database + schema validation)

**Middleware Stack Architecture (`server.rs` lines 184-211):**
Production-ready middleware layers applied in order:
- **Tracing**: HTTP request/response logging with structured data
- **Security headers**: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`
- **Request tracking**: `X-Request-ID` header for request correlation
- **CORS**: Cross-origin resource sharing (development: allow all)
- **Authentication**: Token validation and user context injection
- **Authorization**: Role-based access control per route group

**Static File Integration (`server.rs` lines 223-246):**
Unified server serving both API and frontend:
- **API routes**: Nested under `/api/v1` prefix
- **Documentation**: `/api-docs` (HTML) and `/api-docs/openapi.json`
- **Static files**: React SPA served from configured build directory
- **Fallback handling**: SPA routing with `index.html` fallback
- **Security validation**: Build path validation to prevent directory traversal

**Error Handling System (`error.rs`):**
Comprehensive error types with HTTP status code mapping:
- **Database errors**: SQLx errors → 500 Internal Server Error
- **Authentication errors**: Unauthorized (401), Forbidden (403), Token Expired
- **Validation errors**: Field-specific validation with 400 Bad Request
- **Business logic errors**: NotFound (404), UserAlreadyExists (409)
- **Consistent JSON responses**: Standardized error format across all endpoints

**Route Organization Pattern:**
```rust
// Grouped by security level, not by domain
let public_routes = Router::new().route("/health", get(health));
let protected_routes = Router::new()
    .route("/tasks", post(create_task))
    .layer(auth_middleware);
let admin_routes = Router::new()
    .route("/admin/health", get(detailed_health))
    .layer(admin_middleware)
    .layer(auth_middleware);
```

**Deep Dive Questions:**
1. **Architecture Question**: How do the four security tiers (public/protected/moderator/admin) work with Axum's middleware layering system?
2. **Documentation Question**: How does the OpenAPI generation system use Rust type information to create accurate API documentation?
3. **Health Question**: What's the difference between Kubernetes liveness, readiness, and startup probes, and how does each endpoint serve different monitoring needs?
4. **Security Question**: How do the security headers and CORS configuration protect against common web vulnerabilities?
5. **Integration Question**: How does the unified server serve both API endpoints and static React files without conflicts?

**🔍 Teaching Experiments:**
1. **Middleware Order**: Rearrange middleware layers and observe how authentication/authorization breaks
2. **Health Monitoring**: Test each health endpoint under different failure conditions (database down, etc.)
3. **OpenAPI Generation**: Add a new endpoint and see how the documentation auto-updates
4. **Route Conflicts**: Try to add conflicting routes and see Axum's compile-time protection
5. **Security Headers**: Use browser dev tools to inspect the security headers on API responses

**💡 API Design Insights:**
- **Security-First Routing**: Groups routes by security requirements, not business domains
- **Comprehensive Documentation**: Every endpoint documented with OpenAPI for client generation
- **Production Health Checks**: Multiple health endpoints optimized for different operational needs
- **Unified Architecture**: Single server binary serves both API and static frontend
- **Type-Safe API**: Rust's type system ensures request/response consistency
- **Layered Security**: Multiple middleware layers provide defense in depth
- **Error Consistency**: Standardized error responses across all endpoints

**🌐 API Architecture Flow:**
```
Incoming Request
├── CORS & Security Headers (always applied)
├── Request Tracing & Logging (all requests)
├── Route Matching (/api/v1/* vs static files)
├── Authentication Middleware (protected routes)
├── Authorization Middleware (role-based routes)
├── Handler Execution (business logic)
├── Response Formatting (consistent JSON)
└── Error Handling (standardized error responses)
```

**Performance & Monitoring Characteristics:**
- **Total endpoints**: 36 routes with 34 OpenAPI documented endpoints
- **Health checks**: Sub-millisecond response times for basic health
- **Documentation**: Zero-overhead documentation generation at compile time
- **Static serving**: Efficient file serving with proper caching headers
- **Request tracing**: Structured logging for all HTTP requests
- **Error responses**: Consistent JSON format with appropriate HTTP status codes

---

### **Lesson 7: User Management (`starter/src/users/`)**
*"Managing people at scale"*

**Learning Objectives:**
- Master the comprehensive user management system: 4 files, 12 HTTP endpoints
- Understand the three-tier authorization patterns (ownership, hierarchy, cross-user)
- See how Argon2 password hashing and user lifecycle management work
- Grasp the CLI admin tools for direct database operations

**Starter-Specific Materials:**
- `starter/src/users/mod.rs` - User management module structure
- `starter/src/users/models.rs` - User data structures and validation (200+ lines)
- `starter/src/users/services.rs` - User lifecycle management with Argon2 (300+ lines)
- `starter/src/users/api.rs` - 12 user management HTTP endpoints (580+ lines)
- `starter/src/cli/mod.rs` - CLI module structure  
- `starter/src/cli/models.rs` - CLI command definitions
- `starter/src/cli/services.rs` - Admin business logic
- `starter/src/cli/api.rs` - CLI command execution
- `starter/tests/users/` - User management test suite (17 comprehensive tests)
- `docs/guides/12-user-management.md` - User management design

**🎯 Key Insights for Teaching:**

**The 12 User Management Endpoints (`api.rs`):**
Comprehensive lifecycle management with three authorization patterns:

**Self-Management (3 endpoints):**
1. **`GET /users/me/profile`** - Get own profile (ownership-based)
2. **`PUT /users/me/profile`** - Update own profile (ownership-based)  
3. **`PUT /users/me/password`** - Change own password (with verification)
4. **`DELETE /users/me`** - Delete own account (soft delete with confirmation)

**User Administration (5 endpoints):**
5. **`GET /users`** - List all users (Moderator+, paginated)
6. **`POST /users`** - Create new user (Admin only)
7. **`GET /users/{id}`** - Get user by ID (ownership/hierarchy-based)
8. **`PUT /users/{id}/profile`** - Update user profile (Admin only)
9. **`PUT /users/{id}/status`** - Activate/deactivate users (Moderator+)
10. **`PUT /users/{id}/role`** - Change user roles (Admin only)
11. **`POST /users/{id}/reset-password`** - Force password reset (Moderator+)
12. **`DELETE /users/{id}`** - Delete user (Admin only)

**Analytics Endpoint:**
13. **`GET /admin/users/stats`** - Comprehensive user statistics (Admin only)

**User Model Architecture (`models.rs`):**
The `User` struct demonstrates production user management:
- **Password security**: `password_hash` with `#[serde(skip_serializing)]` (line 13)
- **Role integration**: Direct `UserRole` enum integration (line 15)
- **Audit tracking**: `created_at`, `updated_at`, `last_login_at` timestamps
- **Soft deletion**: `is_active` boolean for account deactivation
- **Email verification**: `email_verified` for email confirmation workflows

**UserProfile Separation Pattern:**
The `to_profile()` method (lines 32-43) demonstrates data layer separation:
- **Security**: Excludes sensitive fields (`password_hash`, `updated_at`)
- **API-friendly**: Only includes client-relevant data
- **Consistent**: All endpoints return `UserProfile`, never raw `User`

**Password Security (`services.rs`):**
Argon2 implementation following security best practices:
```rust
let salt = SaltString::generate(&mut OsRng);  // Cryptographic random salt
let argon2 = Argon2::default();               // Standard Argon2 parameters
let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
```

**Validation System (`models.rs` lines 75-90):**
Three-level validation with specific business rules:
- **Email validation**: 3-254 characters, must contain '@'
- **Username validation**: 3-50 characters, alphanumeric + underscore/hyphen
- **Password validation**: 8+ characters, complexity requirements

**Authorization Patterns Demonstrated:**
1. **Ownership-based**: Users can modify their own resources
2. **Hierarchy-based**: Higher roles can access lower roles' data
3. **Cross-user operations**: Admin-only actions that affect other users

**RBAC Integration Examples:**
```rust
// Ownership check for user profile access
rbac_services::can_access_user_profile(&auth_user, id, target_user.role)?;

// Role requirement for user creation
rbac_services::require_admin(&auth_user)?;

// Hierarchical access for status changes
rbac_services::require_moderator_or_higher(&auth_user)?;
```

**CLI Admin Tools (`cli/` module):**
Direct database access for operational tasks:
- **`cargo run -- admin task-stats`** - Task statistics bypassing API
- **`cargo run -- admin list-tasks`** - Show all users' tasks
- **`cargo run -- admin clear-completed`** - Cleanup across all users

**Deep Dive Questions:**
1. **Authorization Question**: How do the three authorization patterns (ownership, hierarchy, cross-user) work together in the user management endpoints?
2. **Security Question**: Why does the `User` struct use `#[serde(skip_serializing)]` on `password_hash`, and how does `to_profile()` ensure security?
3. **Validation Question**: What are the specific validation rules for email, username, and password, and why these limits?
4. **Password Question**: How does the Argon2 implementation provide security through salt generation and parameter selection?
5. **CLI Question**: How do the CLI admin commands bypass API authentication, and what are the trade-offs?

**🔍 Teaching Experiments:**
1. **Authorization Testing**: Create users with different roles and test cross-user operations to see access patterns
2. **Password Security**: Generate multiple password hashes for the same password to observe salt randomization
3. **Validation Edge Cases**: Test boundary conditions for email/username/password validation
4. **Role Hierarchy**: Test moderator access to admin profiles to see protection mechanisms
5. **CLI vs API**: Compare task statistics from CLI admin tools vs API endpoints

**💡 User Management Design Insights:**
- **Security-First Architecture**: Password hashing, data separation, authorization checks
- **Role-Based Operations**: Different capabilities based on user hierarchy
- **Audit-Friendly**: Comprehensive tracking of user lifecycle events  
- **Soft Deletion**: Preserves data for compliance and recovery
- **API Consistency**: All endpoints follow same response patterns
- **Validation Centralization**: Business rules enforced at model level
- **CLI Operations**: Direct database access for administrative tasks

**👤 User Lifecycle Flow:**
```
1. Registration → Validation → Password hashing → User creation
2. Authentication → Session creation → Profile access
3. Profile updates → Validation → Database update → Activity tracking
4. Role changes → Admin authorization → RBAC update → Capability change
5. Account deletion → Confirmation → Soft delete → Session cleanup
```

**Performance & Security Characteristics:**
- **Password hashing**: Argon2 with random salt (computationally expensive by design)
- **Database queries**: Indexed lookups on email, username, and ID
- **Authorization checks**: In-memory role comparisons (fast)
- **Soft deletion**: Preserves referential integrity while hiding accounts
- **Session integration**: Coordinates with auth module for complete lifecycle

---

### **Lesson 8: Testing & Quality (`starter/tests/`)**
*"How we know the system works"*

**Learning Objectives:**
- Master the comprehensive testing approach: 91 integration tests across 14 files
- Understand database isolation with per-test database creation
- See the 9-step quality pipeline and 10-scenario chaos testing framework
- Grasp the three-tier testing strategy (unit, integration, chaos)

**Starter-Specific Materials:**
- `starter/tests/` - Integration test suite (14 files, 3,994 lines of test code)
- `starter/tests/helpers/` - Test infrastructure (test_app, db isolation, utilities)
- `starter/tests/auth/` - Authentication tests (edge cases included)
- `starter/tests/tasks/` - Task system tests
- `starter/tests/users/` - User management tests  
- `starter/tests/cli/` - CLI functionality tests
- `scripts/check.sh` - 9-step quality pipeline (comprehensive validation)
- `scripts/test-chaos.sh` - Chaos testing framework (10 scenarios, Docker-based)
- `scripts/test-with-curl.sh` - API endpoint testing (44+ endpoint validation)
- `docs/guides/08-testing.md` - Testing philosophy
- `docs/guides/09-chaos-testing.md` - Chaos testing framework

**🎯 Key Insights for Teaching:**

**Comprehensive Test Coverage:**
- **91 integration tests** across all system components
- **14 test files** organized by domain (auth, tasks, users, cli, health, middleware)
- **3,994 lines** of test code ensuring system reliability
- **Real HTTP testing** with spawned test servers on random ports

**Database Isolation Architecture (`helpers/test_app.rs`):**
The testing infrastructure demonstrates production-grade test isolation:
```rust
// Each test gets its own isolated database
let test_db = create_test_db().await.expect("Failed to create test database");
config.database.database = test_db.name.clone();
let database = Database { pool: test_db.pool.clone() };
```
- **Per-test databases**: Each test creates a fresh PostgreSQL database
- **Automatic cleanup**: Test databases are dropped after test completion
- **Real database operations**: Tests use actual PostgreSQL, not mocks
- **Migration validation**: Each test runs the full migration stack

**TestApp Architecture (`helpers/test_app.rs` lines 8-14):**
Production-like test environment with:
- **Real HTTP server**: Spawned on random port for each test
- **Complete application state**: Database, config, middleware stack
- **HTTP client**: Configured reqwest client for API testing
- **Authentication helpers**: Token management for protected endpoints

**Nine-Step Quality Pipeline (`scripts/check.sh`):**
Comprehensive validation before any commit:
1. **Web frontend build** - Early frontend validation
2. **Cargo check** - Compilation verification with SQLx offline mode
3. **Cargo fmt** - Code formatting validation
4. **Cargo clippy** - Linting and best practices
5. **SQLx prepare** - Query cache generation for offline compilation
6. **Unit tests** - Library-level testing
7. **Integration tests** - Full system testing with real database
8. **API endpoint testing** - Live API validation
9. **Static file serving** - Production deployment validation

**Chaos Testing Framework (`scripts/test-chaos.sh`):**
Ten distinct failure scenarios testing system resilience:
1. **Baseline** - Normal operation validation
2. **Database failure** - PostgreSQL connectivity loss
3. **Server restart** - HTTP server recovery
4. **Worker restart** - Background task processor recovery
5. **Task flood** - High-load performance testing
6. **Circuit breaker** - Failure cascade prevention
7. **Mixed chaos** - Multiple simultaneous failures
8. **Recovery testing** - System restoration verification
9. **Multi-worker chaos** - Docker-based worker scaling/failure
10. **Dynamic scaling** - Elastic worker capacity testing

**API Testing Coverage (`scripts/test-with-curl.sh`):**
Comprehensive endpoint validation:
- **44+ endpoint tests** covering all documented APIs
- **Authentication flow testing** (register → login → protected operations)
- **RBAC validation** (role-based access control enforcement)
- **Error response validation** (proper HTTP status codes)
- **Task type registration** (worker-server communication)
- **Dead letter queue testing** (failed task management)

**Test Infrastructure Helpers (`tests/helpers/`):**
Production-grade test utilities:
- **`test_app.rs`** - Complete application spawning with isolation
- **`test_data.rs`** - Test data factories for users, tasks, and scenarios
- **`db.rs`** - Database isolation and cleanup utilities
- **`utils.rs`** - HTTP testing helpers and assertion utilities

**Integration Test Organization:**
```
tests/
├── auth/          # Authentication & session management
├── users/         # User management & RBAC
├── tasks/         # Background job processing
├── cli/           # Command-line interface
├── health/        # Health check endpoints
├── middleware/    # Auth & RBAC middleware
├── api/           # General API functionality
└── helpers/       # Test infrastructure
```

**Deep Dive Questions:**
1. **Architecture Question**: How does the per-test database isolation prevent test interference while maintaining test speed?
2. **Quality Question**: What are the 9 steps in the quality pipeline and why is each step necessary before commits?
3. **Chaos Question**: How do the 10 chaos testing scenarios simulate real-world production failures?
4. **Coverage Question**: How do the 91 integration tests ensure comprehensive system validation beyond unit tests?
5. **Infrastructure Question**: How does the TestApp architecture provide realistic testing environments?

**🔍 Teaching Experiments:**
1. **Test Isolation**: Run multiple tests simultaneously and verify database isolation
2. **Quality Pipeline**: Break each step of `check.sh` and see what failures it catches
3. **Chaos Testing**: Run chaos scenarios against a live system and observe failure modes
4. **API Coverage**: Add a new endpoint and see how test-with-curl.sh validates it
5. **Database Migration**: Test with missing migrations and see test failure patterns

**💡 Testing Strategy Insights:**
- **Three-Tier Testing**: Unit tests → Integration tests → Chaos tests
- **Real Infrastructure**: Uses actual PostgreSQL, not mocks or in-memory databases
- **Production Simulation**: Test environment matches production deployment
- **Comprehensive Coverage**: Every endpoint, middleware, and failure mode tested
- **Automated Quality Gates**: Cannot commit without passing all quality checks
- **Resilience Validation**: Chaos testing ensures system can handle real-world failures
- **Continuous Integration Ready**: All tests designed for CI/CD pipelines

**🧪 Testing Architecture Flow:**
```
Quality Pipeline:
1. Frontend Build → 2. Compilation → 3. Formatting → 4. Linting
         ↓
5. SQLx Cache → 6. Unit Tests → 7. Integration Tests
         ↓
8. API Testing → 9. Static Serving → ✅ Commit Ready

Chaos Testing:
Docker Compose → System Spawn → Failure Injection → Recovery Validation
```

**Performance & Reliability Characteristics:**
- **Test execution time**: ~40-45 seconds for complete quality pipeline
- **Database isolation**: Each test gets fresh PostgreSQL instance
- **Chaos testing duration**: 5-15 minutes depending on scenario difficulty
- **API test coverage**: 44+ endpoints validated with real HTTP calls
- **Integration test count**: 91 tests covering all system components
- **Code coverage**: 3,994 lines of test code ensuring system reliability

---

## 🌐 PHASE 2: FRONTEND INTEGRATION (Lessons 10-14)

### 📊 **Lesson 9: Monitoring & Observability (`starter/src/monitoring/`)**
*"If you can't measure it, you can't manage it"*

**Learning Objectives:**
- Master the comprehensive monitoring system with 14 API endpoints, 4 database tables, and real-time observability patterns
- Understand production-grade monitoring with events, metrics, alerts, and incidents
- See RBAC integration with 3-tier permissions (User → Moderator → Admin)
- Grasp Prometheus metrics export and timeline reconstruction

**Monitoring-Specific Materials:**
- `starter/src/monitoring/` - Complete monitoring module (5 files, ~2,000 lines)
- `starter/migrations/006_monitoring.up.sql` - 4-table schema with PostgreSQL enums
- `docs/guides/15-monitoring-and-observability.md` - 891-line implementation guide
- `starter/tests/monitoring/` - Comprehensive test suite
- `docs/monitoring.md` - API reference and integration patterns

**🎯 Teaching Goals:**
Guide students through implementing a comprehensive monitoring system that demonstrates industry-standard observability patterns.

**🔍 Core Concepts to Teach:**
- [ ] **4-Table Schema**: events, metrics, alerts, incidents with JSONB and PostgreSQL enums
- [ ] **14 API Endpoints**: Event collection, metrics submission, alert management, incident tracking
- [ ] **RBAC Integration**: 3-tier permissions with role-based access control
- [ ] **Prometheus Export**: Time-series metrics in industry-standard format
- [ ] **Timeline Reconstruction**: Automated incident analysis with event correlation

**🧪 Hands-On Teaching Activities:**

1. **30-Second Setup Demo**:
   ```bash
   # Show how monitoring is included by default
   ./scripts/dev-server.sh
   
   # Create first monitoring event
   curl -X POST http://localhost:3000/api/v1/monitoring/events \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $TOKEN" \
     -d '{"event_type": "log", "source": "demo", "message": "First event!"}'
   ```

2. **Database Schema Exploration**:
   - Walk through migration 006 and explain PostgreSQL enums
   - Show relationship between events, metrics, alerts, and incidents
   - Demonstrate JSONB flexibility for tags and labels

3. **API Endpoint Tour**:
   - Events: Create, query, retrieve with filtering
   - Metrics: Submit, query, Prometheus export
   - Alerts: Rule creation (moderator+ required)
   - Incidents: Lifecycle management with RBAC
   - Statistics: System health overview

4. **Real-Time Monitoring**:
   - Create events from different sources
   - Submit metrics with labels and timestamps
   - Trigger alerts and create incidents
   - Export timeline for incident analysis

**🎭 Student Discovery Moments:**

*"Wait, this exports Prometheus metrics automatically?"*
- Show `/api/v1/monitoring/metrics/prometheus` endpoint
- Explain industry-standard observability patterns
- Demonstrate how metrics become dashboards

*"The timeline rebuilds incidents automatically?"*
- Create incident, add related events
- Show timeline reconstruction with correlation
- Explain root cause analysis patterns

*"RBAC controls who can create alerts?"*
- Test alert creation with different user roles
- Show moderator+ requirement for system operations
- Demonstrate ownership-based incident updates

**🔧 Implementation Patterns to Emphasize:**

```rust
// Show clean monitoring integration patterns
use starter::monitoring::services;

// Log application events
let event = services::create_event(&mut conn, CreateEventRequest {
    event_type: "log".to_string(),
    source: "user-service".to_string(),
    message: Some("User action completed".to_string()),
    level: Some("info".to_string()),
    tags: HashMap::from([
        ("user_id".to_string(), json!(user.id)),
        ("action".to_string(), json!("profile_update"))
    ]),
    payload: HashMap::new(),
    timestamp: None,
}).await?;

// Track performance metrics
let metric = services::create_metric(&mut conn, CreateMetricRequest {
    name: "response_time_ms".to_string(),
    metric_type: MetricType::Histogram,
    value: duration.as_millis() as f64,
    labels: HashMap::from([
        ("endpoint".to_string(), "/api/v1/users".to_string()),
        ("method".to_string(), "PUT".to_string())
    ]),
    timestamp: None,
}).await?;
```

**📊 Teaching the 4-Week Progressive Implementation:**

**Week 1: Foundation**
- Database schema and migrations
- Basic event and metric collection
- Understanding monitoring data model

**Week 2: API Development**
- Implementing CRUD endpoints
- Adding RBAC protection
- Request/response validation

**Week 3: Advanced Features**
- Alert rule management
- Incident lifecycle tracking
- Timeline reconstruction

**Week 4: Production Integration**
- Prometheus metrics export
- Performance optimization
- Real-world monitoring patterns

**✅ Student Success Criteria:**
- [ ] Can explain all 4 monitoring database tables and their relationships
- [ ] Understands the 14 API endpoints and their RBAC requirements
- [ ] Can create events, metrics, alerts, and incidents programmatically
- [ ] Knows how to export metrics for external monitoring systems
- [ ] Can correlate events into incident timelines for analysis

**🎓 Advanced Extensions:**
- Integrate monitoring into existing task handlers
- Build custom alert rules for business metrics
- Create monitoring dashboards using exported data
- Implement automated incident response workflows

**📖 Required Reading:**
- `docs/guides/15-monitoring-and-observability.md` - Complete implementation guide
- `docs/monitoring.md` - API reference and integration patterns
- Study existing monitoring tests for usage patterns

**🔗 Connects To:**
- **Previous Lessons**: Authentication (RBAC), Tasks (integration), API Layer (endpoints)
- **Next Lessons**: React Frontend (dashboard integration), Admin Dashboard (real monitoring data)

---

### **Lesson 10: React Frontend Overview (`web/src/`)**
*"Now that we know the server, let's meet the client"*

**Learning Objectives:**
- Master the complete React 18 frontend: 89 TypeScript files, 17,548 lines of code
- Understand TanStack Router file-based routing with auto-generated route tree
- See Vite build configuration with API proxy and Tailwind CSS 4 integration  
- Grasp the production-ready project structure and development workflow

**Frontend-Specific Materials:**
- `web/src/main.tsx` - React 18 application bootstrap (49 lines)
- `web/src/routeTree.gen.ts` - Auto-generated route tree from file structure
- `web/src/integrations/tanstack-query/root-provider.tsx` - Query client setup (16 lines)
- `web/vite.config.ts` - Build configuration with proxy and plugins (44 lines)
- `web/src/components/ui/` - shadcn/ui component library (38 components)
- `web/src/types/api.ts` - Auto-generated OpenAPI types (2,515 lines!)
- `web/package.json` - Modern dependency stack (84 lines)
- `web/playwright.config.ts` - E2E testing configuration

**🎯 Key Insights for Teaching:**

**React 18 Application Bootstrap (`main.tsx`):**
The entry point showcases professional React setup with modern patterns:
```typescript
// TanStack Router with production configuration
const router = createRouter({
  routeTree,                      // Auto-generated from file structure
  context: { ...TanStackQueryProvider.getContext() },
  defaultPreload: "intent",       // Preload on hover/focus for performance
  scrollRestoration: true,        // Preserve scroll position on navigation
  defaultStructuralSharing: true, // Optimize re-renders
  defaultPreloadStaleTime: 0,     // Fresh data on preload
});

// Type-safe router registration
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
```

**File-Based Routing System:**
The `routeTree.gen.ts` demonstrates automatic route generation:
- **13 auto-generated routes** from file structure
- **Type-safe navigation** with full TypeScript support
- **Nested layouts** with proper parent-child relationships
- **Route code splitting** enabled by TanStack Router plugin

**Vite Build Configuration (`vite.config.ts`):**
Production-ready build setup with modern tooling:
```typescript  
export default defineConfig({
  server: {
    cors: false,
    proxy: {
      '/api/v1': {                    // Proxy API calls to backend
        target: 'http://localhost:3000',
        changeOrigin: true,
        secure: false
      }
    }
  },
  plugins: [
    TanStackRouterVite({ autoCodeSplitting: true }),  // File-based routing
    viteReact(),                                      // React support
    tailwindcss(),                                    // Tailwind CSS 4
  ],
  resolve: {
    alias: {
      '@': resolve(..., 'src'),     // Clean import paths
    },
  },
});
```

**TanStack Query Integration:**
Simple but powerful server state management setup:
```typescript
// root-provider.tsx - Centralized query client
const queryClient = new QueryClient();

export function getContext() {
  return { queryClient };
}

export function Provider({ children }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}
```

**shadcn/ui Component Library:**
38 production-ready UI components built on Radix UI primitives:
- **Accessible by default** - ARIA attributes, keyboard navigation
- **Customizable styling** - Tailwind CSS with CSS variables
- **TypeScript support** - Full type safety for props and variants
- **Consistent design system** - Unified look and feel across the app

**Project Architecture Overview:**
```
web/src/
├── components/
│   ├── ui/              # 38 shadcn/ui components (buttons, forms, etc.)
│   ├── auth/            # Authentication forms and guards
│   ├── layout/          # Admin layout, headers, sidebars  
│   └── admin/           # Dashboard widgets and analytics
├── routes/              # File-based routing structure
│   ├── __root.tsx       # Root layout with providers
│   ├── index.tsx        # Home page
│   ├── auth/            # Authentication routes
│   └── admin/           # Protected admin routes
├── lib/                 # Utilities and shared logic
├── hooks/               # Custom React hooks
├── types/               # TypeScript definitions
└── integrations/        # Third-party service integrations
```

**Development Workflow Features:**
- **Hot Module Replacement** - Instant updates without full page refresh
- **API Proxy** - Seamless backend integration during development  
- **Type Generation** - Automatic TypeScript types from backend OpenAPI
- **Code Splitting** - Automatic route-based bundles for optimal loading
- **Linting & Formatting** - Biome for consistent code quality

**Route Structure Analysis:**
The auto-generated `routeTree.gen.ts` shows 13 distinct routes:
- **Public routes**: `/`, `/auth/login`, `/auth/register`
- **Admin routes**: `/admin/*` with nested layouts
- **Demo routes**: `/demo/tanstack-query` for development
- **Dynamic routes**: `/admin/tasks/$taskId`, `/admin/users/$userId`

**Type Safety Implementation:**
Complete type safety chain from backend to frontend:
1. **Backend OpenAPI generation** - Rust derives OpenAPI schema
2. **Type generation** - `pnpm run generate-api` creates TypeScript types
3. **API client integration** - Fully typed HTTP calls
4. **Component prop validation** - TypeScript ensures correct usage

**Deep Dive Questions:**
1. **Bootstrap Question**: How does the `main.tsx` setup demonstrate React 18 best practices with StrictMode and provider composition?
2. **Routing Question**: How does TanStack Router's file-based system generate the route tree and enable type-safe navigation?
3. **Build Question**: How does the Vite configuration integrate multiple tools (React, TanStack Router, Tailwind) into a cohesive build process?
4. **Integration Question**: How does the API proxy in Vite enable seamless frontend-backend development?
5. **Architecture Question**: How do the 38 shadcn/ui components provide a consistent design system while maintaining flexibility?

**🔍 Teaching Experiments:**
1. **Route Generation**: Add a new route file and observe the automatic route tree regeneration
2. **Component Library**: Explore different shadcn/ui components and their prop variations
3. **Build Process**: Run `pnpm dev` vs `pnpm build` to see development vs production modes
4. **API Integration**: Test the `/api/v1` proxy by making requests from the frontend
5. **Type Safety**: Modify the backend API and regenerate types to see the integration

**💡 Frontend Architecture Insights:**
- **Modern React Patterns**: Uses React 18 features with proper provider composition
- **File-Based Organization**: Routes and components follow intuitive file structure
- **Build-Time Optimization**: Automatic code splitting and route-based bundles
- **Developer Experience**: Hot reloading, TypeScript intellisense, integrated tooling
- **Production Ready**: Optimized builds with proper asset handling and caching
- **Type-First Development**: End-to-end type safety from API to UI

**🌐 Development Workflow:**
```
File Change Detection (Vite)
        ↓
Hot Module Replacement
        ↓
TypeScript Compilation
        ↓
Route Tree Regeneration (TanStack Router)
        ↓
Component Re-render (React)
        ↓
API Calls → Proxy → Backend (Dev Mode)
```

**Build & Performance Characteristics:**
- **Development server**: Sub-second startup with instant HMR
- **Production build**: Optimized bundles with tree shaking and minification
- **Route code splitting**: Automatic lazy loading for optimal performance
- **Asset optimization**: Image optimization and CSS extraction
- **Bundle analysis**: Clear separation of vendor, route, and component chunks
- **Type checking**: Fast TypeScript compilation with incremental builds

---

### **Lesson 11: Authentication Frontend (`web/src/components/auth/`)**
*"How users log in through the browser"*

**Learning Objectives:**
- Master the authentication components: LoginForm (145 lines), RegisterForm (227 lines), RoleGuard (172 lines)
- Understand sophisticated RBAC system with component-level and route-level protection
- See production-grade form validation with Zod schemas and React Hook Form
- Grasp the smart token refresh system and context-based state management

**Frontend Auth Materials:**
- `web/src/components/auth/LoginForm.tsx` - Login component with smart redirect (145 lines)
- `web/src/components/auth/RegisterForm.tsx` - Registration with password confirmation (227 lines)
- `web/src/components/auth/RoleGuard.tsx` - Comprehensive RBAC component (172 lines)
- `web/src/lib/auth/context.tsx` - Authentication context with token refresh (281 lines)
- `web/src/lib/rbac/types.ts` - RBAC utilities and type definitions (220 lines)
- Connection to backend: `starter/src/auth/api.rs` (6 auth endpoints)

**🎯 Key Insights for Teaching:**

**LoginForm Production Implementation (`LoginForm.tsx`):**
Sophisticated form with comprehensive UX patterns:
```typescript
const loginSchema = z.object({
  email: z.string().email("Please enter a valid email address"),
  password: z.string().min(1, "Password is required"),
});

// Smart redirect after successful login
const search = useSearch({ from: "/auth/login" });
const redirectTo = (search as Record<string, string>)?.redirect || "/admin";
navigate({ to: redirectTo });
```

**RegisterForm Advanced Validation (`RegisterForm.tsx`):**
Multi-layer validation with Zod schema composition:
```typescript
const registerSchema = z.object({
  username: z.string()
    .min(3, "Username must be at least 3 characters")
    .max(50, "Username must be at most 50 characters")
    .regex(/^[a-zA-Z0-9_-]+$/, "Username can only contain letters, numbers, hyphens, and underscores"),
  email: z.string()
    .email("Please enter a valid email address")
    .max(254, "Email must be at most 254 characters"),
  password: z.string()
    .min(8, "Password must be at least 8 characters")
    .max(128, "Password must be at most 128 characters"),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],  // Target the confirmation field for error
});
```

**RoleGuard Component Architecture (`RoleGuard.tsx`):**
Comprehensive RBAC system with multiple protection patterns:

**Four Protection Modes:**
1. **Role-based**: `<RoleGuard requiredRole="admin">` - Simple role requirement
2. **Resource-based**: `<RoleGuard resource="users" permission="manage">` - Granular permissions
3. **User-specific**: `<RoleGuard targetUserId={userId}>` - Access to specific user data
4. **Custom logic**: `<RoleGuard customCheck={(user) => user.id === ownerId}>` - Custom validation

```typescript
// Advanced RoleGuard with multiple fallback strategies
export function RoleGuard({
  children,
  requiredRole,
  resource,
  permission,
  targetUserId,
  customCheck,
  fallback = null,
  loading = null,
}: RoleGuardProps) {
  const { user, authenticated, loading: authLoading } = useAuth();

  // Progressive permission checking
  if (customCheck) {
    hasAccess = customCheck(user);
  } else if (requiredRole) {
    hasAccess = hasRoleOrHigher(user.role, requiredRole);
  } else if (resource && permission) {
    hasAccess = canAccessResource(user, resource, permission, targetUserId);
  }
}
```

**RBAC Type System (`rbac/types.ts`):**
Production-grade role hierarchy with numerical levels:
```typescript
// Role hierarchy enables >= comparisons
export const ROLE_HIERARCHY: Record<UserRole, number> = {
  user: 1,
  moderator: 2,
  admin: 3,
} as const;

// Complex resource-based permission matrix
export function canAccessResource(
  user: AuthUser,
  resource: Resource,
  permission: Permission,
  targetUserId?: string,
): boolean {
  // Admin bypass - can do everything
  if (isAdmin(user)) return true;
  
  switch (resource) {
    case "users":
      if (permission === "read") {
        // Moderator+ can read all, users can read own
        return isModeratorOrHigher(user) || 
               Boolean(targetUserId && user.id === targetUserId);
      }
      // Additional permission logic...
  }
}
```

**Smart Authentication Context (`auth/context.tsx`):**
Advanced token management with automatic refresh scheduling:

**Token Refresh Algorithm:**
```typescript
// Smart refresh scheduling - prevents session interruptions
const refreshTime = Math.max(
  timeUntilExpiration * 0.25,  // Refresh at 75% of token lifetime
  5 * 60 * 1000,               // But at least 5 minutes before expiry
);

const timeoutId = setTimeout(async () => {
  const success = await refreshToken();
  if (!success) {
    console.log("Token refresh failed, user will be logged out");
  }
}, refreshTime);
```

**Context Provider Patterns:**
The auth context demonstrates production-grade state management:
- **Persistent storage** - Token survives browser refresh via localStorage
- **Automatic rehydration** - Context state restored on app start
- **Multiple logout options** - Single session vs all devices (`logoutAll()`)
- **Error resilience** - Network failures handled gracefully
- **Activity tracking** - Every API call updates token expiration

**Form Validation & UX Patterns:**

**Real-Time Validation:**
Both forms use React Hook Form with Zod resolver for performance:
```typescript
const form = useForm<LoginFormData>({
  resolver: zodResolver(loginSchema),
  defaultValues: { email: "", password: "" },
});

// Loading states prevent double submission
const [isLoading, setIsLoading] = useState(false);
<Button type="submit" disabled={isLoading}>
  {isLoading ? "Signing In..." : "Sign In"}
</Button>
```

**Registration Success Flow:**
```typescript
// Success state with automatic redirect
if (success) {
  return (
    <Alert>
      <AlertDescription>
        Registration successful! Redirecting to login page...
      </AlertDescription>
    </Alert>
  );
}

// Auto-redirect after 2 seconds
setTimeout(() => {
  navigate({ to: "/auth/login" });
}, 2000);
```

**RBAC Integration Patterns:**

**Component-Level Protection:**
```typescript
// Fine-grained component access control
<RoleGuard resource="users" permission="manage">
  <UserManagementTools />
</RoleGuard>

// User-specific resource access
<RoleGuard resource="users" permission="read" targetUserId={userId}>
  <UserProfile />
</RoleGuard>
```

**Imperative Permission Checking:**
```typescript
// usePermissions hook for complex logic
const { checkRole, checkResource, isAdmin } = usePermissions();

if (checkResource("tasks", "delete", taskId)) {
  // Show delete button
}
```

**Deep Dive Questions:**
1. **Architecture Question**: How does the RoleGuard component implement four different protection patterns with a single interface?
2. **Validation Question**: How do the Zod schemas provide both compile-time TypeScript safety and runtime validation?
3. **Token Question**: How does the smart refresh algorithm balance security (fresh tokens) with UX (no interruptions)?
4. **RBAC Question**: How does the numerical role hierarchy enable complex permission matrices with simple comparisons?
5. **Context Question**: How does the auth context coordinate token storage, refresh scheduling, and component re-renders?

**🔍 Teaching Experiments:**
1. **RoleGuard Testing**: Create components with different RBAC requirements and test with various user roles
2. **Form Validation**: Test edge cases in registration form to see Zod validation in action
3. **Token Lifecycle**: Use browser dev tools to observe localStorage, token refresh timing, and cleanup
4. **Permission Matrix**: Test the complex resource-based permissions with different user scenarios
5. **Registration Flow**: Complete the full registration → login → redirect flow to understand UX

**💡 Authentication Architecture Insights:**
- **Multi-Modal Protection**: RoleGuard supports role, resource, user-specific, and custom protection
- **Production UX**: Loading states, error handling, success feedback, smart redirects
- **Type-Safe RBAC**: Complete TypeScript integration from backend UserRole to frontend components
- **Smart Token Management**: Automatic refresh prevents session interruptions while maintaining security
- **Performance Optimized**: React Hook Form reduces re-renders, context updates efficiently
- **Accessibility Ready**: Proper ARIA labels, error associations, keyboard navigation

**🔐 RBAC Permission Matrix:**
```
Resource: Users
├── Read: Moderator+ (all) | User (own only)
├── Write: Admin (all) | User (own only)  
└── Delete: Admin (all) | User (own only)

Resource: Tasks
├── Read: Moderator+ (all) | User (own only)
├── Write: All authenticated users
└── Manage: Moderator+ (all) | User (own only)
```

**Authentication Flow Diagram:**
```
Form Submission
      ↓
Zod Schema Validation (Client)
      ↓
React Hook Form Handling
      ↓
API Client Call (Typed)
      ↓
Backend Authentication
      ↓
Token + User Response
      ↓
Context State Update
      ↓
localStorage Persistence
      ↓
Smart Refresh Scheduling
      ↓
Protected Component Access
```

**Performance & Security Characteristics:**
- **Bundle size**: Efficient tree-shaking with Zod and React Hook Form
- **Validation performance**: Client-side validation prevents unnecessary API calls
- **Token security**: 75% lifetime refresh strategy with 5-minute minimum buffer
- **Memory efficiency**: Context uses structural sharing to minimize re-renders
- **Error handling**: Comprehensive error states with user-friendly messages

---

### **Lesson 12: Admin Dashboard (`web/src/components/admin/`)**
*"Building production monitoring dashboards"*

**Learning Objectives:**
- Master the comprehensive admin dashboard: 10 custom components, 3,267 lines of monitoring code
- Understand real-time system monitoring with TanStack Query auto-refresh
- See production-grade data visualization with Recharts integration
- Grasp the AdminLayout pattern with ProtectedRoute and sidebar navigation

**Admin Dashboard Materials:**
- `web/src/components/admin/` - 10 custom admin components (3,267 lines total)
- `web/src/routes/admin/index.tsx` - Main dashboard route (351 lines)
- `web/src/components/layout/AdminLayout.tsx` - Layout with auth protection (26 lines)
- `web/src/components/admin/SystemMetrics.tsx` - Real-time metrics (337 lines)
- `web/src/components/admin/TaskAnalytics.tsx` - Task performance charts (291 lines)
- `web/src/components/admin/UserActivityAnalytics.tsx` - User analytics (414 lines)
- `web/src/components/admin/HealthStatusCards.tsx` - Health monitoring (275 lines)
- `web/src/components/admin/DependencyMonitor.tsx` - System dependency tracking (382 lines)

**🎯 Key Insights for Teaching:**

**Admin Dashboard Route (`admin/index.tsx`):**
The main dashboard demonstrates production monitoring patterns:
```typescript
function AdminDashboard() {
  // Real-time data fetching with different intervals
  const { data: taskStats, isLoading: isLoadingStats } = useTaskStats(10000);
  const { data: healthStatus } = useHealthBasic(15000);
  const { data: currentUser } = useCurrentUser(30000);

  // Generate trend data for visualization
  const trendData = useMemo(() => {
    const data = [];
    for (let i = 7; i >= 0; i--) {
      const completed = Math.floor(Math.random() * 20) + (taskStats?.completed || 0) * 0.1;
      const failed = Math.floor(Math.random() * 5) + (taskStats?.failed || 0) * 0.1;
      data.push({ day: i, completed, failed, total: completed + failed });
    }
    return data;
  }, [taskStats]);
}
```

**StatsCard Component (`StatsCard.tsx`):**
Reusable metric display component with trend indicators:
```typescript
interface StatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon: LucideIcon;
  trend?: {
    value: number;
    isPositive: boolean;
  };
}

// Usage in dashboard
<StatsCard
  title="Success Rate"
  value={`${taskStats?.total ? Math.round(((taskStats.completed || 0) / taskStats.total) * 100) : 0}%`}
  description="Task completion rate"
  icon={TrendingUp}
  trend={{ value: 2.5, isPositive: true }}
/>
```

**TaskAnalytics Component (`TaskAnalytics.tsx`):**
Advanced analytics with multiple chart types:
```typescript
export function TaskAnalytics() {
  const taskStatsQuery = useQuery({
    queryKey: ["tasks", "stats"],
    queryFn: () => apiClient.getTaskStats(),
    refetchInterval: 10000, // Real-time updates every 10 seconds
  });

  // Status distribution pie chart data
  const statusChartData = useMemo(() => {
    if (!stats) return [];
    return [
      { name: "Completed", value: stats.completed, color: "#10B981" },
      { name: "Pending", value: stats.pending, color: "#F59E0B" },
      { name: "Running", value: stats.running, color: "#3B82F6" },
      { name: "Failed", value: stats.failed, color: "#EF4444" },
      { name: "Cancelled", value: stats.cancelled, color: "#6B7280" },
      { name: "Retrying", value: stats.retrying, color: "#8B5CF6" },
    ].filter((item) => item.value > 0);
  }, [stats]);
}
```

**SystemMetrics Component (`SystemMetrics.tsx`):**
Real-time system monitoring with health indicators:
```typescript
export function SystemMetrics() {
  const basicHealthQuery = useQuery({
    queryKey: ["health", "basic"],
    queryFn: () => apiClient.getHealth(),
    refetchInterval: 30000,
  });

  const getHealthScore = () => {
    if (!detailedHealthQuery.data?.data?.checks) return 0;
    const checks = Object.values(detailedHealthQuery.data.data.checks);
    const healthyChecks = checks.filter(
      (check: ComponentHealth) => check.status === "healthy"
    ).length;
    return Math.round((healthyChecks / checks.length) * 100);
  };
}
```

**RecentActivity Component (`RecentActivity.tsx`):**
Activity feed with mock data and proper typing:
```typescript
type ActivityType = "task_completed" | "task_failed" | "user_login" | "system_event";

interface Activity {
  id: string;
  type: ActivityType;
  title: string;
  description: string;
  timestamp: string;
  user?: string;
}

const activityConfig = {
  task_completed: {
    icon: CheckCircle,
    color: "text-green-600",
    badge: "default" as const,
  },
  // ... other activity types
};
```

**UserActivityAnalytics Component (`UserActivityAnalytics.tsx`):**
Advanced user analytics with time-series data:
```typescript
export function UserActivityAnalytics() {
  // Generate mock user activity data for demonstration
  const userActivityData: UserActivityData[] = useMemo(() => {
    const now = new Date();
    const data: UserActivityData[] = [];

    for (let i = 6; i >= 0; i--) {
      const date = new Date(now.getTime() - i * 24 * 60 * 60 * 1000);
      const dayName = date.toLocaleDateString("en-US", { weekday: "short" });

      data.push({
        timeframe: dayName,
        active_users: Math.floor(Math.random() * 50) + 20,
        new_registrations: Math.floor(Math.random() * 10) + 1,
        login_attempts: Math.floor(Math.random() * 100) + 50,
        task_creation: Math.floor(Math.random() * 30) + 10,
      });
    }
    return data;
  }, []);
}
```

**AdminLayout Pattern (`AdminLayout.tsx`):**
Clean layout with authentication and navigation:
```typescript
export function AdminLayout({ children }: AdminLayoutProps) {
  return (
    <ProtectedRoute>
      <SidebarProvider>
        <AdminSidebar />
        <SidebarInset>
          <AdminHeader />
          <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
            <main className="flex-1">{children}</main>
          </div>
        </SidebarInset>
      </SidebarProvider>
    </ProtectedRoute>
  );
}
```

**DependencyMonitor Component (`DependencyMonitor.tsx`):**
Comprehensive system dependency tracking:
```typescript
export const DependencyMonitor = memo(function DependencyMonitor() {
  const detailedHealthQuery = useQuery({
    queryKey: ["health", "detailed"],
    queryFn: () => apiClient.getDetailedHealth(),
    refetchInterval: 15000,
  });

  // Type guards for API responses with unknown data
  const isProbeResponse = (data: unknown): data is ProbeResponse => {
    return (
      typeof data === "object" &&
      data !== null &&
      typeof (data as ProbeResponse).status === "string"
    );
  };
}
```

**Real-Time Data Patterns:**
All admin components use consistent auto-refresh intervals:
- **High-frequency**: 10-15 seconds for critical metrics (task stats, health probes)
- **Medium-frequency**: 30 seconds for general health data
- **Low-frequency**: 60+ seconds for user analytics and trends

**Data Visualization Integration:**
Components use Recharts for sophisticated visualizations:
- **Area Charts**: Task trends over time
- **Pie Charts**: Task status distribution  
- **Bar Charts**: Performance metrics
- **Progress Bars**: Success rates and health scores
- **Custom Components**: Real-time status badges and indicators

**Component Architecture Patterns:**

**Memoization for Performance:**
```typescript
export const HealthStatusCards = memo(function HealthStatusCards() {
  // Heavy computation memoized to prevent unnecessary re-renders
});
```

**Loading States and Skeletons:**
```typescript
if (taskStatsQuery.isLoading) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
      <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
    </div>
  );
}
```

**Deep Dive Questions:**
1. **Architecture Question**: How do the 10 admin components work together to create a cohesive monitoring dashboard?
2. **Real-time Question**: How do the different auto-refresh intervals (10s, 15s, 30s) optimize performance while maintaining data freshness?
3. **Visualization Question**: How does the integration with Recharts enable sophisticated data visualization with minimal custom code?
4. **Layout Question**: How does the AdminLayout pattern combine authentication, navigation, and responsive design?
5. **Performance Question**: How do memoization, loading states, and efficient data fetching create a smooth user experience?

**🔍 Teaching Experiments:**
1. **Component Integration**: Build a new admin component following the established patterns
2. **Real-time Updates**: Modify refresh intervals and observe the impact on data freshness vs performance
3. **Chart Customization**: Add new chart types to TaskAnalytics using Recharts components
4. **Layout Responsive**: Test the AdminLayout responsiveness across different screen sizes
5. **Mock Data**: Replace mock data with real API integration to see the data flow

**💡 Admin Dashboard Insights:**
- **Production Monitoring**: Real-time system health, task performance, and user analytics
- **Component Reusability**: StatsCard and other primitives used across multiple dashboards
- **Type Safety**: Complete TypeScript integration with API response types
- **Performance Optimized**: Memoization, efficient queries, and skeleton loading states
- **Responsive Design**: Mobile-friendly layouts with collapsible sidebar navigation
- **Extensible Architecture**: Easy to add new monitoring components and visualizations

**📊 Dashboard Component Breakdown:**
```
Admin Components (3,267 lines):
├── TaskAnalytics.tsx (291 lines) - Performance charts and metrics
├── SystemMetrics.tsx (337 lines) - Real-time system monitoring  
├── UserActivityAnalytics.tsx (414 lines) - User behavior analytics
├── DependencyMonitor.tsx (382 lines) - System dependency tracking
├── HealthStatusCards.tsx (275 lines) - Health probe monitoring
├── HealthTrends.tsx (413 lines) - Historical health trending
├── RealTimeNotifications.tsx (448 lines) - Live notification system
├── RecentActivity.tsx (138 lines) - Activity feed with mock data
├── StatsCard.tsx (48 lines) - Reusable metric display component
└── HealthIndicator.tsx (59 lines) - Simple status indicator
```

**Real-Time Monitoring Features:**
- **System Health**: CPU, memory, disk usage with trend indicators
- **Task Performance**: Success rates, throughput, failure analysis
- **User Analytics**: Active users, registrations, login patterns
- **Dependency Tracking**: Database, external services, network connectivity
- **Activity Feeds**: Recent system events with real-time updates
- **Alert System**: Health threshold monitoring with visual indicators

**Performance & User Experience:**
- **Efficient Updates**: Staggered refresh intervals prevent API overload
- **Visual Feedback**: Loading skeletons, progress bars, status indicators
- **Responsive Charts**: Recharts integration provides mobile-friendly visualizations
- **Memory Optimization**: React.memo prevents unnecessary re-renders
- **Error Handling**: Graceful degradation when API calls fail

---

### **Lesson 13: API Integration (`web/src/lib/api/`)**
*"How frontend and backend stay in sync"*

**Learning Objectives:**
- Master the comprehensive API client: 50+ typed methods in 410 lines
- Understand auto-generated TypeScript types from OpenAPI (2,515 lines)
- See centralized query hooks with consistent caching and error handling
- Grasp the complete type safety chain from Rust backend to React frontend

**API Integration Materials:**
- `web/src/lib/api/client.ts` - Complete HTTP client with 50+ API methods (410 lines)
- `web/src/types/api.ts` - Auto-generated OpenAPI TypeScript types (2,515 lines)
- `web/src/hooks/useApiQueries.ts` - Centralized query hooks with consistent caching (168 lines)
- `web/package.json` - API type generation script: `"generate-api": "npx openapi-typescript ../docs/openapi.json -o src/types/api.ts"`
- Backend source: `starter/src/openapi.rs` generates the OpenAPI schema

**🎯 Key Insights for Teaching:**

**Complete API Client Architecture (`lib/api/client.ts`):**
```typescript
// Comprehensive API client with full type safety
class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const token = getAuthToken();

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(options.headers as Record<string, string>),
    };

    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }

    try {
      const response = await fetch(url, { ...options, headers });
      if (!response.ok) {
        const errorData: ApiError = await response.json();
        throw new Error(errorData.error.message || `HTTP ${response.status}`);
      }
      return await response.json();
    } catch (error) {
      console.error(`API request failed: ${endpoint}`, error);
      throw error;
    }
  }
}
```

**Auto-Generated Type Safety (`types/api.ts`):**
The 2,515-line type file provides complete frontend-backend synchronization:
- **All API endpoints typed** - Every route has TypeScript definitions
- **Request/response schemas** - Input and output types for all operations
- **Component types** - Complex nested objects like `ComponentHealth`, `UserRole`
- **Auto-regeneration** - `pnpm run generate-api` keeps types in sync

**Centralized Query Hooks (`hooks/useApiQueries.ts`):**
```typescript
// Consistent query patterns with standardized caching
export function useHealthBasic(refetchInterval?: number): UseQueryResult<HealthData> {
  return useQuery({
    queryKey: ["health", "basic"],
    queryFn: async () => {
      const response = await apiClient.getHealth();
      if (!response.data) {
        throw new Error("No health data received");
      }
      return response.data;
    },
    refetchInterval: refetchInterval ?? REFETCH_INTERVALS.NORMAL,
  });
}

// Standard refetch intervals prevent cache collisions
const REFETCH_INTERVALS = {
  FAST: 5000,    // 5 seconds - for real-time components
  NORMAL: 15000, // 15 seconds - for regular updates
  SLOW: 30000,   // 30 seconds - for less critical data
} as const;
```

**Type-Safe API Methods (50+ methods):**
The API client provides comprehensive coverage:
```typescript
// Authentication methods
async login(data: LoginRequest): Promise<LoginResponse>
async register(data: RegisterRequest): Promise<UserProfileResponse>
async logout(): Promise<BasicResponse>
async refreshToken(): Promise<RefreshResponse>

// User management methods
async getUsers(params?: {...}): Promise<UserListResponse>
async createUser(data: {...}): Promise<UserProfileResponse>
async updateUserRole(id: string, data: {...}): Promise<UserProfileResponse>

// Task management methods
async getTasks(params?: {...}): Promise<TaskListResponse>
async createTask(data: CreateTaskRequest): Promise<TaskResponse>
async getTaskStats(): Promise<TaskStatsResponse>

// Health monitoring methods
async getHealth(): Promise<HealthResponse>
async getDetailedHealth(): Promise<DetailedHealthResponse>
async getLivenessProbe(): Promise<ProbeResponse>
```

**Token Management Integration:**
```typescript
// Automatic token storage and retrieval
export const setAuthToken = (token: string | null) => {
  authToken = token;
  if (token) {
    localStorage.setItem("auth_token", token);
  } else {
    localStorage.removeItem("auth_token");
  }
};

// Auto-store token on successful login
async login(data: LoginRequest): Promise<LoginResponse> {
  const response = await this.request<LoginResponse>("/auth/login", {
    method: "POST",
    body: JSON.stringify(data),
  });

  if (response.data?.session_token) {
    setAuthToken(response.data.session_token);
  }
  return response;
}
```

**Query Key Consistency:**
```typescript
// Centralized query keys prevent cache collisions
export const QUERY_KEYS = {
  health: {
    basic: ["health", "basic"] as const,
    detailed: ["health", "detailed"] as const,
    liveness: ["health", "liveness"] as const,
  },
  tasks: {
    stats: ["tasks", "stats"] as const,
    list: (filters?: Record<string, string>) => ["tasks", "list", filters] as const,
    detail: (id: string) => ["tasks", "detail", id] as const,
  },
  users: {
    me: ["auth", "me"] as const,
    list: (filters?: Record<string, string>) => ["users", "list", filters] as const,
  },
} as const;
```

**Deep Dive Questions:**
1. **Type Safety Question**: How do the 2,515 lines of auto-generated types ensure zero runtime type errors?
2. **Caching Question**: How do the centralized query hooks prevent cache collisions and ensure consistent data?
3. **Error Handling Question**: How does the API client provide uniform error handling across all 50+ methods?
4. **Integration Question**: How does the type generation process keep frontend and backend perfectly synchronized?
5. **Performance Question**: How do the standardized refetch intervals optimize network usage and data freshness?

**🔍 Teaching Experiments:**
1. **Type Generation**: Modify a backend endpoint and regenerate types to see the integration
2. **Query Caching**: Test different refetch intervals and observe caching behavior
3. **Error Scenarios**: Simulate network failures and invalid responses to test error handling
4. **Token Management**: Test the automatic token storage and authentication flow
5. **API Method Usage**: Use different API methods and observe the consistent patterns

**💡 API Integration Insights:**
- **Complete Type Safety**: Zero runtime type errors through comprehensive code generation
- **Centralized Patterns**: Consistent query hooks, error handling, and caching strategies
- **Performance Optimized**: Intelligent refetch intervals and query key management
- **Developer Experience**: Auto-completion, type checking, and uniform API surface
- **Maintainable Architecture**: Single source of truth for API communication
- **Production Ready**: Robust error handling, token management, and retry logic

**Deep Dive Questions:**
1. How does the API client handle authentication?
2. How are TypeScript types generated from the backend OpenAPI spec?
3. What's the error handling strategy for API calls?
4. How does TanStack Query manage caching and state?
5. How would you add a new API endpoint to the frontend?

---

### **Lesson 14: Testing Frontend (`web/e2e/` & `web/src/test/`)**
*"Ensuring the UI works end-to-end"*

**Learning Objectives:**
- Master Playwright E2E testing: 4 test files, 194 total lines of test code
- Understand the 3-tier testing strategy: smoke (400ms), single-browser (11s), multi-browser (5-10min)
- See production-ready test configuration with CI/CD integration and multi-device support
- Grasp the comprehensive 9-step frontend quality pipeline

**Frontend Testing Materials:**
- `web/e2e/auth.spec.ts` - Authentication flow testing (97 lines)
- `web/e2e/api-health.spec.ts` - API integration testing (45 lines)
- `web/e2e/example.spec.ts` - Core functionality testing (39 lines)
- `web/e2e/smoke.spec.ts` - Ultra-fast smoke tests (13 lines)
- `web/playwright.config.ts` - Multi-browser test configuration (86 lines)
- `web/scripts/check-web.sh` - Comprehensive quality pipeline (341 lines)

**🎯 Key Insights for Teaching:**

**Playwright Production Configuration:**
```typescript
// Multi-environment test setup
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  
  // Advanced failure capture
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',  
    video: 'retain-on-failure',
  },

  // Smart browser matrix
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    ...(process.env.PLAYWRIGHT_SMOKE_ONLY ? [] : [
      { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
      { name: 'webkit', use: { ...devices['Desktop Safari'] } },
      { name: 'Mobile Chrome', use: { ...devices['Pixel 5'] } },
      { name: 'Mobile Safari', use: { ...devices['iPhone 12'] } },
    ]),
  ],
});
```

**3-Tier Testing Strategy:**
```bash
# Ultra-fast feedback loop (~10s total, ~400ms E2E)
./scripts/check-web.sh --skip-lint --smoke

# Balanced testing (~11s total, ~1.1s E2E)  
./scripts/check-web.sh --skip-lint

# Comprehensive validation (~5-10min)
./scripts/check-web.sh --skip-lint --full
```

**Authentication Flow Testing (`auth.spec.ts`):**
Complete user journey validation with 97 lines:
```typescript
test.describe('Authentication', () => {
  test('complete registration and login flow', async ({ page }) => {
    // Test registration form
    await page.goto('/auth/register');
    await page.fill('[data-testid="username"]', testUser.username);
    await page.fill('[data-testid="email"]', testUser.email);
    await page.fill('[data-testid="password"]', testUser.password);
    await page.fill('[data-testid="confirmPassword"]', testUser.password);
    
    // Submit and verify success
    await page.click('[data-testid="register-button"]');
    await expect(page.locator('text=Registration successful')).toBeVisible();
    
    // Test automatic redirect
    await page.waitForURL('/auth/login');
    
    // Test login flow
    await page.fill('[data-testid="email"]', testUser.email);
    await page.fill('[data-testid="password"]', testUser.password);
    await page.click('[data-testid="login-button"]');
    
    // Verify dashboard access
    await expect(page).toHaveURL('/admin');
    await expect(page.locator('text=Dashboard')).toBeVisible();
  });
});
```

**Comprehensive Quality Pipeline (`check-web.sh`):**
9-step validation with configurable execution modes:
```bash
# Configurable test execution
--skip-lint: Skip linting/formatting (~2s saved)
--smoke: Ultra-fast mode with minimal coverage
--full: Complete multi-browser validation
--max-failures=N: Stop after N failures
--timeout=N: Per-test timeout (default: 5000ms)
--global-timeout=N: Suite timeout (15s smoke, 90s single, 300s full)

# Quality pipeline steps:
Step 1/9: Dependencies validation
Step 2/9: API type generation from OpenAPI
Step 3/9: TypeScript compilation and type checking  
Step 4/9: Biome linting and formatting
Step 5/9: Production build testing
Step 6/9: Unit test execution (Vitest)
Step 7/9: Component integration tests
Step 8/9: E2E testing (Playwright)
Step 9/9: Bundle analysis and optimization
```

**Deep Dive Questions:**
1. **Testing Strategy Question**: How do the 3 test modes (smoke/single/full) balance speed vs coverage for different development scenarios?
2. **Configuration Question**: How does the Playwright configuration support both local development and CI/CD environments?
3. **Coverage Question**: How do the 4 test files (194 lines) provide comprehensive frontend validation?
4. **Performance Question**: How does the smoke mode achieve 400ms E2E execution while maintaining meaningful coverage?
5. **Quality Question**: How does the 9-step pipeline ensure production-ready frontend code?

**💡 Frontend Testing Insights:**
- **Performance Optimized**: 3-tier strategy from 400ms smoke to comprehensive multi-browser
- **Production Ready**: CI/CD integration, retry logic, failure artifacts
- **Comprehensive Coverage**: Authentication, API health, core functionality, smoke tests
- **Developer Experience**: Fast feedback loops with configurable execution modes
- **Quality Assurance**: 9-step pipeline validates every aspect of frontend code

---

## 🔧 PHASE 3: CUSTOMIZATION & MASTERY (Lessons 15-16)

### **Lesson 15: The Rename Script - Making It Yours**
*"Transform the starter into your own system"*

**Learning Objectives:**
- Master the comprehensive project transformation: 314-line rename script with validation
- Understand the systematic renaming process: 497-line test suite ensuring reliability
- See production-grade project customization with backup and rollback capabilities
- Grasp the complete file transformation affecting 15+ file types across the entire stack

**Customization Materials:**
- `scripts/rename-project.sh` - Complete transformation script (314 lines)
- `scripts/test-rename-project.sh` - Comprehensive validation suite (497 lines)
- Files affected: `Cargo.toml`, `package.json`, Docker configs, documentation, source code
- Backup system with timestamp-based recovery
- Validation system ensuring zero broken functionality

**🎯 Key Insights for Teaching:**

**Comprehensive Rename Script (`rename-project.sh`):**
```bash
#!/bin/bash
# 314-line production-grade transformation script

# Input validation with Rust package naming conventions
if [[ ! "$NEW_NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
    echo -e "${RED}❌ Error: Invalid project name format${NC}"
    echo "Project name must:"
    echo "- Start with a letter or underscore"
    echo "- Contain only letters, numbers, and underscores"
    echo "- Follow Rust package naming conventions"
    echo "Good examples: my_project, awesome_app, backend_service"
    echo "Bad examples: 123project, my-project, project.name"
    exit 1
fi

# Check if already renamed with safety confirmation
if [ ! -d "starter" ]; then
    echo -e "${YELLOW}⚠️  Warning: 'starter' directory not found${NC}"
    echo "This project may have already been renamed, or you're in the wrong directory."
    read -p "Continue anyway? (y/N): " confirm
    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Backup system with timestamp
BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
echo -e "${YELLOW}📦 Creating backup in backup_${BACKUP_TIMESTAMP}/${NC}"
mkdir -p "backup_${BACKUP_TIMESTAMP}"
cp -r starter/ "backup_${BACKUP_TIMESTAMP}/"

# Docker service cleanup (environment variables will change)
echo -e "${BLUE}🐳 Stopping Docker services (environment will change)...${NC}"
docker-compose down --remove-orphans 2>/dev/null || true

# Smart file replacement with cross-platform support
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" -o -name "*.yml" \) \
    -not -path "./target/*" -not -path "./.git/*" -not -path "./backup_*/*" \
    -exec grep -l "starter" {} \; | while read -r file; do
    
    echo "  Updating: $file"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS-specific sed syntax
        sed -i '' "s/cargo run --bin starter/cargo run --bin $NEW_NAME/g" "$file"
        sed -i '' "s/use starter::/use ${NEW_NAME}::/g" "$file"
        sed -i '' "s/starter::/${NEW_NAME}::/g" "$file"
    else
        # Linux sed syntax  
        sed -i "s/cargo run --bin starter/cargo run --bin $NEW_NAME/g" "$file"
        sed -i "s/use starter::/use ${NEW_NAME}::/g" "$file"
        sed -i "s/starter::/${NEW_NAME}::/g" "$file"
    fi
done
```

**Automated Testing Suite (`test-rename-project.sh`):**
497 lines of comprehensive validation:
```bash
#!/bin/bash
# Automated testing for rename-project.sh script

# Test matrix execution
run_rename_test() {
    local project_name="$1"
    local test_dir="/tmp/rename-test-${project_name}"
    
    # Create isolated test environment
    cp -r "$PROJECT_ROOT" "$test_dir"
    cd "$test_dir"
    
    # Execute rename script
    timeout $TIMEOUT ./scripts/rename-project.sh "$project_name"
    
    # Comprehensive validation:
    # 1. Directory structure verification
    # 2. File content validation
    # 3. Compilation testing (Rust + TypeScript)
    # 4. Service startup verification
    # 5. API endpoint testing
    # 6. Database migration validation
    # 7. Docker build testing
    # 8. Clean shutdown verification
}

# Multiple test scenarios
test_scenarios=(
    "hello_world"
    "my_awesome_project" 
    "production_system"
    "custom_backend"
)
```

**File Transformation Coverage:**
The rename script affects 15+ file types across the entire stack:

**Backend Files:**
- `Cargo.toml` - Package name and workspace members
- `src/` - Module names and internal references
- `migrations/` - Database schema references
- `docs/` - API documentation updates

**Frontend Files:**
- `package.json` - Package name and metadata
- `src/` - Component names and references
- `public/` - Static asset references
- `e2e/` - Test configuration updates

**Infrastructure Files:**
- `docker-compose.yaml` - Service names and networks
- `Dockerfile.prod` - Build labels and metadata
- `.github/workflows/` - CI/CD pipeline names
- `README.md` - Project description and instructions

**Environment & Config:**
- `.env.example` - Variable prefix updates
- Script references throughout `/scripts/`
- Documentation links and references

**Production-Grade Features:**

**Backup & Recovery:**
```bash
# Automatic backup before any changes
BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
echo "📦 Creating backup in backup_${BACKUP_TIMESTAMP}/"
cp -r starter/ "backup_${BACKUP_TIMESTAMP}/"

# Rollback capability
if [ $? -ne 0 ]; then
    echo "❌ Rename failed, backup available at backup_${BACKUP_TIMESTAMP}/"
    exit 1
fi
```

**Validation System:**
```bash
# Cross-platform compatibility (macOS/Linux)
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/starter/$NEW_NAME/g" Cargo.toml
else
    sed -i "s/starter/$NEW_NAME/g" Cargo.toml
fi

# Docker service cleanup
docker-compose down --remove-orphans 2>/dev/null || true
```

**Deep Dive Questions:**
1. **Transformation Question**: How does the 314-line script systematically transform 15+ file types while maintaining system integrity?
2. **Validation Question**: How does the 497-line test suite ensure zero functionality breaks during transformation?
3. **Recovery Question**: How do the backup and rollback systems protect against transformation failures?
4. **Compatibility Question**: How does the script handle cross-platform differences (macOS vs Linux)?
5. **Extensibility Question**: How could you extend the rename script to handle additional customization scenarios?

**💡 Project Customization Insights:**
- **Comprehensive Coverage**: Transforms entire stack from database to frontend in single operation
- **Production Safety**: Backup system with timestamp-based recovery and rollback capabilities  
- **Cross-Platform**: Works reliably on macOS and Linux development environments
- **Automated Validation**: 497-line test suite ensures transformation reliability
- **Zero Downtime**: Proper service cleanup and restart coordination during transformation

---

### **Lesson 16: Mastery Demonstration**
*"Prove you own this system completely - build something real using established patterns"*

**Learning Objectives:**
- Create a new task handler following the actual patterns in `starter/src/tasks/handlers.rs` (356 lines)
- Master the rename script validation process using the actual `scripts/test-rename-project.sh` (497 lines)
- Extend Playwright E2E testing following patterns in `web/e2e/auth.spec.ts` (98 lines)
- Demonstrate production deployment using the established Docker and script infrastructure

**🎯 Mastery Challenge: Build a Custom Task Handler System**
Students prove mastery by implementing a complete task handler following the exact patterns found in the actual codebase.

**Challenge Scope: Custom "Invoice Processing" Task Handler**
Based on the real task handler patterns in `starter/src/tasks/handlers.rs`, create a production-ready invoice processing system.

**📋 Phase 1: Task Handler Implementation (Following Real Patterns)**

**1.1 Handler Structure (Based on `ReportGenerationTaskHandler` lines 143-175):**
```rust
/// Custom invoice processing task handler
pub struct InvoiceProcessingTaskHandler;

#[async_trait]
impl TaskHandler for InvoiceProcessingTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Use actual macros from helpers.rs:
        let (invoice_id, customer_id, amount) = 
            extract_fields!(context.payload, "invoice_id", "customer_id", "amount")?;
        
        // Use typed field extraction like DelayTaskHandler (lines 230-248):
        let tax_rate = context
            .payload
            .get("tax_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        // Follow exact error handling pattern from existing handlers
        if amount.parse::<f64>().is_err() {
            return Err(TaskError::Execution(
                "Invalid amount format".to_string(),
            ));
        }
        
        // Simulate processing like existing handlers (with real timing)
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        
        // Return structured result following TaskResult patterns
        let result = serde_json::json!({
            "invoice_id": invoice_id,
            "customer_id": customer_id,
            "amount": amount,
            "tax_rate": tax_rate,
            "processed_at": chrono::Utc::now(),
            "status": "processed"
        });
        
        Ok(TaskResult::success(result))
    }
}
```

**1.2 Handler Registration (Following `register_example_handlers` lines 334-355):**
Must add to the actual registration function:
```rust
// Add to existing register_example_handlers function
processor
    .register_handler("invoice_processing".to_string(), InvoiceProcessingTaskHandler)
    .await;
```

**1.3 Validation Using Real Helper Macros:**
Demonstrate mastery of the actual helper system from `tasks/helpers.rs`:
- Use `extract_fields!` macro (lines 58-70) for multiple field extraction
- Use `require_field!` macro (lines 11-18) for single required fields  
- Use `require_typed_field!` macro (lines 40-50) for typed field validation
- Use `TaskError::missing_field()` helper (lines 323-325)
- Use `TaskError::invalid_field_type()` helper (lines 327-330)

**📋 Phase 2: E2E Testing Extension (Following Real Playwright Patterns)**

**2.1 Authentication Flow Integration (Based on `auth.spec.ts` lines 29-69):**
Extend the existing authentication test to include invoice processing:
```typescript
test('complete authentication and invoice processing flow', async ({ page }) => {
  // Follow exact timestamp pattern from auth.spec.ts line 31
  const timestamp = Date.now();
  const email = `invoicetest_${timestamp}@example.com`;
  const password = 'SecurePassword123!';

  // Use established registration pattern (lines 40-50)
  await page.goto('/auth/register');
  await page.locator('input[type="email"]').fill(email);
  await page.locator('input[type="password"]').first().fill(password);
  await page.locator('input[type="password"]').last().fill(password);
  await page.locator('button:has-text("Create Account")').click();
  
  // Follow automatic redirect pattern (line 50)
  await page.waitForURL('**/auth/login');
  
  // Complete login and test invoice creation
  await page.locator('input[type="email"]').fill(email);
  await page.locator('input[type="password"]').fill(password);
  await page.locator('button:has-text("Sign In")').click();
  
  // Add invoice processing test here
  await page.waitForLoadState('networkidle');
  // Test creating invoice processing task through UI
});
```

**2.2 Form Validation Testing (Following `example.spec.ts` patterns):**
```typescript
test('invoice form validation', async ({ page }) => {
  // Follow the navigation pattern from example.spec.ts (lines 28-38)
  const response = await page.goto('/invoices/new');
  const status = response?.status() || 0;
  expect([200, 201, 202, 204, 301, 302, 404]).toContain(status);
  
  // Test form validation if page exists
  if (response?.status() && response.status() < 400) {
    // Follow validation testing pattern
    await expect(page.locator('body')).toBeVisible();
  }
});
```

**📋 Phase 3: Rename Script Mastery (Using Real Scripts)**

**3.1 Project Rename Process (Using actual `rename-project.sh` 314 lines):**
Students must successfully execute the complete rename process:

**Step 1**: Execute rename with validation
```bash
# Use actual script with custom name 
./scripts/rename-project.sh invoice_processor --verbose

# This will:
# - Stop Docker services (lines 71-78) 
# - Create backup with timestamp (lines 81-89)
# - Rename starter/ directory (lines 92-95)
# - Update workspace members (lines 97-107)
# - Replace all occurrences (lines 121-164)
# - Update environment variables (lines 167-195)
# - Restart Docker services (lines 222-258)
```

**Step 2**: Validate using test script (`test-rename-project.sh` 497 lines)
```bash
# Run comprehensive validation
./scripts/test-rename-project.sh invoice_processor --verbose

# This validates all 4 phases:
# 1. Environment setup (lines 148-279)
# 2. Rename script execution (lines 281-311) 
# 3. Pattern validation (lines 313-358)
# 4. Quality checks (lines 360-380)
```

**3.2 Quality Pipeline Integration:**
Must pass all checks in the actual quality pipeline:
```bash
# Run complete quality validation (check.sh)
./scripts/check.sh

# All 9 steps must pass:
# 1. Frontend build  2. Compilation  3. Formatting  4. Linting
# 5. SQLx cache  6. Unit tests  7. Integration tests
# 8. API testing  9. Static serving
```

**📋 Phase 4: Production Deployment (Using Real Infrastructure)**

**4.1 Docker Deployment (Using actual `Dockerfile.prod`):**
Deploy the renamed system using established patterns:
```bash
# Build production image
docker build -f Dockerfile.prod -t invoice-processor .

# Deploy using docker-compose.prod.yaml
docker-compose -f docker-compose.prod.yaml up -d
```

**4.2 Chaos Testing Integration:**
Add invoice processing to chaos testing scenarios:
```bash
# Test resilience using actual chaos framework
./scripts/test-chaos.sh --scenarios "task-flood,worker-restart" --difficulty 2

# Must achieve 100% task completion including new invoice handlers
```

**4.3 API Testing Validation:**
Extend the actual API testing:
```bash
# Validate new endpoints using established testing
./scripts/test-with-curl.sh

# Must test invoice processing task creation and status endpoints
```

**📊 Success Criteria (Based on Actual Metrics):**

**Handler Implementation Mastery:**
- ✅ **Follows exact patterns** from existing 6 handlers in `handlers.rs`
- ✅ **Uses actual helper macros** from `tasks/helpers.rs` correctly
- ✅ **Implements proper error handling** using `TaskError` variants
- ✅ **Returns structured results** following `TaskResult` patterns
- ✅ **Registers handler correctly** in the existing registration system

**Testing Integration Mastery:**
- ✅ **E2E tests follow patterns** from `auth.spec.ts` and `example.spec.ts`
- ✅ **Playwright configuration** uses existing `playwright.config.ts` setup
- ✅ **Authentication flow** follows the exact 98-line pattern
- ✅ **Form validation** follows established testing approaches
- ✅ **Error cases handled** like existing tests

**Rename Script Mastery:**
- ✅ **314-line rename script** executes without errors
- ✅ **497-line test validation** passes all 4 phases
- ✅ **Quality pipeline** completes in under 60 seconds
- ✅ **Docker services** restart successfully with new environment
- ✅ **Database migrations** work with renamed system

**Production Deployment Mastery:**
- ✅ **Docker production build** completes successfully
- ✅ **Health endpoints** respond correctly at `/api/v1/health/*`
- ✅ **Chaos testing** achieves 100% completion with new handlers
- ✅ **API testing** validates all endpoints including new ones
- ✅ **System monitoring** works through admin CLI commands

**🎓 Mastery Validation Process:**

**Phase A: Code Review (20 minutes)**
Instructor validates implementation against actual patterns:
- Task handler follows exact structure from `handlers.rs`
- Uses macros from `helpers.rs` correctly
- Error handling matches existing patterns
- Result format follows `TaskResult` structure

**Phase B: System Testing (20 minutes)**
Student demonstrates working system:
- Rename script execution with validation
- E2E tests run successfully
- Quality pipeline passes completely
- Handler processes tasks correctly

**Phase C: Production Deployment (20 minutes)**
Student deploys and operates the system:
- Docker build and deployment
- Health check validation
- Chaos testing execution
- API endpoint testing

**🔍 Common Implementation Pitfalls:**

**Handler Implementation Errors:**
- 🚫 **Not using macros**: Students implement manual field extraction instead of using `extract_fields!`
- 🚫 **Wrong error types**: Using generic errors instead of `TaskError` variants
- 🚫 **Improper registration**: Not adding handler to `register_example_handlers()`
- 🚫 **Inconsistent timing**: Not following the established sleep patterns for simulation

**Testing Integration Errors:**
- 🚫 **New test files**: Creating separate test files instead of extending existing ones
- 🚫 **Different patterns**: Not following the exact authentication flow from `auth.spec.ts`
- 🚫 **Missing navigation**: Not handling redirects like existing tests
- 🚫 **Hardcoded values**: Not using timestamp patterns for unique data

**Rename Script Errors:**
- 🚫 **Manual editing**: Trying to manually update files instead of using the script
- 🚫 **Skipping validation**: Not running the test validation script
- 🚫 **Docker issues**: Not properly handling Docker service restarts
- 🚫 **Environment problems**: Not updating environment variables correctly

**📈 Advanced Mastery Extensions:**

**For Exceptional Students:**

**Advanced Handler Features:**
- Implement deadline checking like `DelayTaskHandler` (lines 249-288)
- Add retry logic integration with `RetryStrategy`
- Create handler-specific metadata patterns
- Implement batch processing capabilities

**Advanced Testing:**
- Add chaos testing scenarios specific to invoice processing
- Create performance benchmarks for the new handler
- Implement integration tests that validate end-to-end flows
- Add monitoring and alerting for handler performance

**Advanced Deployment:**
- Set up actual cloud deployment (AWS, GCP, Azure)
- Configure production monitoring and logging
- Implement blue-green deployment patterns
- Create infrastructure as code (Terraform/CloudFormation)

**💡 Success Indicators for Instructors:**

**True Mastery Signals:**
- ✨ **Pattern Recognition**: Student immediately identifies which existing handler to model after
- ✨ **Macro Usage**: Student uses helper macros naturally without referring to documentation
- ✨ **Error Handling**: Student anticipates error cases before they encounter them
- ✨ **System Integration**: Student understands how their handler fits into the broader system
- ✨ **Testing Mindset**: Student writes tests before implementing features

**Teaching Moments:**
- When students try to deviate from patterns, show them why consistency matters
- When students skip validation, demonstrate how the test scripts catch issues
- When students fear the rename script, walk through the backup and recovery process
- When students struggle with deployment, show how the scripts handle complexity

This mastery demonstration is grounded in the actual codebase patterns and ensures students can truly extend and operate the system in real-world scenarios.

---

## 🎯 Teaching Methodology

### Starter-Specific Approach

1. **Always Reference Actual Code**: Every concept ties to specific files in the codebase
2. **Follow the Data Flow**: Trace every request through the actual system
3. **Use Real Examples**: All exercises use the actual starter features
4. **Debug Real Issues**: Practice on actual bugs and edge cases
5. **Build Real Extensions**: Add actual features to the running system

### Progress Tracking

**Level 1: Observer** - Can follow along with code explanations
**Level 2: Navigator** - Can find and understand specific files and functions
**Level 3: Modifier** - Can make changes without breaking the system
**Level 4: Architect** - Can design and implement new features
**Level 5: Master** - Can explain and teach the entire system to others

### Assessment Through Real Tasks

Instead of theoretical questions, students demonstrate understanding by:
- Adding actual features to the codebase
- Fixing real bugs in the system
- Extending existing functionality
- Successfully using the rename script
- Deploying and operating their customized system

### Interactive Teaching Patterns

**When Students Ask Questions:**
1. **Guide to Source Code**: "Let's look at the actual implementation in `starter/src/auth/services.rs`"
2. **Encourage Experimentation**: "What happens if you modify this function and rerun the tests?"
3. **Connect to Bigger Picture**: "How does this auth module connect to the RBAC system?"
4. **Use Real Examples**: "Let's trace a login request through the entire system"
5. **Build on Understanding**: "Now that you understand auth, let's see how tasks use it"

**Lesson Flow Pattern:**
1. **Set Context**: "We're now going to master the `starter/src/tasks/` module"
2. **Explore Together**: Guide through actual files and code
3. **Question Understanding**: Ask about specific implementations
4. **Encourage Experiments**: "Try modifying the retry logic and see what happens"
5. **Connect Concepts**: "How does this relate to what we learned about the database?"
6. **Demonstrate Mastery**: "Can you add a new task type following the existing pattern?"

---

## 💡 Success Metrics

### Student Mastery Indicators

- **Code Fluency**: Can navigate and understand any file in the codebase
- **System Thinking**: Understands how all components work together
- **Debug Capability**: Can systematically find and fix problems
- **Extension Ability**: Can add new features following existing patterns
- **Teaching Ability**: Can explain any part of the system to others

### Course Success

- Student creates a successfully customized system using the rename script
- Student can add substantial new features independently
- Student demonstrates deep understanding through code modifications
- Student shows confidence in deploying and operating the system
- Student becomes capable of teaching others about the system

### Red Flags to Watch For

- **Surface Learning**: Can repeat information but can't apply it
- **Magic Box Thinking**: Treats parts of the system as unknowable
- **Copy-Paste Programming**: Can't modify existing patterns
- **Fear of Breaking**: Won't experiment with the code
- **Isolation Thinking**: Can't see how components connect

### Intervention Strategies

- **For Struggling Students**: Return to concrete examples, use more analogies, slower pace
- **For Advanced Students**: Challenge with optimization tasks, architectural questions
- **For Confused Students**: Back up to fundamentals, use different explanations
- **For Bored Students**: Advanced challenges, mentoring opportunities

---

## 🔧 Practical Teaching Tools

### Key Commands for Teaching

```bash
# System exploration
find starter/src -name "*.rs" | head -20  # Show main Rust files
find web/src -name "*.tsx" | head -10     # Show main React files
wc -l starter/src/**/*.rs                 # Lines of code to master

# Development workflow
./scripts/dev-server.sh                   # Start the system
./scripts/check.sh                        # Quality checks
./scripts/test-with-curl.sh               # API testing
./scripts/test-chaos.sh                   # Resilience testing

# System understanding
cargo run -- admin task-stats             # Admin operations
cargo run -- export-openapi               # API documentation
```

### Useful Analogies for Teaching

- **Databases**: Filing cabinets with organized folders and labels
- **APIs**: Restaurant servers taking orders and bringing food
- **Authentication**: Checking ID at a club entrance
- **Background Tasks**: Postal service - you drop it off, they handle delivery
- **Caching**: Keeping frequently used tools on your workbench
- **Load Balancing**: Multiple checkout lines at a grocery store
- **Migrations**: Renovating a house room by room while people live in it

### Common Student Questions & Responses

**"Why Rust instead of Python/JavaScript?"**
→ "Let's look at the performance characteristics in `scripts/test-chaos.sh` and see how this system handles load"

**"How does this scale to millions of users?"**
→ "Great question! Let's examine the database design and connection pooling in `starter/src/database.rs`"

**"What happens if the database goes down?"**
→ "Let's test that! We can simulate database failures using the chaos testing framework"

**"Can I add my own features?"**
→ "Absolutely! That's the goal. Let's start by understanding the existing patterns, then you can extend them"

---

*"I learned very early the difference between knowing the name of something and knowing something." - Richard P. Feynman*

*The goal isn't to know about web development - it's to truly understand THIS specific system completely.*