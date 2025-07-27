# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Constraints and Guidelines

- **Project Nature Guidance:**
  * TONE DOWN with STARTER NATURE OF THE PROJECT
  * NEVER SAY PRODUCTION OR ENTERPRISE READY

## Testing Commands

- **Integration Tests**: `cd starter && cargo nextest run` (38 tests, ~10 seconds)
- **API Testing**: `./scripts/test-with-curl.sh [host] [port]` (26 endpoint tests)
  - Default: `./scripts/test-with-curl.sh` (localhost:3000)
  - Custom: `./scripts/test-with-curl.sh localhost 8080`
  - HTTPS: `./scripts/test-with-curl.sh api.example.com 443`
- **Server Management**: 
  - Start: `./scripts/server.sh [port]` (default port 3000)
  - Stop: `./scripts/stop-server.sh [port]`
  - Worker: `./scripts/worker.sh`

## Project Scripts

Key development scripts in `/scripts/`:
- `server.sh` - Start development server with custom port
- `worker.sh` - Start background task worker
- `test-with-curl.sh` - Comprehensive API endpoint testing
- `reset-all.sh` - Database reset (requires `--reset-database` flag)

## Development Workflow

1. **Start Services**: `./scripts/server.sh && ./scripts/worker.sh`
2. **Run Tests**: `cd starter && cargo nextest run`
3. **API Testing**: `./scripts/test-with-curl.sh`
4. **Stop Services**: `./scripts/stop-server.sh`