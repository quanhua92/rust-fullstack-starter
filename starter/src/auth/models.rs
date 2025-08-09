use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub last_refreshed_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session can be refreshed (not expired and not too recently refreshed)
    pub fn can_refresh(&self, min_refresh_interval_minutes: i64) -> bool {
        if self.is_expired() {
            return false;
        }

        if let Some(last_refreshed_at) = self.last_refreshed_at {
            let min_next_refresh =
                last_refreshed_at + chrono::Duration::minutes(min_refresh_interval_minutes);
            Utc::now() >= min_next_refresh
        } else {
            // Never refreshed before, can refresh
            true
        }
    }

    /// Calculate new expiration time for refresh
    pub fn calculate_refresh_expiration(&self, extend_hours: i64) -> DateTime<Utc> {
        Utc::now() + chrono::Duration::hours(extend_hours)
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    #[schema(example = "johndoe")]
    pub username: Option<String>,
    #[schema(example = "john@example.com")]
    pub email: Option<String>,
    #[schema(example = "securepassword123")]
    pub password: String,
    pub user_agent: Option<String>,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<()> {
        // Ensure exactly one of username or email is provided
        match (&self.username, &self.email) {
            (Some(username), None) => {
                if username.trim().is_empty() {
                    return Err(Error::validation("username", "Username cannot be empty"));
                }
                crate::users::models::validate_username(username)?;
            }
            (None, Some(email)) => {
                if email.trim().is_empty() {
                    return Err(Error::validation("email", "Email cannot be empty"));
                }
                crate::users::models::validate_email(email)?;
            }
            (Some(_), Some(_)) => {
                return Err(Error::validation(
                    "login",
                    "Provide either username or email, not both",
                ));
            }
            (None, None) => {
                return Err(Error::validation(
                    "login",
                    "Either username or email must be provided",
                ));
            }
        }

        if self.password.is_empty() {
            return Err(Error::validation("password", "Password cannot be empty"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "john@example.com")]
    pub email: String,
    #[schema(example = "securepassword123")]
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<()> {
        crate::users::models::validate_username(&self.username)?;
        crate::users::models::validate_email(&self.email)?;
        crate::users::models::validate_password(&self.password)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub user: crate::users::models::UserProfile,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefreshResponse {
    pub expires_at: DateTime<Utc>,
    pub refreshed_at: DateTime<Utc>,
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
