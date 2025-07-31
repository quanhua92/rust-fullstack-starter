# Test Rename Project Guide

This guide describes the automated testing process for the `rename-project.sh` script to ensure it works correctly across different scenarios.

## Overview

The rename project testing framework validates that:
1. The rename script correctly transforms all references from `starter` to a new project name
2. Docker services are properly managed (stopped, restarted) during the rename process
3. Database is reset and migrations are applied with new credentials
4. All quality checks pass after renaming (compilation, formatting, linting, tests)
5. The complex chaos testing framework continues to work with renamed containers and references
6. Environment variable transformations work correctly (STARTER__ → NEW_NAME__)

## Test Script Usage

### Basic Usage
```bash
# Test with default name 'hello'
./scripts/test-rename-project.sh

# Test with custom name
./scripts/test-rename-project.sh myproject

# Test with verbose output
./scripts/test-rename-project.sh myproject --verbose

# Test with custom attempt number
./scripts/test-rename-project.sh myproject --attempt 05
```

### Options
- `PROJECT_NAME`: Target project name (default: 'hello')
- `--verbose`: Enable detailed logging throughout the process
- `--attempt NUMBER`: Specify attempt number (default: auto-incremented)
- `--keep-on-failure`: Don't clean up test directory if tests fail
- `--timeout SECONDS`: Set timeout for operations (default: 600)

## What Gets Tested

### 1. Directory Structure Setup
- Creates isolated test environment in `tmp/attempt-0X-PROJECTNAME`
- Copies essential project files (buildscale/, scripts/, Cargo.*, CLAUDE.md)
- Sets up proper `starter/` directory structure
- Updates Cargo.toml files to reference `starter`

### 2. Rename Process Testing
- Runs `rename-project.sh` with verbose logging
- Validates Docker service shutdown before changes
- Validates all file transformations occur correctly
- Verifies environment variable prefix updates (STARTER__ → NEW_NAME__)
- Validates Docker service restart with new environment
- Validates database reset and migration execution
- Checks that backup is created
- Verifies directory renaming

### 3. Pattern Validation
The test validates that these patterns are correctly transformed:

#### Basic Patterns
- `starter` → `PROJECT_NAME` in source files
- `cargo run --bin starter` → `cargo run --bin PROJECT_NAME`
- `starter::` → `PROJECT_NAME::`
- `starter_` → `PROJECT_NAME_`

#### Docker Patterns (Critical for Chaos Testing)
- `/app/starter` → `/app/PROJECT_NAME`
- `chaos-starter-` → `chaos-PROJECT_NAME-`
- `rust-fullstack-starter-` → `rust-fullstack-PROJECT_NAME-`

#### Cargo Patterns
- `--manifest-path starter/` → `--manifest-path PROJECT_NAME/`
- `members = ["starter"]` → `members = ["PROJECT_NAME"]`
- `name = "starter"` → `name = "PROJECT_NAME"`

#### Script Patterns
- `cd starter` → `cd PROJECT_NAME`
- `PROJECT_NAME="starter"` → `PROJECT_NAME="PROJECT_NAME"`
- `starter-server` → `PROJECT_NAME-server`
- `starter-worker` → `PROJECT_NAME-worker`

#### Environment Variable Patterns
- `STARTER__SERVER__PORT` → `PROJECT_NAME__SERVER__PORT`
- `STARTER__DATABASE__USER` → `PROJECT_NAME__DATABASE__USER`
- `with_prefix("STARTER")` → `with_prefix("PROJECT_NAME")`
- `starter_user` → `PROJECT_NAME_user`
- `starter_db` → `PROJECT_NAME_db`

### 4. Quality Validation
After renaming, the test runs the comprehensive quality check suite:

- **Compilation Check**: `cargo check --all-targets --all-features` (with SQLx offline mode and fallback)
- **Code Formatting**: `cargo fmt --check` (with auto-fix if needed)
- **Linting**: `cargo clippy -- -D warnings`
- **SQLx Validation**: Query cache validation with automatic database restart if needed
- **Unit Tests**: Library tests
- **Integration Tests**: Full test suite with `cargo nextest run`
- **OpenAPI Export**: Documentation generation
- **Code Quality**: Additional checks for TODOs, debug prints, etc.

**New SQLx Fallback Testing**: The quality checks now include testing of the multi-level fallback system:
1. SQLX_OFFLINE=true cargo check (fast path)
2. If fails: cargo sqlx prepare (regenerate cache)
3. If still fails: Docker service restart + database reset + retry

## Expected Results

### Success Criteria
✅ All files copied correctly  
✅ Docker services stopped before environment changes
✅ Rename script completes without errors  
✅ All pattern replacements successful (including environment variables)
✅ Docker services restarted with new environment
✅ Database reset and migrations completed successfully
✅ Backup created properly  
✅ Compilation succeeds (with SQLx fallback if needed)
✅ All tests pass (typically 70+ integration tests)  
✅ Code formatting validated  
✅ Linting passes  
✅ OpenAPI documentation generates  

### Timing Expectations
- **Setup Phase**: ~5-10 seconds
- **Rename Phase**: ~10-30 seconds (includes Docker restart and database reset)
- **Quality Checks**: ~300-600 seconds (5-10 minutes for full test suite)
- **Total Runtime**: ~350-650 seconds (6-11 minutes)

**Note**: Docker service management and database reset add ~10-20 seconds to the rename phase but ensure reliable quality checks.

## Test Scenarios

### Standard Test Cases
1. **Basic Rename**: `hello` - Simple, valid project name
2. **Underscore Name**: `my_project` - Tests snake_case handling
3. **Long Name**: `awesome_backend_service` - Tests longer names
4. **Single Character**: `x` - Tests minimal valid name

### Edge Cases Tested
- Names starting with underscore: `_internal`
- Mixed case handling in different contexts
- Special characters in file paths
- Large project structures

## Troubleshooting

### Common Issues

#### Test Timeout
```bash
# Increase timeout for slower systems
./scripts/test-rename-project.sh myproject --timeout 600
```

#### Permission Issues
```bash
# Ensure scripts are executable
chmod +x scripts/*.sh
```

#### Compilation Failures
Usually indicates pattern replacement issues. Check:
- Cargo.toml workspace members
- Import statements in Rust files
- Binary name references
- Environment variable prefix mismatches

#### SQLx/Database Issues
New automatic handling, but may still occur:
- **SQLx prepare failures**: Should auto-restart Docker and reset database
- **Database connection errors**: Check Docker service status
- **Environment variable mismatches**: Verify STARTER__ → NEW_NAME__ transformation

#### Test Failures
Look for:
- Missing file transformations
- Incorrect pattern replacements
- Docker service startup failures
- Database migration failures

### Debug Mode
```bash
# Run with maximum verbosity and keep files on failure
./scripts/test-rename-project.sh testproject --verbose --keep-on-failure
```

#### Iterative Fix Workflow

When quality checks fail due to missed patterns in the rename script:

1. **Identify Missing Patterns**: Quality check failures reveal which `starter` references weren't replaced
2. **Fix rename-project.sh**: Add the missing patterns to the replacement logic
3. **Create New Attempt**: The test framework auto-increments attempt numbers (attempt-02, attempt-03, etc.)
4. **Re-run Test**: Test the fixes in a fresh environment

**Example Workflow:**
```bash
# First attempt fails due to missed 'use starter::' imports
./scripts/test-rename-project.sh hello
# ❌ Quality checks fail - compilation errors in test files

# Fix the rename-project.sh script to handle 'use starter' patterns
# Edit scripts/rename-project.sh to add missing sed patterns

# Re-run test - creates attempt-02-hello automatically
./scripts/test-rename-project.sh hello  
# ✅ Quality checks pass - all patterns now handled correctly
```

**Common Missing Patterns to Add:**
- `use starter::` → `use PROJECT_NAME::` (import statements)
- `use starter;` → `use PROJECT_NAME;` (single imports)
- `use starter{` → `use PROJECT_NAME{` (multi-imports)
- `--manifest-path starter/` → `--manifest-path PROJECT_NAME/` (cargo commands)
- `cd starter$` → `cd PROJECT_NAME` (directory changes)
- `starter::config::` → `PROJECT_NAME::config::`
- `starter::types::` → `PROJECT_NAME::types::`
- `starter::AppConfig` → `PROJECT_NAME::AppConfig`
- `STARTER__` → `PROJECT_NAME__` (environment variable prefixes)
- `with_prefix("STARTER")` → `with_prefix("PROJECT_NAME")` (config prefixes)
- `starter_user` → `PROJECT_NAME_user` (database defaults)

This iterative approach ensures the rename script becomes more robust with each test failure, eventually handling all edge cases in the codebase.

## Known Issues and Fixes

### Critical Patterns That Were Missing

During development, several critical patterns were discovered that caused quality check failures:

#### 1. Import Statement Patterns
**Problem**: Rust `use` statements weren't being updated
**Symptoms**: Compilation errors like `use of unresolved module or unlinked crate 'starter'`
**Fix**: Added patterns for all import variations:
```bash
sed -i '' "s/use starter::/use ${NEW_NAME}::/g" "$file"
sed -i '' "s/use starter;/use ${NEW_NAME};/g" "$file"  
sed -i '' "s/use starter{/use ${NEW_NAME}{/g" "$file"
```

#### 2. Cargo Manifest Path Patterns
**Problem**: Cargo commands using `--manifest-path starter/` failed
**Symptoms**: `manifest path 'starter/Cargo.toml' does not exist`
**Fix**: Added pattern for cargo manifest paths:
```bash
sed -i '' "s/--manifest-path starter\//--manifest-path ${NEW_NAME}\//g" "$file"
```

#### 3. Directory Change Patterns
**Problem**: Scripts using `cd starter` failed
**Symptoms**: `cd: starter: No such file or directory`
**Fix**: Added pattern for directory changes:
```bash
sed -i '' "s/cd starter$/cd $NEW_NAME/g" "$file"
```

#### 4. Output Formatting Issues
**Problem**: Raw escape codes like `\033[0;34m` appearing in output
**Symptoms**: Messy terminal output with visible escape sequences
**Fix**: Changed `echo` to `echo -e` for color code processing:
```bash
# Before (shows raw codes)
echo "   ${BLUE}cargo run --bin $NEW_NAME -- --help${NC}"

# After (shows colors)
echo -e "   ${BLUE}cargo run --bin $NEW_NAME -- --help${NC}"
```

### Quality Check Performance

**Issue**: Quality checks taking 5-10 minutes, causing timeouts
**Root Cause**: Comprehensive test suite includes:
- 50+ integration tests via `cargo nextest run`
- Full compilation with all features
- SQLx database validation
- Clippy linting with strict warnings
- Code formatting validation

**Solution**: Increased default timeout from 300s to 600s (10 minutes)

### Test Directory Cleanup

The test framework automatically cleans up test directories on completion. Use `--keep-on-failure` to preserve directories when debugging failures.

## Integration with Development Workflow

### Pre-commit Testing
```bash
# Add to your workflow before committing rename script changes
./scripts/test-rename-project.sh && echo "Rename script validated ✅"
```

### CI/CD Integration
The test script is designed to work in CI environments:
- Non-interactive (no user prompts)
- Clear exit codes (0 = success, 1 = failure)
- Structured output for parsing
- Timeout handling for automated environments

## File Structure After Testing

```
tmp/attempt-01-hello/
├── backup_TIMESTAMP/          # Backup of original files
├── hello/                     # Renamed project directory
│   ├── src/                   # Source code with updated references
│   ├── tests/                 # Tests with updated imports
│   ├── Cargo.toml            # Updated package name
│   └── ...
├── scripts/                   # Scripts with updated references
├── Cargo.toml                # Updated workspace members
└── CLAUDE.md                 # Updated documentation
```

## Validation Checklist

After running the test, verify:

- [ ] No compilation errors
- [ ] All tests pass
- [ ] Docker services properly stopped and restarted
- [ ] Database reset and migrations completed
- [ ] Environment variables transformed (STARTER__ → NEW_NAME__)
- [ ] Docker container names updated correctly
- [ ] Chaos testing scripts reference new names
- [ ] API documentation generated successfully
- [ ] No remaining references to 'starter' in critical files
- [ ] Backup created and contains original files
- [ ] Quality check script passes completely (including SQLx fallback)

## Performance Notes

The test is optimized for speed:
- Only copies essential files (no web/, docs/, .git/)
- Uses cargo check before full compilation
- Parallelizes independent operations where possible
- Includes timeout protection for CI environments

## Extending the Test

To add new validation scenarios:

1. **Add Pattern Tests**: Update the pattern validation section
2. **Add Quality Checks**: Extend the check.sh validation
3. **Add Edge Cases**: Include new project name formats
4. **Add Integration Tests**: Test with real chaos testing scenarios

This comprehensive testing ensures the rename script works reliably across all project components and maintains the integrity of the complex development infrastructure.