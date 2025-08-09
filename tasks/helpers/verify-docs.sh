#!/bin/bash

# Single Bulletproof Documentation Verification Script
# Works reliably every time, all modes

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERBOSE=false
QUICK=false
FULL=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    local level="$1"; shift; local msg="$*"
    case "$level" in
        ERROR)   echo -e "${RED}✗${NC} $msg" ;;
        SUCCESS) echo -e "${GREEN}✓${NC} $msg" ;;
        WARN)    echo -e "${YELLOW}⚠${NC} $msg" ;;
        INFO)    echo -e "${BLUE}ℹ${NC} $msg" ;;
        DEBUG)   [ "$VERBOSE" = true ] && echo -e "${BLUE}🔍${NC} $msg" ;;
        *) echo "$msg" ;;
    esac
}

show_help() {
    cat << 'EOF'
verify.sh - Simple Documentation Verification

USAGE:
    verify.sh           # Standard verification
    verify.sh --quick   # Fast check (structures only)  
    verify.sh --full    # Everything including URLs
    verify.sh --verbose # Show detailed output
    verify.sh --help    # This help

That's it! Works the same way every time.
EOF
}

verify_all() {
    log INFO "Documentation Verification"
    local errors=0
    
    # 1. Check environment
    log DEBUG "Checking environment..."
    if [ ! -d "$PROJECT_ROOT/docs" ] || [ ! -d "$PROJECT_ROOT/starter/src" ]; then
        log ERROR "Required directories not found"
        return 1
    fi
    
    for tool in rg find; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            log ERROR "Required tool not found: $tool"
            return 1
        fi
    done
    
    # 2. Check code structures
    log INFO "Checking code structures..."
    
    # API Response check - updated for new modular structure
    local doc_api=$(rg "ApiResponse.*\{" "$PROJECT_ROOT/docs" | wc -l | tr -d ' ')
    local code_api=0
    if [ -f "$PROJECT_ROOT/starter/src/api/response.rs" ]; then
        code_api=$(rg "pub struct ApiResponse" "$PROJECT_ROOT/starter/src/api/response.rs" | wc -l | tr -d ' ')
    fi
    
    if [ "$code_api" -gt 0 ]; then
        log SUCCESS "ApiResponse: code($code_api) docs($doc_api)"
    else
        log WARN "ApiResponse not found in code"
    fi
    
    # Database tables - exclude PostgreSQL system tables (pg_*)
    log DEBUG "Checking database tables..."
    local doc_tables migration_tables
    
    doc_tables=$(find "$PROJECT_ROOT/docs" -name "*.md" -not -path "*/ideas/*" \
                -exec rg "INSERT INTO|FROM|JOIN|CREATE TABLE" {} + 2>/dev/null | \
                rg -o "(?:INSERT INTO|FROM|JOIN|CREATE TABLE)\\s+([a-z_]+)" -r '$1' | \
                grep -v -E "^(app_user|gcr|rust|books)$" | \
                grep -v "^pg_" | \
                sort -u) || doc_tables=""
    
    migration_tables=$(find "$PROJECT_ROOT/starter/migrations" -name "*.sql" \
                      -exec cat {} + 2>/dev/null | \
                      rg -o "CREATE TABLE ([a-z_]+)" -r '$1' | \
                      sort -u) || migration_tables=""
    
    if [ -n "$doc_tables" ] && [ -n "$migration_tables" ]; then
        while read -r table; do
            if [ -n "$table" ]; then
                if echo "$migration_tables" | grep -q "^$table$"; then
                    log DEBUG "✓ Table '$table' found"
                else
                    log ERROR "Table '$table' in docs but not in migrations"
                    errors=$((errors + 1))
                fi
            fi
        done <<< "$doc_tables"
    fi
    
    # Skip CLI command verification (not needed)
    
    # 3. File references (if not quick mode) - use temp file to preserve error count
    if [ "$QUICK" != true ]; then
        log INFO "Checking file references..."
        
        local temp_file="/tmp/verify_refs_$$"
        find "$PROJECT_ROOT/docs" -name "*.md" -exec rg "\\./[a-zA-Z0-9][a-zA-Z0-9/_.-]*" -Ho {} + 2>/dev/null > "$temp_file"
        
        while IFS=: read -r file_path match; do
            if [ -n "$match" ]; then
                local check_path="$PROJECT_ROOT/${match#./}"
                if [ -f "$check_path" ] || [ -d "$check_path" ]; then
                    log DEBUG "✓ Found: $match"
                else
                    log ERROR "Missing: $match (from $(basename "$file_path"))"
                    errors=$((errors + 1))
                fi
            fi
        done < "$temp_file"
        
        rm -f "$temp_file"
    fi
    
    # 4. URLs (if full mode)
    if [ "$FULL" = true ]; then
        log INFO "Checking URLs..."
        if command -v curl >/dev/null 2>&1; then
            local temp_urls="/tmp/verify_urls_$$"
            find "$PROJECT_ROOT/docs" -name "*.md" -exec rg "https?://[a-zA-Z0-9][a-zA-Z0-9._/-]*" -o {} + 2>/dev/null | \
            sort -u > "$temp_urls"
            
            while read -r url; do
                if [ -n "$url" ]; then
                    if timeout 10 curl -s -f -L "$url" >/dev/null 2>&1; then
                        log DEBUG "✓ URL: $url"
                    else
                        log ERROR "URL not accessible: $url"
                        errors=$((errors + 1))
                    fi
                fi
            done < "$temp_urls"
            
            rm -f "$temp_urls"
        else
            log WARN "curl not found - skipping URL checks"
        fi
    fi
    
    # Results
    echo ""
    if [ $errors -eq 0 ]; then
        log SUCCESS "Documentation verification passed!"
    else
        log ERROR "Found $errors issues"
    fi
    
    return $errors
}

# Parse arguments
while [ $# -gt 0 ]; do
    case $1 in
        --help|-h) show_help; exit 0 ;;
        --verbose|-v) VERBOSE=true; shift ;;
        --quick|-q) QUICK=true; shift ;;
        --full|-f) FULL=true; shift ;;
        --dry-run) log INFO "Would run verification"; exit 0 ;;
        *) log ERROR "Unknown option: $1"; exit 1 ;;
    esac
done

# Show help if no args
if [ "$VERBOSE" = false ] && [ "$QUICK" = false ] && [ "$FULL" = false ]; then
    log INFO "Documentation Verification - Usage:"
    log INFO "  $0           # Standard check"
    log INFO "  $0 --quick   # Fast check"  
    log INFO "  $0 --full    # Complete check"
    log INFO "  $0 --verbose # Detailed output"
    log INFO "  $0 --help    # Show help"
    echo ""
fi

# Run verification
verify_all