#!/bin/bash

# Show help if requested
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    echo "Usage: $0"
    echo ""
    echo "Check all prerequisites for Rust Full-Stack Starter:"
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

echo "🔍 Checking prerequisites for Rust Full-Stack Starter..."
echo ""

EXIT_CODE=0

# Check Docker
echo -n "🐳 Docker: "
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version | cut -d' ' -f3 | cut -d',' -f1)
    echo "✅ Found ($DOCKER_VERSION)"
    
    # Check version requirement (20.10+)
    MAJOR_VERSION=$(echo $DOCKER_VERSION | cut -d'.' -f1)
    MINOR_VERSION=$(echo $DOCKER_VERSION | cut -d'.' -f2)
    if [ "$MAJOR_VERSION" -gt 20 ] || ([ "$MAJOR_VERSION" -eq 20 ] && [ "$MINOR_VERSION" -ge 10 ]); then
        echo "   ✅ Version meets requirement (20.10+)"
    else
        echo "   ⚠️  Version $DOCKER_VERSION may be too old (recommended: 20.10+)"
    fi
    
    # Check if Docker daemon is running
    if docker ps &> /dev/null; then
        echo "   ✅ Docker daemon is running"
    else
        echo "   ❌ Docker daemon is not running. Please start Docker Desktop."
        EXIT_CODE=1
    fi
else
    echo "❌ Not found"
    echo "   📥 Install from: https://docker.com/get-started (version 20.10+)"
    EXIT_CODE=1
fi

# Check Docker Compose
echo -n "🏗️  Docker Compose: "
if command -v docker-compose &> /dev/null; then
    COMPOSE_VERSION=$(docker-compose --version | cut -d' ' -f3 | cut -d',' -f1 | sed 's/v//')
    echo "✅ Found (standalone: $COMPOSE_VERSION)"
    # Check version requirement (2.0+) - only if version is numeric
    if [[ $COMPOSE_VERSION =~ ^[0-9]+\.[0-9]+ ]]; then
        MAJOR_VERSION=$(echo $COMPOSE_VERSION | cut -d'.' -f1)
        if [ "$MAJOR_VERSION" -ge 2 ]; then
            echo "   ✅ Version meets requirement (2.0+)"
        else
            echo "   ⚠️  Version $COMPOSE_VERSION may be too old (recommended: 2.0+)"
        fi
    else
        echo "   ✅ Version looks good"
    fi
elif docker compose version &> /dev/null; then
    COMPOSE_VERSION=$(docker compose version --short 2>/dev/null || echo "integrated")
    echo "✅ Found (integrated: $COMPOSE_VERSION)"
    echo "   ✅ Integrated Docker Compose (modern version)"
else
    echo "❌ Not found"
    echo "   📥 Install Docker Desktop or standalone docker-compose"
    EXIT_CODE=1
fi

# Check Rust/Cargo
echo -n "🦀 Rust: "
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo "✅ Found ($RUST_VERSION)"
    
    # Check minimum version (1.75+)
    RUST_MAJOR=$(echo $RUST_VERSION | cut -d'.' -f1)
    RUST_MINOR=$(echo $RUST_VERSION | cut -d'.' -f2)
    if [ "$RUST_MAJOR" -gt 1 ] || ([ "$RUST_MAJOR" -eq 1 ] && [ "$RUST_MINOR" -ge 75 ]); then
        echo "   ✅ Version meets minimum requirement (1.75+)"
    else
        echo "   ⚠️  Version $RUST_VERSION is below recommended 1.75+"
        echo "   📥 Update with: rustup update"
    fi
else
    echo "❌ Not found"
    echo "   📥 Install from: https://rustup.rs"
    EXIT_CODE=1
fi

# Check sqlx-cli
echo -n "🗃️  sqlx-cli: "
if command -v sqlx &> /dev/null; then
    SQLX_VERSION=$(sqlx --version | cut -d' ' -f2)
    echo "✅ Found ($SQLX_VERSION)"
else
    echo "⚠️  Not found (will auto-install when needed)"
    echo "   💡 To install now: cargo install sqlx-cli --no-default-features --features postgres"
fi

# Check optional tools
echo ""
echo "🔧 Optional tools:"

echo -n "   📊 jq: "
if command -v jq &> /dev/null; then
    echo "✅ Found (JSON parsing in scripts)"
else
    echo "⚪ Not found (scripts will use python3 fallback)"
fi

echo -n "   🐍 python3: "
if command -v python3 &> /dev/null; then
    echo "✅ Found (JSON parsing fallback)"
else
    echo "⚪ Not found (jq recommended for script features)"
fi

# Summary
echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "🎉 All required prerequisites satisfied!"
    echo "   Ready to run: ./scripts/dev-server.sh"
else
    echo "❌ Missing required prerequisites. Please install missing tools and run again."
fi

echo ""
echo "📚 For more info, see: docs/getting-started.md"

exit $EXIT_CODE