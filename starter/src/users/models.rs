use crate::error::Error;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
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

pub fn validate_email(email: &str) -> Result<()> {
    if email.len() < 3 || !email.contains('@') || email.len() > 254 {
        return Err(Error::validation("email", "Invalid email format"));
    }
    Ok(())
}

pub fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 50 {
        return Err(Error::validation(
            "username",
            "Username must be between 3 and 50 characters",
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::validation(
            "username",
            "Username can only contain letters, numbers, underscores, and hyphens",
        ));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(Error::validation(
            "password",
            "Password must be at least 8 characters long",
        ));
    }
    if password.len() > 128 {
        return Err(Error::validation(
            "password",
            "Password must be less than 128 characters",
        ));
    }
    Ok(())
}
