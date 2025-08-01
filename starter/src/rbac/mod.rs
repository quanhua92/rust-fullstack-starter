pub mod middleware;
pub mod models;
pub mod services;

// Re-export main types for convenience
pub use middleware::{require_permission, require_role, require_role_or_higher};
pub use models::{Permission, Resource, UserRole};
pub use services::{check_permission, has_role_or_higher};
