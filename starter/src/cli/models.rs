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
    /// Generate code from templates
    Generate {
        #[command(subcommand)]
        generator: GenerateCommands,
    },
    /// Revert generated code (removes files and reverts migrations)
    Revert {
        #[command(subcommand)]
        revert: RevertCommands,
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

#[derive(Subcommand)]
pub enum GenerateCommands {
    /// Generate a complete module with API, models, services, tests, and migrations
    Module {
        /// Module name (e.g., "books", "users")
        name: String,
        /// Template to use (basic, production)
        #[arg(long, default_value = "basic")]
        template: String,
        /// Dry run - show what would be created
        #[arg(long)]
        dry_run: bool,
        /// Force overwrite existing files
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum RevertCommands {
    /// Revert a generated module (removes files and reverts migrations)
    Module {
        /// Module name to revert (e.g., "books", "users")
        name: String,
        /// Skip all confirmation prompts (DANGEROUS)
        #[arg(long)]
        yes: bool,
        /// Only show what would be reverted without doing it
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
