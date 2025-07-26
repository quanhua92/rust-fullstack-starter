use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{types::Result, error::Error};

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
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
    pub user: crate::users::models::UserProfile,
}