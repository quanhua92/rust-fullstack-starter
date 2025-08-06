# Module Generator

The module generator provides a powerful way to quickly scaffold complete CRUD modules with API endpoints, business logic, database migrations, and comprehensive tests.

## Overview

The generator creates:
- **API routes** with full REST endpoints
- **Data models** with validation and serialization
- **Business services** with database operations
- **Database migrations** with proper indexes
- **Integration tests** covering all functionality
- **Type-safe code** using sqlx! macros for compile-time validation

## Quick Start

```bash
# Generate a basic module
cargo run -- generate module books --template basic

# Generate with production features
cargo run -- generate module products --template production

# Force overwrite existing files
cargo run -- generate module inventory --template basic --force

# Preview without creating files
cargo run -- generate module orders --template production --dry-run
```

## Commands

### Generate Module

```bash
cargo run -- generate module <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Module name (e.g., "books", "users", "products")

**Options:**
- `--template <TEMPLATE>` - Template to use (default: "basic")
  - `basic` - Simple CRUD with pagination and search
  - `production` - Advanced features with filtering, bulk operations, cursors
- `--force` - Overwrite existing files without prompting
- `--dry-run` - Show what would be created without actually creating files

### Revert Module

```bash
cargo run -- revert module <NAME> [OPTIONS]
```

**Arguments:**
- `<NAME>` - Module name to revert

**Options:**
- `--yes` - Skip all confirmation prompts ⚠️ **DANGEROUS**
- `--dry-run` - Show what would be reverted without doing it

## Templates

### Basic Template

The basic template provides:
- ✅ **CRUD Operations** - Create, Read, Update, Delete
- ✅ **Pagination** - Offset-based with configurable limits
- ✅ **Search** - Text search across name and description fields
- ✅ **Validation** - Input validation with meaningful error messages
- ✅ **Tests** - Complete integration test suite
- ✅ **Authentication** - All endpoints require valid authentication

**Generated Structure:**
```
src/books/
├── api.rs          # REST API endpoints
├── models.rs       # Data models and request/response types
├── services.rs     # Business logic and database operations
└── mod.rs          # Module exports

tests/books/
└── mod.rs          # Integration tests

migrations/
├── XXX_books.up.sql    # Database schema
└── XXX_books.down.sql  # Rollback script
```

**API Endpoints:**
- `GET /api/v1/books` - List books with pagination and search
- `GET /api/v1/books/{id}` - Get book by ID
- `POST /api/v1/books` - Create new book
- `PUT /api/v1/books/{id}` - Update book
- `DELETE /api/v1/books/{id}` - Delete book

### Production Template

The production template includes everything from basic plus:
- ✅ **Advanced Filtering** - Status, priority ranges, date filters
- ✅ **Dual Pagination** - Both offset-based and cursor-based
- ✅ **Bulk Operations** - Create, update, delete multiple items
- ✅ **Status Management** - Enum-based status with database constraints
- ✅ **Metadata Support** - JSON fields with GIN indexing
- ✅ **Performance** - Multiple database indexes and triggers
- ✅ **Error Handling** - Comprehensive error handling with skip options

**Additional Features:**
- Status enum: `active`, `inactive`, `pending`, `archived`
- Priority integer field with range filtering
- Metadata JSONB field for flexible data storage
- Auto-updating `updated_at` trigger
- Comprehensive database indexes for performance

**Additional API Endpoints:**
- `POST /api/v1/products/bulk` - Bulk create
- `PUT /api/v1/products/bulk` - Bulk update
- `DELETE /api/v1/products/bulk` - Bulk delete

**Query Parameters:**
- `limit`, `offset` - Standard pagination
- `cursor` - Cursor-based pagination
- `search` - Text search
- `status` - Filter by status (comma-separated)
- `min_priority`, `max_priority` - Priority range
- `created_after`, `created_before` - Date range
- `sort_by`, `sort_order` - Sorting options

## Workflow

### 1. Generate Module

```bash
cargo run -- generate module books --template basic
```

**Output:**
```
🚀 Generating module 'books' using 'basic' template
📁 Created directory: src/books
📄 Created: src/books/api.rs
📄 Created: src/books/models.rs
📄 Created: src/books/services.rs
📄 Created: src/books/mod.rs
📄 Created: tests/books/mod.rs
📄 Created: migrations/008_books.up.sql
📄 Created: migrations/008_books.down.sql
📝 Updated: src/lib.rs
✅ Module generation completed!
```

### 2. Run Migration

```bash
cd starter && sqlx migrate run
```

### 3. Update Query Cache

```bash
# Use the prepare script for reliability
./scripts/prepare-sqlx.sh
```

### 4. Run Quality Checks

```bash
# Comprehensive quality checks (recommended - includes compilation, linting, tests)
./scripts/check.sh
```

### 5. Add Module Declaration (Manual)

Add to `src/lib.rs`:
```rust
pub mod books;
```

### 6. Add Routes (Manual)

Add to `src/server.rs`:
```rust
use crate::books::api::books_routes;

// In build_router function:
.nest("/api/v1/books", books_routes())
```

### 7. Update OpenAPI (Manual)

Add to `src/openapi.rs`:
```rust
use crate::books::models::*;
```

## Safety and Revert

### Interactive Revert

```bash
cargo run -- revert module books
```

**Interactive prompts:**
```
📋 Revert plan for module 'books':
   ⚠️  Revert database migration #8
   🗑️  Delete 2 migration files
   🗑️  Delete module directory: src/books
   🗑️  Delete test directory: tests/books
   📝 Manual step: Remove module declaration from lib.rs

⚠️  WARNING: This operation will permanently delete files and revert database migrations!

❓ Revert database migration #8? [y/N]: 
❓ Delete all generated files? [y/N]: 

📋 Manual cleanup required:
   📝 Remove from lib.rs: pub mod books;
   📝 Remove from server.rs: use crate::books::api::books_routes;
   📝 Remove from server.rs: .nest("/api/v1/books", books_routes())
   📝 Remove from openapi.rs: any imports from books::models
```

### Automated Revert

```bash
cargo run -- revert module books --yes
```

⚠️ **WARNING**: The `--yes` flag skips all safety prompts. Use with caution!

### Preview Revert

```bash
cargo run -- revert module books --dry-run
```

Shows what would be reverted without making any changes.

## Compile-Time Safety

All generated code uses `sqlx!` macros for compile-time query validation:

```rust
let books = sqlx::query_as!(
    Book,
    "SELECT id, name, description, created_at, updated_at 
     FROM books 
     ORDER BY created_at DESC 
     LIMIT $1 OFFSET $2",
    limit,
    offset
)
.fetch_all(&database.pool)
.await?;
```

**Benefits:**
- ✅ **Compile-time validation** - Queries checked against actual database schema
- ✅ **Type safety** - Automatic type inference from database columns
- ✅ **Migration enforcement** - Code won't compile without proper migrations
- ✅ **Refactoring safety** - Schema changes caught at compile time

## Testing

### Automated Template Testing

Use the dedicated template testing script for comprehensive API validation:

```bash
# Test generated module with real HTTP requests
./scripts/test-template-with-curl.sh products        # Test on default port 3000
./scripts/test-template-with-curl.sh books 8080      # Test on custom port
./scripts/test-template-with-curl.sh --help          # Show comprehensive help
```

**Template Testing Features:**
- ✅ **Automatic Authentication** - Handles user registration and login flow
- ✅ **Complete CRUD Testing** - Tests all endpoints with real data
- ✅ **Search Validation** - Tests search parameters and response formatting
- ✅ **Error Handling** - Validates 404s and unauthorized access responses
- ✅ **RBAC Integration** - Tests role-based access controls properly
- ✅ **Colorized Output** - Clear success/failure indicators with detailed feedback

### Integration Tests

Generated modules include comprehensive integration tests:

```bash
# Run specific module tests
cargo nextest run books

# Run all tests
cargo nextest run
```

**Test Coverage:**
- ✅ **CRUD workflow** - Complete create, read, update, delete cycle
- ✅ **Search functionality** - Text search validation
- ✅ **Access control** - Authentication and authorization tests
- ✅ **Validation** - Input validation and error handling
- ✅ **Bulk operations** - (Production template only)

### Generator System Testing

Test the entire generator system end-to-end:

```bash
# Complete automated testing (both templates)
./scripts/test-generate.sh              # ~2-3 minutes, comprehensive

# Quick validation testing (basic template only)
./scripts/test-generate-simple.sh      # ~30-60 seconds, fast validation
```

## Best Practices

### Naming Conventions

- **Module names**: lowercase, singular (e.g., `book`, `user`, `product`)
- **Generated plurals**: automatic (e.g., `book` → `books`, `user` → `users`)
- **Database tables**: plural form (e.g., `books`, `users`, `products`)

### Template Selection

**Choose Basic When:**
- Building MVPs or prototypes
- Simple CRUD requirements
- Learning the codebase
- Time-sensitive projects

**Choose Production When:**
- Building production applications
- Need advanced filtering and search
- Require bulk operations
- Performance is critical
- Complex business requirements

### Development Workflow

1. **Start with Basic** - Use basic template for initial development
2. **Manual Integration** - Follow the 3-step integration process (lib.rs → server.rs → openapi.rs)
3. **Test Early** - Run integration tests immediately after generation and integration
4. **Customize Gradually** - Modify generated code to fit specific needs
5. **Use Revert Safely** - Always use `--dry-run` first, then interactive mode with manual cleanup
6. **Version Control** - Commit generated code before customization

## Troubleshooting

### Common Issues

**Migration Conflicts:**
```bash
error: migration X was previously applied but has been modified
```
**Solution:** Use `cargo run -- revert module <name> --yes` then regenerate

**Compilation Errors:**
```bash
error: relation "table_name" does not exist
```
**Solution:** Run `sqlx migrate run`, then use `./scripts/prepare-sqlx.sh` and `./scripts/check.sh`

**SQLx Cache Issues:**
```bash
error: query does not match cached data
```
**Solution:** Use `./scripts/prepare-sqlx.sh` to reliably update the query cache

**Template Testing Failures:**
```bash
./scripts/test-template-with-curl.sh fails with connection errors
```
**Solution:** Ensure server is running first: `./scripts/server.sh`, then run template tests

**Route Integration Issues:**
```bash
Templates generate but API endpoints return 404
```
**Solution:** Manually integrate module in three steps:
```rust
// 1. Add to src/lib.rs:
pub mod books;

// 2. Add to src/server.rs:
use crate::books::api::books_routes;
// In router: .nest("/books", books_routes())

// 3. Add to src/openapi.rs:
use crate::books::models::*;
```

**Test Failures:**
```bash
error: cannot find function `create_test_app`
```
**Solution:** Generated tests use existing test infrastructure - ensure all dependencies are available

### Testing Workflow Issues

**Server Not Starting:**
```bash
./scripts/test-template-with-curl.sh: Server is not running on port 3000
```
**Solution:** Start server first: `./scripts/server.sh` (runs in background on port 3000)

**Authentication Failures:**
```bash
Template tests fail with 401 Unauthorized
```
**Solution:** Template test script handles authentication automatically - check server logs for database issues

### Getting Help

- Check [troubleshooting guide](troubleshooting.md)
- Review [testing documentation](guides/08-testing.md)
- Examine existing modules for patterns
- Use `--dry-run` to preview changes
- Run `./scripts/test-template-with-curl.sh --help` for testing options