use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// User roles with hierarchy: User < Moderator < Admin
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
pub enum UserRole {
    /// Regular user - can only access their own resources
    User = 1,
    /// Moderator - can manage user tasks and access user data
    Moderator = 2,
    /// Administrator - full system access
    Admin = 3,
}

impl UserRole {
    /// Check if this role has the same or higher privileges than the required role
    pub fn has_role_or_higher(&self, required_role: UserRole) -> bool {
        *self >= required_role
    }

    /// Get the string representation without allocating
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
        }
    }

    /// Get all roles that this role includes (including itself)
    pub fn included_roles(&self) -> Vec<UserRole> {
        match self {
            UserRole::User => vec![UserRole::User],
            UserRole::Moderator => vec![UserRole::User, UserRole::Moderator],
            UserRole::Admin => vec![UserRole::User, UserRole::Moderator, UserRole::Admin],
        }
    }

    /// Check if this role can perform an action on a resource
    pub fn can_access(&self, resource: Resource, permission: Permission) -> bool {
        match (self, resource, permission) {
            // Admin can do everything
            (UserRole::Admin, _, _) => true,

            // Moderator permissions
            (UserRole::Moderator, Resource::Tasks, Permission::Read) => true,
            (UserRole::Moderator, Resource::Tasks, Permission::Write) => true,
            (UserRole::Moderator, Resource::Tasks, Permission::Delete) => true,
            (UserRole::Moderator, Resource::Users, Permission::Read) => true,
            (UserRole::Moderator, Resource::Users, Permission::Write) => true,
            (UserRole::Moderator, Resource::Users, Permission::Delete) => false, // Moderators can't delete users
            (UserRole::Moderator, Resource::Admin, _) => false,                  // No admin access

            // User permissions - only their own resources
            (UserRole::User, Resource::Tasks, Permission::Read) => true, // Own tasks only (checked elsewhere)
            (UserRole::User, Resource::Tasks, Permission::Write) => true, // Own tasks only
            (UserRole::User, Resource::Tasks, Permission::Delete) => true, // Own tasks only
            (UserRole::User, Resource::Users, Permission::Read) => true, // Own profile only
            (UserRole::User, Resource::Users, Permission::Write) => true, // Own profile only
            (UserRole::User, Resource::Admin, _) => false,
            (UserRole::User, _, Permission::Delete) => false, // Users can't delete others' resources
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for UserRole {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(UserRole::User),
            "moderator" => Ok(UserRole::Moderator),
            "admin" => Ok(UserRole::Admin),
            _ => Err(Error::validation("role", &format!("Invalid role: {s}"))),
        }
    }
}

// SQLx conversion for database storage/retrieval

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        role.to_string()
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        UserRole::from_str(&s).unwrap_or(UserRole::User)
    }
}

// SQLx traits for database encoding/decoding
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserRole {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(UserRole::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for UserRole {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        // Use &str encoder with as_str() - no allocation needed
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for UserRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

/// Resources that can be protected by RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    /// Task-related endpoints
    Tasks,
    /// User-related endpoints  
    Users,
    /// Admin-only endpoints
    Admin,
}

/// Types of permissions that can be granted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Read access to resources
    Read,
    /// Write/modify access to resources
    Write,
    /// Delete access to resources
    Delete,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resource::Tasks => write!(f, "tasks"),
            Resource::Users => write!(f, "users"),
            Resource::Admin => write!(f, "admin"),
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Delete => write!(f, "delete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        assert!(UserRole::Admin > UserRole::Moderator);
        assert!(UserRole::Moderator > UserRole::User);
        assert!(UserRole::Admin > UserRole::User);
    }

    #[test]
    fn test_has_role_or_higher() {
        assert!(UserRole::Admin.has_role_or_higher(UserRole::User));
        assert!(UserRole::Admin.has_role_or_higher(UserRole::Moderator));
        assert!(UserRole::Admin.has_role_or_higher(UserRole::Admin));

        assert!(UserRole::Moderator.has_role_or_higher(UserRole::User));
        assert!(UserRole::Moderator.has_role_or_higher(UserRole::Moderator));
        assert!(!UserRole::Moderator.has_role_or_higher(UserRole::Admin));

        assert!(UserRole::User.has_role_or_higher(UserRole::User));
        assert!(!UserRole::User.has_role_or_higher(UserRole::Moderator));
        assert!(!UserRole::User.has_role_or_higher(UserRole::Admin));
    }

    #[test]
    fn test_role_from_string() {
        assert_eq!(UserRole::from_str("user").unwrap(), UserRole::User);
        assert_eq!(
            UserRole::from_str("moderator").unwrap(),
            UserRole::Moderator
        );
        assert_eq!(UserRole::from_str("admin").unwrap(), UserRole::Admin);
        assert_eq!(UserRole::from_str("USER").unwrap(), UserRole::User);
        assert!(UserRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_role_to_string() {
        assert_eq!(UserRole::User.to_string(), "user");
        assert_eq!(UserRole::Moderator.to_string(), "moderator");
        assert_eq!(UserRole::Admin.to_string(), "admin");
    }

    #[test]
    fn test_permissions() {
        // Admin can do everything
        assert!(UserRole::Admin.can_access(Resource::Tasks, Permission::Read));
        assert!(UserRole::Admin.can_access(Resource::Admin, Permission::Write));

        // Moderator can manage tasks and users but not admin
        assert!(UserRole::Moderator.can_access(Resource::Tasks, Permission::Write));
        assert!(UserRole::Moderator.can_access(Resource::Users, Permission::Read));
        assert!(!UserRole::Moderator.can_access(Resource::Admin, Permission::Read));

        // User has limited permissions
        assert!(UserRole::User.can_access(Resource::Tasks, Permission::Read));
        assert!(!UserRole::User.can_access(Resource::Admin, Permission::Read));
    }
}
