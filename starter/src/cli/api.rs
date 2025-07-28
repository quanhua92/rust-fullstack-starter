use super::{
    models::{Cli, Commands},
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
        // Initialize logging
        tracing_subscriber::fmt::init();

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
