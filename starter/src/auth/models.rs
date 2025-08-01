use crate::{error::Error, types::Result};
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
    pub last_activity_at: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
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
