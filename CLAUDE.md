# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Constraints and Guidelines

- **Project Nature Guidance:**
  * TONE DOWN with STARTER NATURE OF THE PROJECT
  * NEVER SAY PRODUCTION OR ENTERPRISE READY

## Testing Commands

- **Integration Tests**: `cd starter && cargo nextest run` (40 tests, ~10 seconds)
- **API Testing**: `./scripts/test-with-curl.sh [host] [port]` (29 endpoint tests)
  - Default: `./scripts/test-with-curl.sh` (localhost:3000)
  - Custom: `./scripts/test-with-curl.sh localhost 8080`
  - HTTPS: `./scripts/test-with-curl.sh api.example.com 443`
- **Server Management**: 
  - Start: `./scripts/server.sh [port]` (default port 3000)
  - Stop: `./scripts/stop-server.sh [port]`
  - Worker: `./scripts/worker.sh`

## Health Endpoints

Available health check endpoints:
- `/health` - Basic health check (status, version, uptime)
- `/health/detailed` - Detailed health with dependency checks
- `/health/live` - Kubernetes liveness probe (minimal checks)
- `/health/ready` - Kubernetes readiness probe (dependency validation)
- `/health/startup` - Kubernetes startup probe (initialization checks)

## Project Scripts

Key development scripts in `/scripts/`:
- `server.sh` - Start development server with custom port
- `worker.sh` - Start background task worker
- `test-with-curl.sh` - Comprehensive API endpoint testing
- `reset-all.sh` - Database reset (requires `--reset-database` flag)
- `rename-project.sh` - Automated project renaming with validation
- `deploy-prod.sh` - Production deployment with Docker
- `dev-server.sh` - Complete development environment setup

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
2. **Run Tests**: `cargo nextest run` (40 integration tests)
3. **API Testing**: `./scripts/test-with-curl.sh` (29 endpoint tests)
4. **Stop Services**: `./scripts/stop-server.sh 3000`