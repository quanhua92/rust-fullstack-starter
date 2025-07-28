# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Constraints and Guidelines

- **Project Nature Guidance:**
  * TONE DOWN with STARTER NATURE OF THE PROJECT
  * NEVER SAY PRODUCTION OR ENTERPRISE READY

## Testing Commands

- **Quality Checks**: `./scripts/check.sh` (**RUN BEFORE EVERY COMMIT** - comprehensive quality validation)
  - Runs: cargo check, fmt, clippy, sqlx prepare, unit tests, integration tests
  - ~30-60 seconds for complete validation
  - Required for all commits to maintain code quality
- **Integration Tests**: `cd starter && cargo nextest run` (53 tests, ~12 seconds)
- **API Testing**: `./scripts/test-with-curl.sh [host] [port]` (40+ endpoint tests)
  - Default: `./scripts/test-with-curl.sh` (localhost:3000)
  - Custom: `./scripts/test-with-curl.sh localhost 8080`
  - HTTPS: `./scripts/test-with-curl.sh api.example.com 443`
  - **NEW**: Includes task type registration testing (`POST/GET /tasks/types`)
- **Chaos Testing**: `./scripts/test-chaos.sh [options]` (Docker-based resilience testing with automatic image building)
  - Basic: `./scripts/test-chaos.sh` (difficulty 1, all scenarios)
  - Advanced: `./scripts/test-chaos.sh --difficulty 3 --scenarios "db-failure,task-flood"`
  - Output: Results saved to `/tmp/chaos-test-report.md` and `/tmp/api-test-*.txt`
- **Server Management**: 
  - Start: `./scripts/server.sh [port]` (default port 3000)
  - Stop: `./scripts/stop-server.sh [port]`
  - Worker: `./scripts/worker.sh`

## Health Endpoints

Available health check endpoints:
- `/health` - Basic health check (status, version, uptime, includes documentation links)
- `/health/detailed` - Detailed health with dependency checks
- `/health/live` - Kubernetes liveness probe (minimal checks)
- `/health/ready` - Kubernetes readiness probe (dependency validation)
- `/health/startup` - Kubernetes startup probe (initialization checks)

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
- `server.sh` - Start development server with custom port
- `worker.sh` - Start background task worker
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

1. **Start Services**: `./scripts/dev-server.sh 3000` (complete environment)
   - Or manually: `./scripts/server.sh && ./scripts/worker.sh`
   - **IMPORTANT**: Workers must start to register task types before creating tasks
2. **Quality Checks**: `./scripts/check.sh` (**MANDATORY before every commit**)
   - Validates: formatting, linting, compilation, SQLx, tests
3. **API Testing**: `./scripts/test-with-curl.sh` (40+ endpoint tests)
4. **Chaos Testing**: `./scripts/test-chaos.sh` (Docker-based resilience validation)
5. **Stop Services**: `./scripts/stop-server.sh 3000`

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
  - **Phase 1 (0-60s)**: Optimal capacity test with 5 workers
  - **Phase 2 (60-120s)**: Capacity reduction stress test with 2 workers
  - **Phase 3 (120-150s)**: Gradual scale-up (+1 worker every 10s)
  - **Phase 4 (150-240s)**: Completion monitoring with full capacity
  - Tests system's ability to handle worker scaling operations while maintaining 100% task completion
  - Demonstrates resilience during resource constraints and validates scaling behavior
  - Success criteria: 100% completion within 4 minutes total

## Task Type Registration System

**BREAKING CHANGE**: As of recent updates, the system requires task type registration before tasks can be created.

### Key Changes:
- **API Validation**: `POST /tasks` now validates task types against registered handlers
- **Worker Registration**: Workers automatically register task types on startup via `POST /tasks/types`
- **New Endpoints**: 
  - `GET /tasks/types` - List registered task types (public)
  - `POST /tasks/types` - Register task type (public, used by workers)
- **Test Updates**: Integration tests now use `TestDataFactory::new_with_task_types()` for automatic registration
- **Error Handling**: Unregistered task types return 400 validation errors instead of 200/201

### Impact on Development:
- **Start workers before creating tasks** - API will reject tasks for unregistered types
- **Tests updated** - All 53 integration tests pass with new validation
- **Docker-based chaos testing** - All scenarios now run in isolated containers with proper resource limits