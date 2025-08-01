use crate::error::Error;
use crate::rbac::UserRole;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn is_moderator_or_higher(&self) -> bool {
        self.role.has_role_or_higher(UserRole::Moderator)
    }

    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role,
            is_active: self.is_active,
            email_verified: self.email_verified,
            created_at: self.created_at,
            last_login_at: self.last_login_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
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

// New request/response models for user management endpoints

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

impl UpdateProfileRequest {
    pub fn validate(&self) -> Result<()> {
        if let Some(ref username) = self.username {
            validate_username(username)?;
        }
        if let Some(ref email) = self.email {
            validate_email(email)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<()> {
        validate_password(&self.new_password)?;
        if self.current_password == self.new_password {
            return Err(Error::validation(
                "new_password",
                "New password must be different from current password",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DeleteAccountRequest {
    pub password: String,
    pub confirmation: String,
}

impl DeleteAccountRequest {
    pub fn validate(&self) -> Result<()> {
        if self.confirmation != "DELETE" {
            return Err(Error::validation(
                "confirmation",
                "Must provide 'DELETE' as confirmation",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

impl UpdateUserProfileRequest {
    pub fn validate(&self) -> Result<()> {
        if let Some(ref username) = self.username {
            validate_username(username)?;
        }
        if let Some(ref email) = self.email {
            validate_email(email)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserStatusRequest {
    pub is_active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRoleRequest {
    pub role: UserRole,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub new_password: String,
    pub require_change: Option<bool>,
    pub reason: Option<String>,
}

impl ResetPasswordRequest {
    pub fn validate(&self) -> Result<()> {
        validate_password(&self.new_password)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DeleteUserRequest {
    pub reason: Option<String>,
    pub hard_delete: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserStats {
    pub total_users: i64,
    pub active_users: i64,
    pub inactive_users: i64,
    pub email_verified: i64,
    pub email_unverified: i64,
    pub by_role: UserRoleStats,
    pub recent_registrations: RecentRegistrations,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserRoleStats {
    pub user: i64,
    pub moderator: i64,
    pub admin: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RecentRegistrations {
    pub last_24h: i64,
    pub last_7d: i64,
    pub last_30d: i64,
}
