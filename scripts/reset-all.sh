#!/bin/bash

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Parse command line arguments
RESET_DATABASE=false
FORCE_REMOTE=false
parse_standard_args "$@"

# Handle help or custom parsing
if [ "$HELP_REQUESTED" = true ]; then
    show_standard_help "$0" "Reset all starter processes and optionally database:"
    echo "Options:"
    echo "  --reset-database    Also reset the database (DANGEROUS!)"
    echo "  --force-remote      Allow database operations on remote hosts (VERY DANGEROUS!)"
    echo ""
    echo "By default, only stops processes and cleans up files."
    echo "Database reset requires explicit flag for safety."
    echo "Remote database operations are blocked unless --force-remote is used."
    exit 0
fi

# Check for --reset-database and --force-remote flags
for arg in "$@"; do
    case $arg in
        --reset-database)
            RESET_DATABASE=true
            ;;
        --force-remote)
            FORCE_REMOTE=true
            ;;
        --*)
            print_status "error" "Unknown option: $arg"
            print_status "info" "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to check if database host is local
is_local_host() {
    local db_url="$1"
    # Extract host from database URL
    # URL format: postgresql://user:pass@host:port/db
    local host=$(echo "$db_url" | sed -n 's/.*@\([^:]*\):.*/\1/p')
    
    # If sed didn't extract host, try alternative parsing
    if [ -z "$host" ]; then
        host=$(echo "$db_url" | sed -n 's/.*@\([^/]*\)\/.*/\1/p' | cut -d':' -f1)
    fi
    
    # Check if host is localhost or 127.0.0.1 or empty (defaults to localhost)
    case "$host" in
        "localhost"|"127.0.0.1"|"::1"|"")
            return 0  # Local host
            ;;
        *)
            return 1  # Remote host
            ;;
    esac
}

# Function to confirm remote database operation
confirm_remote_operation() {
    local db_url="$1"
    local host=$(echo "$db_url" | sed -n 's/.*@\([^:]*\):.*/\1/p')
    
    if [ -z "$host" ]; then
        host=$(echo "$db_url" | sed -n 's/.*@\([^/]*\)\/.*/\1/p' | cut -d':' -f1)
    fi
    
    print_status "warning" "⚠️  DANGER: About to perform database operation on REMOTE host: $host"
    print_status "warning" "This could affect production or shared databases!"
    echo ""
    read -p "Are you absolutely sure you want to proceed? Type 'yes' to continue: " confirmation
    
    if [ "$confirmation" != "yes" ]; then
        print_status "info" "Operation cancelled for safety"
        return 1
    fi
    
    return 0
}

# Validate project directory
validate_project_root

print_status "step" "Resetting all servers and workers..."

# Stop all processes
print_status "info" "Stopping all starter processes..."
stop_processes "starter server" "server"
stop_processes "starter worker" "worker"

# Clean up common ports
for PORT in 3000 8080; do
    kill_port $PORT
done

# Clean up PID and log files
print_status "info" "Cleaning up PID files..."
rm -f /tmp/starter-server-*.pid /tmp/starter-worker-*.pid

print_status "info" "Cleaning up old log files..."
find /tmp -name "starter-*.log" -mtime +1 -delete 2>/dev/null || true

# Database reset (only if explicitly requested)
if [ "$RESET_DATABASE" = true ]; then
    run_cmd "Resetting database" bash -c "cd starter && sqlx database reset -y" || {
        print_status "warning" "Database reset failed, continuing anyway..."
    }
    
    # Also drop test template database to avoid version mismatch issues
    print_status "info" "Dropping test template database..."
    
    # Get database config from root .env file
    # Source .env file if it exists to get DATABASE_URL with proper parsing
    if [ -f ".env" ]; then
        # Source .env file in a subshell to avoid polluting current environment
        # Use set -a to automatically export all variables
        set -a
        source .env 2>/dev/null || true
        set +a
    fi
    
    # Use sourced DATABASE_URL or fallback to default
    DB_URL="${DATABASE_URL:-postgresql://starter_user:starter_pass@localhost:5432/starter_db}"
    
    # Safety check: ensure we're working with local database
    if ! is_local_host "$DB_URL"; then
        if [ "$FORCE_REMOTE" = true ]; then
            print_status "warning" "🚨 --force-remote flag detected, proceeding with remote database operation"
        else
            if ! confirm_remote_operation "$DB_URL"; then
                print_status "error" "Remote database operation cancelled for safety"
                print_status "info" "Use --force-remote flag to override this safety check"
                exit 1
            fi
        fi
    fi
    
    # Proceed with dropping test template database
    ADMIN_URL="${DB_URL%/*}/postgres"
    if psql "$ADMIN_URL" -c 'DROP DATABASE IF EXISTS "starter_test_template"' 2>/dev/null; then
        print_status "success" "Test template database dropped successfully"
    else
        print_status "warning" "Failed to drop test template database (it may not exist)"
    fi
    
    DATABASE_STATUS="and database reset"
else
    print_status "info" "Database preserved (use --reset-database to reset database)"
    print_status "info" "To reset database manually: cd starter && sqlx database reset -y"
    DATABASE_STATUS="(database preserved)"
fi

# Wait for cleanup
sleep 2

print_status "success" "Reset complete! All processes stopped $DATABASE_STATUS."
echo ""
print_status "step" "Next steps:"
echo "   Start server: ./scripts/server.sh [port]"
echo "   Start worker: ./scripts/worker.sh [--id ID]"
echo "   Multiple workers: ./scripts/worker.sh --id 1 && ./scripts/worker.sh --id 2"
echo "   Run tests: ./scripts/test_tasks_integration.sh"
if [ "$RESET_DATABASE" = false ]; then
    echo ""
    print_status "info" "To also reset database next time: ./scripts/reset-all.sh --reset-database"
fi