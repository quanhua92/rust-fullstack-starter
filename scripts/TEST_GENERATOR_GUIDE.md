# Generator Testing Guide

This guide explains how to test the module generator system, both for validating the existing functionality and for developing new templates.

## Table of Contents

1. [Overview](#overview)
2. [Quick Testing](#quick-testing)
3. [Manual Testing Workflow](#manual-testing-workflow)
4. [Template Development Testing](#template-development-testing)
5. [Integration Testing](#integration-testing)
6. [Troubleshooting](#troubleshooting)
7. [Adding New Templates](#adding-new-templates)

## Overview

The module generator system has several testing layers:

- **Automated Script**: `./scripts/test-generate.sh` - Comprehensive automated testing
- **Manual Workflow**: Step-by-step testing of individual features
- **Unit Tests**: Generated code includes unit tests that validate basic functionality
- **Integration Tests**: Full API endpoint testing (requires manual 3-step integration: lib.rs → server.rs → openapi.rs)

## Quick Testing

### Run Complete Test Suite

```bash
# From project root
./scripts/test-generate.sh
```

This will test:
- ✅ Basic template generation and workflow
- ✅ Production template generation and workflow  
- ✅ Compilation with sqlx! macros
- ✅ Database migrations
- ✅ Unit tests
- ✅ Force overwrite functionality
- ✅ Dry-run functionality
- ✅ Revert functionality
- ✅ Error handling

**Expected Duration**: ~2-3 minutes

### Run Quick Test Suite

```bash
# From project root
./scripts/test-generate-simple.sh
```

This provides faster validation with:
- ✅ Basic template generation
- ✅ Core functionality verification
- ✅ Compilation validation
- ✅ Quick cleanup

**Expected Duration**: ~30-60 seconds

### Quick Manual Test

```bash
# Generate a test module (from project root)
cargo run -- generate module quicktest --template basic --force

# Follow the workflow
cd starter && sqlx migrate run
cd .. && ./scripts/prepare-sqlx.sh  

# Manual Integration (3 steps - required for testing):
# Step 1: Add to src/lib.rs: pub mod quicktest;
# Step 2: Add to src/server.rs: use + routes (for full testing)
# Step 3: Add to src/openapi.rs: model imports (for API docs)

./scripts/check.sh

# Test it works (after manual integration)
cd starter && cargo nextest run quicktest

# Clean up (from project root)
cargo run -- revert module quicktest --yes
# Then manually remove from lib.rs, server.rs, openapi.rs
```

## Manual Testing Workflow

### Testing Basic Template

```bash
cd starter

# 1. Generate module with name transformations display
cargo run -- generate module book --template basic

# Expected output should show:
# 📝 Name transformations:
#    Module name (singular): book
#    Module name (plural):   books
#    Struct name:           Book
#    Table name:            books

# 2. Run migration
sqlx migrate run

# 3. Update query cache (use script for reliability)
cd .. && ./scripts/prepare-sqlx.sh

# 4. Run quality checks (recommended - includes compilation, linting, tests)
./scripts/check.sh
# Should complete successfully

# 5. Run unit tests
cargo nextest run books
# Should pass 2 tests: book_creation, book_update

# 6. Test dry-run functionality
cargo run -- generate module dryruntest --template basic --dry-run
# Should show what would be created without creating files

# 7. Test revert dry-run
cargo run -- revert module book --dry-run
# Should show revert plan without making changes

# 8. Test revert functionality
cargo run -- revert module book --yes
# Should clean up all files and revert migration

# 9. Verify clean state
cargo check
# Should compile cleanly
```

### Testing Production Template

```bash
cd starter

# 1. Generate production module
cargo run -- generate module product --template production

# 2. Check advanced features in generated files
cat src/products/models.rs
# Should contain: ProductStatus enum, priority field, metadata JSONB

cat migrations/*_products.up.sql  
# Should contain: enum types, indexes, triggers, GIN indexes

# 3. Follow standard workflow
sqlx migrate run
cd .. && ./scripts/prepare-sqlx.sh
./scripts/check.sh

# 4. Run unit tests
cargo nextest run products

# 5. Clean up
cargo run -- revert module product --yes
```

### Testing Force Overwrite

```bash
cd starter

# 1. Generate module
cargo run -- generate module test --template basic
sqlx migrate run

# 2. Test force overwrite
cargo run -- generate module test --template basic --force
# Should overwrite without prompting

# 3. Clean up
cargo run -- revert module test --yes
```

### Testing Error Conditions

```bash
cd starter

# 1. Test invalid template
cargo run -- generate module test --template nonexistent
# Should fail with clear error message

# 2. Test revert non-existent module  
cargo run -- revert module nonexistent --dry-run
# Should handle gracefully with appropriate message
```

## Template Development Testing

### When Creating New Templates

1. **Create Template Structure**:
   ```bash
   mkdir templates/my-template
   cd templates/my-template
   ```

2. **Required Template Files**:
   ```
   templates/my-template/
   ├── api.rs          # API endpoints
   ├── models.rs       # Data models  
   ├── services.rs     # Business logic
   ├── mod.rs          # Module exports
   ├── tests.rs        # Integration tests
   ├── up.sql          # Database migration
   └── down.sql        # Migration rollback
   ```

3. **Template Development Cycle**:
   ```bash
   # Generate with your template
   cargo run -- generate module testmodule --template my-template

   # Manual Integration (required for testing):
   # Step 1: Add to src/lib.rs: pub mod testmodule;
   # Step 2: Add to src/server.rs: use + routes 
   # Step 3: Add to src/openapi.rs: model imports

   # Test compilation
   sqlx migrate run
   cd .. && ./scripts/prepare-sqlx.sh
   ./scripts/check.sh

   # Fix any compilation errors in templates
   # Revert and try again
   cargo run -- revert module testmodule --yes
   # Then manually remove from lib.rs, server.rs, openapi.rs
   
   # Repeat until working
   ```

4. **Template Validation Checklist**:
   - [ ] Uses correct placeholder format (`__MODULE_NAME__`, `__MODULE_STRUCT__`, etc.)
   - [ ] Compiles without errors after migration
   - [ ] Uses sqlx! macros (not string concatenation)
   - [ ] Includes proper error handling
   - [ ] Has meaningful unit tests in models.rs
   - [ ] Integration tests use existing test infrastructure
   - [ ] Migration has proper rollback (down.sql)

### Required Placeholders

| Placeholder | Example | Usage |
|-------------|---------|-------|
| `__MODULE_NAME__` | `book` | Variable names, function parameters |
| `__MODULE_NAME_PLURAL__` | `books` | URLs, directory names, table names |
| `__MODULE_STRUCT__` | `Book` | Struct names, type names |
| `__MODULE_TABLE__` | `books` | Database table names |

### Template Testing Commands

```bash
# Test placeholder replacement
cargo run -- generate module testitem --template my-template --dry-run

# Verify transformations are correct:
# testitem -> testitems (plural)
# testitem -> Testitem (struct)
# testitem -> testitems (table)

# Test actual generation
cargo run -- generate module testitem --template my-template
```

## Integration Testing

### Template-Specific API Testing

Use the dedicated template testing script for comprehensive API validation:

```bash
# Test generated module with real HTTP requests
./scripts/test-template-with-curl.sh products        # Test on default port 3000
./scripts/test-template-with-curl.sh books 8080      # Test on custom port
./scripts/test-template-with-curl.sh --help          # Show help and options
```

**Features of Template Testing Script:**
- ✅ **Automatic Authentication** - Handles user registration and login
- ✅ **Complete CRUD Testing** - Tests all endpoints with real data
- ✅ **Search Validation** - Tests search parameters and filters
- ✅ **Error Handling** - Validates 404s and unauthorized access
- ✅ **RBAC Integration** - Tests role-based access controls
- ✅ **Colorized Output** - Clear success/failure indicators

**Integration Workflow (Manual Steps Required):**
```bash
# 1. Generate and setup module
cargo run -- generate module products --template production
cd starter && sqlx migrate run
cd .. && ./scripts/prepare-sqlx.sh

# 2. Manual Integration (3 steps - prevents accidental commits):
# Step 1: Add to src/lib.rs: pub mod products;
# Step 2: Add to src/server.rs: use crate::products::api::products_routes;
#         and .nest("/products", products_routes())
# Step 3: Add to src/openapi.rs: use crate::products::models::*;

# 3. Start server and test
./scripts/server.sh                                  # Start server on port 3000
./scripts/test-template-with-curl.sh products        # Test the API endpoints

# 4. Clean up
cargo run -- revert module products --yes
# Then manually remove from lib.rs, server.rs, openapi.rs
```

### Manual Integration Testing

For deeper integration testing, follow the 3-step manual integration process:

1. **Add to `src/lib.rs`**:
   ```rust
   pub mod books;
   ```

2. **Add to `src/server.rs`**:
   ```rust
   use crate::books::api::books_routes;
   
   // In build_router function:
   .nest("/api/v1/books", books_routes())
   ```

3. **Add to `src/openapi.rs`**:
   ```rust
   use crate::books::models::*;
   ```

**Important**: This manual integration prevents generated modules from accidentally being committed to your repository. Always remember to remove these integrations when testing is complete.

3. **Run Integration Tests**:
   ```bash
   cargo nextest run books --no-fail-fast
   ```

4. **Test API Endpoints**:
   ```bash
   # Start server
   cargo run -- server --port 3000
   
   # In another terminal, test endpoints
   curl http://localhost:3000/api/v1/books
   ```

## Troubleshooting

### Common Issues

**Compilation Errors after Generation**:
```bash
error: relation "books" does not exist
```
**Solution**: Run `sqlx migrate run`, then use `./scripts/prepare-sqlx.sh` and `./scripts/check.sh` (from project root)

**SQLx Cache Issues**:
```bash  
error: query does not match cached data
```
**Solution**: Use `./scripts/prepare-sqlx.sh` to update cache reliably (from project root)

**Template Not Found**:
```bash
Error: Template 'custom' not found in templates directory
```
**Solution**: Ensure template directory exists in `templates/custom/`

**Migration Conflicts**:
```bash
error: migration X was previously applied but has been modified
```
**Solution**: Use revert command first: `cargo run -- revert module name --yes`

### Debug Template Issues

1. **Check Template Syntax**:
   ```bash
   # Generate with dry-run to see output without creating files
   cargo run -- generate module debug --template my-template --dry-run
   ```

2. **Verify Placeholder Replacement**:
   ```bash
   # Look at generated files to ensure placeholders were replaced
   cat src/debugs/models.rs
   # Should show "Debug" not "__MODULE_STRUCT__"
   ```

3. **Test Minimal Template**:
   Start with working basic template and modify incrementally.

### Reset Environment

If tests leave system in bad state:

```bash
# Reset database
./scripts/reset-all.sh --reset-database

# Clean generated files manually if needed
rm -rf src/test* tests/test* migrations/*_test*

# Remove from lib.rs if needed
sed -i '/pub mod test/d' src/lib.rs
```

## Adding New Templates

### Template Creation Process

1. **Copy Existing Template**:
   ```bash
   cp -r templates/basic templates/my-template
   ```

2. **Modify Template Files**:
   - Update functionality in `api.rs`, `models.rs`, `services.rs`
   - Ensure `up.sql` and `down.sql` are complementary
   - Update `tests.rs` for new functionality

3. **Test Template**:
   ```bash
   ./scripts/test-generate.sh
   # Or manual testing as described above
   ```

4. **Document Template**:
   Add template description to `docs/module-generator.md`

### Template Best Practices

- **Start Simple**: Begin with basic template and add features incrementally
- **Test Frequently**: Generate and test after each significant change
- **Use Real Patterns**: Base templates on actual production code patterns
- **Include Tests**: Every template should have meaningful unit tests
- **Document Features**: Clearly document what the template provides

### Template Validation

Before submitting new templates:

```bash
# 1. Run automated tests
./scripts/test-generate.sh

# 2. Test edge cases
cargo run -- generate module "edge-case" --template my-template
cargo run -- generate module "s" --template my-template  # Single letter
cargo run -- generate module "items" --template my-template  # Already plural

# 3. Test all functionality
# - Generation works
# - Migration works  
# - Compilation works
# - Tests pass
# - Revert works
# - Force overwrite works
# - Dry-run works

# 4. Verify cleanup
./scripts/check.sh  # Should be clean after all tests
```

## Continuous Integration

### Automated Testing in CI

Add to CI pipeline:

```bash
# Test generator system
./scripts/test-generate.sh

# Ensure clean state after tests
./scripts/check.sh
```

### Template Validation

For new templates, CI should verify:
- All required files present
- Template compiles successfully
- Generated tests pass
- Migration is reversible
- No leftover files after revert

This ensures the generator system remains reliable as new templates are added.

## Summary

The generator testing system provides multiple layers of validation:

1. **Automated Script** (`test-generate.sh`) - Primary testing method
2. **Manual Workflow** - For detailed debugging and development
3. **Unit Tests** - Generated code self-validation
4. **Integration Tests** - Full API testing (manual route registration required)

Always run the automated script before making changes to templates or generator logic. For template development, use the manual workflow to iterate quickly and debug issues.

The system is designed to be safe - dry-run functionality lets you preview changes, and the revert command ensures clean rollback of any changes made during testing.