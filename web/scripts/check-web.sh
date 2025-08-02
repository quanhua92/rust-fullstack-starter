#!/bin/bash

# Comprehensive web frontend quality check script
# Runs all quality checks: format, lint, type check, build, and tests

set -e

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
print_status "step" "📦 Step 1/8: Checking dependencies..."
if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules" ]; then
    print_status "warning" "Dependencies need to be installed/updated"
    run_cmd "Installing dependencies" pnpm install
else
    print_status "success" "Dependencies are up to date"
fi

# 2. Generate API types from OpenAPI spec
print_status "step" "🔄 Step 2/8: Generating API types from OpenAPI spec..."
if [ -f "../docs/openapi.json" ]; then
    run_cmd "Generating API types" pnpm run generate-api
else
    print_status "warning" "OpenAPI spec not found at ../docs/openapi.json"
    print_status "info" "Run './scripts/check.sh' from project root first"
fi

# 3. TypeScript type checking
run_cmd "📝 Step 3/8: TypeScript type checking" pnpm exec tsc --noEmit

# 4. Biome linting
if ! run_cmd "📎 Step 4/8: Biome linting" pnpm run lint; then
    print_status "info" "Run 'pnpm run format' to fix formatting issues"
    exit 1
fi

# 5. Biome formatting check
if ! run_cmd "🎨 Step 5/8: Code formatting check" pnpm run format --write=false; then
    print_status "info" "Run 'pnpm run format' to fix formatting"
    exit 1
fi

# 6. Biome comprehensive check
run_cmd "🔍 Step 6/8: Biome comprehensive check" pnpm run check

# 7. Build check
run_cmd "🏗️ Step 7/8: Production build test" pnpm run build

# 8. Unit/Integration tests
run_cmd "🧪 Step 8/8: Running tests" pnpm run test

# Additional quality checks
print_status "step" "🔍 Additional quality checks..."

# Check for console statements in source files (excluding dev/debug files)
if find src -name "*.ts" -o -name "*.tsx" | grep -v "dev\|debug\|test" | xargs grep -l "console\." 2>/dev/null | head -5 >/dev/null; then
    print_status "warning" "Found console statements in source files (consider removing for production)"
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
echo "   ✅ TypeScript, linting, and formatting"
echo "   ✅ Build and tests"
echo "   ✅ Code quality analysis"

print_status "info" "Ready to continue development!"