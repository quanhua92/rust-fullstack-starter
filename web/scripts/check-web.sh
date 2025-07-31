#!/bin/bash

# Comprehensive web frontend quality check script
# Runs all quality checks: format, lint, type check, build, and tests

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

echo -e "${CYAN}🚀 Running comprehensive web frontend quality checks...${NC}"
echo -e "${BLUE}================================${NC}"

# Get project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$WEB_ROOT/.." && pwd)"
cd "$WEB_ROOT"

echo -e "${BLUE}📁 Working directory: $WEB_ROOT${NC}"

# Check if pnpm is available
if ! command -v pnpm >/dev/null 2>&1; then
    echo -e "${RED}❌ pnpm is required but not installed!${NC}"
    echo -e "${YELLOW}💡 Install pnpm: npm install -g pnpm${NC}"
    exit 1
fi

# 1. Install dependencies if needed
echo -e "\n${BLUE}📦 Step 1/9: Checking dependencies...${NC}"
if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules" ]; then
    echo -e "${YELLOW}⚠️  Dependencies need to be installed/updated${NC}"
    if ! pnpm install; then
        echo -e "${RED}❌ Failed to install dependencies!${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Dependencies installed${NC}"
else
    echo -e "${GREEN}✅ Dependencies are up to date${NC}"
fi

# 2. Generate API types from OpenAPI spec
echo -e "\n${BLUE}🔄 Step 2/9: Generating API types from OpenAPI spec...${NC}"
if [ -f "../docs/openapi.json" ]; then
    if ! pnpm run generate-api; then
        echo -e "${RED}❌ Failed to generate API types!${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ API types generated successfully${NC}"
else
    echo -e "${YELLOW}⚠️  OpenAPI spec not found at ../docs/openapi.json${NC}"
    echo -e "${YELLOW}   Run './scripts/check.sh' from project root first${NC}"
fi

# 3. TypeScript type checking
echo -e "\n${BLUE}📝 Step 3/9: Running TypeScript type checking...${NC}"
if ! pnpm exec tsc --noEmit; then
    echo -e "${RED}❌ TypeScript type checking failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ TypeScript type checking passed${NC}"

# 5. Biome linting
echo -e "\n${BLUE}📎 Step 5/10: Running Biome linting...${NC}"
if ! pnpm run lint; then
    echo -e "${RED}❌ Biome linting failed!${NC}"
    echo -e "${YELLOW}💡 Run 'pnpm run format' to fix formatting issues${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Biome linting passed${NC}"

# 6. Biome formatting check
echo -e "\n${BLUE}🎨 Step 6/10: Checking code formatting...${NC}"
if ! pnpm run format --write=false; then
    echo -e "${RED}❌ Code formatting issues found!${NC}"
    echo -e "${YELLOW}💡 Run 'pnpm run format' to fix formatting${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Code formatting is correct${NC}"

# 7. Biome comprehensive check
echo -e "\n${BLUE}🔍 Step 7/10: Running Biome comprehensive check...${NC}"
if ! pnpm run check; then
    echo -e "${RED}❌ Biome comprehensive check failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Biome comprehensive check passed${NC}"

# 8. Build check
echo -e "\n${BLUE}🏗️  Step 8/10: Testing production build...${NC}"
if ! pnpm run build; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Production build successful${NC}"

# 9. Unit/Integration tests
echo -e "\n${BLUE}🧪 Step 9/10: Running tests...${NC}"
if ! pnpm run test; then
    echo -e "${RED}❌ Tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ All tests passed${NC}"

# 10. Additional quality checks
echo -e "\n${BLUE}🔍 Step 10/10: Additional quality checks...${NC}"

# Check for console.log in source files (excluding dev/debug files)
if find src -name "*.ts" -o -name "*.tsx" | grep -v "dev\|debug\|test" | xargs grep -l "console\.log\|console\.warn\|console\.error" 2>/dev/null | head -5; then
    echo -e "${YELLOW}⚠️  Found console statements in source files (consider removing for production)${NC}"
fi

# Check for TODO/FIXME in source files
if find src -name "*.ts" -o -name "*.tsx" | xargs grep -l "TODO\|FIXME" 2>/dev/null | head -5; then
    echo -e "${YELLOW}⚠️  Found TODO/FIXME in source files (consider addressing)${NC}"
fi

# Check for unused dependencies (if depcheck is available)
if command -v depcheck >/dev/null 2>&1; then
    echo -e "${BLUE}📦 Checking for unused dependencies...${NC}"
    if ! depcheck --quiet; then
        echo -e "${YELLOW}⚠️  Found potential unused dependencies${NC}"
        echo -e "${YELLOW}   Review the output above and consider removing unused packages${NC}"
    else
        echo -e "${GREEN}✅ No unused dependencies found${NC}"
    fi
else
    echo -e "${YELLOW}💡 Install depcheck for dependency analysis: pnpm add -g depcheck${NC}"
fi

# Check bundle size (if built)
if [ -d "dist" ]; then
    echo -e "${BLUE}📊 Analyzing bundle size...${NC}"
    bundle_size=$(du -sh dist | cut -f1)
    echo -e "${CYAN}📦 Bundle size: ${bundle_size}${NC}"
    
    # Check for large files in bundle
    large_files=$(find dist -type f -size +1M 2>/dev/null | head -3)
    if [ -n "$large_files" ]; then
        echo -e "${YELLOW}⚠️  Large files in bundle (>1MB):${NC}"
        echo "$large_files" | while read -r file; do
            size=$(du -sh "$file" | cut -f1)
            echo -e "${YELLOW}   $file ($size)${NC}"
        done
    fi
fi

# Check for proper environment configuration
if [ ! -f ".env.example" ] && [ ! -f ".env.local" ]; then
    echo -e "${YELLOW}⚠️  No environment configuration examples found${NC}"
    echo -e "${YELLOW}   Consider adding .env.example for environment variable documentation${NC}"
fi

# Validate shadcn components are properly installed
if [ ! -d "src/components/ui" ]; then
    echo -e "${YELLOW}⚠️  shadcn/ui components directory not found${NC}"
else
    ui_components=$(find src/components/ui -name "*.tsx" | wc -l)
    echo -e "${CYAN}🎨 Found ${ui_components} shadcn/ui components${NC}"
fi

# Check if API client is properly configured
if [ -f "src/lib/api/client.ts" ]; then
    echo -e "${GREEN}✅ API client configuration found${NC}"
else
    echo -e "${YELLOW}⚠️  API client configuration not found${NC}"
fi

# Check if authentication system is set up
if [ -f "src/lib/auth/context.tsx" ]; then
    echo -e "${GREEN}✅ Authentication system configured${NC}"
else
    echo -e "${YELLOW}⚠️  Authentication system not configured${NC}"
fi

# Calculate total time
end_time=$(date +%s)
duration=$((end_time - start_time))

echo -e "\n${BLUE}================================${NC}"
echo -e "${GREEN}🎉 All web frontend quality checks passed!${NC}"
echo -e "${CYAN}⏱️  Total time: ${duration}s${NC}"
echo -e "${BLUE}✨ Web frontend is ready for development${NC}"

# Optional: Show summary of what was checked
echo -e "\n${BLUE}📋 Summary of checks performed:${NC}"
echo -e "   ✅ Dependencies installation/validation"
echo -e "   ✅ API types generation from OpenAPI spec"
echo -e "   ✅ TypeScript type checking"
echo -e "   ✅ Biome linting"
echo -e "   ✅ Code formatting (Biome)"
echo -e "   ✅ Comprehensive code quality (Biome)"
echo -e "   ✅ Production build"
echo -e "   ✅ Unit/Integration tests"
echo -e "   ✅ Code quality analysis"

echo -e "\n${CYAN}🚀 Ready to continue with Phase 2 development!${NC}"