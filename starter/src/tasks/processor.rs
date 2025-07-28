use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::database::Database;
use crate::tasks::{
    handlers::TaskHandler,
    retry::CircuitBreaker,
    types::{
        CreateTaskRequest, Task, TaskContext, TaskError, TaskFilter, TaskPriority, TaskResult,
        TaskResult2, TaskStats, TaskStatus,
    },
};

pub type TaskHandlerFn = Box<dyn TaskHandler + Send + Sync>;

#[derive(Clone)]
pub struct TaskProcessor {
    database: Database,
    handlers: Arc<RwLock<HashMap<String, TaskHandlerFn>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    semaphore: Arc<Semaphore>,
    config: ProcessorConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub poll_interval: Duration,
    pub task_timeout: Duration,
    pub max_concurrent_tasks: usize,
    pub batch_size: usize,
    pub enable_circuit_breaker: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            task_timeout: Duration::from_secs(300), // 5 minutes
            max_concurrent_tasks: 10,
            batch_size: 50,
            enable_circuit_breaker: true,
        }
    }
}

impl TaskProcessor {
    pub fn new(database: Database, config: ProcessorConfig) -> Self {
        Self {
            database,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            config,
        }
    }

    /// Register a task handler for a specific task type
    pub async fn register_handler<H>(&self, task_type: String, handler: H)
    where
        H: TaskHandler + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(task_type.clone(), Box::new(handler));

        if self.config.enable_circuit_breaker {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.insert(task_type.clone(), CircuitBreaker::default());
        }

        info!("Registered task handler for type: {}", task_type);
    }

    /// Check if a task type has a registered handler
    pub async fn has_handler(&self, task_type: &str) -> bool {
        let handlers = self.handlers.read().await;
        handlers.contains_key(task_type)
    }

    /// Create a new task
    pub async fn create_task(&self, request: CreateTaskRequest) -> TaskResult2<Task> {
        let mut conn = self.database.pool.acquire().await?;

        let task_id = Uuid::new_v4();
        let retry_strategy_json = serde_json::to_value(&request.retry_strategy)?;
        let metadata_json = serde_json::to_value(&request.metadata)?;
        let max_attempts = request.retry_strategy.max_attempts() as i32;

        let task = sqlx::query_as!(
            Task,
            r#"
            INSERT INTO tasks (
                id, task_type, payload, status, priority, retry_strategy, 
                max_attempts, current_attempt, created_at, updated_at, 
                scheduled_at, created_by, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING 
                id, task_type, payload, 
                status as "status: TaskStatus", 
                priority as "priority: TaskPriority",
                retry_strategy, max_attempts, current_attempt, last_error,
                created_at, updated_at, scheduled_at, started_at, completed_at,
                created_by, metadata
            "#,
            task_id,
            request.task_type,
            request.payload,
            TaskStatus::Pending as TaskStatus,
            request.priority as TaskPriority,
            retry_strategy_json,
            max_attempts,
            0,
            Utc::now(),
            Utc::now(),
            request.scheduled_at,
            request.created_by,
            metadata_json
        )
        .fetch_one(&mut *conn)
        .await?;

        debug!("Created task {} of type {}", task.id, task.task_type);
        Ok(task)
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: Uuid) -> TaskResult2<Option<Task>> {
        let mut conn = self.database.pool.acquire().await?;

        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT 
                id, task_type, payload, 
                status as "status: TaskStatus", 
                priority as "priority: TaskPriority",
                retry_strategy, max_attempts, current_attempt, last_error,
                created_at, updated_at, scheduled_at, started_at, completed_at,
                created_by, metadata
            FROM tasks 
            WHERE id = $1
            "#,
            task_id
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(task)
    }

    /// List tasks with filtering
    pub async fn list_tasks(&self, filter: TaskFilter) -> TaskResult2<Vec<Task>> {
        let mut conn = self.database.pool.acquire().await?;

        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT 
                id, task_type, payload, 
                status as "status: TaskStatus", 
                priority as "priority: TaskPriority",
                retry_strategy, max_attempts, current_attempt, last_error,
                created_at, updated_at, scheduled_at, started_at, completed_at,
                created_by, metadata
            FROM tasks 
            WHERE ($1::TEXT IS NULL OR task_type = $1)
              AND ($2::task_status IS NULL OR status = $2)
              AND ($3::task_priority IS NULL OR priority = $3)
              AND ($4::UUID IS NULL OR created_by = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
            ORDER BY priority DESC, created_at ASC
            LIMIT $7
            OFFSET $8
            "#,
            filter.task_type,
            filter.status as Option<TaskStatus>,
            filter.priority as Option<TaskPriority>,
            filter.created_by,
            filter.created_after,
            filter.created_before,
            filter.limit.unwrap_or(100),
            filter.offset.unwrap_or(0)
        )
        .fetch_all(&mut *conn)
        .await?;

        Ok(tasks)
    }

    /// Get task statistics
    pub async fn get_stats(&self) -> TaskResult2<TaskStats> {
        let mut conn = self.database.pool.acquire().await?;

        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'running') as running,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled,
                COUNT(*) FILTER (WHERE status = 'retrying') as retrying
            FROM tasks
            "#
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(TaskStats {
            total: stats.total.unwrap_or(0),
            pending: stats.pending.unwrap_or(0),
            running: stats.running.unwrap_or(0),
            completed: stats.completed.unwrap_or(0),
            failed: stats.failed.unwrap_or(0),
            cancelled: stats.cancelled.unwrap_or(0),
            retrying: stats.retrying.unwrap_or(0),
        })
    }

    /// Start the task processor worker loop
    pub async fn start_worker(&self) -> TaskResult2<()> {
        info!(
            "Starting task processor worker with config: {:?}",
            self.config
        );

        let mut interval = interval(self.config.poll_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.process_batch().await {
                error!("Error processing task batch: {}", e);
            }
        }
    }

    /// Process a batch of ready tasks
    async fn process_batch(&self) -> TaskResult2<()> {
        let tasks = self.fetch_ready_tasks().await?;

        if tasks.is_empty() {
            return Ok(());
        }

        info!("Processing {} ready tasks", tasks.len());

        let mut handles = Vec::new();

        for task in tasks {
            let processor = self.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = processor.process_task(task).await {
                    error!("Error processing task: {}", e);
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task processing handle error: {}", e);
            }
        }

        Ok(())
    }

    /// Fetch ready tasks from database
    async fn fetch_ready_tasks(&self) -> TaskResult2<Vec<Task>> {
        let mut conn = self.database.pool.acquire().await?;

        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT 
                id, task_type, payload, 
                status as "status: TaskStatus", 
                priority as "priority: TaskPriority",
                retry_strategy, max_attempts, current_attempt, last_error,
                created_at, updated_at, scheduled_at, started_at, completed_at,
                created_by, metadata
            FROM tasks 
            WHERE (status = 'pending' OR status = 'retrying')
              AND (scheduled_at IS NULL OR scheduled_at <= NOW())
            ORDER BY priority DESC, created_at ASC
            LIMIT $1
            "#,
            self.config.batch_size as i64
        )
        .fetch_all(&mut *conn)
        .await?;

        Ok(tasks)
    }

    /// Process a single task
    async fn process_task(&self, mut task: Task) -> TaskResult2<()> {
        // Acquire semaphore permit to limit concurrency
        let _permit = self.semaphore.acquire().await.unwrap();

        debug!("Processing task {} of type {}", task.id, task.task_type);

        // Update task status to running
        if let Err(e) = self.update_task_status(task.id, TaskStatus::Running).await {
            error!("Failed to update task status to running: {}", e);
            return Err(e);
        }

        let context = TaskContext::from(&task);

        // Check circuit breaker if enabled
        if self.config.enable_circuit_breaker {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            if let Some(cb) = circuit_breakers.get_mut(&task.task_type) {
                if !cb.should_allow_operation() {
                    let error = "Circuit breaker is open";
                    warn!("Task {} blocked by circuit breaker", task.id);
                    self.mark_task_failed(task.id, error).await?;
                    return Ok(());
                }
            }
        }

        // Execute task with timeout
        let result = {
            let handlers = self.handlers.read().await;
            match handlers.get(&task.task_type) {
                Some(handler) => {
                    // Execute handler with timeout
                    timeout(self.config.task_timeout, handler.handle(context)).await
                }
                None => {
                    let error = format!("No handler registered for task type: {}", task.task_type);
                    error!("{}", error);
                    self.mark_task_failed(task.id, &error).await?;
                    return Err(TaskError::HandlerNotFound(task.task_type));
                }
            }
        };

        match result {
            Ok(Ok(task_result)) => {
                // Task completed successfully
                if self.config.enable_circuit_breaker {
                    let mut circuit_breakers = self.circuit_breakers.write().await;
                    if let Some(cb) = circuit_breakers.get_mut(&task.task_type) {
                        cb.record_success();
                    }
                }
                self.mark_task_completed(task.id, task_result).await?;
                info!("Task {} completed successfully", task.id);
            }
            Ok(Err(e)) => {
                // Task failed
                if self.config.enable_circuit_breaker {
                    let mut circuit_breakers = self.circuit_breakers.write().await;
                    if let Some(cb) = circuit_breakers.get_mut(&task.task_type) {
                        cb.record_failure();
                    }
                }

                task.current_attempt += 1;
                let error_msg = e.to_string();
                let task_id = task.id;
                let current_attempt = task.current_attempt;

                if task.can_retry() {
                    self.schedule_retry(task, &error_msg).await?;
                    warn!(
                        "Task {} failed, scheduled for retry (attempt {})",
                        task_id, current_attempt
                    );
                } else {
                    self.mark_task_failed(task_id, &error_msg).await?;
                    error!(
                        "Task {} failed permanently after {} attempts",
                        task_id, current_attempt
                    );
                }
            }
            Err(_) => {
                // Task timed out
                let error = "Task execution timed out";
                if self.config.enable_circuit_breaker {
                    let mut circuit_breakers = self.circuit_breakers.write().await;
                    if let Some(cb) = circuit_breakers.get_mut(&task.task_type) {
                        cb.record_failure();
                    }
                }

                task.current_attempt += 1;
                let task_id = task.id;

                if task.can_retry() {
                    self.schedule_retry(task, error).await?;
                    warn!("Task {} timed out, scheduled for retry", task_id);
                } else {
                    self.mark_task_failed(task_id, error).await?;
                    error!("Task {} timed out permanently", task_id);
                }
            }
        }

        Ok(())
    }

    /// Update task status
    async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        let started_at = if matches!(status, TaskStatus::Running) {
            Some(Utc::now())
        } else {
            None
        };

        let completed_at = if matches!(
            status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, started_at = COALESCE($3, started_at), completed_at = COALESCE($4, completed_at)
            WHERE id = $5
            "#,
            status as TaskStatus,
            Utc::now(),
            started_at,
            completed_at,
            task_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Mark task as completed
    async fn mark_task_completed(&self, task_id: Uuid, result: TaskResult) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        let metadata_json = serde_json::to_value(&result.metadata)?;

        sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, completed_at = $3, metadata = metadata || $4
            WHERE id = $5
            "#,
            TaskStatus::Completed as TaskStatus,
            Utc::now(),
            Utc::now(),
            metadata_json,
            task_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Mark task as failed
    async fn mark_task_failed(&self, task_id: Uuid, error: &str) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, completed_at = $3, last_error = $4
            WHERE id = $5
            "#,
            TaskStatus::Failed as TaskStatus,
            Utc::now(),
            Utc::now(),
            error,
            task_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Schedule task for retry
    async fn schedule_retry(&self, task: Task, error: &str) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        let retry_strategy = task.get_retry_strategy()?;
        let delay = retry_strategy.calculate_delay(task.current_attempt as u32);

        let scheduled_at =
            delay.map(|delay| Utc::now() + chrono::Duration::from_std(delay).unwrap());

        sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, current_attempt = $3, last_error = $4, scheduled_at = $5
            WHERE id = $6
            "#,
            TaskStatus::Retrying as TaskStatus,
            Utc::now(),
            task.current_attempt,
            error,
            scheduled_at,
            task.id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, completed_at = $3
            WHERE id = $4 AND status IN ('pending', 'retrying')
            "#,
            TaskStatus::Cancelled as TaskStatus,
            Utc::now(),
            Utc::now(),
            task_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    /// Retry a failed task
    pub async fn retry_task(&self, task_id: Uuid) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        // Reset task to pending and clear error state
        let result = sqlx::query!(
            r#"
            UPDATE tasks 
            SET status = $1, updated_at = $2, current_attempt = 0, last_error = NULL, 
                scheduled_at = NULL, started_at = NULL, completed_at = NULL
            WHERE id = $3 AND status = 'failed'
            "#,
            TaskStatus::Pending as TaskStatus,
            Utc::now(),
            task_id
        )
        .execute(&mut *conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskError::NotFound(task_id));
        }

        Ok(())
    }

    /// Delete a task permanently
    pub async fn delete_task(&self, task_id: Uuid) -> TaskResult2<()> {
        let mut conn = self.database.pool.acquire().await?;

        let result = sqlx::query!(
            r#"
            DELETE FROM tasks 
            WHERE id = $1 AND status IN ('completed', 'failed', 'cancelled')
            "#,
            task_id
        )
        .execute(&mut *conn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskError::NotFound(task_id));
        }

        Ok(())
    }

    /// Get dead letter queue (failed tasks)
    pub async fn get_dead_letter_queue(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> TaskResult2<Vec<Task>> {
        let filter = TaskFilter {
            status: Some(TaskStatus::Failed),
            limit,
            offset,
            ..Default::default()
        };
        self.list_tasks(filter).await
    }
}
