#!/bin/bash

# Script to create a new module from a template
# Usage: ./scripts/create-new-module.sh [OPTIONS] MODULE_NAME
# Examples:
#   ./scripts/create-new-module.sh notes
#   ./scripts/create-new-module.sh --singular note --plural notes note
#   ./scripts/create-new-module.sh --template basic --delete notes

set -e

# Load common functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

validate_project_root

# Default values
TEMPLATE_NAME="basic"
DELETE_MODE=false
ONLY_DELETE=false
DRY_RUN=false
YES_MODE=false
SINGULAR_NAME=""
PLURAL_NAME=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --template)
            TEMPLATE_NAME="$2"
            shift 2
            ;;
        --delete)
            DELETE_MODE=true
            shift
            ;;
        --only-delete)
            ONLY_DELETE=true
            DELETE_MODE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --yes|-y)
            YES_MODE=true
            shift
            ;;
        --singular)
            SINGULAR_NAME="$2"
            shift 2
            ;;
        --plural)
            PLURAL_NAME="$2"
            shift 2
            ;;
        --help|-h)
            cat << 'EOF'
DESCRIPTION:
    Generate a new CRUD module from a template with full API endpoints, database migrations,
    tests, and OpenAPI documentation. Creates a complete resource module following the
    existing codebase patterns for users, tasks, and monitoring modules.

USAGE:
    ./scripts/create-new-module.sh [OPTIONS] MODULE_NAME

ARGUMENTS:
    MODULE_NAME                The name of the module to create (e.g., 'notes', 'tasks', 'documents')
                              Used as the base name for auto-detecting singular/plural forms.
                              Must start with lowercase letter and contain only letters, numbers, underscores.

OPTIONS:
    --template NAME           Template to use for code generation (default: 'basic')
                              Templates are located in starter/templates/
                              The 'basic' template provides standard CRUD operations with RBAC

    --delete                  Clean up a previously generated module attempt, then create new module
                              Removes source files, migrations, and test files
                              Prompts for confirmation unless --yes is used
                              Does NOT modify server.rs, lib.rs, or openapi.rs automatically

    --only-delete             Clean up a previously generated module attempt without creating new module
                              Useful for cleanup after testing or failed attempts
                              Same cleanup as --delete but stops after deletion

    --dry-run                 Preview mode - show what would be created without making changes
                              Displays file paths and content previews for all generated files
                              Useful for understanding the template output before committing
                              Automatically skips all confirmation prompts

    --yes, -y                 Non-interactive mode - skip all confirmation prompts
                              Useful for CI/CD scripts and automated workflows
                              Does not affect --dry-run behavior

    --singular NAME           Override auto-detected singular form of the resource name
                              Example: --singular person (instead of auto-detected 'people')
                              Used for struct names, function names, and API endpoints

    --plural NAME             Override auto-detected plural form of the resource name  
                              Example: --plural people (instead of auto-detected 'persons')
                              Used for table names, collection endpoints, and module directories

    --help, -h                Show this detailed help information

AUTO-DETECTION LOGIC:
    If no --singular/--plural specified, the script auto-detects based on MODULE_NAME:
    - If MODULE_NAME ends with 's': assumes plural, removes 's' for singular
      Example: 'notes' → singular='note', plural='notes'
    - If MODULE_NAME doesn't end with 's': assumes singular, adds 's' for plural  
      Example: 'document' → singular='document', plural='documents'

GENERATED FILES:
    Source Code:
    - starter/src/MODULE_PLURAL/api.rs      (REST endpoints with RBAC)
    - starter/src/MODULE_PLURAL/models.rs   (Structs, validation, request/response types)
    - starter/src/MODULE_PLURAL/services.rs (Database operations, business logic)
    - starter/src/MODULE_PLURAL/mod.rs      (Module declaration)
    
    Database:
    - starter/migrations/XXX_MODULE_PLURAL.up.sql    (CREATE table with indexes)
    - starter/migrations/XXX_MODULE_PLURAL.down.sql  (DROP cleanup)
    
    Tests:
    - starter/tests/MODULE_PLURAL/mod.rs    (Integration tests with RBAC scenarios)

MANUAL STEPS REQUIRED:
    The script generates template files but requires manual integration:
    1. Add 'pub mod MODULE_PLURAL;' to starter/src/lib.rs
    2. Add API import to starter/src/server.rs
    3. Add CRUD routes to protected_routes section in starter/src/server.rs
    4. Add admin stats route to admin_routes section in starter/src/server.rs  
    5. Add 'mod MODULE_PLURAL;' to starter/tests/lib.rs
    6. Add OpenAPI schemas to starter/src/openapi.rs
    7. Run database migrations: sqlx migrate run
    8. Test the module: cargo nextest run MODULE_PLURAL

EXAMPLES:
    # Basic usage - creates notes module (note/notes)
    ./scripts/create-new-module.sh notes

    # Preview what would be generated without creating files
    ./scripts/create-new-module.sh --dry-run notes

    # Non-interactive creation for CI/CD
    ./scripts/create-new-module.sh --yes notes

    # Handle irregular plurals manually
    ./scripts/create-new-module.sh --singular person --plural people person
    ./scripts/create-new-module.sh --singular child --plural children child

    # Use different template (if available)
    ./scripts/create-new-module.sh --template advanced documents

    # Clean up previous failed attempt, then create new
    ./scripts/create-new-module.sh --delete notes

    # Clean up only (useful for testing cleanup)
    ./scripts/create-new-module.sh --only-delete notes

    # Combine options
    ./scripts/create-new-module.sh --dry-run --singular category --plural categories category

TEMPLATE STRUCTURE:
    Templates use placeholder substitution:
    - __MODULE_NAME__ → singular form (e.g., 'note')
    - __MODULE_NAME_PLURAL__ → plural form (e.g., 'notes')  
    - __MODULE_STRUCT__ → PascalCase struct name (e.g., 'Note')
    - __MODULE_TABLE__ → database table name (e.g., 'notes')

    The 'basic' template provides:
    - Full CRUD operations (Create, Read, Update, Delete)
    - RBAC permission checks (users can access own resources, admins see all)
    - Input validation and error handling
    - OpenAPI/Swagger documentation
    - Database migrations with proper indexes
    - Comprehensive integration tests
    - Admin statistics endpoint

AI AGENT NOTES:
    This script is designed to work with AI coding assistants. After running the script:
    1. Look for "IMPORTANT_NEXT_STEPS" in the output
    2. Follow the numbered steps exactly as shown
    3. Each step provides the exact code to add and file locations
    4. Use --dry-run first to understand what will be generated
    5. The generated code follows existing codebase patterns from users/tasks/monitoring modules
EOF
            exit 0
            ;;
        -*)
            print_status "error" "Unknown option: $1"
            print_status "info" "Use --help for usage information"
            exit 1
            ;;
        *)
            if [ -z "$MODULE_NAME" ]; then
                MODULE_NAME="$1"
            else
                print_status "error" "Multiple module names provided: $MODULE_NAME and $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate required arguments
if [ -z "$MODULE_NAME" ]; then
    print_status "error" "MODULE_NAME is required"
    print_status "info" "Usage: $0 [OPTIONS] MODULE_NAME"
    print_status "info" "Use --help for more information"
    exit 1
fi

# Validate inputs
if [[ ! "$MODULE_NAME" =~ ^[a-z][a-z0-9_]*$ ]]; then
    print_status "error" "Module name must start with lowercase letter and contain only lowercase letters, numbers, and underscores"
    exit 1
fi

# Smart plural/singular handling
if [ -n "$SINGULAR_NAME" ] && [ -n "$PLURAL_NAME" ]; then
    # User provided both explicitly
    MODULE_NAME_SINGULAR="$SINGULAR_NAME"
    MODULE_NAME_PLURAL="$PLURAL_NAME"
elif [ -n "$SINGULAR_NAME" ]; then
    # User provided singular, auto-generate plural
    MODULE_NAME_SINGULAR="$SINGULAR_NAME"
    MODULE_NAME_PLURAL="${SINGULAR_NAME}s"
elif [ -n "$PLURAL_NAME" ]; then
    # User provided plural, try to derive singular
    MODULE_NAME_PLURAL="$PLURAL_NAME"
    if [[ "$PLURAL_NAME" =~ s$ ]]; then
        MODULE_NAME_SINGULAR="${PLURAL_NAME%s}"  # Remove trailing 's'
    else
        MODULE_NAME_SINGULAR="$PLURAL_NAME"
    fi
else
    # Auto-detect based on module name
    if [[ "$MODULE_NAME" =~ s$ ]]; then
        # Ends with 's', assume it's plural
        MODULE_NAME_PLURAL="$MODULE_NAME"
        MODULE_NAME_SINGULAR="${MODULE_NAME%s}"  # Remove trailing 's'
    else
        # Doesn't end with 's', assume it's singular
        MODULE_NAME_SINGULAR="$MODULE_NAME"
        MODULE_NAME_PLURAL="${MODULE_NAME}s"
    fi
fi

# Convert to different case formats
MODULE_STRUCT=$(echo "$MODULE_NAME_SINGULAR" | sed 's/^./\U&/' | sed 's/_\(.\)/\U\1/g')  # PascalCase
MODULE_TABLE="$MODULE_NAME_PLURAL"

print_status "info" "Using singular: '$MODULE_NAME_SINGULAR', plural: '$MODULE_NAME_PLURAL'"

# Helper function for confirmation prompts
confirm_action() {
    local message="$1"
    
    if [ "$YES_MODE" = true ] || [ "$DRY_RUN" = true ]; then
        return 0  # Auto-confirm in yes mode or dry-run
    fi
    
    echo -n "$message (y/N): "
    read -r response
    case "$response" in
        [yY]|[yY][eE][sS])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Helper function for dry run operations
dry_run_execute() {
    local description="$1"
    local command="$2"
    
    if [ "$DRY_RUN" = true ]; then
        print_status "info" "[DRY-RUN] $description"
        print_status "info" "  Command: $command"
    else
        eval "$command"
    fi
}

# Helper function for dry run file creation
dry_run_create_file() {
    local file_path="$1"
    local content="$2"
    
    if [ "$DRY_RUN" = true ]; then
        print_status "info" "[DRY-RUN] Would create file: $file_path"
        echo "--- Content preview (first 10 lines) ---"
        echo "$content" | head -10
        echo "--- End preview ---"
        echo ""
    else
        echo "$content" > "$file_path"
    fi
}

if [ "$DRY_RUN" = true ]; then
    print_status "warning" "🔍 DRY RUN MODE - No files will be created or modified"
    echo ""
fi

# Check if template exists
TEMPLATE_DIR="starter/templates/$TEMPLATE_NAME"
if [ ! -d "$TEMPLATE_DIR" ]; then
    print_status "error" "Template '$TEMPLATE_NAME' not found in $TEMPLATE_DIR"
    exit 1
fi

MODULE_DIR="starter/src/$MODULE_NAME_PLURAL"
TEST_DIR="starter/tests/$MODULE_NAME_PLURAL"

# Handle delete mode
if [ "$DELETE_MODE" = true ]; then
    print_status "warning" "🧹 Cleaning up previous attempt for module '$MODULE_NAME_PLURAL'"
    
    if ! confirm_action "⚠️  This will permanently delete module files. Continue?"; then
        print_status "info" "Operation cancelled"
        exit 0
    fi
    
    # Remove module directory
    if [ -d "$MODULE_DIR" ] || [ "$DRY_RUN" = true ]; then
        dry_run_execute "Remove module directory" "rm -rf '$MODULE_DIR'"
        if [ "$DRY_RUN" = false ]; then
            print_status "info" "  Removed $MODULE_DIR"
        fi
    fi
    
    # Remove migration files (check both singular and plural patterns)
    for migration in starter/migrations/*_${MODULE_NAME_PLURAL}.*.sql starter/migrations/*_${MODULE_NAME_SINGULAR}.*.sql starter/migrations/*_${MODULE_NAME}.*.sql; do
        if [ -f "$migration" ] || [ "$DRY_RUN" = true ]; then
            dry_run_execute "Remove migration file" "rm '$migration'"
            if [ "$DRY_RUN" = false ] && [ -f "$migration" ]; then
                print_status "info" "  Removed $(basename "$migration")"
            elif [ "$DRY_RUN" = true ]; then
                print_status "info" "[DRY-RUN] Would remove: $(basename "$migration")"
            fi
        fi
    done
    
    # Remove test directory
    if [ -d "$TEST_DIR" ] || [ "$DRY_RUN" = true ]; then
        dry_run_execute "Remove test directory" "rm -rf '$TEST_DIR'"
        if [ "$DRY_RUN" = false ]; then
            print_status "info" "  Removed $TEST_DIR"
        fi
    fi
    
    if [ "$DRY_RUN" = false ]; then
        print_status "success" "Cleanup completed for module '$MODULE_NAME_PLURAL'"
    else
        print_status "success" "[DRY-RUN] Cleanup preview completed for module '$MODULE_NAME_PLURAL'"
    fi
    
    print_status "warning" "⚠️  IMPORTANT: You may need to manually clean up:"
    print_status "warning" "   - Remove 'pub mod $MODULE_NAME_PLURAL;' from starter/src/lib.rs"
    print_status "warning" "   - Remove API imports and routes from starter/src/server.rs"
    print_status "warning" "   - Remove 'mod $MODULE_NAME_PLURAL;' from starter/tests/lib.rs"
    print_status "warning" "   - Remove module from starter/src/openapi.rs"
    
    # Exit early if only cleanup was requested
    if [ "$ONLY_DELETE" = true ]; then
        print_status "info" "Only cleanup requested - exiting without creating new module"
        exit 0
    fi
    
    exit 0
fi

# Check if module already exists
if [ -d "$MODULE_DIR" ] && [ "$DRY_RUN" = false ]; then
    print_status "error" "Module '$MODULE_NAME_PLURAL' already exists in $MODULE_DIR"
    print_status "info" "Use --delete flag to clean up: $0 --delete $MODULE_NAME"
    exit 1
fi

if [ "$DRY_RUN" = true ]; then
    print_status "info" "[DRY-RUN] Would create new module '$MODULE_NAME_PLURAL' from template '$TEMPLATE_NAME'"
else
    print_status "info" "Creating new module '$MODULE_NAME_PLURAL' from template '$TEMPLATE_NAME'"
    
    # Confirm creation in interactive mode
    if ! confirm_action "📝 Create new module '$MODULE_NAME_PLURAL' with template '$TEMPLATE_NAME'?"; then
        print_status "info" "Operation cancelled"
        exit 0
    fi
fi

# Create module directory
print_status "step" "Creating module directory: $MODULE_DIR"
dry_run_execute "Create module directory" "mkdir -p '$MODULE_DIR'"

# Copy and process template files
print_status "step" "Copying and processing template files"
for file in "$TEMPLATE_DIR"/*.rs; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        print_status "info" "  Processing $filename"
        
        # Process the file with sed replacements
        processed_content=$(sed -e "s/__MODULE_NAME__/$MODULE_NAME_SINGULAR/g" \
            -e "s/__MODULE_NAME_PLURAL__/$MODULE_NAME_PLURAL/g" \
            -e "s/__MODULE_STRUCT__/$MODULE_STRUCT/g" \
            -e "s/__MODULE_TABLE__/$MODULE_TABLE/g" \
            "$file")
        
        dry_run_create_file "$MODULE_DIR/$filename" "$processed_content"
    fi
done

# Handle migration files
print_status "step" "Creating database migrations"

# Find the next migration number
LAST_MIGRATION=$(ls starter/migrations/ | grep -E '^[0-9]+_' | sort -n | tail -1 | sed 's/^\([0-9]*\)_.*/\1/')
if [ -z "$LAST_MIGRATION" ]; then
    NEXT_NUMBER="001"
else
    NEXT_NUMBER=$(printf "%03d" $((10#$LAST_MIGRATION + 1)))
fi

# Copy and process migration files
for sql_file in "$TEMPLATE_DIR"/*.sql; do
    if [ -f "$sql_file" ]; then
        sql_filename=$(basename "$sql_file")
        migration_name="${NEXT_NUMBER}_${MODULE_NAME_PLURAL}.${sql_filename}"
        print_status "info" "  Creating migration: $migration_name"
        
        processed_sql=$(sed -e "s/__MODULE_NAME__/$MODULE_NAME_SINGULAR/g" \
            -e "s/__MODULE_NAME_PLURAL__/$MODULE_NAME_PLURAL/g" \
            -e "s/__MODULE_STRUCT__/$MODULE_STRUCT/g" \
            -e "s/__MODULE_TABLE__/$MODULE_TABLE/g" \
            "$sql_file")
        
        dry_run_create_file "starter/migrations/$migration_name" "$processed_sql"
    fi
done

# Create integration test file
print_status "step" "Creating integration test file"
dry_run_execute "Create test directory" "mkdir -p '$TEST_DIR'"

if [ -f "$TEMPLATE_DIR/tests.rs" ]; then
    processed_test=$(sed -e "s/__MODULE_NAME__/$MODULE_NAME_SINGULAR/g" \
        -e "s/__MODULE_NAME_PLURAL__/$MODULE_NAME_PLURAL/g" \
        -e "s/__MODULE_STRUCT__/$MODULE_STRUCT/g" \
        -e "s/__MODULE_TABLE__/$MODULE_TABLE/g" \
        "$TEMPLATE_DIR/tests.rs")
    
    dry_run_create_file "$TEST_DIR/mod.rs" "$processed_test"
    if [ "$DRY_RUN" = false ]; then
        print_status "info" "  Created test file: $TEST_DIR/mod.rs"
    fi
fi

if [ "$DRY_RUN" = true ]; then
    print_status "success" "✅ [DRY-RUN] Module '$MODULE_NAME_PLURAL' template preview completed!"
else
    print_status "success" "✅ Module '$MODULE_NAME_PLURAL' template files created successfully!"
fi

# Important next steps that need manual intervention
print_status "warning" "🤖 IMPORTANT_NEXT_STEPS - AI Agent Instructions:"
echo ""
echo "STEP 1: Add module declaration to starter/src/lib.rs"
echo "   Add this line after 'pub mod users;': pub mod $MODULE_NAME_PLURAL;"
echo ""
echo "STEP 2: Add API import to starter/src/server.rs" 
echo "   Add this line after 'users::api as users_api,': ${MODULE_NAME_PLURAL}::api as ${MODULE_NAME_PLURAL}_api,"
echo ""
echo "STEP 3: Add CRUD routes to starter/src/server.rs in protected_routes section"
echo "   Add these lines after the users routes:"
echo "        .route(\"/${MODULE_NAME_PLURAL}\", get(${MODULE_NAME_PLURAL}_api::list_${MODULE_NAME_PLURAL}))"
echo "        .route(\"/${MODULE_NAME_PLURAL}\", post(${MODULE_NAME_PLURAL}_api::create_${MODULE_NAME_SINGULAR}))"
echo "        .route(\"/${MODULE_NAME_PLURAL}/{id}\", get(${MODULE_NAME_PLURAL}_api::get_${MODULE_NAME_SINGULAR}_by_id))"
echo "        .route(\"/${MODULE_NAME_PLURAL}/{id}\", put(${MODULE_NAME_PLURAL}_api::update_${MODULE_NAME_SINGULAR}))"
echo "        .route(\"/${MODULE_NAME_PLURAL}/{id}\", delete(${MODULE_NAME_PLURAL}_api::delete_${MODULE_NAME_SINGULAR}))"
echo ""
echo "STEP 4: Add admin stats route to starter/src/server.rs in admin_routes section"
echo "   Add this line after '/admin/users/stats': .route(\"/admin/${MODULE_NAME_PLURAL}/stats\", get(${MODULE_NAME_PLURAL}_api::get_${MODULE_NAME_PLURAL}_stats))"
echo ""
echo "STEP 5: Add test module to starter/tests/lib.rs"
echo "   Add this line: mod $MODULE_NAME_PLURAL;"
echo ""
echo "STEP 6: Add OpenAPI integration to starter/src/openapi.rs"
echo "   Add these imports after the existing model imports:"
echo "   crate::${MODULE_NAME_PLURAL}::models::{"
echo "       ${MODULE_STRUCT}, ${MODULE_STRUCT}Response, Create${MODULE_STRUCT}Request,"
echo "       Update${MODULE_STRUCT}Request, ${MODULE_STRUCT}Stats"
echo "   },"
echo ""
echo "   Add these API endpoints to the paths() section:"
echo "       crate::${MODULE_NAME_PLURAL}::api::create_${MODULE_NAME_SINGULAR},"
echo "       crate::${MODULE_NAME_PLURAL}::api::get_${MODULE_NAME_SINGULAR}_by_id,"
echo "       crate::${MODULE_NAME_PLURAL}::api::list_${MODULE_NAME_PLURAL},"
echo "       crate::${MODULE_NAME_PLURAL}::api::update_${MODULE_NAME_SINGULAR},"
echo "       crate::${MODULE_NAME_PLURAL}::api::delete_${MODULE_NAME_SINGULAR},"
echo "       crate::${MODULE_NAME_PLURAL}::api::get_${MODULE_NAME_PLURAL}_stats,"
echo ""
echo "   Add these schemas to the components/schemas array:"
echo "       ${MODULE_STRUCT}, ${MODULE_STRUCT}Response, Create${MODULE_STRUCT}Request,"
echo "       Update${MODULE_STRUCT}Request, ${MODULE_STRUCT}Stats,"
echo ""
echo "STEP 7: Run database migrations: sqlx migrate run"
echo ""
echo "STEP 8: Test compilation and integration: cargo check"
echo ""
echo "STEP 9: Run comprehensive quality checks: ./scripts/check.sh"
echo "   This will:"
echo "   - Run all tests including your new module"
echo "   - Export updated OpenAPI specification to docs/openapi.json"
echo "   - Generate frontend API types"
echo "   - Validate code quality with clippy and formatting"
echo ""
echo "STEP 10: Test your module specifically: cargo nextest run $MODULE_NAME_PLURAL"
echo ""
echo "Generated files:"
echo "  - $MODULE_DIR/ (module source code)"
echo "  - starter/migrations/${NEXT_NUMBER}_${MODULE_NAME_PLURAL}.up.sql"  
echo "  - starter/migrations/${NEXT_NUMBER}_${MODULE_NAME_PLURAL}.down.sql"
echo "  - $TEST_DIR/mod.rs (integration tests)"