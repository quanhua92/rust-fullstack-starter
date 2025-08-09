use crate::tasks::retry::RetryStrategy;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Task status for background job queue
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
            TaskStatus::Retrying => "retrying",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "running" => Ok(TaskStatus::Running),
            "completed" => Ok(TaskStatus::Completed),
            "failed" => Ok(TaskStatus::Failed),
            "cancelled" => Ok(TaskStatus::Cancelled),
            "retrying" => Ok(TaskStatus::Retrying),
            _ => Err(Error::validation("task_status", "Invalid task status")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Normal => "normal",
            TaskPriority::High => "high",
            TaskPriority::Critical => "critical",
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "low" => Ok(TaskPriority::Low),
            "normal" => Ok(TaskPriority::Normal),
            "high" => Ok(TaskPriority::High),
            "critical" => Ok(TaskPriority::Critical),
            _ => Err(Error::validation("task_priority", "Invalid task priority")),
        }
    }
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
    pub fn get_retry_strategy(&self) -> std::result::Result<RetryStrategy, serde_json::Error> {
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
    const MAX_TASK_TYPE_LEN: usize = 128;
    const MAX_PAYLOAD_SIZE_BYTES: usize = 1_024 * 1_024; // 1MB
    const MAX_METADATA_KEY_LEN: usize = 128;
    const MAX_METADATA_VALUE_SIZE_BYTES: usize = 4 * 1_024; // 4KB
    const MAX_TOTAL_METADATA_SIZE_BYTES: usize = 64 * 1_024; // 64KB
    const MAX_SCHEDULE_FUTURE_DAYS: i64 = 365;
    const MAX_SCHEDULE_PAST_HOURS: i64 = 1;

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

    /// Validates the CreateTaskRequest for security and correctness
    pub fn validate(&self) -> std::result::Result<(), String> {
        // Validate task_type is not empty and contains only safe characters
        if self.task_type.is_empty() {
            return Err("Task type cannot be empty".to_string());
        }

        if self.task_type.len() > Self::MAX_TASK_TYPE_LEN {
            return Err(format!(
                "Task type cannot exceed {} characters",
                Self::MAX_TASK_TYPE_LEN
            ));
        }

        // Only allow alphanumeric characters, underscores, and hyphens in task_type
        if !self
            .task_type
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(
                "Task type can only contain alphanumeric characters, underscores, and hyphens"
                    .to_string(),
            );
        }

        // Validate payload is a reasonable size (prevent DoS attacks)
        let payload_str = self.payload.to_string();
        if payload_str.len() > Self::MAX_PAYLOAD_SIZE_BYTES {
            return Err(format!(
                "Task payload cannot exceed {}MB",
                Self::MAX_PAYLOAD_SIZE_BYTES / (1024 * 1024)
            ));
        }

        // Validate metadata keys and values
        for (key, value) in &self.metadata {
            if key.is_empty() || key.len() > Self::MAX_METADATA_KEY_LEN {
                return Err(format!(
                    "Metadata keys must be 1-{} characters long",
                    Self::MAX_METADATA_KEY_LEN
                ));
            }

            // Only allow alphanumeric characters, underscores, and hyphens in metadata keys
            if !key
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err("Metadata keys can only contain alphanumeric characters, underscores, and hyphens".to_string());
            }

            let value_str = value.to_string();
            if value_str.len() > Self::MAX_METADATA_VALUE_SIZE_BYTES {
                return Err(format!(
                    "Metadata values cannot exceed {}KB",
                    Self::MAX_METADATA_VALUE_SIZE_BYTES / 1024
                ));
            }
        }

        // Validate total metadata size
        let total_metadata_size: usize = self
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.to_string().len())
            .sum();
        if total_metadata_size > Self::MAX_TOTAL_METADATA_SIZE_BYTES {
            return Err(format!(
                "Total metadata size cannot exceed {}KB",
                Self::MAX_TOTAL_METADATA_SIZE_BYTES / 1024
            ));
        }

        // Validate scheduled_at is not too far in the future
        if let Some(scheduled_at) = self.scheduled_at {
            let now = chrono::Utc::now();
            let max_future = now + chrono::Duration::days(Self::MAX_SCHEDULE_FUTURE_DAYS);
            if scheduled_at > max_future {
                return Err(format!(
                    "Tasks cannot be scheduled more than {} days in the future",
                    Self::MAX_SCHEDULE_FUTURE_DAYS
                ));
            }

            // Don't allow scheduling tasks too far in the past
            let max_past = now - chrono::Duration::hours(Self::MAX_SCHEDULE_PAST_HOURS);
            if scheduled_at < max_past {
                return Err(format!(
                    "Tasks cannot be scheduled more than {} hour(s) in the past",
                    Self::MAX_SCHEDULE_PAST_HOURS
                ));
            }
        }

        Ok(())
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
        Self::Execution(format!("Invalid '{field}' field type, expected {expected}"))
    }
}

pub type TaskResult2<T> = std::result::Result<T, TaskError>;

// IMPORTANT: From<String> implementations are REQUIRED by SQLx query_as! macros
//
// These implementations exist solely to support SQLx's query_as! macro which
// automatically converts database strings to enums. Without these, query_as!
// compilation will fail with "the trait bound `TaskStatus: From<String>`" errors.
//
// DESIGN CHOICE: We use safe fallbacks with error logging instead of panicking
// to prevent server crashes from corrupted database data. TaskStatus falls back
// to 'failed' to prevent re-execution of corrupted tasks. The sqlx::Decode
// implementation (below) provides proper error handling when used directly.
impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        TaskStatus::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid task_status in database: '{}', falling back to 'failed'",
                s
            );
            TaskStatus::Failed
        })
    }
}

// Required by SQLx query_as! macro - see TaskStatus implementation above for details
impl From<String> for TaskPriority {
    fn from(s: String) -> Self {
        TaskPriority::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid task_priority in database: '{}', falling back to 'normal'",
                s
            );
            TaskPriority::Normal
        })
    }
}

// SQLx implementations for TaskStatus
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TaskStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(TaskStatus::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for TaskStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for TaskStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx implementations for TaskPriority
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TaskPriority {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(TaskPriority::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for TaskPriority {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for TaskPriority {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}
