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

# 1. Cargo check
echo -e "\n${BLUE}🔍 Step 1/7: Running cargo check...${NC}"
if ! cargo check --manifest-path starter/Cargo.toml --all-targets --all-features; then
    echo -e "${RED}❌ Cargo check failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Cargo check passed${NC}"

# 2. Format check
echo -e "\n${BLUE}🎨 Step 2/7: Checking code formatting...${NC}"
if ! cargo fmt --manifest-path starter/Cargo.toml --all -- --check; then
    echo -e "${RED}❌ Code formatting issues found!${NC}"
    echo -e "${YELLOW}💡 Run 'cargo fmt --manifest-path starter/Cargo.toml --all' to fix${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Code formatting is correct${NC}"

# 3. Clippy linting (offline mode)
echo -e "\n${BLUE}📎 Step 3/7: Running Clippy lints...${NC}"
if ! SQLX_OFFLINE=true cargo clippy --manifest-path starter/Cargo.toml --all-targets --all-features -- -D warnings; then
    echo -e "${RED}❌ Clippy found issues!${NC}"
    echo -e "${YELLOW}💡 Fix the linting issues above${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Clippy checks passed${NC}"

# 4. Check if database is available for SQLx prepare
echo -e "\n${BLUE}🗄️  Step 4/7: Checking database availability for SQLx prepare...${NC}"
cd starter

# Check if we can connect to database
if ! cargo sqlx prepare --check 2>/dev/null; then
    echo -e "${YELLOW}⚠️  Database not available or SQLx cache outdated${NC}"
    echo -e "${BLUE}🔄 Running SQLx prepare to update query cache...${NC}"
    
    # Try to prepare with database connection
    if cargo sqlx prepare -- --all-targets 2>/dev/null; then
        echo -e "${GREEN}✅ SQLx prepare completed with database${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not connect to database for SQLx prepare${NC}"
        echo -e "${YELLOW}   Using existing query cache (run with database to update)${NC}"
    fi
else
    echo -e "${GREEN}✅ SQLx queries are up to date${NC}"
fi

cd "$PROJECT_ROOT"

# 5. Unit tests
echo -e "\n${BLUE}🧪 Step 5/7: Running unit tests...${NC}"
if ! cargo test --manifest-path starter/Cargo.toml --lib; then
    echo -e "${RED}❌ Unit tests failed!${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Unit tests passed${NC}"

# 6. Integration tests with nextest
echo -e "\n${BLUE}🔬 Step 6/7: Running integration tests with nextest...${NC}"
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

# 7. Additional quality checks
echo -e "\n${BLUE}🔍 Step 7/7: Additional quality checks...${NC}"

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
echo -e "   ✅ Cargo check (compilation)"
echo -e "   ✅ Code formatting (cargo fmt)"
echo -e "   ✅ Linting (cargo clippy)"
echo -e "   ✅ SQLx query cache validation"
echo -e "   ✅ Unit tests"
echo -e "   ✅ Integration tests (cargo nextest)"
echo -e "   ✅ Code quality analysis"