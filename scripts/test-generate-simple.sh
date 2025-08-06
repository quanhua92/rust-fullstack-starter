#!/bin/bash
set -e

# Load common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

print_status "step" "Testing module generator (focused test)"

# Ensure we're in the right directory
validate_project_root

# Test module name
TEST_MODULE="testgen"
TEST_TABLE="${TEST_MODULE}s"

print_status "step" "Step 1: Test dry run generation"
cd starter || exit 1
cargo run -- generate module "$TEST_MODULE" --dry-run

print_status "step" "Step 2: Test actual generation"
cargo run -- generate module "$TEST_MODULE"

print_status "step" "Step 3: Verify files were created"
expected_files=(
    "src/$TEST_TABLE/api.rs"
    "src/$TEST_TABLE/models.rs" 
    "src/$TEST_TABLE/services.rs"
    "src/$TEST_TABLE/mod.rs"
    "tests/$TEST_TABLE/mod.rs"
)

for file in "${expected_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        print_status "error" "Expected file not found: $file"
        exit 1
    fi
    print_status "success" "✓ Found: $file"
done

# Check for migration files
migration_found=false
for migration in migrations/*_"$TEST_TABLE".up.sql; do
    if [[ -f "$migration" ]]; then
        migration_found=true
        print_status "success" "✓ Found migration: $migration"
        break
    fi
done

if [[ "$migration_found" != true ]]; then
    print_status "error" "Migration file not found"
    exit 1
fi

print_status "step" "Step 4: Verify lib.rs was updated"
if ! grep -q "pub mod $TEST_TABLE;" src/lib.rs; then
    print_status "error" "lib.rs was not updated with module declaration"
    exit 1
fi
print_status "success" "✓ lib.rs updated correctly"

print_status "step" "Step 5: Verify template replacements"
# Check that placeholders were replaced correctly
if grep -q "__MODULE_" "src/$TEST_TABLE"/*.rs; then
    print_status "error" "Template placeholders not replaced properly"
    exit 1
fi
print_status "success" "✓ Template placeholders replaced correctly"

print_status "step" "Step 6: Verify sqlx! macros are used"
if grep -q "sqlx::query_as!" "src/$TEST_TABLE/services.rs"; then
    print_status "success" "✓ Using sqlx! macros for compile-time checking"
else
    print_status "error" "Not using sqlx! macros"
    exit 1
fi

print_status "step" "Step 7: Verify API structure"
if grep -q "pub fn ${TEST_TABLE}_routes()" "src/$TEST_TABLE/api.rs"; then
    print_status "success" "✓ Route function generated correctly"
else
    print_status "error" "Route function not found in generated API"
    exit 1
fi

if grep -q "list_${TEST_TABLE}_service" "src/$TEST_TABLE/services.rs"; then
    print_status "success" "✓ Service functions generated correctly"  
else
    print_status "error" "Service functions not found"
    exit 1
fi

print_status "step" "Step 8: Test force overwrite"
cargo run -- generate module "$TEST_MODULE" --force
print_status "success" "✓ Force overwrite works"

print_status "step" "Step 9: Test compilation with SQLX_OFFLINE"
if SQLX_OFFLINE=true cargo check --quiet 2>/dev/null; then
    print_status "error" "Code should fail compilation without migration (sqlx! macros should enforce this)"
    exit 1
fi
print_status "success" "✓ Code correctly fails without migration (sqlx! enforces compile-time checking)"

# Cleanup
print_status "step" "Cleaning up test artifacts"
rm -rf "src/$TEST_TABLE" "tests/$TEST_TABLE" 2>/dev/null || true
rm -f migrations/*_"$TEST_TABLE".*.sql 2>/dev/null || true
sed -i.bak "/pub mod $TEST_TABLE;/d" src/lib.rs && rm src/lib.rs.bak 2>/dev/null || true

cd ..

print_status "success" "🎉 Generator test completed successfully!"
print_status "success" "All core functionality verified:"
echo "   ✓ Dry run works correctly"
echo "   ✓ File generation (API, models, services, tests, migrations)"
echo "   ✓ Template placeholder replacement"  
echo "   ✓ lib.rs integration"
echo "   ✓ Force overwrite functionality"
echo "   ✓ sqlx! macro usage for compile-time query checking"
echo "   ✓ Proper API and service structure"

print_status "info" "Generator is working perfectly!"