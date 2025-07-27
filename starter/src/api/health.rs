use crate::types::{AppState, ApiResponse, HealthStatus, ComponentHealth};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use std::collections::HashMap;

/// Basic health check endpoint
pub async fn health_check() -> impl IntoResponse {
    let health_data = serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    });
    Json(ApiResponse::success(health_data))
}

/// Comprehensive health check with dependencies
pub async fn health_detailed(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = HashMap::new();

    // Check database connection
    let db_health = check_database(&state).await;
    checks.insert("database".to_string(), db_health);

    // Determine overall status
    let all_healthy = checks.values().all(|check| check.status == "healthy");
    let overall_status = if all_healthy { "healthy" } else { "unhealthy" };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        checks,
    };

    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(ApiResponse::success(health_status)))
}

/// Check database connectivity
async fn check_database(state: &AppState) -> ComponentHealth {
    match sqlx::query("SELECT 1").fetch_one(&state.database.pool).await {
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