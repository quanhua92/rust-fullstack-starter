use crate::auth::AuthUser;
use crate::Error;
use crate::rbac::models::{Permission, Resource, UserRole};
use uuid::Uuid;

/// Check if a user has the required permission for a resource
pub fn check_permission(
    user: &AuthUser,
    resource: Resource,
    permission: Permission,
) -> Result<(), Error> {
    if user.role.can_access(resource, permission) {
        Ok(())
    } else {
        Err(Error::Forbidden(format!(
            "Insufficient permissions: {} role cannot {} {}",
            user.role, permission, resource
        )))
    }
}

/// Check if a user has the specified role or higher
pub fn has_role_or_higher(user: &AuthUser, required_role: UserRole) -> bool {
    user.role.has_role_or_higher(required_role)
}

/// Check if a user can access a specific task based on ownership and role
pub fn can_access_task(user: &AuthUser, task_created_by: Option<Uuid>) -> Result<(), Error> {
    match user.role {
        // Admin and Moderator can access any task
        UserRole::Admin | UserRole::Moderator => Ok(()),
        // Users can only access their own tasks
        UserRole::User => {
            if let Some(created_by) = task_created_by {
                if created_by == user.id {
                    Ok(())
                } else {
                    Err(Error::NotFound("Task not found".to_string())) // Prevent enumeration
                }
            } else {
                // System tasks (no created_by) are not accessible to regular users
                Err(Error::NotFound("Task not found".to_string()))
            }
        }
    }
}

/// Check if a user can access another user's profile
/// This function requires the target user's role for complete authorization check
pub fn can_access_user_profile(
    user: &AuthUser,
    target_user_id: Uuid,
    target_user_role: UserRole,
) -> Result<(), Error> {
    match user.role {
        // Admin can access any user profile
        UserRole::Admin => Ok(()),
        // Moderators can access user profiles but NOT admin profiles
        UserRole::Moderator => {
            if target_user_role == UserRole::Admin {
                Err(Error::NotFound("User not found".to_string())) // Hide admin existence from moderators
            } else {
                Ok(())
            }
        }
        // Users can only access their own profile
        UserRole::User => {
            if user.id == target_user_id {
                Ok(())
            } else {
                Err(Error::NotFound("User not found".to_string())) // Prevent enumeration
            }
        }
    }
}

/// Check if a user can perform administrative actions
pub fn require_admin(user: &AuthUser) -> Result<(), Error> {
    match user.role {
        UserRole::Admin => Ok(()),
        _ => Err(Error::Forbidden("Admin access required".to_string())),
    }
}

/// Check if a user can perform moderator-level actions
pub fn require_moderator_or_higher(user: &AuthUser) -> Result<(), Error> {
    if user.role.has_role_or_higher(UserRole::Moderator) {
        Ok(())
    } else {
        Err(Error::Forbidden("Moderator access required".to_string()))
    }
}

/// Check if a user can access a resource based on ownership and role
/// Admin/Moderator can access any resource, users can only access their own
pub fn can_access_own_resource(user: &AuthUser, resource_owner: Uuid) -> Result<(), Error> {
    match user.role {
        // Admin and Moderator can access any resource
        UserRole::Admin | UserRole::Moderator => Ok(()),
        // Users can only access their own resources
        UserRole::User => {
            if resource_owner == user.id {
                Ok(())
            } else {
                Err(Error::NotFound("Resource not found".to_string())) // Prevent enumeration
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_user(role: &str) -> AuthUser {
        AuthUser {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: role.to_string().into(),
        }
    }

    #[test]
    fn test_check_permission() {
        let admin = create_test_user("admin");
        let moderator = create_test_user("moderator");
        let user = create_test_user("user");

        // Admin can do everything
        assert!(check_permission(&admin, Resource::Admin, Permission::Write).is_ok());
        assert!(check_permission(&admin, Resource::Tasks, Permission::Delete).is_ok());

        // Moderator can manage tasks and users but not admin
        assert!(check_permission(&moderator, Resource::Tasks, Permission::Write).is_ok());
        assert!(check_permission(&moderator, Resource::Users, Permission::Read).is_ok());
        assert!(check_permission(&moderator, Resource::Admin, Permission::Read).is_err());

        // User has limited permissions
        assert!(check_permission(&user, Resource::Tasks, Permission::Read).is_ok());
        assert!(check_permission(&user, Resource::Admin, Permission::Read).is_err());
    }

    #[test]
    fn test_can_access_task() {
        let admin = create_test_user("admin");
        let moderator = create_test_user("moderator");
        let user = create_test_user("user");
        let other_user_id = Uuid::new_v4();

        // Admin and moderator can access any task
        assert!(can_access_task(&admin, Some(other_user_id)).is_ok());
        assert!(can_access_task(&moderator, Some(other_user_id)).is_ok());
        assert!(can_access_task(&admin, None).is_ok());

        // User can only access their own tasks
        assert!(can_access_task(&user, Some(user.id)).is_ok());
        assert!(can_access_task(&user, Some(other_user_id)).is_err());
        assert!(can_access_task(&user, None).is_err());
    }

    #[test]
    fn test_role_requirements() {
        let admin = create_test_user("admin");
        let moderator = create_test_user("moderator");
        let user = create_test_user("user");

        // Admin requirements
        assert!(require_admin(&admin).is_ok());
        assert!(require_admin(&moderator).is_err());
        assert!(require_admin(&user).is_err());

        // Moderator requirements
        assert!(require_moderator_or_higher(&admin).is_ok());
        assert!(require_moderator_or_higher(&moderator).is_ok());
        assert!(require_moderator_or_higher(&user).is_err());
    }

    #[test]
    fn test_can_access_own_resource() {
        let admin = create_test_user("admin");
        let moderator = create_test_user("moderator");
        let user = create_test_user("user");
        let other_user_id = Uuid::new_v4();

        // Admin and moderator can access any resource
        assert!(can_access_own_resource(&admin, other_user_id).is_ok());
        assert!(can_access_own_resource(&moderator, other_user_id).is_ok());

        // User can only access their own resource
        assert!(can_access_own_resource(&user, user.id).is_ok());
        assert!(can_access_own_resource(&user, other_user_id).is_err());
    }
}
