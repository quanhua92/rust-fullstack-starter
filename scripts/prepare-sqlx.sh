#!/bin/bash

# SQLx Prepare Script
# Updates SQLx query cache for offline compilation

set -e

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Get project directories (SCRIPT_DIR already set above)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
STARTER_DIR="$PROJECT_ROOT/starter"

print_status "step" "SQLx Prepare - Updating query cache for offline compilation"
echo "================================================="

# Check if starter directory exists
if [ ! -d "$STARTER_DIR" ]; then
    echo -e "${RED}❌ Error: starter directory not found at $STARTER_DIR${NC}"
    exit 1
fi

# Check if Cargo.toml exists in starter directory
if [ ! -f "$STARTER_DIR/Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Cargo.toml not found in starter directory${NC}"
    exit 1
fi

echo -e "${BLUE}📁 Working directory: $STARTER_DIR${NC}"
echo ""

# Change to starter directory and run sqlx prepare
cd "$STARTER_DIR"

echo -e "${BLUE}🗄️  Running SQLx prepare with all targets...${NC}"
if cargo sqlx prepare --all -- --all-targets; then
    echo ""
    echo -e "${GREEN}✅ SQLx query cache updated successfully${NC}"
    echo -e "${GREEN}   Query data written to .sqlx directory${NC}"
    echo -e "${GREEN}   Ready for offline compilation${NC}"
else
    echo ""
    echo -e "${RED}❌ SQLx prepare failed${NC}"
    echo -e "${RED}   Make sure database is running and accessible${NC}"
    exit 1
fi

echo ""
echo "================================================="
echo -e "${GREEN}🎉 SQLx prepare completed successfully!${NC}"