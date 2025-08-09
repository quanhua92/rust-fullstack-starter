use super::{
    models::{Cli, Commands, GenerateCommands, RevertCommands},
    services::{TaskTypeService, execute_admin_command},
};
use crate::{AppConfig, Database, core::server, tasks};
use clap::Parser;

/// Main CLI application handler
pub struct CliApp {
    config: AppConfig,
}

impl CliApp {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Parse and execute CLI commands
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging with info level by default
        tracing_subscriber::fmt()
            .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
            .init();

        // Load environment variables
        dotenvy::dotenv().ok();

        let cli = Cli::parse();
        let config = AppConfig::load()?;
        let app = CliApp::new(config);

        app.execute_command(cli.command).await
    }

    /// Execute a parsed CLI command
    pub async fn execute_command(
        &self,
        command: Commands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Server { port } => self.run_server(port).await,
            Commands::Worker => self.run_worker().await,
            Commands::HealthCheck => self.run_health_check().await,
            Commands::ExportOpenApi { output } => self.export_openapi(output).await,
            Commands::Admin { admin_command } => self.run_admin_command(admin_command).await,
            Commands::Generate { generator } => self.run_generate_command(generator).await,
            Commands::Revert { revert } => self.run_revert_command(revert).await,
        }
    }

    /// Run the web server
    async fn run_server(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.config.clone();

        // Override port from CLI if provided
        if port != 8080 {
            config.server.port = port;
        }

        let database = Database::connect(&config).await?;
        database.migrate().await?;
        database.ensure_initial_admin(&config).await?;

        server::start_server(config, database).await?;
        Ok(())
    }

    /// Run the background worker
    async fn run_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        let database = Database::connect(&self.config).await?;
        database.migrate().await?;

        // Create task processor with configuration
        let processor_config = tasks::processor::ProcessorConfig {
            poll_interval: self.config.poll_interval(),
            task_timeout: std::time::Duration::from_secs(300),
            max_concurrent_tasks: self.config.worker.concurrency,
            batch_size: 50,
            enable_circuit_breaker: true,
        };

        let processor = tasks::processor::TaskProcessor::new(database, processor_config);

        // Register example task handlers
        tasks::handlers::register_example_handlers(&processor).await;

        // Register task types with the API
        if let Err(e) = TaskTypeService::register_task_types_with_api(None).await {
            eprintln!("Warning: Failed to register task types with API: {e}");
            eprintln!(
                "Worker will continue, but new tasks may be rejected until types are registered"
            );
        }

        println!(
            "Background worker starting with {} max concurrent tasks",
            self.config.worker.concurrency
        );

        // Start the worker loop
        processor.start_worker().await?;
        Ok(())
    }

    /// Run health check for Docker/Kubernetes
    async fn run_health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple health check for Docker/Kubernetes
        // Exit code 0 = healthy, non-zero = unhealthy
        let config = match AppConfig::load() {
            Ok(config) => config,
            Err(_) => std::process::exit(1),
        };

        // Try to connect to database
        match Database::connect(&config).await {
            Ok(database) => {
                // Test database connectivity with a simple query
                match sqlx::query("SELECT 1").fetch_one(&database.pool).await {
                    Ok(_) => {
                        println!("OK");
                        std::process::exit(0);
                    }
                    Err(_) => {
                        eprintln!("Database query failed");
                        std::process::exit(1);
                    }
                }
            }
            Err(_) => {
                eprintln!("Database connection failed");
                std::process::exit(1);
            }
        }
    }

    /// Export OpenAPI specification to file
    async fn export_openapi(&self, output: String) -> Result<(), Box<dyn std::error::Error>> {
        use crate::core::openapi;
        use std::fs;
        use std::path::Path;

        // Check if we're running from the correct location
        let is_running_from_root = Path::new("templates").exists()
            && Path::new("starter").exists()
            && Path::new("starter/Cargo.toml").exists();

        let is_running_from_starter = Path::new("src").exists()
            && Path::new("Cargo.toml").exists()
            && Path::new("../templates").exists();

        if !is_running_from_root && !is_running_from_starter {
            eprintln!("❌ You are not running from the correct location.");
            eprintln!("   Expected to find 'starter' folder or 'templates' folder.");
            eprintln!("   This command should be run from the project root.");
            eprintln!();
            print!("   Continue anyway? (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                return Err("Export cancelled by user".into());
            }
        }

        let json = openapi::openapi_json();

        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&output).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&output, json)?;
        println!("✅ OpenAPI specification exported to: {output}");
        Ok(())
    }

    /// Run admin commands
    async fn run_admin_command(
        &self,
        admin_command: super::models::AdminCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database = Database::connect(&self.config).await?;
        execute_admin_command(database, admin_command).await
    }

    /// Run generate commands
    async fn run_generate_command(
        &self,
        generator: GenerateCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match generator {
            GenerateCommands::Module {
                name,
                template,
                dry_run,
                force,
            } => self.generate_module(name, template, dry_run, force).await,
        }
    }

    /// Generate a module from templates
    async fn generate_module(
        &self,
        name: String,
        template: String,
        dry_run: bool,
        force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::collections::HashMap;
        use std::fs;
        use std::path::Path;

        println!("🚀 Generating module '{name}' using '{template}' template");

        if dry_run {
            println!("🔍 DRY RUN MODE - No files will be created");
        }

        // Create singular/plural forms
        let singular = &name;
        let plural = if name.ends_with('s') {
            name.clone()
        } else {
            format!("{singular}s")
        }; // Simple pluralization
        let struct_name = capitalize_first(singular);
        let table_name = &plural;

        // Show transformations to user
        println!();
        println!("📝 Name transformations:");
        println!("   Module name (singular): {singular}");
        println!("   Module name (plural):   {plural}");
        println!("   Struct name:           {struct_name}");
        println!("   Table name:            {table_name}");
        println!();

        // Template replacements
        let mut replacements = HashMap::new();
        replacements.insert("__MODULE_NAME__", singular);
        replacements.insert("__MODULE_NAME_PLURAL__", &plural);
        replacements.insert("__MODULE_STRUCT__", &struct_name);
        replacements.insert("__MODULE_TABLE__", table_name);

        // Determine template directory based on working directory with multiple checks
        let template_dir = if Path::new("templates").exists()
            && Path::new("starter").exists()
            && Path::new("starter/Cargo.toml").exists()
        {
            // Running from project root: has templates/ + starter/ + starter/Cargo.toml
            format!("./templates/{template}")
        } else if Path::new("src").exists()
            && Path::new("Cargo.toml").exists()
            && Path::new("../templates").exists()
        {
            // Running from starter directory: has src/ + Cargo.toml + ../templates
            format!("../templates/{template}")
        } else {
            return Err("Cannot determine project structure. Must run from project root (with templates/ and starter/ directories) or from starter directory (with src/ and Cargo.toml)".into());
        };
        if !Path::new(&template_dir).exists() {
            return Err(format!("Template '{template}' not found in templates directory").into());
        }

        // Determine working directory and paths
        let (module_dir, test_dir, migrations_dir, _lib_rs_path, _sqlx_working_dir) =
            determine_project_paths(&plural)?;

        // Check for existing files and prompt if necessary
        let module_exists = Path::new(&module_dir).exists();
        let test_exists = Path::new(&test_dir).exists();

        if !dry_run && (module_exists || test_exists) {
            if !force {
                use std::io::{self, Write};

                // Print big warning in red
                println!("\n🚨 \x1b[31m\x1b[1mWARNING: FILES ALREADY EXIST!\x1b[0m 🚨");
                println!("\x1b[31mThe following files/directories will be OVERWRITTEN:\x1b[0m");

                if module_exists {
                    println!("   \x1b[91m📁 Module directory: {module_dir}\x1b[0m");
                }
                if test_exists {
                    println!("   \x1b[91m📁 Test directory: {test_dir}\x1b[0m");
                }

                println!("\n\x1b[33m⚠️  This will permanently replace existing files!\x1b[0m");
                print!("\n❓ \x1b[1mAre you sure you want to continue? [y/N]: \x1b[0m");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().to_lowercase().starts_with('y') {
                    println!("\n\x1b[32m✅ Operation cancelled - no files were modified\x1b[0m");
                    println!("\x1b[36m💡 To skip this prompt, use: --force\x1b[0m");
                    println!(
                        "\x1b[36m💡 To preview changes without writing files, use: --dry-run\x1b[0m"
                    );
                    return Ok(());
                }

                println!("\n\x1b[33m⚠️  Proceeding with file overwrite...\x1b[0m");
            } else {
                // Force mode - show what we're overwriting but proceed
                println!(
                    "\n\x1b[33m🔥 FORCE MODE: Overwriting existing files without prompting\x1b[0m"
                );
                if module_exists {
                    println!("   \x1b[93m📁 Overwriting: {module_dir}\x1b[0m");
                }
                if test_exists {
                    println!("   \x1b[93m📁 Overwriting: {test_dir}\x1b[0m");
                }
                println!();
            }
        }

        let mut files_created = Vec::new();
        if !dry_run {
            fs::create_dir_all(&module_dir)?;
        }
        println!("📁 Created directory: {module_dir}");

        // Copy and process template files
        let template_files = [
            ("api.rs", "api.rs"),
            ("models.rs", "models.rs"),
            ("services.rs", "services.rs"),
            ("mod.rs", "mod.rs"),
        ];

        for (template_file, output_file) in template_files {
            let template_path = format!("{template_dir}/{template_file}");
            let output_path = format!("{module_dir}/{output_file}");

            if Path::new(&template_path).exists() {
                let content = fs::read_to_string(&template_path)?;
                let processed = process_template(&content, &replacements);

                if !dry_run {
                    fs::write(&output_path, processed)?;
                }
                files_created.push(output_path.clone());
                println!("📄 Created: {output_path}");
            }
        }

        // Create test directory and file
        if !dry_run {
            fs::create_dir_all(&test_dir)?;
        }

        let test_template_path = format!("{template_dir}/tests.rs");
        if Path::new(&test_template_path).exists() {
            let test_content = fs::read_to_string(&test_template_path)?;
            let processed_test = process_template(&test_content, &replacements);
            let test_output = format!("{test_dir}/mod.rs");

            if !dry_run {
                fs::write(&test_output, processed_test)?;
            }
            files_created.push(test_output.clone());
            println!("📄 Created: {test_output}");
        }

        // Create migrations
        let migration_number = get_next_migration_number(&migrations_dir)?;

        let migration_files = [
            ("up.sql", format!("{migration_number:03}_{plural}.up.sql")),
            (
                "down.sql",
                format!("{migration_number:03}_{plural}.down.sql"),
            ),
        ];

        for (template_file, output_file) in migration_files {
            let template_path = format!("{template_dir}/{template_file}");
            let output_path = format!("{migrations_dir}/{output_file}");

            if Path::new(&template_path).exists() {
                let content = fs::read_to_string(&template_path)?;
                let processed = process_template(&content, &replacements);

                if !dry_run {
                    fs::write(&output_path, processed)?;
                }
                files_created.push(output_path.clone());
                println!("📄 Created: {output_path}");
            }
        }

        // Note: lib.rs update is now a manual step for consistency

        println!("✅ Module generation completed!");
        let files_count = files_created.len();
        println!("📄 Files created: {files_count}");

        if !dry_run {
            println!("\n📋 Next steps - run these commands:");
            println!("   1. Run the migration:");
            println!("      cd starter && sqlx migrate run");
            println!();
            println!("   2. Update sqlx cache (IMPORTANT: remove old cache first):");
            println!("      rm -rf starter/.sqlx");
            println!("      ./scripts/prepare-sqlx.sh");
            println!();
            println!(
                "   3. Run quality checks (recommended - includes compilation, linting, tests):"
            );
            println!("      ./scripts/check.sh");
            println!();
            println!("   4. Add module to lib.rs (manual step):");
            println!("      - Add: pub mod {plural};");
            println!();
            println!("   5. Add routes to server.rs (manual step):");
            println!("      - Import: use crate::{plural}::api::{plural}_routes;");
            println!(
                "      - Add route: .nest(\"/{plural}\", {plural}_routes()) INSIDE protected_routes"
            );
            println!(
                "      - NOTE: Add nest() BEFORE .layer(auth_middleware) for proper authentication"
            );
            println!();
            println!("   6. Add to openapi.rs (manual step):");
            match template.as_str() {
                "basic" => {
                    println!(
                        "      - Import: use crate::{plural}::{{models::*, api::List{struct_name}Query}};"
                    );
                    println!("      - Add endpoints to paths() section");
                    println!("      - Add models to schemas() section");
                }
                "production" => {
                    println!("      - Import: use crate::{plural}::models::*;");
                    println!("      - Add endpoints to paths() section");
                    println!(
                        "      - Add models to schemas() section (BulkOperationResponse needs <T> generic)"
                    );
                }
                _ => {
                    println!("      - Import: use crate::{plural}::models::*;");
                    println!("      - Add endpoints to paths() section");
                    println!("      - Add models to schemas() section");
                }
            }
        }

        Ok(())
    }

    /// Run revert command
    async fn run_revert_command(
        &self,
        revert: RevertCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match revert {
            RevertCommands::Module { name, yes, dry_run } => {
                self.revert_module(&name, yes, dry_run).await
            }
        }
    }

    /// Revert a generated module
    async fn revert_module(
        &self,
        name: &str,
        yes: bool,
        dry_run: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::io::{self, Write};
        use std::path::Path;

        let plural = if name.ends_with('s') {
            name.to_string()
        } else {
            format!("{name}s")
        };

        println!("🔍 Analyzing module '{name}' for revert...");

        // Determine working directory and paths
        let (module_dir, test_dir, migrations_dir, lib_rs_path, sqlx_working_dir) =
            determine_project_paths(&plural)?;

        let module_exists = Path::new(&module_dir).exists();
        let test_exists = Path::new(&test_dir).exists();

        // Find migration files
        let mut migration_files = Vec::new();
        let mut migration_number = None;

        if Path::new(&migrations_dir).exists() {
            let entries = fs::read_dir(&migrations_dir)?;
            for entry in entries {
                let entry = entry?;
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if filename_str.contains(&plural) {
                    migration_files.push(entry.path());
                    // Extract migration number for revert
                    if let Some(num_str) = filename_str.split('_').next()
                        && let Ok(num) = num_str.parse::<u32>()
                    {
                        migration_number = Some(num);
                    }
                }
            }
        }

        // Check lib.rs
        let lib_rs_has_module = if Path::new(&lib_rs_path).exists() {
            let lib_content = fs::read_to_string(&lib_rs_path)?;
            lib_content.contains(&format!("pub mod {plural};"))
        } else {
            false
        };

        // Show what will be done
        println!("\n📋 Revert plan for module '{name}':");

        if let Some(num) = migration_number {
            println!("   ⚠️  Revert database migration #{num}");
        }

        if !migration_files.is_empty() {
            let file_count = migration_files.len();
            println!("   🗑️  Delete {file_count} migration files");
            for file in &migration_files {
                let file_display = file.display();
                println!("       - {file_display}");
            }
        }

        if module_exists {
            println!("   🗑️  Delete module directory: {module_dir}");
        }

        if test_exists {
            println!("   🗑️  Delete test directory: {test_dir}");
        }

        if lib_rs_has_module {
            println!("   📝 Manual step: Remove module declaration from lib.rs");
        }

        if !module_exists && migration_files.is_empty() && !test_exists && !lib_rs_has_module {
            println!("   ✅ No files found for module '{name}' - nothing to revert");
            return Ok(());
        }

        if dry_run {
            println!("\n🔍 DRY RUN - No changes will be made");
            return Ok(());
        }

        // Interactive confirmations unless --yes is provided
        if !yes {
            println!(
                "\n⚠️  WARNING: This operation will permanently delete files and revert database migrations!"
            );

            if let Some(num) = migration_number {
                print!("\n❓ Revert database migration #{num}? [y/N]: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("❌ Operation cancelled by user");
                    return Ok(());
                }
            }

            if module_exists || test_exists || !migration_files.is_empty() {
                print!("\n❓ Delete all generated files? [y/N]: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("❌ Operation cancelled by user");
                    return Ok(());
                }
            }
        }

        println!("\n🚀 Starting revert process...");

        // Step 1: Revert migration if exists
        if let Some(migration_to_revert) = migration_number {
            println!("📦 Checking migration safety...");

            // Safety check: only allow reverting the latest migration
            let latest_migration = get_latest_migration_number(&migrations_dir)?;
            if migration_to_revert != latest_migration {
                return Err(format!(
                    "Cannot revert module '{name}' because its migration #{migration_to_revert} is not the latest one (latest is #{latest_migration}).\n\
                    This could cause database schema corruption.\n\
                    \n\
                    To proceed safely:\n\
                    1. Revert newer migrations manually with 'cd starter && sqlx migrate revert'\n\
                    2. Then use this command to revert module '{name}'\n\
                    \n\
                    Or handle the migration manually and use --dry-run to see what files would be cleaned up."
                ).into());
            }

            println!("✅ Migration #{migration_to_revert} is the latest - safe to revert");
            println!("📦 Reverting database migration...");

            let output = std::process::Command::new("sqlx")
                .args(["migrate", "revert"])
                .current_dir(&sqlx_working_dir)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("✅ Database migration reverted successfully");
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        println!("⚠️  Migration revert warning: {stderr}");
                        println!("   (This might be expected if migration was already reverted)");
                    }
                }
                Err(e) => {
                    println!("⚠️  Failed to run sqlx migrate revert: {e}");
                    println!("   You may need to run 'sqlx migrate revert' manually");
                }
            }
        }

        // Step 2: Delete migration files
        for file in migration_files {
            if let Err(e) = fs::remove_file(&file) {
                let file_display = file.display();
                println!("⚠️  Failed to delete {file_display}: {e}");
            } else {
                let file_display = file.display();
                println!("🗑️  Deleted: {file_display}");
            }
        }

        // Step 3: Delete module directory
        if module_exists {
            if let Err(e) = fs::remove_dir_all(&module_dir) {
                println!("⚠️  Failed to delete {module_dir}: {e}");
            } else {
                println!("🗑️  Deleted: {module_dir}");
            }
        }

        // Step 4: Delete test directory
        if test_exists {
            if let Err(e) = fs::remove_dir_all(&test_dir) {
                println!("⚠️  Failed to delete {test_dir}: {e}");
            } else {
                println!("🗑️  Deleted: {test_dir}");
            }
        }

        // Step 5: Manual lib.rs cleanup (not automated)
        if lib_rs_has_module {
            println!("\n📋 Manual cleanup required:");
            println!("   📝 Remove from lib.rs: pub mod {plural};");
            println!("   📝 Remove from server.rs: use crate::{plural}::api::{plural}_routes;");
            println!("   📝 Remove from server.rs: .nest(\"/api/v1/{plural}\", {plural}_routes())");
            println!("   📝 Remove from openapi.rs: any imports from {plural}::models");
        }

        println!("\n✅ Module '{name}' reverted successfully!");
        println!("\n📋 You may want to run:");
        println!("   cargo check  # Verify compilation");

        Ok(())
    }
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Process template content by replacing placeholders
fn process_template(
    content: &str,
    replacements: &std::collections::HashMap<&str, &String>,
) -> String {
    let mut result = content.to_string();
    for (placeholder, replacement) in replacements {
        result = result.replace(placeholder, replacement);
    }
    result
}

/// Get the next migration number
fn get_next_migration_number(migrations_dir: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let latest = get_latest_migration_number(migrations_dir)?;
    Ok(latest + 1)
}

fn get_latest_migration_number(migrations_dir: &str) -> Result<u32, Box<dyn std::error::Error>> {
    use std::fs;

    if !std::path::Path::new(migrations_dir).exists() {
        return Ok(0);
    }

    let mut max_number = 0;
    let entries = fs::read_dir(migrations_dir)?;

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        if let Some(number_str) = filename_str.split('_').next()
            && let Ok(number) = number_str.parse::<u32>()
        {
            max_number = max_number.max(number);
        }
    }

    Ok(max_number)
}

/// Project paths structure for organize paths based on working directory
type ProjectPaths = (String, String, String, String, String);

/// Determine project paths based on current working directory
/// Returns (module_dir, test_dir, migrations_dir, lib_rs_path, sqlx_working_dir)
fn determine_project_paths(plural: &str) -> Result<ProjectPaths, Box<dyn std::error::Error>> {
    use std::path::Path;

    if Path::new("templates").exists()
        && Path::new("starter").exists()
        && Path::new("starter/Cargo.toml").exists()
    {
        // Running from project root: has templates/ + starter/ + starter/Cargo.toml
        Ok((
            format!("starter/src/{plural}"),
            format!("starter/tests/{plural}"),
            "starter/migrations".to_string(),
            "starter/src/lib.rs".to_string(),
            "starter".to_string(),
        ))
    } else if Path::new("src").exists()
        && Path::new("Cargo.toml").exists()
        && Path::new("../templates").exists()
    {
        // Running from starter directory: has src/ + Cargo.toml + ../templates
        Ok((
            format!("src/{plural}"),
            format!("tests/{plural}"),
            "migrations".to_string(),
            "src/lib.rs".to_string(),
            ".".to_string(),
        ))
    } else {
        Err("Cannot determine project structure. Must run from project root (with templates/ and starter/ directories) or from starter directory (with src/ and Cargo.toml)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_app_creation() {
        let config = AppConfig::default();
        let app = CliApp::new(config);
        // Basic test to ensure the app can be created
        assert_eq!(app.config.server.port, 8080);
    }
}
