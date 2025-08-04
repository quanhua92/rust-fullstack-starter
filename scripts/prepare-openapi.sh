#!/bin/bash

# prepare-openapi.sh - Generate and update OpenAPI specification
# This script uses the built-in export-openapi CLI command to update OpenAPI files

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

print_status "step" "Preparing OpenAPI specification..."

validate_project_root

print_status "step" "Building Rust project to ensure latest OpenAPI schema..."
run_cmd "Building project" cargo build

print_status "step" "Exporting OpenAPI specification..."
run_cmd "Exporting to docs/openapi.json" cargo run -- export-openapi

# Also update the legacy location for backwards compatibility
if [[ -f "docs/openapi.json" ]]; then
    cp docs/openapi.json starter/docs/openapi.json
    print_status "success" "Updated starter/docs/openapi.json (legacy location)"
fi

print_status "step" "Regenerating TypeScript API types..."
if [[ -d "web" ]]; then
    cd web
    if command -v pnpm >/dev/null 2>&1; then
        run_cmd "Generating TypeScript types" pnpm run generate-api
        print_status "success" "Updated web/src/types/api.ts"
    else
        print_status "warning" "pnpm not found, skipping TypeScript type generation"
    fi
    cd ..
else
    print_status "warning" "web directory not found, skipping TypeScript type generation"
fi

print_status "step" "Validating updated files..."

# Check that the key enums have lowercase values
if [[ -f "docs/openapi.json" ]]; then
    TASK_PRIORITY=$(jq -r '.components.schemas.TaskPriority.enum[0]' docs/openapi.json 2>/dev/null || echo "null")
    TASK_STATUS=$(jq -r '.components.schemas.TaskStatus.enum[0]' docs/openapi.json 2>/dev/null || echo "null")

    if [[ "$TASK_PRIORITY" == "low" && "$TASK_STATUS" == "pending" ]]; then
        print_status "success" "Enum values are correctly lowercase"
    else
        print_status "warning" "Enum values may not be lowercase (TaskPriority: $TASK_PRIORITY, TaskStatus: $TASK_STATUS)"
    fi

    # Validate file sizes
    MAIN_SIZE=$(stat -f%z docs/openapi.json 2>/dev/null || stat -c%s docs/openapi.json 2>/dev/null || echo "0")
    if [[ "$MAIN_SIZE" -gt 1000 ]]; then
        print_status "success" "OpenAPI specification updated successfully (${MAIN_SIZE} bytes)"
    else
        print_status "error" "OpenAPI file seems too small (${MAIN_SIZE} bytes)"
        exit 1
    fi
else
    print_status "error" "OpenAPI file not found after export"
    exit 1
fi

print_status "success" "OpenAPI specification preparation complete"
print_status "info" "Files updated:"
print_status "info" "  - docs/openapi.json"
print_status "info" "  - starter/docs/openapi.json"
if [[ -f "web/src/types/api.ts" ]]; then
    print_status "info" "  - web/src/types/api.ts"
fi

print_status "info" "You can now commit these changes with the updated OpenAPI specification"