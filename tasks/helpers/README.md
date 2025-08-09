# Documentation Verification Helpers

This directory contains automated scripts for comprehensive documentation verification in the Rust Fullstack Starter project.

## 🎯 Overview

These scripts implement a zero-tolerance approach to documentation accuracy, ensuring perfect synchronization between documentation and actual implementation through systematic, automated verification.

## 📁 Scripts

### Main Entry Point

**`docs-verify.sh`** - Primary orchestrator script
```bash
# Quick verification (recommended for development)
./tasks/helpers/docs-verify.sh

# Comprehensive verification (recommended for CI)
./tasks/helpers/docs-verify.sh --full --check-urls

# Fast check for pre-commit hooks
./tasks/helpers/docs-verify.sh --quick
```

### Individual Verifiers

**`verify-structures.sh`** - Code structure verification
- API response types, error enums, configuration structs
- Database table names, CLI command definitions
- Test counts and categories

**`verify-references.sh`** - Reference validation
- File paths, script references, configuration files
- Import paths, command examples
- URLs (optional, requires network)

## 🚀 Quick Start

### Basic Usage
```bash
# Navigate to project root
cd /path/to/rust-fullstack-starter

# Run basic verification
./tasks/helpers/docs-verify.sh

# Run with verbose output
./tasks/helpers/docs-verify.sh --verbose

# Check specific verification types
./tasks/helpers/docs-verify.sh structures references
```

### Common Workflows

**Pre-commit checking:**
```bash
./tasks/helpers/docs-verify.sh --quick
```

**Full verification before release:**
```bash
./tasks/helpers/docs-verify.sh --full --check-urls --verbose
```

**CI/CD integration:**
```bash
./tasks/helpers/docs-verify.sh --format github --sequential
```

**Generate verification report:**
```bash
./tasks/helpers/docs-verify.sh --report > docs-verification-report.json
```

## 🔧 Configuration Options

### Command Line Arguments

| Option | Description | Default |
|--------|-------------|---------|
| `--verbose` | Enable detailed output | `false` |
| `--dry-run` | Show what would be checked | `false` |
| `--format FORMAT` | Output format: human, json, github | `human` |
| `--parallel` | Run checks in parallel | `true` |
| `--sequential` | Run checks one at a time | `false` |
| `--check-urls` | Enable URL validation | `false` |
| `--timeout SECONDS` | Network/command timeout | `30` |
| `--quick` | Fast checks only | `false` |
| `--full` | Comprehensive verification | `false` |

### Environment Variables

```bash
export DOCS_VERIFY_VERBOSE=true       # Enable verbose mode
export DOCS_VERIFY_FORMAT=json        # Set default format
export DOCS_VERIFY_PARALLEL=false     # Disable parallel execution
export DOCS_VERIFY_TIMEOUT=60         # Set longer timeout
```

### Verification Types

| Type | Description | Speed | Network Required |
|------|-------------|-------|------------------|
| `structures` | Code structure matching | Fast | No |
| `references` | File/path validation | Fast | No |
| `urls` | URL accessibility | Slow | Yes |
| `commands` | Command examples | Medium | No |
| `imports` | Import path validation | Fast | No |
| `configs` | Config file references | Fast | No |

## 🧪 Integration Examples

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit
echo "🔍 Verifying documentation..."
./tasks/helpers/docs-verify.sh --quick
if [ $? -ne 0 ]; then
    echo "❌ Documentation verification failed"
    echo "Run: ./tasks/helpers/docs-verify.sh --verbose"
    exit 1
fi
```

### GitHub Actions
```yaml
# .github/workflows/docs-verification.yml
name: Documentation Verification
on:
  pull_request:
    paths: ['docs/**', 'starter/src/**', 'web/src/**']

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify Documentation
        run: ./tasks/helpers/docs-verify.sh --format github --sequential
```

### Development Workflow
```bash
# Add to your .bashrc or .zshrc
alias docs-check='./tasks/helpers/docs-verify.sh --quick'
alias docs-verify='./tasks/helpers/docs-verify.sh --verbose'
alias docs-full='./tasks/helpers/docs-verify.sh --full --check-urls'
```

## 📊 Output Formats

### Human Format (Default)
```
ℹ Documentation Structure Verification
✓ ApiResponse struct: Found in both docs (3) and code (1) 
✗ Error enum name mismatch: docs='ApiError', code='Error'
⚠ Command not available: docker
✓ All documentation references verified successfully!
```

### JSON Format (CI/Automation)
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "status": "failure",
  "exit_code": 1,
  "verification_types": "structures,references",
  "errors_found": 1
}
```

### GitHub Format (Actions)
```
::notice::Documentation verification completed successfully
::error::Documentation verification failed with 1 errors
::error::Run locally with --verbose for detailed information
```

## 🔍 Pattern Detection

### Structural Patterns
- API response structures: `ApiResponse.*{` → `pub struct ApiResponse`
- Error enums: `pub enum.*Error` → actual enum definitions
- Database tables: `INSERT INTO|FROM|JOIN` → migration files
- CLI commands: `cargo run --` → actual CLI help output

### Reference Patterns
- File paths: `./path/file`, `../path/file`, `/absolute/path`
- Scripts: `./scripts/name.sh`, `scripts/name.sh`
- Import paths: `import.*from.*@/` → `web/src/`
- Config files: `.env*`, `package.json`, `Cargo.toml`

### Flexible Matching
- Text changes don't break verification
- Multiple file extensions checked automatically
- Base directory resolution for relative paths
- Graceful handling of optional components

## 🐛 Troubleshooting

### Common Issues

**"Required tool not found: rg"**
```bash
# macOS
brew install ripgrep

# Ubuntu/Debian
apt install ripgrep

# Cargo
cargo install ripgrep
```

**"Script not executable"**
```bash
chmod +x tasks/helpers/*.sh
```

**"Verification script not found"**
```bash
# Ensure you're in the project root
cd /path/to/rust-fullstack-starter

# Check script exists
ls -la tasks/helpers/
```

**Network timeouts with URL checking**
```bash
# Increase timeout
./tasks/helpers/docs-verify.sh --check-urls --timeout 60

# Or skip URL checking
./tasks/helpers/docs-verify.sh --exclude urls
```

### Debug Mode
```bash
# Enable verbose output
./tasks/helpers/docs-verify.sh --verbose

# Dry run to see what would be checked
./tasks/helpers/docs-verify.sh --dry-run

# Check specific patterns only
./tasks/helpers/verify-structures.sh api-responses --verbose
./tasks/helpers/verify-references.sh files scripts --verbose
```

## 📈 Performance

| Mode | Duration | Use Case |
|------|----------|----------|
| Quick | 5-15s | Pre-commit hooks |
| Standard | 10-30s | Development workflow |
| Full | 30-60s | Pre-release verification |
| With URLs | +10-30s | Complete validation |

### Optimization Tips
- Use `--quick` for frequent checks
- Use `--parallel` (default) for faster execution
- Use `--exclude urls` to skip network checks
- Use `--sequential` only for debugging

## 🔄 Maintenance

### Adding New Patterns
1. Edit verification scripts to add new detection logic
2. Update pattern documentation in this README
3. Test with various documentation examples
4. Add integration tests

### Updating for New Features
1. Check if new code structures need verification patterns
2. Update database table patterns for new migrations
3. Add new CLI command patterns
4. Test against actual documentation

### Performance Tuning
1. Profile slow verification patterns
2. Optimize regex patterns for speed
3. Cache results where appropriate
4. Consider parallel execution of individual checks

## 🎯 Best Practices

### For Contributors
- Run `docs-verify --quick` before committing
- Use `--verbose` to understand failures
- Fix issues immediately rather than batching
- Test documentation changes with `--full`

### For Maintainers  
- Run `--full --check-urls` before releases
- Update verification patterns for new features
- Monitor script performance and optimize
- Keep this README updated with new patterns

### For CI/CD
- Use `--format github` for proper annotations
- Set appropriate timeouts for network conditions
- Use `--sequential` for predictable execution order
- Generate JSON reports for dashboards

---

*These scripts implement the battle-tested methodology that found and fixed 12 critical documentation inaccuracies in the original systematic verification. They ensure this accuracy is maintained automatically going forward.*