use crate::error::Error;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export user models from the users module (domain-driven organization)
pub use crate::users::models::{
    CreateUserRequest, User, UserProfile, validate_email, validate_password, validate_username,
};

// Session models
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

// API Keys model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing)] // Never serialize key hash
    pub key_hash: String,
    pub key_prefix: String,
    pub created_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub permissions: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
}

// Task models for background processing
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: i32,
    pub max_retries: i32,
    pub retry_count: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub const STATUS_PENDING: &'static str = "pending";
    pub const STATUS_RUNNING: &'static str = "running";
    pub const STATUS_COMPLETED: &'static str = "completed";
    pub const STATUS_FAILED: &'static str = "failed";
    pub const STATUS_CANCELLED: &'static str = "cancelled";

    pub fn is_finished(&self) -> bool {
        matches!(
            self.status.as_str(),
            Self::STATUS_COMPLETED | Self::STATUS_FAILED | Self::STATUS_CANCELLED
        )
    }
}

// Request/Response models for API

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub payload: serde_json::Value,
    pub priority: Option<i32>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl CreateTaskRequest {
    pub fn validate(&self) -> Result<()> {
        if self.task_type.trim().is_empty() {
            return Err(Error::validation("task_type", "Task type cannot be empty"));
        }
        if self.task_type.len() > 50 {
            return Err(Error::validation(
                "task_type",
                "Task type must be 50 characters or less",
            ));
        }
        Ok(())
    }
}
