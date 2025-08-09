use crate::Error;
use crate::Result;
use crate::rbac::UserRole;
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
    // Basic length checks
    if email.len() < 3 || email.len() > 254 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Check for exactly one @ symbol
    let at_count = email.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Split into local and domain parts
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    // Validate local part
    if local_part.is_empty() || local_part.len() > 64 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Local part cannot start or end with dots
    if local_part.starts_with('.') || local_part.ends_with('.') {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // No consecutive dots in local part
    if local_part.contains("..") {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Check for valid characters in local part (simplified RFC compliance)
    for c in local_part.chars() {
        if !c.is_alphanumeric() && !matches!(c, '.' | '_' | '-' | '+') {
            return Err(Error::validation("email", "Invalid email format"));
        }
    }

    // Validate domain part
    if domain_part.is_empty() || domain_part.len() > 253 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Domain cannot start or end with hyphens or dots
    if domain_part.starts_with('-')
        || domain_part.ends_with('-')
        || domain_part.starts_with('.')
        || domain_part.ends_with('.')
    {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Check for valid domain format (must contain at least one dot for TLD)
    if !domain_part.contains('.') {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Split domain into labels
    let domain_labels: Vec<&str> = domain_part.split('.').collect();
    if domain_labels.len() < 2 {
        return Err(Error::validation("email", "Invalid email format"));
    }

    // Validate each domain label
    for label in &domain_labels {
        if label.is_empty() || label.len() > 63 {
            return Err(Error::validation("email", "Invalid email format"));
        }

        // Label cannot start or end with hyphen
        if label.starts_with('-') || label.ends_with('-') {
            return Err(Error::validation("email", "Invalid email format"));
        }

        // Check for valid characters in domain labels
        for c in label.chars() {
            if !c.is_alphanumeric() && c != '-' {
                return Err(Error::validation("email", "Invalid email format"));
            }
        }
    }

    // TLD (last label) should be at least 2 characters and only letters
    let tld = domain_labels.last().unwrap();
    if tld.len() < 2 || !tld.chars().all(|c| c.is_alphabetic()) {
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

    // Check password strength requirements
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| {
        matches!(
            c,
            '!' | '@'
                | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '('
                | ')'
                | '_'
                | '+'
                | '-'
                | '='
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | ';'
                | ':'
                | ','
                | '.'
                | '<'
                | '>'
                | '?'
        )
    });

    let strength_count = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if strength_count < 3 {
        return Err(Error::validation(
            "password",
            "Password must contain at least 3 of: uppercase letters, lowercase letters, numbers, special characters",
        ));
    }

    // Check for common weak passwords
    if is_common_password(password) {
        return Err(Error::validation(
            "password",
            "Password is too common. Please choose a stronger password",
        ));
    }

    Ok(())
}

/// Check if password is in a list of common weak passwords
/// Uses constant-time comparison to prevent timing attacks
/// Checks both exact match and lowercase version to prevent case bypass
fn is_common_password(password: &str) -> bool {
    static COMMON_PASSWORDS: once_cell::sync::Lazy<Vec<&'static str>> =
        once_cell::sync::Lazy::new(|| {
            vec![
                "password",
                "123456",
                "password123",
                "admin",
                "qwerty",
                "letmein",
                "welcome",
                "monkey",
                "1234567890",
                "abc123",
                "password1",
                "123456789",
                "welcome123",
                "admin123",
                "qwerty123",
                "12345678",
                "111111",
                "123123",
                "1234567",
                "1q2w3e4r",
                "asdfgh",
                "zxcvbn",
                "qwertyui",
                "000000",
                "1234",
                "iloveyou",
                "dragon",
                "sunshine",
                "princess",
                "azerty",
                "trustno1",
                "123qwe",
                // Additional common variants
                "password123!",
                "admin123!",
                "qwerty123!",
                "welcome123!",
                "password1!",
                "admin1",
                "qwerty1",
                "welcome1",
            ]
        });

    // Check both the password as-is and its lowercase version
    // Use constant-time comparison to prevent timing attacks
    let password_lower = password.to_lowercase();

    let mut found = false;
    for common_password in COMMON_PASSWORDS.iter() {
        // Check exact match
        if constant_time_eq(password.as_bytes(), common_password.as_bytes()) {
            found = true;
            // Continue the loop to maintain constant time
        }
        // Check lowercase match
        if constant_time_eq(password_lower.as_bytes(), common_password.as_bytes()) {
            found = true;
            // Continue the loop to maintain constant time
        }
    }
    found
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        // Even for different lengths, we should do some work to maintain timing
        let mut _result = 0u8;
        for (i, &byte_a) in a.iter().enumerate() {
            let byte_b = b.get(i % b.len().max(1)).copied().unwrap_or(0);
            _result |= byte_a ^ byte_b;
        }
        // Ensure we've done similar work regardless of length difference
        for _ in 0..b.len().saturating_sub(a.len()) {
            _result |= 0;
        }
        false
    } else {
        let mut result = 0u8;
        for (byte_a, byte_b) in a.iter().zip(b.iter()) {
            result |= byte_a ^ byte_b;
        }
        result == 0
    }
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
