use clap::{Parser, Subcommand};
use serde_json::json;
use sqlx::Row;
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
    /// Health check for Docker/Kubernetes
    #[command(name = "health-check")]
    HealthCheck,
    /// Export OpenAPI specification to docs folder
    #[command(name = "export-openapi")]
    ExportOpenApi {
        /// Output file path (default: docs/openapi.json)
        #[arg(long, default_value = "docs/openapi.json")]
        output: String,
    },
    /// Admin commands for direct database access
    Admin {
        #[command(subcommand)]
        admin_command: AdminCommands,
    },
}

#[derive(Subcommand)]
enum AdminCommands {
    /// List tasks with optional filtering
    #[command(name = "list-tasks")]
    ListTasks {
        /// Filter by task status (pending, running, completed, failed)
        #[arg(long)]
        status: Option<String>,
        /// Filter by task type
        #[arg(long)]
        task_type: Option<String>,
        /// Limit number of results
        #[arg(long, default_value = "50")]
        limit: i32,
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
    /// Show task statistics
    #[command(name = "task-stats")]
    TaskStats {
        /// Filter by task tag in metadata
        #[arg(long)]
        tag: Option<String>,
    },
    /// Clear completed tasks older than specified days
    #[command(name = "clear-completed")]
    ClearCompleted {
        #[arg(long, default_value = "7")]
        older_than_days: i32,
        #[arg(long)]
        dry_run: bool,
    },
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

            // Register task types with the API
            if let Err(e) = register_task_types_with_api(&config).await {
                eprintln!("Warning: Failed to register task types with API: {e}");
                eprintln!(
                    "Worker will continue, but new tasks may be rejected until types are registered"
                );
            }

            println!(
                "Background worker starting with {} max concurrent tasks",
                config.worker.concurrency
            );

            // Start the worker loop
            processor.start_worker().await?;

            Ok(())
        }
        Commands::HealthCheck => {
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
        Commands::ExportOpenApi { output } => {
            // Export OpenAPI specification to file
            use starter::openapi;
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
        Commands::Admin { admin_command } => {
            let config = AppConfig::load()?;
            let database = Database::connect(&config).await?;

            match admin_command {
                AdminCommands::ListTasks {
                    status: _,
                    task_type: _,
                    limit,
                    verbose,
                } => {
                    // Use working dynamic query approach
                    let tasks = sqlx::query("SELECT id, task_type, status::text as status, priority::text as priority, created_at, updated_at, metadata FROM tasks ORDER BY created_at DESC LIMIT $1")
                        .bind(limit as i64)
                        .fetch_all(&database.pool)
                        .await?;

                    if tasks.is_empty() {
                        println!("No tasks found");
                        return Ok(());
                    }

                    println!("📋 Found {} tasks:", tasks.len());
                    for task in tasks {
                        let id: uuid::Uuid = task.get("id");
                        let task_type: String = task.get("task_type");
                        let status: String = task.get("status");
                        let priority: String = task.get("priority");
                        let created_at: chrono::DateTime<chrono::Utc> = task.get("created_at");
                        let updated_at: chrono::DateTime<chrono::Utc> = task.get("updated_at");
                        let metadata: serde_json::Value = task.get("metadata");

                        if verbose {
                            println!(
                                "🔸 {} | {} | {} | Priority: {} | Created: {} | Updated: {}",
                                id,
                                task_type,
                                status,
                                priority,
                                created_at.format("%Y-%m-%d %H:%M:%S"),
                                updated_at.format("%Y-%m-%d %H:%M:%S")
                            );
                            println!("   Metadata: {metadata}");
                        } else {
                            println!(
                                "🔸 {} | {} | {} | {}",
                                id,
                                task_type,
                                status,
                                created_at.format("%H:%M:%S")
                            );
                        }
                    }
                    Ok(())
                }
                AdminCommands::TaskStats { tag } => {
                    let stats = if let Some(ref tag_filter) = tag {
                        println!("📊 Task Statistics for tag '{tag_filter}':");
                        sqlx::query("SELECT status::text as status, COUNT(*) as count FROM tasks WHERE metadata->>'tag' = $1 GROUP BY status")
                            .bind(tag_filter)
                            .fetch_all(&database.pool)
                            .await?
                    } else {
                        println!("📊 Overall Task Statistics:");
                        sqlx::query("SELECT status::text as status, COUNT(*) as count FROM tasks GROUP BY status")
                            .fetch_all(&database.pool)
                            .await?
                    };

                    let mut total = 0i64;
                    for stat in &stats {
                        let status: String = stat.get("status");
                        let count: i64 = stat.get("count");
                        total += count;
                        println!("  {status}: {count}");
                    }
                    println!("  Total: {total}");

                    // Additional timing stats for completed tasks
                    let avg_duration_query = if let Some(ref tag_filter) = tag {
                        sqlx::query("SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration FROM tasks WHERE status = 'completed' AND metadata->>'tag' = $1")
                            .bind(tag_filter)
                            .fetch_optional(&database.pool)
                            .await?
                    } else {
                        sqlx::query("SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration FROM tasks WHERE status = 'completed'")
                            .fetch_optional(&database.pool)
                            .await?
                    };

                    if let Some(result) = avg_duration_query {
                        if let Ok(duration) = result.try_get::<f64, _>("avg_duration") {
                            println!("  Average completion time: {duration:.2}s");
                        }
                    }

                    Ok(())
                }
                AdminCommands::ClearCompleted {
                    older_than_days,
                    dry_run,
                } => {
                    let cutoff_time =
                        chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);

                    let count_result = sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE status = 'completed' AND updated_at < $1")
                        .bind(cutoff_time)
                        .fetch_one(&database.pool)
                        .await?;

                    let count: i64 = count_result.get("count");

                    if dry_run {
                        println!(
                            "🔍 DRY RUN: Would delete {count} completed tasks older than {older_than_days} days"
                        );
                    } else {
                        let result = sqlx::query(
                            "DELETE FROM tasks WHERE status = 'completed' AND updated_at < $1",
                        )
                        .bind(cutoff_time)
                        .execute(&database.pool)
                        .await?;

                        println!(
                            "🗑️  Deleted {} completed tasks older than {older_than_days} days",
                            result.rows_affected()
                        );
                    }

                    Ok(())
                }
            }
        }
    }
}

/// Register task types with the API server
async fn register_task_types_with_api(
    _config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to determine the API server URL (worker might be connecting to a different host)
    let base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let client = reqwest::Client::new();

    // Define task types that this worker handles
    let task_types = [
        ("email", "Email notification tasks"),
        ("data_processing", "Data processing and analysis tasks"),
        ("file_cleanup", "File system cleanup tasks"),
        ("report_generation", "Report generation tasks"),
        ("webhook", "Webhook notification tasks"),
        (
            "delay_task",
            "Delay/sleep tasks for testing and chaos scenarios",
        ),
    ];

    for (task_type, description) in task_types.iter() {
        let response = client
            .post(format!("{base_url}/tasks/types"))
            .header("Content-Type", "application/json")
            .json(&json!({
                "task_type": task_type,
                "description": description
            }))
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Registered task type: {task_type}");
        } else {
            eprintln!(
                "⚠️  Failed to register task type '{}': {}",
                task_type,
                response.status()
            );
        }
    }

    Ok(())
}
