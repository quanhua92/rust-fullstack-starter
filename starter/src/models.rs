use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::types::Result;
use crate::error::Error;

// User models with proper validation
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]  // Never serialize password hash
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    pub const ROLE_ADMIN: &'static str = "admin";
    pub const ROLE_USER: &'static str = "user";

    pub fn is_admin(&self) -> bool {
        self.role == Self::ROLE_ADMIN
    }

    /// Convert User to UserProfile (removes sensitive data)
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            created_at: self.created_at,
            last_login_at: self.last_login_at,
        }
    }
}

// User profile for API responses (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

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
    #[serde(skip_serializing)]  // Never serialize key hash
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
        matches!(self.status.as_str(), 
            Self::STATUS_COMPLETED | Self::STATUS_FAILED | Self::STATUS_CANCELLED)
    }
}

// Request/Response models for API
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<()> {
        validate_username(&self.username)?;
        validate_email(&self.email)?;
        validate_password(&self.password)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
    pub user_agent: Option<String>,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<()> {
        if self.username_or_email.trim().is_empty() {
            return Err(Error::validation("username_or_email", "Username or email cannot be empty"));
        }
        if self.password.is_empty() {
            return Err(Error::validation("password", "Password cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserProfile,
}

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
            return Err(Error::validation("task_type", "Task type must be 50 characters or less"));
        }
        Ok(())
    }
}

// Validation helpers
pub fn validate_email(email: &str) -> Result<()> {
    if email.len() < 3 || !email.contains('@') || email.len() > 254 {
        return Err(Error::validation("email", "Invalid email format"));
    }
    Ok(())
}

pub fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 50 {
        return Err(Error::validation("username", "Username must be between 3 and 50 characters"));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::validation("username", "Username can only contain letters, numbers, underscores, and hyphens"));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(Error::validation("password", "Password must be at least 8 characters long"));
    }
    if password.len() > 128 {
        return Err(Error::validation("password", "Password must be less than 128 characters"));
    }
    Ok(())
}