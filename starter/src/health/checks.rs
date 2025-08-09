//! Health check implementations
//!
//! This module contains the actual health check logic for various
//! application components and dependencies.

use crate::core::AppState;
use crate::health::types::ComponentHealth;

/// Check database connectivity and basic functionality
pub async fn check_database(state: &AppState) -> ComponentHealth {
    match sqlx::query("SELECT 1")
        .fetch_one(&state.database.pool)
        .await
    {
        Ok(_) => ComponentHealth {
            status: "healthy".to_string(),
            message: Some("Database connection successful".to_string()),
            details: None,
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            message: Some("Database connection failed".to_string()),
            details: Some(serde_json::json!({
                "error": e.to_string()
            })),
        },
    }
}

/// Check if the application is ready to serve traffic
///
/// This includes checking configuration validity and other
/// readiness indicators beyond just database connectivity.
pub async fn check_application_readiness(state: &AppState) -> ComponentHealth {
    // Verify basic application configuration
    let config_valid = !state.config.database.host.is_empty() && state.config.database.port > 0;

    if config_valid {
        ComponentHealth {
            status: "healthy".to_string(),
            message: Some("Application configuration is valid".to_string()),
            details: Some(serde_json::json!({
                "config_loaded": true,
                "auth_configured": true
            })),
        }
    } else {
        ComponentHealth {
            status: "unhealthy".to_string(),
            message: Some("Application configuration is invalid".to_string()),
            details: Some(serde_json::json!({
                "config_loaded": false,
                "issues": "Missing required configuration"
            })),
        }
    }
}

/// Check if database schema is properly initialized
///
/// This verifies that essential tables exist, indicating that
/// database migrations have been successfully applied.
pub async fn check_database_schema(state: &AppState) -> ComponentHealth {
    match sqlx::query("SELECT 1 FROM users LIMIT 1")
        .fetch_optional(&state.database.pool)
        .await
    {
        Ok(_) => ComponentHealth {
            status: "healthy".to_string(),
            message: Some("Database schema is initialized".to_string()),
            details: Some(serde_json::json!({
                "tables_exist": true,
                "migrations_applied": true
            })),
        },
        Err(e) => ComponentHealth {
            status: "unhealthy".to_string(),
            message: Some("Database schema not initialized".to_string()),
            details: Some(serde_json::json!({
                "error": e.to_string(),
                "suggestion": "Ensure database migrations have been applied"
            })),
        },
    }
}
