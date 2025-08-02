# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Constraints and Guidelines

- **Project Nature Guidance:**
  * TONE DOWN with STARTER NATURE OF THE PROJECT
  * NEVER SAY PRODUCTION OR ENTERPRISE READY

## Testing Commands

- **Quality Checks**: `./scripts/check.sh` (**RUN BEFORE EVERY COMMIT** - comprehensive quality validation)
  - Runs: web build (early), cargo check, fmt, clippy, sqlx prepare, unit tests, integration tests, static serving smoke test
  - 9 steps total with health endpoint polling for reliable testing
  - ~40-45 seconds for complete validation including frontend
  - Required for all commits to maintain code quality
- **Integration Tests**: `cd starter && cargo nextest run` (119 tests, ~17 seconds)
- **API Testing**: `./scripts/test-with-curl.sh [host] [port]` (44+ endpoint tests)
  - Default: `./scripts/test-with-curl.sh` (localhost:3000)
  - Custom: `./scripts/test-with-curl.sh localhost 8080`
  - HTTPS: `./scripts/test-with-curl.sh api.example.com 443`
  - **NEW**: Includes task type registration and comprehensive user management testing (`POST/GET /api/v1/tasks/types`, user lifecycle endpoints)
- **Chaos Testing**: `./scripts/test-chaos.sh [options]` (Docker-based resilience testing with automatic image building)
  - Basic: `./scripts/test-chaos.sh` (difficulty 1, all scenarios, clean database by default)
  - Advanced: `./scripts/test-chaos.sh --difficulty 3 --scenarios "db-failure,task-flood"`
  - Keep Data: `./scripts/test-chaos.sh --keep-database` (preserve existing database state)
  - Output: Results saved to `/tmp/chaos-test-report.md` and `/tmp/api-test-*.txt`
- **Server Management**: 
  - Complete environment: `./scripts/dev-server.sh [options]` (database + web build + unified server)
  - Quick start: `./scripts/server.sh [port] [-f]` (unified API + frontend serving, auto-builds web if needed)
  - Stop: `./scripts/stop-server.sh [port]`
  - Worker: `./scripts/worker.sh [--id ID] [-f]` (--id for concurrent workers, -f for foreground mode)

## Health Endpoints

Available health check endpoints:
- `/api/v1/health` - Basic health check (status, version, uptime, includes documentation links)
- `/api/v1/health/detailed` - Detailed health with dependency checks
- `/api/v1/health/live` - Kubernetes liveness probe (minimal checks)
- `/api/v1/health/ready` - Kubernetes readiness probe (dependency validation)
- `/api/v1/health/startup` - Kubernetes startup probe (initialization checks)

## API Documentation

Comprehensive OpenAPI documentation is available:
- **Interactive Docs**: `/api-docs` - HTML page with overview and Swagger UI links
- **OpenAPI Schema**: `/api-docs/openapi.json` - Complete OpenAPI 3.0 specification
- **Local Export**: `cargo run -- export-openapi` - Export to `docs/openapi.json` for versioning
- **Features**: All endpoints documented with request/response examples, authentication support, type definitions
- **Client Generation**: Use schema to generate TypeScript, Python, or other language clients

## Project Scripts

Key development scripts in `/scripts/`:
- `check.sh` - **Comprehensive quality checks (run before every commit)**
- `prepare-sqlx.sh` - **Update SQLx query cache for offline compilation**
- `server.sh` - Start development server with custom port
- `worker.sh` - Start background task worker (supports concurrent workers with --id)
- `build-web.sh` - **Build React frontend to web/dist directory**
- `test-with-curl.sh` - Comprehensive API endpoint testing
- `test-chaos.sh` - Chaos testing framework for resilience validation
- `reset-all.sh` - Database reset (requires `--reset-database` flag)
- `rename-project.sh` - Automated project renaming with validation
- `deploy-prod.sh` - Production deployment with Docker
- `dev-server.sh` - Complete development environment setup

### Chaos Testing Helpers in `/scripts/helpers/`:
- `auth-helper.sh` - Create test users and authentication tokens
- `task-flood.sh` - Generate high task loads for performance testing
- `delay-task-flood.sh` - **NEW**: Create delay tasks with configurable deadlines for worker chaos testing
- `multi-worker-chaos.sh` - **NEW**: Docker Compose scaling for multi-worker chaos testing
- `task-completion-monitor.sh` - **NEW**: Monitor task completion against deadlines with statistics
- `service-chaos.sh` - Simulate service failures (server, worker, database)

## Starter Features

This starter template includes comprehensive development infrastructure:
- **Docker Configuration**: Multi-stage builds with development and testing setups (`Dockerfile.prod`, `docker-compose.yaml`, `docker-compose.chaos.yaml`)
- **Container Testing**: Docker-based chaos testing with container isolation and resource limits
- **CI/CD Examples**: GitHub Actions workflows demonstrating testing, security, and deployment patterns (`.github/workflows/`)
- **Development Tools**: Automated scripts for testing, quality checks, and development workflow
- **Testing Framework**: Integration testing with database isolation and Docker-based resilience testing
- **Project Customization**: Automated project renaming and adaptation tools

## Development Workflow

### Backend Development (Rust API)
1. **Start Services**: Choose your preferred method
   - **Complete environment**: `./scripts/dev-server.sh` (database + web build + unified server) **[RECOMMENDED]**
   - **Quick restart**: `./scripts/server.sh && ./scripts/worker.sh` (auto-builds web if needed)
   - **Manual foreground**: `./scripts/server.sh -f` + `./scripts/worker.sh -f` (separate terminals)
   - **Multiple workers**: `./scripts/worker.sh --id 1` + `./scripts/worker.sh --id 2` (concurrent workers)
   - **IMPORTANT**: Workers must start to register task types before creating tasks
2. **Quality Checks**: `./scripts/check.sh` (**MANDATORY before every commit**)
   - Validates: web build, formatting, linting, compilation, SQLx, tests, static serving
3. **API Testing**: `./scripts/test-with-curl.sh` (44+ endpoint tests including web serving)
4. **Chaos Testing**: `./scripts/test-chaos.sh` (Docker-based resilience validation)
5. **Stop Services**: `./scripts/stop-server.sh 3000`

### Frontend Development (React/TypeScript)
**Multi-phase development workflow with quality checks:**

1. **Implement Phase**: Work on specific feature phase (authentication, admin portal, user management, etc.)
2. **Quality Validation**: `cd web && ./scripts/check-web.sh` (**RUN BEFORE EVERY COMMIT**)
   - Dependencies validation and API type generation
   - TypeScript type checking and compilation
   - Biome linting and code formatting
   - Production build testing
   - Unit/integration tests
   - Code quality analysis and bundle optimization
3. **Fix Issues**: Address any failures from quality checks
4. **Commit Phase**: Commit completed phase without push to mark milestone
5. **Next Phase**: Proceed to next development phase

### Full-Stack Development (React + Rust)
**Unified development workflow with static file serving:**

1. **Complete Environment**: `./scripts/dev-server.sh` (One command for everything)
   - Starts database if needed
   - Builds frontend automatically
   - Serves unified API + static files
   - Available at `http://localhost:3000`
   - Single deployment artifact for production
2. **Manual Build**: `./scripts/build-web.sh` (Build React app to web/dist only)
3. **Quality Validation**: Both backend and frontend checks
   - Backend: `./scripts/check.sh`
   - Frontend: `cd web && ./scripts/check-web.sh`

**Web Quality Checks**: `web/scripts/check-web.sh`
- **Dependencies**: Validates pnpm dependencies and installation
- **API Types**: Auto-generates TypeScript types from `../docs/openapi.json`
- **TypeScript**: Full type checking with `tsc --noEmit`
- **Linting**: Biome linting with auto-fix suggestions
- **Formatting**: Code formatting validation with Biome
- **Build**: Production build testing with Vite
- **Tests**: Unit and integration test execution
- **Analysis**: Bundle size analysis, unused dependencies, code quality checks
- **Components**: Validates shadcn/ui components and API client setup

**Web Project Structure**:
- Modern React 18 with TanStack Router (file-based routing)
- TanStack Query for server state management
- shadcn/ui@canary components with Tailwind CSS 4
- TypeScript with auto-generated API types
- Authentication system with session-based auth and RBAC
- Admin portal with sidebar navigation and dashboard
- Comprehensive quality checking and production build validation

## CLI Module Architecture

The application uses a modular CLI structure located in `starter/src/cli/`:

### Module Organization:
- **`api.rs`** - Command execution and application entry point (`CliApp::run()`)
- **`models.rs`** - CLI command definitions using Clap framework
- **`services.rs`** - Business logic and database operations (`AdminService`)
- **`mod.rs`** - Module organization and public exports
- **`tests.rs`** - Unit tests for CLI functionality (11 tests)

### Testing:
- **Unit Tests**: `starter/src/cli/tests.rs` (11 tests)
- **Integration Tests**: `starter/tests/cli/mod.rs` (8 tests)
- **Total Coverage**: 19 CLI tests with 100% pass rate

## Admin CLI Commands

For direct database access (useful during chaos testing and debugging):

```bash
# Task statistics (bypasses API authentication, shows all tasks regardless of user ownership)
cargo run -- admin task-stats

# Task statistics with tag filter
cargo run -- admin task-stats --tag "baseline"

# List recent tasks (shows all users' tasks - RBAC-aware)
cargo run -- admin list-tasks --limit 10

# List tasks with verbose details (includes user context and role information)
cargo run -- admin list-tasks --verbose

# Clear old completed tasks (cleans tasks from all users - dry run)
cargo run -- admin clear-completed --dry-run

# Clear completed tasks older than 7 days (across all users)
cargo run -- admin clear-completed
```

**CLI Architecture**: The CLI functionality follows the same modular pattern as `auth/`, `users/`, and `rbac/` modules with dedicated `api.rs`, `models.rs`, and `services.rs` files. The main application entry point (`main.rs`) has been simplified to just 6 lines, delegating all CLI logic to the dedicated CLI module.

**Note**: Admin commands access the database directly, bypassing API authentication and RBAC checks. Commands show data from all users regardless of ownership. Useful for monitoring during chaos testing when API may be unreliable.

## Role-Based Access Control (RBAC) System

This project implements comprehensive three-tier RBAC with hierarchical permissions:

### Role Hierarchy
- **User (Level 1)**: Can only access own tasks and profile
- **Moderator (Level 2)**: Can manage all user tasks and view all profiles  
- **Admin (Level 3)**: Full system access including admin-only endpoints

### RBAC Development Patterns

**Using RBAC in API handlers:**
```rust
use crate::rbac::services as rbac_services;

// Check task access permissions
rbac_services::can_access_task(&auth_user, task.created_by)?;

// Require moderator or higher
rbac_services::require_moderator_or_higher(&auth_user)?;

// Check specific permissions
rbac_services::check_permission(&auth_user, Resource::Tasks, Permission::Write)?;
```

**Role-based middleware for routes:**
```rust
// Moderator routes
let moderator_routes = Router::new()
    .route("/users", get(users_api::list_users))
    .layer(middleware::from_fn(moderator_middleware));

// Admin routes  
let admin_routes = Router::new()
    .route("/admin/settings", get(admin_settings))
    .layer(middleware::from_fn(admin_middleware));
```

### Admin Account Setup

Set `STARTER__INITIAL_ADMIN_PASSWORD` in `.env` to automatically create admin account:
- Username: `admin`
- Email: `admin@example.com`
- Role: `Admin` (full system access)

### RBAC Module Architecture

The RBAC system is organized in `starter/src/rbac/`:
- **`models.rs`** - UserRole enum, Permission/Resource definitions, SQLx integration
- **`services.rs`** - Authorization business logic and permission checking
- **`middleware.rs`** - Role-based route protection middleware
- **`mod.rs`** - Module organization and public exports

## Pre-Commit Requirements

**ALWAYS run `./scripts/check.sh` before every commit.** This script:
- Verifies code compilation (`cargo check`)
- Validates formatting (`cargo fmt --check`)
- Runs linting (`cargo clippy`)
- Updates SQLx query cache (`cargo sqlx prepare`)
- Executes unit tests (`cargo test --lib`)
- Runs integration tests (`cargo nextest run`)
- Performs additional quality checks

The repository includes a pre-commit hook that automatically runs these checks.

## Chaos Testing Scenarios

Available chaos testing scenarios:
- `baseline` - Normal functionality validation
- `db-failure` - Database connection failure resilience
- `server-restart` - HTTP server restart recovery
- `worker-restart` - Background worker restart handling
- `task-flood` - High load performance testing
- `circuit-breaker` - Circuit breaker activation/recovery
- `mixed-chaos` - Multiple simultaneous failures
- `recovery` - Recovery time measurement
- `multi-worker-chaos` - **NEW**: Docker-based multi-worker testing with container scaling and failures
  - Uses Docker Compose scaling to manage multiple worker containers
  - Simulates random container failures and automatic restarts
  - Tests task completion under worker chaos with configurable delays and deadlines
  - Configurable difficulty levels (1-6) affect worker count, task delays, and failure intervals
  - **Level 6 (Catastrophic)**: Designed to fail - tests impossible workloads and container failure handling
- `dynamic-scaling` - **NEW**: Dynamic worker scaling with 4-phase resilience testing
  - **Phase 1 (0-20s)**: Optimal capacity test with 5 workers
  - **Phase 2 (20-40s)**: Capacity reduction stress test with 2 workers
  - **Phase 3 (40-49s)**: Gradual scale-up (+1 worker every 3s: 2→3→4→5)
  - **Phase 4 (49s-deadline)**: Completion monitoring with full capacity
  - Tests system's ability to handle worker scaling operations while maintaining 100% task completion
  - Demonstrates resilience during resource constraints and validates scaling behavior
  - Success criteria: 100% completion within time limits (varies by difficulty: 300s→180s)
  - **See detailed documentation**: `docs/guides/09-chaos-testing.md`

## Task Type Registration System

**BREAKING CHANGE**: As of recent updates, the system requires task type registration before tasks can be created.

### Key Changes:
- **API Validation**: `POST /api/v1/tasks` now validates task types against registered handlers
- **Worker Registration**: Workers automatically register task types on startup via `POST /api/v1/tasks/types`
- **New Endpoints**: 
  - `GET /api/v1/tasks/types` - List registered task types (public)
  - `POST /api/v1/tasks/types` - Register task type (public, used by workers)
- **Test Updates**: Integration tests now use `TestDataFactory::new_with_task_types()` for automatic registration
- **Error Handling**: Unregistered task types return 400 validation errors instead of 200/201

### Impact on Development:
- **Start workers before creating tasks** - API will reject tasks for unregistered types
- **Tests updated** - All 95 integration tests pass with new validation
- **Docker-based chaos testing** - All scenarios now run in isolated containers with proper resource limits

## User Management System

**NEW**: Comprehensive user lifecycle management with 12 endpoints for profile management, administration, and analytics.

### User Management Features
- **Profile Self-Management** - Users control their own data (update profile, change password, delete account)
- **Hierarchical Administration** - Moderators manage users, admins manage moderators and system
- **Comprehensive Analytics** - Detailed user statistics for operational insights
- **Security-First Design** - Password verification, audit trails, and data privacy
- **RBAC Enforcement** - Demonstrates three authorization patterns (ownership, hierarchy, cross-user)
- **Soft Delete Protection** - Data preservation with recovery options

### User Management Endpoints (12 total)
| Endpoint | Method | Access Level | Description |
|----------|--------|-------------|-------------|
| **Profile Management** | | | |
| `/api/v1/users/me/profile` | PUT | User | Update own profile (username, email) |
| `/api/v1/users/me/password` | PUT | User | Change own password with verification |
| `/api/v1/users/me` | DELETE | User | Delete own account (soft delete) |
| **User Administration** | | | |
| `/api/v1/users` | GET | Moderator+ | List all users (paginated) |
| `/api/v1/users` | POST | Admin | Create new user account |
| `/api/v1/users/{id}` | GET | Owner/Moderator+ | Get user profile by ID |
| `/api/v1/users/{id}/profile` | PUT | Admin | Update any user's profile |
| `/api/v1/users/{id}/status` | PUT | Moderator+ | Activate/deactivate user accounts |
| `/api/v1/users/{id}/role` | PUT | Admin | Change user roles |
| `/api/v1/users/{id}/reset-password` | POST | Moderator+ | Force password reset |
| `/api/v1/users/{id}` | DELETE | Admin | Delete user account (admin) |
| **Analytics** | | | |
| `/api/v1/admin/users/stats` | GET | Admin | Comprehensive user statistics |

### User Management Module Architecture

The user management system is organized in `starter/src/users/`:
- **`api.rs`** - HTTP endpoints (12 user management handlers)
- **`models.rs`** - Request/response types with validation
- **`services.rs`** - Business logic (password hashing, RBAC checks)
- **`mod.rs`** - Module exports and organization

### User Management Testing
- **Integration Tests**: `starter/tests/users/mod.rs` (17 comprehensive tests)
- **Total Coverage**: User management tests cover all 12 endpoints with positive/negative cases
- **RBAC Testing**: Role-based access control enforcement validation
- **Security Testing**: Password verification, confirmations, and audit trails

### User Management Development Patterns

**Profile Self-Management Pattern:**
```rust
// Users can always update their own resources
pub async fn update_own_profile(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // No additional auth needed - user owns the resource
    let user = user_services::update_user_profile(&mut conn, auth_user.id, request).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

**Administrative Management Pattern:**
```rust
// Admin operations with proper authorization checks
pub async fn create_user(
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    rbac_services::require_admin(&auth_user)?;
    let user = user_services::create_user(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

**Documentation**: Comprehensive user management documentation available at `docs/guides/12-user-management.md`

## GitHub CLI Workflow

**Standard GitHub CLI commands for PR management and code review workflows:**

### Pull Request Management
```bash
# Create pull request with structured description
gh pr create --title "TITLE" --body "$(cat <<'EOF'
## Summary
- Key change 1
- Key change 2
- Key change 3

## Test plan
- [ ] Quality checks passed
- [ ] Tests executed
- [ ] Documentation updated
EOF
)"

# View pull request details and status
gh pr view PR_NUMBER
gh pr status

# List and read pull request reviews
gh pr review list PR_NUMBER
gh api repos/OWNER/REPO/pulls/PR_NUMBER/reviews/REVIEW_ID
gh api repos/OWNER/REPO/pulls/PR_NUMBER/reviews/REVIEW_ID/comments
```

### Code Review Response Workflow
```bash
# Comment on pull request to address reviewer feedback
gh pr comment PR_NUMBER --body "## 🔧 PR Review Feedback Addressed

@REVIEWER_USERNAME Thank you for the comprehensive code review! I've addressed the issues you identified:

## ✅ **Critical Issues Fixed:**
### 1. **Issue Title** 
- **Fixed**: Description of fix
- **Location**: \`file/path:line-range\`
- **Impact**: What this fix accomplishes

### 2. **Another Issue**
- **Fixed**: Description of fix
- **Location**: \`file/path:line-range\`
- **Impact**: What this fix accomplishes

## 📊 **Quality Validation:**
- [x] All tests passing
- [x] Build successful
- [x] Linting/formatting clean
- [x] No breaking changes"
```

### Review Analysis Commands
```bash
# Fetch specific review details with GitHub API
gh api repos/OWNER/REPO/pulls/PR_NUMBER/reviews/REVIEW_ID

# Get review comments for detailed analysis  
gh api repos/OWNER/REPO/pulls/PR_NUMBER/reviews/REVIEW_ID/comments

# View PR conversation and comments
gh pr view PR_NUMBER --comments
```

**Key Success Patterns:**
- **Direct reviewer tagging**: Use `@username` to notify original reviewer
- **Structured responses**: Organize fixes by severity (Critical → High → Medium)
- **Detailed locations**: Include file paths and line numbers for each fix
- **Impact explanation**: Describe what each fix accomplishes  
- **Testing validation**: Show test results and quality metrics
- **Template usage**: Use placeholders (OWNER/REPO/PR_NUMBER) for reusability