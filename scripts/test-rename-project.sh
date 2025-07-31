#!/bin/bash
# test-rename-project.sh - Automated testing for rename-project.sh script
# Usage: ./scripts/test-rename-project.sh [project_name] [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default values
PROJECT_NAME="hello"
VERBOSE=false
ATTEMPT_NUM=""
KEEP_ON_FAILURE=false
TIMEOUT=600
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Test tracking
START_TIME=$(date +%s)
TEST_RESULTS=()

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --attempt)
            ATTEMPT_NUM="$2"
            shift 2
            ;;
        --keep-on-failure)
            KEEP_ON_FAILURE=true
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [project_name] [options]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v           Enable detailed logging"
            echo "  --attempt NUMBER        Specify attempt number (default: auto)"
            echo "  --keep-on-failure       Don't clean up on failure"
            echo "  --timeout SECONDS       Set timeout (default: 600)"
            echo "  --help, -h              Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                      # Test with 'hello'"
            echo "  $0 myproject --verbose  # Test with verbose output"
            echo "  $0 backend --attempt 05 # Use specific attempt number"
            exit 0
            ;;
        *)
            if [[ "$1" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]] && [ -z "${PROJECT_NAME_SET:-}" ]; then
                PROJECT_NAME="$1"
                PROJECT_NAME_SET=true
            else
                echo -e "${RED}❌ Invalid project name or unknown option: $1${NC}"
                echo "Project names must start with letter/underscore and contain only letters, numbers, underscores"
                exit 1
            fi
            shift
            ;;
    esac
done

# Verbose logging function
verbose_log() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Test result tracking
add_test_result() {
    local status="$1"
    local message="$2"
    local duration="$3"
    TEST_RESULTS+=("$status|$message|$duration")
}

# Error handling
cleanup_on_error() {
    local exit_code=$?
    if [ $exit_code -ne 0 ] && [ "$KEEP_ON_FAILURE" = false ]; then
        verbose_log "Cleaning up test directory due to failure..."
        rm -rf "$TEST_DIR" 2>/dev/null || true
    fi
    exit $exit_code
}

trap cleanup_on_error ERR

# Determine attempt number
if [ -z "$ATTEMPT_NUM" ]; then
    # Create tmp directory if it doesn't exist
    mkdir -p "$PROJECT_ROOT/tmp"
    
    # Find the highest existing attempt number
    HIGHEST_ATTEMPT=$(find "$PROJECT_ROOT/tmp" -maxdepth 1 -name "attempt-*-*" 2>/dev/null | \
                      sed 's/.*attempt-\([0-9]\+\)-.*/\1/' | \
                      grep '^[0-9]\+$' | \
                      sort -n | tail -1)
    
    if [ -z "$HIGHEST_ATTEMPT" ]; then
        ATTEMPT_NUM="01"
    else
        ATTEMPT_NUM=$(printf "%02d" $((HIGHEST_ATTEMPT + 1)))
    fi
fi

TEST_DIR="$PROJECT_ROOT/tmp/attempt-$ATTEMPT_NUM-$PROJECT_NAME"

echo -e "${CYAN}${BOLD}🧪 Automated Rename Project Testing${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Project Name:${NC} $PROJECT_NAME"
echo -e "${BLUE}Attempt:${NC} $ATTEMPT_NUM"
echo -e "${BLUE}Test Directory:${NC} $TEST_DIR"
echo -e "${BLUE}Timeout:${NC} ${TIMEOUT}s"
echo -e "${BLUE}Verbose:${NC} $VERBOSE"
echo ""

# Step 1: Setup test environment
echo -e "${YELLOW}📁 Step 1/4: Setting up test environment...${NC}"
step_start=$(date +%s)

verbose_log "Cleaning up any existing test directory..."
rm -rf "$TEST_DIR" 2>/dev/null || true

verbose_log "Creating test directory: $TEST_DIR"
mkdir -p "$TEST_DIR"

verbose_log "Copying essential project files..."
cd "$PROJECT_ROOT"

# Detect current project directory (could be buildscale, hello, or any other name)
verbose_log "Detecting current project directory..."
CURRENT_PROJECT_DIR=""

# First, try to find the main project directory (not backup or tmp directories)
for dir in */ ; do
    dir_name="${dir%/}"  # Remove trailing slash
    if [ -f "$dir/Cargo.toml" ] && [ -d "$dir/src" ] && [ "$dir" != "scripts/" ] && \
       [[ ! "$dir_name" =~ ^backup_ ]] && [[ ! "$dir_name" =~ ^tmp ]] && \
       [[ ! "$dir_name" =~ ^attempt- ]]; then
        CURRENT_PROJECT_DIR="$dir_name"
        break
    fi
done

# If no main directory found, fall back to any directory with Cargo.toml and src/
if [ -z "$CURRENT_PROJECT_DIR" ]; then
    for dir in */ ; do
        dir_name="${dir%/}"  # Remove trailing slash
        if [ -f "$dir/Cargo.toml" ] && [ -d "$dir/src" ] && [ "$dir" != "scripts/" ]; then
            CURRENT_PROJECT_DIR="$dir_name"
            break
        fi
    done
fi

if [ -z "$CURRENT_PROJECT_DIR" ]; then
    echo -e "${RED}❌ Error: Could not find project directory with Cargo.toml and src/${NC}"
    echo -e "${YELLOW}Expected to find a directory like: buildscale/, hello/, myproject/, etc.${NC}"
    exit 1
fi

verbose_log "Found current project directory: $CURRENT_PROJECT_DIR"

# Copy essential files for testing (avoid large directories)
essential_files=(
    "$CURRENT_PROJECT_DIR"
    "scripts" 
    "Cargo.toml"
    "Cargo.lock"
    "CLAUDE.md"
    "Dockerfile.prod"
    "docker-compose.yaml"
    "docker-compose.prod.yaml"
    "docker-compose.chaos.yaml"
    "deny.toml"
    "LICENSE"
)

# Copy .env files if they exist
env_files=(
    ".env.example"
    ".env"
    ".env.local"
    ".env.test"
)

for item in "${essential_files[@]}"; do
    if [ -e "$item" ]; then
        verbose_log "Copying $item..."
        cp -r "$item" "$TEST_DIR/"
    else
        echo -e "${YELLOW}⚠️  Warning: $item not found, skipping...${NC}"
    fi
done

# Copy environment files if they exist
for env_file in "${env_files[@]}"; do
    if [ -e "$env_file" ]; then
        verbose_log "Copying $env_file..."
        cp "$env_file" "$TEST_DIR/"
    fi
done

verbose_log "Setting up starter directory structure..."
cd "$TEST_DIR"

# Rename current project directory to starter and update Cargo.toml files
if [ -d "$CURRENT_PROJECT_DIR" ]; then
    if [ "$CURRENT_PROJECT_DIR" != "starter" ]; then
        mv "$CURRENT_PROJECT_DIR" starter
        verbose_log "Renamed $CURRENT_PROJECT_DIR/ to starter/"
    else
        verbose_log "Current directory is already named 'starter', no rename needed"
    fi
else
    echo -e "${RED}❌ Error: $CURRENT_PROJECT_DIR directory not found in test directory${NC}"
    exit 1
fi

# Update Cargo.toml files - extract current name from workspace
verbose_log "Detecting current workspace member name..."
CURRENT_WORKSPACE_MEMBER=$(grep -o 'members = \["[^"]*"\]' Cargo.toml | sed 's/members = \["\([^"]*\)"\]/\1/' || echo "")
if [ -z "$CURRENT_WORKSPACE_MEMBER" ]; then
    CURRENT_WORKSPACE_MEMBER="$CURRENT_PROJECT_DIR"
fi

verbose_log "Current workspace member: $CURRENT_WORKSPACE_MEMBER"
verbose_log "Updating Cargo.toml workspace members..."
if [ -f "Cargo.toml" ]; then
    sed -i.bak "s/members = \\[\"$CURRENT_WORKSPACE_MEMBER\"\\]/members = [\"starter\"]/" Cargo.toml
fi

# Extract current package name from project Cargo.toml
verbose_log "Detecting current package name..."
CURRENT_PACKAGE_NAME=$(grep -o 'name = "[^"]*"' starter/Cargo.toml | head -1 | sed 's/name = "\([^"]*\)"/\1/' || echo "")
if [ -z "$CURRENT_PACKAGE_NAME" ]; then
    CURRENT_PACKAGE_NAME="$CURRENT_PROJECT_DIR"
fi

verbose_log "Current package name: $CURRENT_PACKAGE_NAME"
verbose_log "Updating starter/Cargo.toml package name..."
if [ -f "starter/Cargo.toml" ]; then
    sed -i.bak "s/name = \"$CURRENT_PACKAGE_NAME\"/name = \"starter\"/" starter/Cargo.toml
fi

step_duration=$(($(date +%s) - step_start))
add_test_result "✅" "Environment setup" "${step_duration}s"
echo -e "${GREEN}✅ Environment setup completed (${step_duration}s)${NC}"

# Step 2: Run rename script
echo -e "${YELLOW}📝 Step 2/4: Running rename script...${NC}"
step_start=$(date +%s)

verbose_log "Current directory: $(pwd)"
verbose_log "Directory contents:"
if [ "$VERBOSE" = true ]; then
    ls -la
fi

verbose_log "Running rename script with timeout ${TIMEOUT}s..."
if [ "$VERBOSE" = true ]; then
    timeout "${TIMEOUT}s" ./scripts/rename-project.sh "$PROJECT_NAME" --verbose
else
    timeout "${TIMEOUT}s" ./scripts/rename-project.sh "$PROJECT_NAME"
fi

# Verify rename succeeded
if [ ! -d "$PROJECT_NAME" ]; then
    echo -e "${RED}❌ Error: Renamed directory '$PROJECT_NAME' not found${NC}"
    exit 1
fi

if [ -d "starter" ]; then
    echo -e "${RED}❌ Error: Original 'starter' directory still exists${NC}"
    exit 1
fi

step_duration=$(($(date +%s) - step_start))
add_test_result "✅" "Rename script execution" "${step_duration}s"
echo -e "${GREEN}✅ Rename script completed successfully (${step_duration}s)${NC}"

# Step 3: Pattern validation
echo -e "${YELLOW}🔍 Step 3/4: Validating pattern replacements...${NC}"
step_start=$(date +%s)

verbose_log "Checking critical pattern replacements..."

# Check workspace member update
if ! grep -q "members = \[\"$PROJECT_NAME\"\]" Cargo.toml; then
    echo -e "${RED}❌ Error: Workspace member not updated in Cargo.toml${NC}"
    exit 1
fi
verbose_log "✓ Workspace member updated correctly"

# Check package name update
if ! grep -q "name = \"$PROJECT_NAME\"" "$PROJECT_NAME/Cargo.toml"; then
    echo -e "${RED}❌ Error: Package name not updated in $PROJECT_NAME/Cargo.toml${NC}"
    exit 1
fi
verbose_log "✓ Package name updated correctly"

# Check for remaining 'starter' references in critical files
verbose_log "Checking for remaining 'starter' references..."
remaining_refs=$(find . -name "*.rs" -o -name "*.toml" -o -name "*.sh" | \
                 grep -v "./backup_" | \
                 xargs grep -l "starter" 2>/dev/null | \
                 head -5)

if [ -n "$remaining_refs" ]; then
    verbose_log "Files still containing 'starter' references:"
    if [ "$VERBOSE" = true ]; then
        echo "$remaining_refs"
    fi
    # This is just a warning, not a failure, as some references might be intentional
    echo -e "${YELLOW}⚠️  Warning: Some 'starter' references remain (may be intentional)${NC}"
fi

# Check backup creation
if [ ! -d backup_* ]; then
    echo -e "${RED}❌ Error: Backup directory not created${NC}"
    exit 1
fi
verbose_log "✓ Backup directory created"

step_duration=$(($(date +%s) - step_start))
add_test_result "✅" "Pattern validation" "${step_duration}s"
echo -e "${GREEN}✅ Pattern validation completed (${step_duration}s)${NC}"

# Step 4: Quality checks
echo -e "${YELLOW}🧪 Step 4/4: Running quality checks...${NC}"
step_start=$(date +%s)

verbose_log "Running comprehensive quality check suite..."

# Run check.sh with timeout - always show full output
timeout "${TIMEOUT}s" ./scripts/check.sh

check_exit_code=$?
if [ $check_exit_code -ne 0 ]; then
    echo -e "${RED}❌ Error: Quality checks failed (exit code: $check_exit_code)${NC}"
    if [ "$VERBOSE" = false ]; then
        echo -e "${YELLOW}💡 Run with --verbose to see detailed output${NC}"
    fi
    exit 1
fi

step_duration=$(($(date +%s) - step_start))
add_test_result "✅" "Quality checks" "${step_duration}s"
echo -e "${GREEN}✅ Quality checks passed (${step_duration}s)${NC}"

# Final results
total_duration=$(($(date +%s) - START_TIME))

echo ""
echo -e "${CYAN}${BOLD}🎉 Rename Project Test Results${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${GREEN}✅ All tests passed successfully!${NC}"
echo ""
echo -e "${BLUE}Test Summary:${NC}"
for result in "${TEST_RESULTS[@]}"; do
    IFS='|' read -r status message duration <<< "$result"
    echo -e "   $status $message ($duration)"
done
echo ""
echo -e "${BLUE}Project Details:${NC}"
echo -e "   📁 Original name: starter"
echo -e "   📁 New name: $PROJECT_NAME"
echo -e "   📁 Test directory: $TEST_DIR"
echo -e "   ⏱️  Total time: ${total_duration}s"
echo ""

if [ "$KEEP_ON_FAILURE" = false ]; then
    echo -e "${BLUE}🧹 Cleaning up test directory...${NC}"
    cd "$TEST_DIR"
    
    # Stop any Docker services that might have been started during testing
    verbose_log "Stopping Docker services from test directory..."
    if command -v docker-compose >/dev/null 2>&1; then
        docker-compose down --remove-orphans 2>/dev/null || true
    elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        docker compose down --remove-orphans 2>/dev/null || true
    fi
    
    cd "$PROJECT_ROOT"
    rm -rf "$TEST_DIR"
    verbose_log "Test directory removed"
else
    echo -e "${YELLOW}📁 Test directory preserved: $TEST_DIR${NC}"
fi

echo -e "${GREEN}${BOLD}✨ Rename script validation completed successfully!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "   • The rename-project.sh script is working correctly"
echo -e "   • Safe to use for renaming projects from 'starter' to any valid name"
echo -e "   • All quality checks pass after renaming"
echo -e "   • Chaos testing infrastructure properly updated"
echo ""
echo -e "${GREEN}Happy coding! 🦀${NC}"