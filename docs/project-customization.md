# Project Customization Guide

This guide explains how to customize the Rust Full-Stack Starter for your own project.

## Automated Project Renaming

The easiest way to customize the starter is using the automated rename script:

### Quick Rename

```bash
# Rename project from "starter" to your project name
./scripts/rename-project.sh my_awesome_project
```

### What Gets Renamed

The script automatically updates:

- **Directory**: `starter/` → `my_awesome_project/`
- **Package name**: In `Cargo.toml` files
- **Binary name**: `cargo run --bin starter` → `cargo run --bin my_awesome_project`
- **Module references**: `starter::auth::jwt` → `my_awesome_project::auth::jwt`
- **Environment variables**: `STARTER__SERVER__PORT` → `MY_AWESOME_PROJECT__SERVER__PORT`
- **Config prefix**: `with_prefix("STARTER")` → `with_prefix("MY_AWESOME_PROJECT")`
- **Database defaults**: `starter_user` → `my_awesome_project_user`, `starter_db` → `my_awesome_project_db`
- **Script configurations**: Log files, process names
- **Documentation**: All examples and command references

### Requirements

Project names must follow Rust package naming conventions:
- ✅ Start with a letter or underscore
- ✅ Contain only letters, numbers, and underscores
- ✅ Use snake_case for consistency

**Good examples**: `my_project`, `awesome_app`, `backend_service`  
**Bad examples**: `123project`, `my-project`, `project.name`

### Safety Features

The rename script includes safety measures:

- **Validation**: Checks project name format before starting
- **Backup**: Creates timestamped backup in `backup_YYYYMMDD_HHMMSS/`
- **Verification**: Tests compilation after renaming
- **Rollback**: Easy to restore from backup if needed

### Example Usage

```bash
# Rename the project
./scripts/rename-project.sh my_blog_api

# Output shows progress:
# 🚀 Renaming project from 'starter' to 'my_blog_api'...
# 📦 Creating backup in backup_20240127_143022/
# 📁 Renaming starter/ directory to my_blog_api/
# 📝 Updating root Cargo.toml workspace members
# 📝 Updating my_blog_api/Cargo.toml package name
# 🔄 Replacing 'starter' with 'my_blog_api' in source files...
# 🔄 Updating environment variable prefixes...
# 🔄 Updating script references...
# 🔄 Updating log file references...
# ✅ Renaming complete!

# Test the renamed project
cargo run --bin my_blog_api -- --help

# Start development
./scripts/server.sh 3000
```

### After Renaming

1. **Update Project Description**: Edit README.md with your project details
2. **Update Environment Variables**: The script renames environment variable prefixes, so update your `.env` file:
   ```bash
   # Before renaming (STARTER__ prefix)
   STARTER__SERVER__PORT=3000
   STARTER__DATABASE__USER=starter_user
   
   # After renaming to "my_blog_api" (MY_BLOG_API__ prefix)
   MY_BLOG_API__SERVER__PORT=3000
   MY_BLOG_API__DATABASE__USER=my_blog_api_user
   ```
3. **Test Everything**: Run tests and start services
4. **Commit Changes**: Initialize or update your git repository

```bash
# Update git repository
git add .
git commit -m "Initial project setup for my_blog_api"
```

## Manual Customization

If you prefer manual customization or need more control:

### 1. Directory Structure

```bash
# Rename main directory
mv starter/ my_project/
```

### 2. Update Cargo.toml Files

```toml
# Root Cargo.toml - Update workspace members
[workspace]
members = ["my_project"]

# my_project/Cargo.toml - Update package name
[package]
name = "my_project"
```

### 3. Update Source Code References

Replace `starter` references in:
- `my_project/src/main.rs` - use statements
- `my_project/src/lib.rs` - module declarations
- Test files - module imports
- Documentation - command examples

### 4. Update Scripts

Edit script files in `scripts/` to use your project name:
- Update `PROJECT_NAME` variable in `server.sh`
- Update log file paths to use your project name
- Update any hardcoded references

## Project Structure Customization

### Adding New Features

The starter is designed to be extended:

```
my_project/
├── src/
│   ├── auth/           # Authentication module
│   ├── users/          # User management
│   ├── tasks/          # Background tasks
│   ├── your_feature/   # Add your custom modules here
│   └── main.rs
```

### Custom Task Types

Add new background task handlers:

```rust
// my_project/src/tasks/handlers.rs
pub async fn handle_your_task(payload: Value) -> Result<(), String> {
    // Your custom task logic
    Ok(())
}

// Register in processor.rs
match task.task_type.as_str() {
    "email" => handle_email_task(payload).await,
    "your_task" => handle_your_task(payload).await,
    // ...
}
```

### Custom API Endpoints

Add new API routes:

```rust
// my_project/src/api/your_feature.rs
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/your-endpoint", get(your_handler))
        .route("/your-endpoint", post(your_create_handler))
}

// my_project/src/server.rs
let app = Router::new()
    .nest("/api/your-feature", your_feature::create_routes())
    // ... other routes
```

## Best Practices

### 1. Follow Project Conventions

- **Naming**: Use snake_case for Rust conventions
- **Modules**: Group related functionality together
- **Tests**: Keep comprehensive test coverage
- **Documentation**: Update docs when adding features

### 2. Maintain Database Migrations

```bash
# Add new migrations as needed
sqlx migrate add your_feature_tables

# Keep migrations in order and reversible
```

### 3. Update Configuration

```rust
// my_project/src/config.rs
pub struct AppConfig {
    // Add your custom configuration fields
    pub your_feature_enabled: bool,
    pub your_api_key: String,
}
```

### 4. Environment Variables

After renaming, your environment variables will use the new project name in uppercase:

```bash
# .env - After renaming to "my_blog_api"
MY_BLOG_API__SERVER__PORT=3000
MY_BLOG_API__DATABASE__HOST=localhost
MY_BLOG_API__DATABASE__USER=my_blog_api_user
MY_BLOG_API__DATABASE__PASSWORD=my_blog_api_pass
MY_BLOG_API__DATABASE__DATABASE=my_blog_api_db

# Custom feature variables
MY_BLOG_API__YOUR_FEATURE_ENABLED=true
MY_BLOG_API__YOUR_API_KEY=your-secret-key
```

The configuration system automatically uses your project's uppercase name as the environment variable prefix.

## Troubleshooting

### Rename Script Issues

**Problem**: "starter directory not found"
```bash
# Make sure you're in the project root
pwd  # Should show rust-fullstack-starter directory
ls   # Should show starter/ directory
```

**Problem**: Compilation errors after rename
```bash
# Check for missed references
grep -r "starter" my_project/src/
# Fix any remaining references manually
```

**Problem**: Tests failing after rename
```bash
# Run tests to identify issues
cargo nextest run
# Update test helper imports if needed
```

### Manual Customization Issues

**Problem**: Binary not found
```bash
# Make sure package name matches in Cargo.toml
cargo run --bin your_project_name -- --help
```

**Problem**: Module not found errors
```bash
# Check use statements in main.rs and lib.rs
# Make sure module names match directory structure
```

## Getting Help

If you encounter issues during customization:

1. **Check the backup**: Restore from the backup directory if needed
2. **Read error messages**: Rust provides helpful compilation errors
3. **Test incrementally**: Make changes in small steps
4. **Use the example**: Compare with the original starter structure

The rename script is designed to be safe and reliable, but if you encounter issues, the backup ensures you can always start over.