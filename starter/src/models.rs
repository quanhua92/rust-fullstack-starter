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
