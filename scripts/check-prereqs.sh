#!/bin/bash

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Show help if requested
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_standard_help "$0" "Check all prerequisites for Rust Full-Stack Starter:"
    echo "Checks for:"
    echo "  • Docker and Docker Compose"
    echo "  • Rust and Cargo (1.75+)"
    echo "  • sqlx-cli (auto-install option)"
    echo "  • Optional tools (jq, python3)"
    echo ""
    echo "Exit codes:"
    echo "  0    All required tools found"
    echo "  1    Missing required tools"
    exit 0
fi

print_status "step" "Checking prerequisites for Rust Full-Stack Starter..."
echo ""

EXIT_CODE=0

# Check Docker
if ! check_dependency "docker" "20.10" "https://docker.com/get-started"; then
    EXIT_CODE=1
else
    # Check if Docker daemon is running
    if docker ps &> /dev/null; then
        echo "   ✅ Docker daemon is running"
    else
        echo "   ❌ Docker daemon is not running. Please start Docker Desktop."
        EXIT_CODE=1
    fi
fi

# Check Docker Compose
echo -n "🏗️  Docker Compose: "
if command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_VERSION=$(docker-compose --version | cut -d' ' -f3 | cut -d',' -f1 | sed 's/v//')
    echo "✅ Found (standalone: $COMPOSE_VERSION)"
elif docker compose version >/dev/null 2>&1; then
    COMPOSE_VERSION=$(docker compose version --short 2>/dev/null || echo "integrated")
    echo "✅ Found (integrated: $COMPOSE_VERSION)"
    echo "   ✅ Integrated Docker Compose (modern version)"
else
    echo "❌ Not found"
    echo "   📥 Install Docker Desktop or standalone docker-compose"
    EXIT_CODE=1
fi

# Check Rust/Cargo
if ! check_dependency "cargo" "1.75" "https://rustup.rs"; then
    EXIT_CODE=1
fi

# Check sqlx-cli
if check_command "sqlx" "cargo install sqlx-cli --no-default-features --features postgres"; then
    :  # sqlx found
else
    print_status "warning" "sqlx-cli not found (will auto-install when needed)"
fi

# Check optional tools
echo ""
echo "🔧 Optional tools:"

check_command "jq" "Recommended for JSON parsing in scripts" && \
    echo "   📊 jq: ✅ Found (JSON parsing in scripts)" || \
    echo "   📊 jq: ⚪ Not found (scripts will use python3 fallback)"

check_command "python3" "Fallback for JSON parsing" && \
    echo "   🐍 python3: ✅ Found (JSON parsing fallback)" || \
    echo "   🐍 python3: ⚪ Not found (jq recommended for script features)"

# Summary
echo ""
if [ $EXIT_CODE -eq 0 ]; then
    print_status "success" "All required prerequisites satisfied!"
    print_status "info" "Ready to run: ./scripts/dev-server.sh"
else
    print_status "error" "Missing required prerequisites. Please install missing tools and run again."
fi

echo ""
print_status "info" "For more info, see: docs/getting-started.md"

exit $EXIT_CODE