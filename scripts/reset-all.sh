#!/bin/bash

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Parse command line arguments
RESET_DATABASE=false
parse_standard_args "$@"

# Handle help or custom parsing
if [ "$HELP_REQUESTED" = true ]; then
    show_standard_help "$0" "Reset all starter processes and optionally database:"
    echo "Options:"
    echo "  --reset-database    Also reset the database (DANGEROUS!)"
    echo ""
    echo "By default, only stops processes and cleans up files."
    echo "Database reset requires explicit flag for safety."
    exit 0
fi

# Check for --reset-database flag
for arg in "$@"; do
    case $arg in
        --reset-database)
            RESET_DATABASE=true
            ;;
        --*)
            print_status "error" "Unknown option: $arg"
            print_status "info" "Use --help for usage information"
            exit 1
            ;;
    esac
done

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
    bash -c "
        # Get database config from root .env file
        DB_URL=\$(grep 'DATABASE_URL' .env 2>/dev/null | cut -d'=' -f2 | tr -d '\"' || echo 'postgresql://starter_user:starter_pass@localhost:5432/starter_db')
        ADMIN_URL=\${DB_URL%/*}/postgres
        
        # Drop template database using psql
        psql \"\$ADMIN_URL\" -c 'DROP DATABASE IF EXISTS \"starter_test_template\"' 2>/dev/null || true
    "
    
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