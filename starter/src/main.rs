use clap::{Parser, Subcommand};
use starter::{AppConfig, Database, server, tasks};

#[derive(Parser)]
#[command(name = "starter")]
#[command(about = "Rust + React Full-Stack Starter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Server {
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Start the background worker
    Worker,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Server { port } => {
            let mut config = AppConfig::load()?;
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
        Commands::Worker => {
            let config = AppConfig::load()?;
            let database = Database::connect(&config).await?;
            database.migrate().await?;
            
            // Create task processor with configuration
            let processor_config = tasks::processor::ProcessorConfig {
                poll_interval: config.poll_interval(),
                task_timeout: std::time::Duration::from_secs(300),
                max_concurrent_tasks: config.worker.concurrency,
                batch_size: 50,
                enable_circuit_breaker: true,
            };
            
            let processor = tasks::processor::TaskProcessor::new(database, processor_config);
            
            // Register example task handlers
            tasks::handlers::register_example_handlers(&processor).await;
            
            println!("Background worker starting with {} max concurrent tasks", config.worker.concurrency);
            
            // Start the worker loop
            processor.start_worker().await?;
            
            Ok(())
        }
    }
}