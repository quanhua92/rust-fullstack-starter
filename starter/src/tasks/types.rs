use crate::tasks::retry::RetryStrategy;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_priority", rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub retry_strategy: serde_json::Value, // Serialized RetryStrategy
    pub max_attempts: i32,
    pub current_attempt: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
}

impl Task {
    pub fn get_retry_strategy(&self) -> Result<RetryStrategy, serde_json::Error> {
        serde_json::from_value(self.retry_strategy.clone())
    }

    pub fn is_ready_to_run(&self) -> bool {
        match self.status {
            TaskStatus::Pending => {
                if let Some(scheduled_at) = self.scheduled_at {
                    scheduled_at <= Utc::now()
                } else {
                    true
                }
            }
            TaskStatus::Retrying => {
                if let Some(scheduled_at) = self.scheduled_at {
                    scheduled_at <= Utc::now()
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn can_retry(&self) -> bool {
        match self.status {
            TaskStatus::Failed => self.current_attempt < self.max_attempts,
            _ => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Cancelled)
    }
}

// API response type for tasks (excludes sensitive internal fields)
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub max_attempts: i32,
    pub current_attempt: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        // Convert JSON metadata to HashMap
        let metadata =
            serde_json::from_value::<std::collections::HashMap<String, serde_json::Value>>(
                task.metadata,
            )
            .unwrap_or_default();

        Self {
            id: task.id,
            task_type: task.task_type,
            status: task.status,
            priority: task.priority,
            max_attempts: task.max_attempts,
            current_attempt: task.current_attempt,
            last_error: task.last_error,
            created_at: task.created_at,
            updated_at: task.updated_at,
            scheduled_at: task.scheduled_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            created_by: task.created_by,
            metadata,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub priority: TaskPriority,
    #[serde(default)]
    pub retry_strategy: RetryStrategy,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CreateTaskRequest {
    pub fn new(task_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            task_type: task_type.into(),
            payload,
            priority: TaskPriority::default(),
            retry_strategy: RetryStrategy::default(),
            scheduled_at: None,
            created_by: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_retry_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.retry_strategy = strategy;
        self
    }

    pub fn with_scheduled_at(mut self, scheduled_at: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }

    pub fn with_created_by(mut self, created_by: Uuid) -> Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task_id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub attempt: i32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<&Task> for TaskContext {
    fn from(task: &Task) -> Self {
        Self {
            task_id: task.id,
            task_type: task.task_type.clone(),
            payload: task.payload.clone(),
            attempt: task.current_attempt,
            metadata: task
                .metadata
                .as_object()
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default(),
            created_by: task.created_by,
            created_at: task.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TaskResult {
    pub fn success(output: serde_json::Value) -> Self {
        Self {
            success: true,
            output: Some(output),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn success_empty() -> Self {
        Self {
            success: true,
            output: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub task_type: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub created_by: Option<Uuid>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            task_type: None,
            status: None,
            priority: None,
            created_by: None,
            created_after: None,
            created_before: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskStats {
    pub total: i64,
    pub pending: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub retrying: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Task not found: {0}")]
    NotFound(Uuid),
    #[error("Invalid task status transition: {from:?} -> {to:?}")]
    InvalidStatusTransition { from: TaskStatus, to: TaskStatus },
    #[error("Task execution error: {0}")]
    Execution(String),
    #[error("Task handler not found: {0}")]
    HandlerNotFound(String),
    #[error("Task timeout")]
    Timeout,
    #[error("Task cancelled")]
    Cancelled,
}

impl TaskError {
    /// Helper for creating missing field errors
    pub fn missing_field(field: &str) -> Self {
        Self::Execution(format!("Missing '{field}' field in payload"))
    }

    /// Helper for creating invalid field type errors
    pub fn invalid_field_type(field: &str, expected: &str) -> Self {
        Self::Execution(format!(
            "Invalid '{field}' field type, expected {expected}"
        ))
    }
}

pub type TaskResult2<T> = Result<T, TaskError>;
