#!/bin/bash

# Build web frontend script
# Builds the React/TypeScript frontend into static files

set -e

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Initialize timing
init_timing

print_status "step" "Building web frontend..."
echo -e "${BLUE}================================${NC}"

# Get project directories (SCRIPT_DIR already set above)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WEB_DIR="$PROJECT_ROOT/web"

echo -e "${BLUE}📁 Project root: $PROJECT_ROOT${NC}"
echo -e "${BLUE}📁 Web directory: $WEB_DIR${NC}"

# Check if web directory exists
if [ ! -d "$WEB_DIR" ]; then
    echo -e "${RED}❌ Web directory not found at: $WEB_DIR${NC}"
    exit 1
fi

cd "$WEB_DIR"

# Check if package.json exists
if [ ! -f "package.json" ]; then
    echo -e "${RED}❌ package.json not found in web directory${NC}"
    exit 1
fi

# Check if pnpm is installed
if ! command -v pnpm >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  pnpm not found, attempting to install...${NC}"
    if command -v npm >/dev/null 2>&1; then
        npm install -g pnpm
    else
        echo -e "${RED}❌ npm not found. Please install Node.js and npm first${NC}"
        exit 1
    fi
fi

# Install dependencies
echo -e "\n${BLUE}📦 Installing dependencies...${NC}"
if ! pnpm install --frozen-lockfile; then
    echo -e "${YELLOW}⚠️  Frozen lockfile failed, trying regular install...${NC}"
    pnpm install
fi

# Run quality checks if available
if [ -f "scripts/check-web.sh" ]; then
    echo -e "\n${BLUE}🔍 Running quality checks...${NC}"
    if ! ./scripts/check-web.sh; then
        echo -e "${YELLOW}⚠️  Quality checks failed, but continuing with build...${NC}"
    fi
fi

# Build the project
echo -e "\n${BLUE}🏗️  Building production bundle...${NC}"
if ! pnpm build; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

# Check if dist directory was created
if [ ! -d "dist" ]; then
    echo -e "${RED}❌ Build output directory 'dist' not found!${NC}"
    exit 1
fi

# Check if index.html was created
if [ ! -f "dist/index.html" ]; then
    echo -e "${RED}❌ index.html not found in build output!${NC}"
    exit 1
fi

# Calculate total time
echo -e "\n${BLUE}================================${NC}"
print_status "success" "Web frontend build completed successfully!"
show_elapsed
echo -e "${BLUE}📂 Build output: $WEB_DIR/dist${NC}"

# Show build summary
echo -e "\n${BLUE}📋 Build Summary:${NC}"
echo -e "   ✅ Dependencies installed"
echo -e "   ✅ Production build created"
echo -e "   ✅ Static files ready for serving"

# Optional: Show build size
if command -v du >/dev/null 2>&1; then
    build_size=$(du -sh dist 2>/dev/null | cut -f1)
    echo -e "   📊 Build size: $build_size"
fi

echo -e "\n${CYAN}💡 To serve the built files, start the Rust server:${NC}"
echo -e "   ${BLUE}./scripts/server.sh${NC}"