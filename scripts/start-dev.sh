#!/bin/bash
set -e

PORT=${1:-3000}

echo "🚀 Rust Full-Stack Starter - Complete Setup"
echo "============================================="
echo ""

# Check prerequisites first
echo "Step 1/4: Checking prerequisites..."
if ! ./scripts/check-prereqs.sh; then
    echo ""
    echo "❌ Prerequisites check failed. Please install missing tools and try again."
    exit 1
fi

echo ""
echo "Step 2/4: Setting up environment..."

# Copy .env if needed
if [ ! -f ".env" ]; then
    cp .env.example .env
    echo "✅ Environment file created (.env)"
else
    echo "✅ Environment file exists (.env)"
fi

echo ""
echo "Step 3/4: Starting development environment..."
./scripts/dev-server.sh $PORT

echo ""
echo "Step 4/4: Ready for development!"
echo ""
echo "🎉 Setup complete! Your development environment is running."
echo ""
echo "📍 Quick Links:"
echo "   • Server: http://localhost:$PORT"
echo "   • Health: http://localhost:$PORT/health"
echo "   • Detailed: http://localhost:$PORT/health/detailed"
echo ""
echo "📚 Next Steps:"
echo "   • Read guides: docs/guides/01-architecture.md"
echo "   • Try API: curl http://localhost:$PORT/health"
echo "   • View logs: tail -f /tmp/starter-server-$PORT.log"
echo ""
echo "🛑 To stop: ./scripts/stop-server.sh $PORT"