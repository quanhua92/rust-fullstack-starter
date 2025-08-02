#!/bin/bash

# Common utilities for Rust Full-Stack Starter scripts
# Source this file in other scripts: source scripts/common.sh

# Color codes for consistent output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Track timing across scripts
START_TIME=""

# Initialize timing
init_timing() {
    START_TIME=$(date +%s)
}

# Show elapsed time
show_elapsed() {
    if [ -n "$START_TIME" ]; then
        local end_time=$(date +%s)
        local elapsed=$((end_time - START_TIME))
        echo -e "${CYAN}⏱️  Total time: ${elapsed}s${NC}"
    fi
}

# Print colored status messages
print_status() {
    local level="$1"
    local message="$2"
    case "$level" in
        "success") echo -e "${GREEN}✅ $message${NC}" ;;
        "error") echo -e "${RED}❌ $message${NC}" ;;
        "warning") echo -e "${YELLOW}⚠️  $message${NC}" ;;
        "info") echo -e "${BLUE}ℹ️  $message${NC}" ;;
        "step") echo -e "${CYAN}🚀 $message${NC}" ;;
        *) echo "$message" ;;
    esac
}

# Check if a command exists
check_command() {
    local cmd="$1"
    local install_msg="$2"
    
    if command -v "$cmd" >/dev/null 2>&1; then
        print_status "success" "$cmd is available"
        return 0
    else
        print_status "error" "$cmd is not installed"
        if [ -n "$install_msg" ]; then
            print_status "info" "$install_msg"
        fi
        return 1
    fi
}

# Check dependency with version requirement
check_dependency() {
    local cmd="$1"
    local required_version="$2"
    local install_url="$3"
    
    echo -n "🔍 $cmd: "
    if command -v "$cmd" >/dev/null 2>&1; then
        local version=$($cmd --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        if [ -n "$version" ]; then
            echo "✅ Found ($version)"
            if [ -n "$required_version" ]; then
                # Basic version comparison (works for most cases)
                if [ "$(printf '%s\n' "$required_version" "$version" | sort -V | head -n1)" = "$required_version" ]; then
                    echo "   ✅ Version meets requirement ($required_version+)"
                else
                    echo "   ⚠️  Version $version may be too old (recommended: $required_version+)"
                fi
            fi
        else
            echo "✅ Found (version unknown)"
        fi
        return 0
    else
        echo "❌ Not found"
        if [ -n "$install_url" ]; then
            echo "   📥 Install from: $install_url"
        fi
        return 1
    fi
}

# Check if we're in the project root
validate_project_root() {
    if [ ! -f "Cargo.toml" ] || [ ! -d "starter" ]; then
        print_status "error" "Please run this script from the project root directory"
        print_status "info" "Current directory: $(pwd)"
        print_status "info" "Expected files: Cargo.toml, starter/"
        exit 1
    fi
}

# Kill processes on a specific port
kill_port() {
    local port="$1"
    local pids=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$pids" ]; then
        print_status "info" "Killing processes on port $port"
        echo "$pids" | xargs kill -9 2>/dev/null || true
        sleep 1
    fi
}

# Stop processes by name pattern
stop_processes() {
    local pattern="$1"
    local description="$2"
    
    if pkill -f "$pattern" 2>/dev/null; then
        print_status "success" "Stopped $description"
        sleep 1
    else
        print_status "info" "No $description processes found"
    fi
}

# Wait for service to be ready
wait_for_service() {
    local url="$1"
    local timeout="${2:-30}"
    local description="${3:-service}"
    
    print_status "info" "Waiting for $description to be ready..."
    local count=0
    while [ $count -lt $timeout ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_status "success" "$description is ready"
            return 0
        fi
        sleep 1
        count=$((count + 1))
        echo -n "."
    done
    echo
    print_status "error" "$description failed to start within ${timeout}s"
    return 1
}

# Run command with error handling
run_cmd() {
    local description="$1"
    shift
    
    print_status "step" "$description"
    if "$@"; then
        print_status "success" "$description completed"
        return 0
    else
        print_status "error" "$description failed"
        return 1
    fi
}

# Show help for common script patterns
show_standard_help() {
    local script_name="$1"
    local description="$2"
    
    echo "Usage: $script_name [options]"
    echo ""
    echo "$description"
    echo ""
    echo "Common options:"
    echo "  -h, --help          Show this help message"
    echo "  -f, --foreground    Run in foreground mode"
    echo "  --port PORT         Specify custom port"
    echo ""
}

# Parse standard arguments
parse_standard_args() {
    FOREGROUND=false
    CUSTOM_PORT=""
    HELP_REQUESTED=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                HELP_REQUESTED=true
                shift
                ;;
            -f|--foreground)
                FOREGROUND=true
                shift
                ;;
            --port)
                CUSTOM_PORT="$2"
                shift 2
                ;;
            *)
                # Return remaining arguments
                break
                ;;
        esac
    done
}

# Get project directories
get_project_dirs() {
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
    WEB_ROOT="$PROJECT_ROOT/web"
    STARTER_ROOT="$PROJECT_ROOT/starter"
    
    # Handle case where script is in web/scripts/
    if [[ "$SCRIPT_DIR" == */web/scripts ]]; then
        PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
        WEB_ROOT="$PROJECT_ROOT/web"
        STARTER_ROOT="$PROJECT_ROOT/starter"
    fi
}

# Docker health check
check_docker_health() {
    if ! command -v docker >/dev/null 2>&1; then
        print_status "error" "Docker is not installed"
        return 1
    fi
    
    if ! docker ps >/dev/null 2>&1; then
        print_status "error" "Docker daemon is not running"
        return 1
    fi
    
    print_status "success" "Docker is available and running"
    return 0
}

# Database health check
check_database() {
    local host="${1:-localhost}"
    local port="${2:-5432}"
    local timeout="${3:-10}"
    
    print_status "info" "Checking database connection..."
    if timeout "$timeout" bash -c "</dev/tcp/$host/$port" 2>/dev/null; then
        print_status "success" "Database is accessible at $host:$port"
        return 0
    else
        print_status "warning" "Database not accessible at $host:$port"
        return 1
    fi
}