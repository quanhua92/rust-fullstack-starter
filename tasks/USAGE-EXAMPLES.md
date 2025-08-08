# Documentation Verification Usage Examples

Quick reference for using the automated documentation verification system.

## 🚀 Common Commands

### Quick Development Check
```bash
# Fast check for daily development
./tasks/helpers/docs-verify.sh --quick

# Same as above but with:
# - Structures verification only
# - Excludes slow URL/command checks
# - ~5-15 seconds execution time
```

### Full Verification
```bash  
# Comprehensive verification before commits/releases
./tasks/helpers/docs-verify.sh --full --check-urls

# Includes:
# - All structure verification
# - All reference validation  
# - URL accessibility checking
# - ~30-60 seconds execution time
```

### Individual Component Checking
```bash
# Check only API structures
./tasks/helpers/verify-structures.sh api-responses error-enums

# Check only file references  
./tasks/helpers/verify-references.sh files scripts configs

# Check with verbose output for debugging
./tasks/helpers/verify-structures.sh --verbose all
```

## 🔧 Integration Examples

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔍 Verifying documentation accuracy..."
./tasks/helpers/docs-verify.sh --quick

if [ $? -ne 0 ]; then
    echo "❌ Documentation verification failed"
    echo "📖 Run './tasks/helpers/docs-verify.sh --verbose' for details"
    echo "🔧 Fix issues before committing"
    exit 1
fi

echo "✅ Documentation verification passed"
```

### GitHub Actions
```yaml
name: Documentation Verification
on:
  pull_request:
    paths: ['docs/**', 'starter/src/**', 'web/src/**']

jobs:
  verify-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Dependencies
        run: |
          # Install ripgrep for pattern matching
          wget https://github.com/BurntSushi/ripgrep/releases/download/13.0.0/ripgrep_13.0.0_amd64.deb
          sudo dpkg -i ripgrep_13.0.0_amd64.deb
          
      - name: Verify Documentation
        run: |
          chmod +x tasks/helpers/*.sh
          ./tasks/helpers/docs-verify.sh --format github --sequential
```

### Development Aliases
```bash
# Add to your ~/.bashrc or ~/.zshrc
alias docs-check='./tasks/helpers/docs-verify.sh --quick'
alias docs-verify='./tasks/helpers/docs-verify.sh --verbose' 
alias docs-full='./tasks/helpers/docs-verify.sh --full --check-urls'
alias docs-report='./tasks/helpers/docs-verify.sh --report'
```

## 📊 Output Examples

### Success Output
```
ℹ Documentation Structure Verification
✓ ApiResponse struct: Found in both docs (3) and code (1)
✓ Error enum name matches: Error
✓ Database table references verified
✓ CLI commands verified against actual help

ℹ Documentation Reference Verification  
✓ Found: ./scripts/dev-server.sh
✓ Found: ./scripts/check.sh
✓ All 24 file path references verified
✓ All 8 script references verified

🎉 All documentation verifications passed!
Documentation is accurate and consistent with implementation
```

### Failure Output
```
ℹ Documentation Structure Verification
✗ ApiResponse field mismatch: docs show 'error' field but code has 'message' field
✗ Error enum name mismatch: docs='ApiError', code='Error'
✓ Database table references verified

❌ Documentation verification failed
Found 2 verification errors
Run with --verbose for detailed information
```

### JSON Output (for CI/reporting)
```json
{
    "timestamp": "2024-01-15T10:30:00Z",
    "project": "Rust Fullstack Starter",
    "verification_types": "structures,references",
    "execution_mode": "parallel",
    "url_checking": false,
    "timeout": 30,
    "status": "success",
    "exit_code": 0,
    "errors_found": 0
}
```

## 🎯 Specific Use Cases

### Before Making Code Changes
```bash
# Establish baseline - ensure docs are currently accurate
./tasks/helpers/docs-verify.sh --verbose

# Make your code changes
# ... edit source code ...

# Update documentation to match
# ... edit docs ...

# Verify changes are consistent
./tasks/helpers/docs-verify.sh --verbose
```

### Before Release
```bash
# Complete verification with network checks
./tasks/helpers/docs-verify.sh --full --check-urls --verbose

# Generate verification report
./tasks/helpers/docs-verify.sh --report > release-verification-report.json

# Check report shows all clear
cat release-verification-report.json | jq '.status'  # Should show "success"
```

### Debugging Documentation Issues
```bash
# Run with maximum verbosity
./tasks/helpers/docs-verify.sh --verbose --sequential

# Check specific problematic patterns
./tasks/helpers/verify-structures.sh api-responses --verbose

# Test individual reference types
./tasks/helpers/verify-references.sh files --verbose
./tasks/helpers/verify-references.sh scripts --verbose
```

### Performance Testing
```bash
# Time the verification
time ./tasks/helpers/docs-verify.sh

# Quick mode timing
time ./tasks/helpers/docs-verify.sh --quick

# Full mode timing  
time ./tasks/helpers/docs-verify.sh --full

# Sequential vs parallel comparison
time ./tasks/helpers/docs-verify.sh --sequential
time ./tasks/helpers/docs-verify.sh --parallel
```

## 🔍 Pattern-Specific Examples

### Structure Verification Patterns
```bash
# API response structures
./tasks/helpers/verify-structures.sh api-responses

# Database table names  
./tasks/helpers/verify-structures.sh database-tables

# Error type definitions
./tasks/helpers/verify-structures.sh error-enums

# CLI command structure
./tasks/helpers/verify-structures.sh cli-commands

# Test count accuracy
./tasks/helpers/verify-structures.sh test-counts
```

### Reference Verification Patterns
```bash
# File path validation
./tasks/helpers/verify-references.sh files

# Script references
./tasks/helpers/verify-references.sh scripts

# Configuration files
./tasks/helpers/verify-references.sh configs

# Frontend import paths
./tasks/helpers/verify-references.sh imports

# Command examples (no network required)
./tasks/helpers/verify-references.sh commands

# URL accessibility (requires network)
./tasks/helpers/verify-references.sh urls --check-urls
```

## 🚨 Troubleshooting Examples

### Missing Dependencies
```bash
# Install ripgrep (required)
brew install ripgrep  # macOS
apt install ripgrep   # Ubuntu/Debian

# Install curl (for URL checking)
brew install curl     # macOS
apt install curl      # Ubuntu/Debian
```

### Permission Issues
```bash
# Make scripts executable
chmod +x tasks/helpers/*.sh

# Check permissions
ls -la tasks/helpers/
```

### Path Issues
```bash
# Ensure you're in project root
cd /path/to/rust-fullstack-starter
pwd  # Should end with 'rust-fullstack-starter'

# Check directory structure
ls -la tasks/helpers/
ls -la docs/
ls -la starter/src/
```

### Network Issues (URL checking)
```bash
# Skip URL checking
./tasks/helpers/docs-verify.sh --exclude urls

# Increase timeout for slow networks
./tasks/helpers/docs-verify.sh --check-urls --timeout 60

# Test URL checking separately
./tasks/helpers/verify-references.sh urls --check-urls --verbose
```

## 📈 Performance Optimization

### Fast Development Workflow
```bash
# Use quick mode for frequent checks
./tasks/helpers/docs-verify.sh --quick

# Exclude slow patterns
./tasks/helpers/docs-verify.sh --exclude commands,urls

# Focus on specific areas you're working on
./tasks/helpers/verify-structures.sh api-responses error-enums
```

### CI Optimization
```bash
# Use sequential mode for predictable timing
./tasks/helpers/docs-verify.sh --sequential --format github

# Skip network-dependent checks in CI
./tasks/helpers/docs-verify.sh --exclude urls

# Set reasonable timeout for CI environment
./tasks/helpers/docs-verify.sh --timeout 45
```

---

*These examples demonstrate the flexible, powerful verification system that maintains zero-tolerance accuracy for documentation in the Rust Fullstack Starter project.*