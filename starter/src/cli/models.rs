use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "starter")]
#[command(about = "Rust + React Full-Stack Starter")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum AdminCommands {
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

/// Task information for CLI display
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: uuid::Uuid,
    pub task_type: String,
    pub status: String,
    pub priority: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

/// Task statistics for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub status: String,
    pub count: i64,
}

/// Overall task statistics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatsSummary {
    pub stats: Vec<TaskStats>,
    pub total: i64,
    pub avg_completion_time: Option<f64>,
}

/// Configuration for CLI admin operations
#[derive(Debug, Clone)]
pub struct AdminConfig {
    pub default_limit: i32,
    pub default_days: i32,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            default_limit: 50,
            default_days: 7,
        }
    }
}
