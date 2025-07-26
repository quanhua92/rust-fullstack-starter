use clap::{Parser, Subcommand};
use starter::{AppConfig, Database, server};

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
            let _database = Database::connect(&config).await?;
            println!("Worker starting with {} concurrency", config.worker.concurrency);
            // Worker implementation will be added in Phase 5
            Ok(())
        }
    }
}