use super::{
    models::{Cli, Commands, GenerateCommands},
    services::{TaskTypeService, execute_admin_command},
};
use crate::{AppConfig, Database, server, tasks};
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
        use crate::openapi;
        use std::fs;

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
            GenerateCommands::Module { name, template, dry_run, force } => {
                self.generate_module(name, template, dry_run, force).await
            }
        }
    }

    /// Generate a module from templates
    async fn generate_module(
        &self,
        name: String,
        template: String,
        dry_run: bool,
        _force: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::collections::HashMap;
        use std::fs;
        use std::path::Path;

        println!("🚀 Generating module '{}' using '{}' template", name, template);
        
        if dry_run {
            println!("🔍 DRY RUN MODE - No files will be created");
        }

        // Create singular/plural forms
        let singular = &name;
        let plural = if name.ends_with('s') {
            name.clone()
        } else {
            format!("{}s", singular)
        }; // Simple pluralization
        let struct_name = capitalize_first(&singular);
        let table_name = &plural;

        // Template replacements
        let mut replacements = HashMap::new();
        replacements.insert("__MODULE_NAME__", singular);
        replacements.insert("__MODULE_NAME_PLURAL__", &plural);
        replacements.insert("__MODULE_STRUCT__", &struct_name);
        replacements.insert("__MODULE_TABLE__", table_name);

        let template_dir = format!("../templates/{}", template);
        if !Path::new(&template_dir).exists() {
            return Err(format!("Template '{}' not found in templates directory", template).into());
        }

        let mut files_created = Vec::new();

        // Create module directory
        let module_dir = format!("src/{}", plural);
        if !dry_run {
            fs::create_dir_all(&module_dir)?;
        }
        println!("📁 Created directory: {}", module_dir);

        // Copy and process template files
        let template_files = [
            ("api.rs", "api.rs"),
            ("models.rs", "models.rs"), 
            ("services.rs", "services.rs"),
            ("mod.rs", "mod.rs"),
        ];

        for (template_file, output_file) in template_files {
            let template_path = format!("{}/{}", template_dir, template_file);
            let output_path = format!("{}/{}", module_dir, output_file);
            
            if Path::new(&template_path).exists() {
                let content = fs::read_to_string(&template_path)?;
                let processed = process_template(&content, &replacements);
                
                if !dry_run {
                    fs::write(&output_path, processed)?;
                }
                files_created.push(output_path.clone());
                println!("📄 Created: {}", output_path);
            }
        }

        // Create test directory and file
        let test_dir = format!("tests/{}", plural);
        if !dry_run {
            fs::create_dir_all(&test_dir)?;
        }

        let test_template_path = format!("{}/tests.rs", template_dir);
        if Path::new(&test_template_path).exists() {
            let test_content = fs::read_to_string(&test_template_path)?;
            let processed_test = process_template(&test_content, &replacements);
            let test_output = format!("{}/mod.rs", test_dir);
            
            if !dry_run {
                fs::write(&test_output, processed_test)?;
            }
            files_created.push(test_output.clone());
            println!("📄 Created: {}", test_output);
        }

        // Create migrations
        let migrations_dir = "migrations";
        let migration_number = get_next_migration_number(migrations_dir)?;
        
        let migration_files = [
            ("up.sql", format!("{:03}_{}.up.sql", migration_number, plural)),
            ("down.sql", format!("{:03}_{}.down.sql", migration_number, plural)),
        ];

        for (template_file, output_file) in migration_files {
            let template_path = format!("{}/{}", template_dir, template_file);
            let output_path = format!("{}/{}", migrations_dir, output_file);
            
            if Path::new(&template_path).exists() {
                let content = fs::read_to_string(&template_path)?;
                let processed = process_template(&content, &replacements);
                
                if !dry_run {
                    fs::write(&output_path, processed)?;
                }
                files_created.push(output_path.clone());
                println!("📄 Created: {}", output_path);
            }
        }

        // Update lib.rs
        let lib_rs_path = "src/lib.rs";
        if Path::new(lib_rs_path).exists() && !dry_run {
            let lib_content = fs::read_to_string(lib_rs_path)?;
            let module_declaration = format!("pub mod {};", plural);
            
            if !lib_content.contains(&module_declaration) {
                let updated_content = format!("{}\n{}", lib_content.trim(), module_declaration);
                fs::write(lib_rs_path, updated_content)?;
                println!("📝 Updated: {}", lib_rs_path);
            }
        }

        println!("✅ Module generation completed!");
        println!("📄 Files created: {}", files_created.len());
        
        if !dry_run {
            println!("\n📋 Next steps - run these commands:");
            println!("   1. Run the migration:");
            println!("      cd starter && sqlx migrate run");
            println!();
            println!("   2. Update sqlx query cache:");
            println!("      cd starter && cargo sqlx prepare");
            println!();
            println!("   3. Test compilation:");
            println!("      cd starter && cargo check");
            println!();
            println!("   4. Add routes to server.rs (manual step):");
            println!("      - Import: use crate::{}::api::{}_routes;", plural, plural);
            println!("      - Add route: .nest(\"/api/v1/{}\", {}_routes())", plural, plural);
            println!();
            println!("   5. Add to openapi.rs (manual step):");
            println!("      - Import the structs from {}::models", plural);
        }

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
fn process_template(content: &str, replacements: &std::collections::HashMap<&str, &String>) -> String {
    let mut result = content.to_string();
    for (placeholder, replacement) in replacements {
        result = result.replace(placeholder, replacement);
    }
    result
}

/// Get the next migration number
fn get_next_migration_number(migrations_dir: &str) -> Result<u32, Box<dyn std::error::Error>> {
    use std::fs;
    
    if !std::path::Path::new(migrations_dir).exists() {
        return Ok(1);
    }

    let mut max_number = 0;
    let entries = fs::read_dir(migrations_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();
        
        if let Some(number_str) = filename_str.split('_').next() {
            if let Ok(number) = number_str.parse::<u32>() {
                max_number = max_number.max(number);
            }
        }
    }

    Ok(max_number + 1)
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
