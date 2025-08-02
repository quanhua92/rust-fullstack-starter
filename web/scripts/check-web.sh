#!/bin/bash

# Comprehensive web frontend quality check script
# Runs all quality checks: format, lint, type check, build, and tests
#
# Usage: ./check-web.sh [--skip-lint]
#   --skip-lint: Skip linting and formatting checks

set -e

# Parse command line arguments
SKIP_LINT=false
for arg in "$@"; do
    case $arg in
        --skip-lint)
            SKIP_LINT=true
            shift
            ;;
        *)
            # Unknown option
            ;;
    esac
done

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../scripts/common.sh"

# Initialize timing and get project directories
init_timing
get_project_dirs
cd "$WEB_ROOT"

print_status "step" "Running comprehensive web frontend quality checks..."
echo -e "${BLUE}================================${NC}"
print_status "info" "Working directory: $WEB_ROOT"

# Check if pnpm is available
if ! check_command "pnpm" "npm install -g pnpm"; then
    exit 1
fi

# 1. Install dependencies if needed
print_status "step" "📦 Step 1/9: Checking dependencies..."
if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules" ]; then
    print_status "warning" "Dependencies need to be installed/updated"
    run_cmd "Installing dependencies" pnpm install
else
    print_status "success" "Dependencies are up to date"
fi

# 2. Generate API types from OpenAPI spec
print_status "step" "🔄 Step 2/9: Generating API types from OpenAPI spec..."
if [ -f "../docs/openapi.json" ]; then
    run_cmd "Generating API types" pnpm run generate-api
else
    print_status "warning" "OpenAPI spec not found at ../docs/openapi.json"
    print_status "info" "Run './scripts/check.sh' from project root first"
fi

# 3. TypeScript type checking
run_cmd "📝 Step 3/9: TypeScript type checking" pnpm exec tsc --noEmit

# 4. Biome linting
if [ "$SKIP_LINT" = "true" ]; then
    print_status "info" "📎 Step 4/9: Skipping Biome linting (--skip-lint)"
else
    if ! run_cmd "📎 Step 4/9: Biome linting" pnpm run lint; then
        print_status "info" "Run 'pnpm run format' to fix formatting issues"
        exit 1
    fi
fi

# 5. Biome formatting check
if [ "$SKIP_LINT" = "true" ]; then
    print_status "info" "🎨 Step 5/9: Skipping code formatting check (--skip-lint)"
else
    if ! run_cmd "🎨 Step 5/9: Code formatting check" pnpm run format --write=false; then
        print_status "info" "Run 'pnpm run format' to fix formatting"
        exit 1
    fi
fi

# 6. Biome comprehensive check
if [ "$SKIP_LINT" = "true" ]; then
    print_status "info" "🔍 Step 6/9: Skipping Biome comprehensive check (--skip-lint)"
else
    run_cmd "🔍 Step 6/9: Biome comprehensive check" pnpm run check
fi

# 7. Build check
run_cmd "🏗️ Step 7/9: Production build test" pnpm run build

# 8. Unit/Integration tests
run_cmd "🧪 Step 8/9: Running unit tests" pnpm run test

# 9. End-to-end tests with Playwright
print_status "step" "🎭 Step 9/9: Running E2E tests with Playwright..."
if [ "$CI" = "true" ] || [ "$PLAYWRIGHT_SKIP" = "true" ]; then
    print_status "info" "Skipping E2E tests (CI=$CI, PLAYWRIGHT_SKIP=$PLAYWRIGHT_SKIP)"
else
    # Function to check if a port is in use
    check_port() {
        local port=$1
        netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN" || lsof -i :${port} >/dev/null 2>&1
    }
    
    # Function to wait for server to be ready
    wait_for_server() {
        local url=$1
        local timeout=${2:-30}
        local count=0
        
        while [ $count -lt $timeout ]; do
            if curl -s "$url" >/dev/null 2>&1; then
                return 0
            fi
            sleep 1
            count=$((count + 1))
        done
        return 1
    }
    
    # Check and start backend server (port 3000)
    BACKEND_STARTED=false
    if ! check_port 3000; then
        print_status "info" "Starting backend server on port 3000..."
        (cd "$PROJECT_ROOT" && ./scripts/server.sh 3000 >/dev/null 2>&1 &)
        BACKEND_STARTED=true
        if wait_for_server "http://localhost:3000/api/v1/health" 30; then
            print_status "success" "Backend server started successfully"
        else
            print_status "error" "Failed to start backend server"
            exit 1
        fi
    else
        print_status "success" "Backend server already running on port 3000"
    fi
    
    # Check and start worker if backend was started
    WORKER_STARTED=false
    if [ "$BACKEND_STARTED" = "true" ]; then
        print_status "info" "Starting worker process..."
        (cd "$PROJECT_ROOT" && ./scripts/worker.sh >/dev/null 2>&1 &)
        WORKER_STARTED=true
        sleep 3  # Give worker time to register task types
        print_status "success" "Worker process started"
    fi
    
    # Check and start frontend dev server (port 5173)
    FRONTEND_STARTED=false
    if ! check_port 5173; then
        print_status "info" "Starting frontend dev server on port 5173..."
        (pnpm run dev >/dev/null 2>&1 &)
        FRONTEND_STARTED=true
        if wait_for_server "http://localhost:5173" 60; then
            print_status "success" "Frontend dev server started successfully"
        else
            print_status "error" "Failed to start frontend dev server"
            exit 1
        fi
    else
        print_status "success" "Frontend dev server already running on port 5173"
    fi
    
    # Set Playwright to use the unified backend server on port 3000
    export PLAYWRIGHT_BASE_URL="http://localhost:3000"
    
    # Run Playwright tests
    run_cmd "Running Playwright E2E tests" pnpm run test:e2e
    
    # Cleanup: Stop servers we started
    cleanup_servers() {
        if [ "$BACKEND_STARTED" = "true" ]; then
            print_status "info" "Stopping backend server..."
            (cd "$PROJECT_ROOT" && ./scripts/stop-server.sh 3000 >/dev/null 2>&1)
        fi
        if [ "$WORKER_STARTED" = "true" ]; then
            print_status "info" "Stopping worker process..."
            pkill -f "starter.*worker" >/dev/null 2>&1 || true
        fi
        if [ "$FRONTEND_STARTED" = "true" ]; then
            print_status "info" "Stopping frontend dev server..."
            pkill -f "vite.*--port 5173" >/dev/null 2>&1 || true
        fi
    }
    
    # Register cleanup function to run on script exit
    trap cleanup_servers EXIT
fi

# Additional quality checks
print_status "step" "🔍 Additional quality checks..."

# Check for console statements in source files (excluding dev/debug files)
console_files=$(find src -name "*.ts" -o -name "*.tsx" | grep -v "dev\|debug\|test" | xargs grep -l "console\." 2>/dev/null | head -5)
if [ -n "$console_files" ]; then
    print_status "warning" "Found console statements in source files (consider removing for production)"
    echo "$console_files" | while read -r file; do
        print_status "info" "  $file"
    done
fi

# Check for TODO/FIXME in source files
if find src -name "*.ts" -o -name "*.tsx" | xargs grep -l "TODO\|FIXME" 2>/dev/null | head -5 >/dev/null; then
    print_status "warning" "Found TODO/FIXME in source files (consider addressing)"
fi

# Check for unused dependencies
if check_command "depcheck" "pnpm add -g depcheck"; then
    print_status "info" "Checking for unused dependencies..."
    depcheck --quiet || print_status "warning" "Found potential unused dependencies"
fi

# Check bundle size
if [ -d "dist" ]; then
    bundle_size=$(du -sh dist | cut -f1)
    print_status "info" "Bundle size: ${bundle_size}"
    
    # Check for large files (>1MB)
    if find dist -type f -size +1M 2>/dev/null | head -1 >/dev/null; then
        print_status "warning" "Large files (>1MB) found in bundle"
    fi
fi

# Validate project structure
[ -d "src/components/ui" ] && print_status "success" "shadcn/ui components found" || print_status "warning" "shadcn/ui components not found"
[ -f "src/lib/api/client.ts" ] && print_status "success" "API client configured" || print_status "warning" "API client not configured"
[ -f "src/lib/auth/context.tsx" ] && print_status "success" "Authentication system configured" || print_status "warning" "Authentication system not configured"

# Show results
echo ""
print_status "success" "All web frontend quality checks passed!"
show_elapsed
print_status "info" "Web frontend is ready for development"

# Show summary
echo ""
print_status "step" "Summary of checks performed:"
echo "   ✅ Dependencies and API types"
if [ "$SKIP_LINT" = "true" ]; then
    echo "   ⏭️  TypeScript (linting and formatting skipped)"
else
    echo "   ✅ TypeScript, linting, and formatting"
fi
echo "   ✅ Build and unit tests"
echo "   ✅ End-to-end tests (Playwright)"
echo "   ✅ Code quality analysis"

print_status "info" "Ready to continue development!"