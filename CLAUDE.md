# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Constraints and Guidelines

- **Project Nature Guidance:**
  * TONE DOWN with STARTER NATURE OF THE PROJECT
  * NEVER SAY PRODUCTION OR ENTERPRISE READY

## Testing Commands

- **Integration Tests**: `cd starter && cargo nextest run` (51 tests, ~12 seconds)
- **API Testing**: `./scripts/test-with-curl.sh [host] [port]` (38 endpoint tests)
  - Default: `./scripts/test-with-curl.sh` (localhost:3000)
  - Custom: `./scripts/test-with-curl.sh localhost 8080`
  - HTTPS: `./scripts/test-with-curl.sh api.example.com 443`
- **Chaos Testing**: `./scripts/test-chaos.sh [options]` (resilience testing with failure simulation)
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
- **Features**: All endpoints documented with request/response examples, authentication support, type definitions
- **Client Generation**: Use schema to generate TypeScript, Python, or other language clients

## Project Scripts

Key development scripts in `/scripts/`:
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
- `service-chaos.sh` - Simulate service failures (server, worker, database)

## Production Features

This starter includes production-ready infrastructure:
- **Docker Configuration**: Multi-stage builds with distroless runtime (`Dockerfile.prod`, `docker-compose.prod.yaml`)
- **Kubernetes Support**: Health probes, deployment manifests, and persistent storage (`k8s/`)
- **CI/CD Pipelines**: GitHub Actions workflows for testing, security, building, and deployment (`.github/workflows/`)
- **Security Scanning**: Automated dependency audits, secret detection, and vulnerability scanning
- **Automated Deployment**: Production deployment scripts with validation and backups
- **Project Customization**: Automated project renaming with comprehensive validation

## Development Workflow

1. **Start Services**: `./scripts/dev-server.sh 3000` (complete environment)
   - Or manually: `./scripts/server.sh && ./scripts/worker.sh`
2. **Run Tests**: `cargo nextest run` (51 integration tests)
3. **API Testing**: `./scripts/test-with-curl.sh` (38 endpoint tests)
4. **Chaos Testing**: `./scripts/test-chaos.sh` (resilience validation)
5. **Stop Services**: `./scripts/stop-server.sh 3000`

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