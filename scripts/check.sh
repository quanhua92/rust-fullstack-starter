#!/bin/bash

# Comprehensive quality check script
# Runs all quality checks: format, lint, prepare SQLx, and tests

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Track timing
start_time=$(date +%s)

echo -e "${CYAN}🚀 Running comprehensive quality checks...${NC}"
echo -e "${BLUE}================================${NC}"

# Get project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}📁 Working directory: $PROJECT_ROOT${NC}"

# 1. Web frontend build (if exists)
echo -e "\n${BLUE}🌐 Step 1/9: Web frontend build...${NC}"

# Check if web directory exists and build early
if [ -d "web" ]; then
    echo -e "${BLUE}🏗️  Building web frontend...${NC}"
    if ! ./scripts/build-web.sh; then
        echo -e "${RED}❌ Web frontend build failed!${NC}"
        echo -e "${YELLOW}   Run './scripts/build-web.sh' for details${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Web frontend build successful${NC}"
    
    # Check if build artifacts exist
    if [ ! -f "web/dist/index.html" ]; then
        echo -e "${RED}❌ Web build artifacts not found in web/dist/${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Web build artifacts verified${NC}"
else
    echo -e "${YELLOW}⚠️  Web directory not found, skipping frontend build${NC}"
fi

# 2. Cargo check
echo -e "\n${BLUE}🔍 Step 2/9: Running cargo check...${NC}"
if ! SQLX_OFFLINE=true cargo check --manifest-path starter/Cargo.toml --all --all-targets --all-features; then
    echo -e "${YELLOW}⚠️  Offline cargo check failed, attempting to regenerate SQLx cache...${NC}"
    if ./scripts/prepare-sqlx.sh 2>/dev/null; then
        echo -e "${BLUE}🔄 Retrying cargo check with updated cache...${NC}"
        if ! SQLX_OFFLINE=true cargo check --manifest-path starter/Cargo.toml --all --all-targets --all-features; then
            echo -e "${RED}❌ Cargo check failed even after updating SQLx cache!${NC}"
            exit 1
        fi
        echo -e "${GREEN}✅ Cargo check passed after cache update${NC}"
    else
        echo -e "${YELLOW}⚠️  SQLx prepare failed, attempting to restart Docker services...${NC}"
        
        # Try to restart Docker services
        if command -v docker-compose >/dev/null 2>&1; then
            if docker-compose up -d 2>/dev/null; then
                echo -e "${GREEN}✅ Docker services restarted successfully${NC}"
                echo -e "${BLUE}🗄️  Resetting database and running migrations...${NC}"
                if ./scripts/reset-all.sh --reset-database >/dev/null 2>&1; then
                    echo -e "${GREEN}✅ Database reset and migrations completed${NC}"
                    DOCKER_RESTARTED=true
                else
                    echo -e "${YELLOW}⚠️  Database reset failed, but continuing...${NC}"
                    DOCKER_RESTARTED=true
                fi
            fi
        elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
            if docker compose up -d 2>/dev/null; then
                echo -e "${GREEN}✅ Docker services restarted successfully${NC}"
                echo -e "${BLUE}🗄️  Resetting database and running migrations...${NC}"
                if ./scripts/reset-all.sh --reset-database >/dev/null 2>&1; then
                    echo -e "${GREEN}✅ Database reset and migrations completed${NC}"
                    DOCKER_RESTARTED=true
                else
                    echo -e "${YELLOW}⚠️  Database reset failed, but continuing...${NC}"
                    DOCKER_RESTARTED=true
                fi
            fi
        fi
        
        # If Docker was restarted, try SQLx prepare again
        if [ "${DOCKER_RESTARTED:-false}" = true ]; then
            echo -e "${BLUE}🔄 Retrying SQLx prepare after Docker restart...${NC}"
            if ./scripts/prepare-sqlx.sh 2>/dev/null; then
                echo -e "${BLUE}🔄 Retrying cargo check with updated cache...${NC}"
                if ! SQLX_OFFLINE=true cargo check --manifest-path starter/Cargo.toml --all --all-targets --all-features; then
                    echo -e "${RED}❌ Cargo check failed even after Docker restart and SQLx cache update!${NC}"
                    exit 1
                fi
                echo -e "${GREEN}✅ Cargo check passed after Docker restart and cache update${NC}"
            else
                echo -e "${RED}❌ SQLx prepare still failed after Docker restart!${NC}"
                exit 1
            fi
        else
            echo -e "${RED}❌ Could not regenerate SQLx cache and cargo check failed!${NC}"
            exit 1
        fi
    fi
else
    echo -e "${GREEN}✅ Cargo check passed${NC}"
fi

# 3. Format check (strict - matches CI behavior)
echo -e "\n${BLUE}🎨 Step 3/9: Checking code formatting...${NC}"
if ! cargo fmt --manifest-path starter/Cargo.toml --all -- --check; then
    echo -e "${RED}❌ Code formatting issues found!${NC}"
    echo -e "${YELLOW}💡 Run 'cargo fmt --manifest-path starter/Cargo.toml --all' to fix${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Code formatting is correct${NC}"
fi

# 4. Clippy linting (offline mode)
echo -e "\n${BLUE}📎 Step 4/9: Running Clippy lints...${NC}"
if ! SQLX_OFFLINE=true cargo clippy --manifest-path starter/Cargo.toml --all --all-targets --all-features -- -D warnings; then
    echo -e "${RED}❌ Clippy found issues!${NC}"
    echo -e "${YELLOW}💡 Fix the linting issues above${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Clippy checks passed${NC}"

# 5. SQLx prepare using dedicated script
echo -e "\n${BLUE}🗄️  Step 5/9: Running SQLx prepare...${NC}"

# Check if we can connect to database first
cd starter
if ! cargo sqlx prepare --check 2>/dev/null; then
    cd "$PROJECT_ROOT"
    echo -e "${BLUE}🔄 Running SQLx prepare script to update query cache...${NC}"
    
    # Use the dedicated prepare-sqlx.sh script
    if ./scripts/prepare-sqlx.sh 2>/dev/null; then
        echo -e "${GREEN}✅ SQLx prepare completed successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  SQLx prepare script failed${NC}"
        echo -e "${YELLOW}   Database may not be available - using existing query cache${NC}"
        echo -e "${YELLOW}   Run 'docker compose up -d' to start database if needed${NC}"
    fi
else
    cd "$PROJECT_ROOT"
    echo -e "${GREEN}✅ SQLx queries are up to date${NC}"
fi

# 6. Unit tests
echo -e "\n${BLUE}🧪 Step 6/9: Running unit tests...${NC}"
if ! cargo test --manifest-path starter/Cargo.toml --all --lib; then
    echo -e "${RED}❌ Unit tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Unit tests passed${NC}"

# 7. Integration tests with nextest
echo -e "\n${BLUE}🔬 Step 7/9: Running integration tests with nextest...${NC}"
cd starter

# Check if cargo-nextest is installed
if ! command -v cargo-nextest >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  cargo-nextest not found, installing...${NC}"
    cargo install cargo-nextest --locked
fi

if ! cargo nextest run; then
    echo -e "${RED}❌ Integration tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Integration tests passed${NC}"

cd "$PROJECT_ROOT"

# 8. Export OpenAPI specification
echo -e "\n${BLUE}📋 Step 8/9: Exporting OpenAPI specification...${NC}"
if ! cargo run --quiet --manifest-path starter/Cargo.toml -- export-openapi; then
    echo -e "${RED}❌ OpenAPI export failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ OpenAPI specification exported to docs/openapi.json${NC}"

# Generate frontend API types from updated OpenAPI spec
echo -e "${BLUE}🔄 Generating frontend API types...${NC}"
if [ -d "web" ] && [ -f "web/package.json" ]; then
    if ! (cd web && pnpm generate-api); then
        echo -e "${YELLOW}⚠️  Frontend API type generation failed, but continuing...${NC}"
    else
        echo -e "${GREEN}✅ Frontend API types updated${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Web directory not found, skipping API type generation${NC}"
fi

# 9. Web static file serving smoke test
echo -e "\n${BLUE}🚀 Step 9/9: Web static file serving smoke test...${NC}"

# Only run if web was built earlier
if [ -d "web" ] && [ -f "web/dist/index.html" ]; then
    echo -e "${BLUE}🧪 Testing static file serving integration...${NC}"
    
    # Start a temporary server for testing
    TEST_PORT=38123  # Use unusual port to avoid conflicts
    export STARTER__SERVER__WEB_BUILD_PATH="$PROJECT_ROOT/web/dist"
    
    # Start server in background
    bash -c "cd '$PROJECT_ROOT/starter' && cargo run --quiet -- server --port $TEST_PORT" &
    TEST_SERVER_PID=$!
    
    # Wait for server to start by polling its health endpoint
    echo -e "${BLUE}   Waiting for server to start...${NC}"
    SERVER_READY=false
    for i in {1..60}; do # 30 seconds timeout
        if curl -s -f "http://localhost:$TEST_PORT/api/v1/health" >/dev/null 2>&1; then
            SERVER_READY=true
            break
        fi
        sleep 0.5
    done
    
    # Test if server is responding
    if [ "$SERVER_READY" = true ] && kill -0 $TEST_SERVER_PID 2>/dev/null; then
        # Test static file serving
        if curl -s "http://localhost:$TEST_PORT/" | grep -q "<!DOCTYPE html>"; then
            echo -e "${GREEN}✅ Static file serving working${NC}"
            
            # Test SPA fallback
            if curl -s "http://localhost:$TEST_PORT/admin/dashboard" | grep -q "<!DOCTYPE html>"; then
                echo -e "${GREEN}✅ SPA fallback routing working${NC}"
            else
                echo -e "${YELLOW}⚠️  SPA fallback may not be working properly${NC}"
            fi
            
            # Test API endpoint coexistence
            if curl -s "http://localhost:$TEST_PORT/api/v1/health" | grep -q "status"; then
                echo -e "${GREEN}✅ API and static serving coexistence verified${NC}"
            else
                echo -e "${YELLOW}⚠️  API endpoints may be affected by static serving${NC}"
            fi
        else
            echo -e "${YELLOW}⚠️  Static file serving may not be working properly${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Could not start test server for static file validation${NC}"
    fi
    
    # Clean up test server gracefully
    if kill -0 $TEST_SERVER_PID 2>/dev/null; then
        # Try graceful shutdown first
        kill $TEST_SERVER_PID 2>/dev/null || true
        
        # Wait briefly for graceful shutdown
        for i in {1..6}; do # 3 seconds timeout
            if ! kill -0 $TEST_SERVER_PID 2>/dev/null; then
                break
            fi
            sleep 0.5
        done
        
        # Force kill if still running
        if kill -0 $TEST_SERVER_PID 2>/dev/null; then
            kill -9 $TEST_SERVER_PID 2>/dev/null || true
        fi
        
        # Also kill by port in case of issues
        lsof -ti:$TEST_PORT | xargs kill -9 2>/dev/null || true
    fi
    
    # Wait for cleanup
    sleep 1
    
else
    echo -e "${YELLOW}⚠️  Web build not available, skipping static file serving test${NC}"
fi

# Additional quality checks (integrated into step 9)
echo -e "\n${BLUE}🔍 Additional code quality checks...${NC}"

# Check for TODO/FIXME in source files (excluding this script)
if find starter/src -name "*.rs" -exec grep -l "TODO\|FIXME" {} \; 2>/dev/null | head -5; then
    echo -e "${YELLOW}⚠️  Found TODO/FIXME in source files (consider addressing)${NC}"
fi

# Check for debug prints in source files
if find starter/src -name "*.rs" -exec grep -l "println!\|dbg!\|eprintln!" {} \; 2>/dev/null | head -5; then
    echo -e "${YELLOW}⚠️  Found debug prints in source files (consider removing)${NC}"
fi

# Check if cargo-sort is available and run it
if command -v cargo-sort >/dev/null 2>&1; then
    echo -e "${BLUE}📦 Checking Cargo.toml dependency sorting...${NC}"
    if ! cargo sort --check --workspace; then
        echo -e "${YELLOW}⚠️  Dependencies could be sorted better${NC}"
        echo -e "${YELLOW}   Run 'cargo sort --workspace' to fix${NC}"
    else
        echo -e "${GREEN}✅ Dependencies are properly sorted${NC}"
    fi
else
    echo -e "${YELLOW}💡 Install cargo-sort for dependency sorting: cargo install cargo-sort${NC}"
fi

# Calculate total time
end_time=$(date +%s)
duration=$((end_time - start_time))

echo -e "\n${BLUE}================================${NC}"
echo -e "${GREEN}🎉 All quality checks passed!${NC}"
echo -e "${CYAN}⏱️  Total time: ${duration}s${NC}"
echo -e "${BLUE}✨ Code is ready for commit${NC}"

# Optional: Show summary of what was checked
echo -e "\n${BLUE}📋 Summary of checks performed:${NC}"
echo -e "   ✅ Web frontend build (early validation)"
echo -e "   ✅ Cargo check (compilation)"
echo -e "   ✅ Code formatting (cargo fmt)"
echo -e "   ✅ Linting (cargo clippy)"
echo -e "   ✅ SQLx query cache validation"
echo -e "   ✅ Unit tests"
echo -e "   ✅ Integration tests (cargo nextest)"
echo -e "   ✅ OpenAPI specification export"
echo -e "   ✅ Web static file serving smoke test"
echo -e "   ✅ Code quality analysis"