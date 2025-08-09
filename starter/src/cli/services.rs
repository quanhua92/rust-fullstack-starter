use super::models::{AdminCommands, TaskInfo, TaskStats, TaskStatsSummary};
use crate::{Database, Error};
use serde_json::json;
use sqlx::Row;

/// Service for handling admin CLI operations
pub struct AdminService {
    database: Database,
}

impl AdminService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// List tasks with optional filtering
    pub async fn list_tasks(
        &self,
        status: Option<String>,
        task_type: Option<String>,
        limit: i32,
        _verbose: bool,
    ) -> Result<Vec<TaskInfo>, Error> {
        // Future enhancement: Add filtering by status and task_type
        let _ = (status, task_type); // Suppress unused warnings

        let tasks = sqlx::query(
            "SELECT id, task_type, status::text as status, priority::text as priority, created_at, updated_at, metadata 
             FROM tasks 
             ORDER BY created_at DESC 
             LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.database.pool)
        .await
        .map_err(Error::Database)?;

        let mut task_infos = Vec::new();
        for task in tasks {
            let task_info = TaskInfo {
                id: task.get("id"),
                task_type: task.get("task_type"),
                status: task.get("status"),
                priority: task.get("priority"),
                created_at: task.get("created_at"),
                updated_at: task.get("updated_at"),
                metadata: task.get("metadata"),
            };
            task_infos.push(task_info);
        }

        Ok(task_infos)
    }

    /// Display tasks in CLI format
    pub fn display_tasks(&self, tasks: &[TaskInfo], verbose: bool) {
        if tasks.is_empty() {
            println!("No tasks found");
            return;
        }

        println!("📋 Found {} tasks:", tasks.len());
        for task in tasks {
            if verbose {
                println!(
                    "🔸 {} | {} | {} | Priority: {} | Created: {} | Updated: {}",
                    task.id,
                    task.task_type,
                    task.status,
                    task.priority,
                    task.created_at.format("%Y-%m-%d %H:%M:%S"),
                    task.updated_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!("   Metadata: {}", task.metadata);
            } else {
                println!(
                    "🔸 {} | {} | {} | {}",
                    task.id,
                    task.task_type,
                    task.status,
                    task.created_at.format("%H:%M:%S")
                );
            }
        }
    }

    /// Get task statistics
    pub async fn get_task_stats(&self, tag: Option<String>) -> Result<TaskStatsSummary, Error> {
        let stats = if let Some(ref tag_filter) = tag {
            sqlx::query(
                "SELECT status::text as status, COUNT(*) as count 
                 FROM tasks 
                 WHERE metadata->>'tag' = $1 
                 GROUP BY status",
            )
            .bind(tag_filter)
            .fetch_all(&self.database.pool)
            .await
            .map_err(Error::Database)?
        } else {
            sqlx::query(
                "SELECT status::text as status, COUNT(*) as count 
                 FROM tasks 
                 GROUP BY status",
            )
            .fetch_all(&self.database.pool)
            .await
            .map_err(Error::Database)?
        };

        let mut task_stats = Vec::new();
        let mut total = 0i64;

        for stat in stats {
            let status: String = stat.get("status");
            let count: i64 = stat.get("count");
            total += count;
            task_stats.push(TaskStats { status, count });
        }

        // Get average completion time
        let avg_duration_query = if let Some(ref tag_filter) = tag {
            sqlx::query(
                "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration 
                 FROM tasks 
                 WHERE status = 'completed' AND metadata->>'tag' = $1",
            )
            .bind(tag_filter)
            .fetch_optional(&self.database.pool)
            .await
            .map_err(Error::Database)?
        } else {
            sqlx::query(
                "SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration 
                 FROM tasks 
                 WHERE status = 'completed'",
            )
            .fetch_optional(&self.database.pool)
            .await
            .map_err(Error::Database)?
        };

        let avg_completion_time =
            avg_duration_query.and_then(|result| result.try_get::<f64, _>("avg_duration").ok());

        Ok(TaskStatsSummary {
            stats: task_stats,
            total,
            avg_completion_time,
        })
    }

    /// Display task statistics in CLI format
    pub fn display_task_stats(&self, summary: &TaskStatsSummary, tag: Option<&String>) {
        if let Some(tag_filter) = tag {
            println!("📊 Task Statistics for tag '{tag_filter}':");
        } else {
            println!("📊 Overall Task Statistics:");
        }

        for stat in &summary.stats {
            println!("  {}: {}", stat.status, stat.count);
        }
        println!("  Total: {}", summary.total);

        if let Some(avg_duration) = summary.avg_completion_time {
            println!("  Average completion time: {avg_duration:.2}s");
        }
    }

    /// Clear completed tasks older than specified days
    pub async fn clear_completed_tasks(
        &self,
        older_than_days: i32,
        dry_run: bool,
    ) -> Result<i64, Error> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);

        let count_result = sqlx::query(
            "SELECT COUNT(*) as count 
             FROM tasks 
             WHERE status = 'completed' AND updated_at < $1",
        )
        .bind(cutoff_time)
        .fetch_one(&self.database.pool)
        .await
        .map_err(Error::Database)?;

        let count: i64 = count_result.get("count");

        if dry_run {
            println!(
                "🔍 DRY RUN: Would delete {count} completed tasks older than {older_than_days} days"
            );
            Ok(count)
        } else {
            let result = sqlx::query(
                "DELETE FROM tasks 
                 WHERE status = 'completed' AND updated_at < $1",
            )
            .bind(cutoff_time)
            .execute(&self.database.pool)
            .await
            .map_err(Error::Database)?;

            let deleted_count = result.rows_affected() as i64;
            println!(
                "🗑️  Deleted {deleted_count} completed tasks older than {older_than_days} days"
            );
            Ok(deleted_count)
        }
    }
}

/// Service for handling task type registration with API
pub struct TaskTypeService;

impl TaskTypeService {
    /// Register task types with the API server
    pub async fn register_task_types_with_api(
        api_base_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let base_url = api_base_url
            .or_else(|| std::env::var("API_BASE_URL").ok())
            .unwrap_or_else(|| "http://localhost:3000".to_string());

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
                .post(format!("{base_url}/api/v1/tasks/types"))
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
}

/// Execute admin command
pub async fn execute_admin_command(
    database: Database,
    admin_command: AdminCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let admin_service = AdminService::new(database);

    match admin_command {
        AdminCommands::ListTasks {
            status,
            task_type,
            limit,
            verbose,
        } => {
            let tasks = admin_service
                .list_tasks(status, task_type, limit, verbose)
                .await?;
            admin_service.display_tasks(&tasks, verbose);
            Ok(())
        }
        AdminCommands::TaskStats { tag } => {
            let summary = admin_service.get_task_stats(tag.clone()).await?;
            admin_service.display_task_stats(&summary, tag.as_ref());
            Ok(())
        }
        AdminCommands::ClearCompleted {
            older_than_days,
            dry_run,
        } => {
            admin_service
                .clear_completed_tasks(older_than_days, dry_run)
                .await?;
            Ok(())
        }
    }
}
