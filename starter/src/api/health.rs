use crate::types::{
    ApiResponse, AppState, ComponentHealth, DetailedHealthResponse, ErrorResponse, HealthResponse,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use std::collections::HashMap;

/// Basic health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    summary = "Basic health check",
    description = "Returns basic application health status with version and uptime",
    responses(
        (status = 200, description = "Application is healthy", body = ApiResponse<HealthResponse>),
        (status = 503, description = "Application is unhealthy", body = ErrorResponse)
    )
)]
pub async fn health() -> impl IntoResponse {
    let health_data = serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
        "documentation": {
            "openapi_json": "/api-docs/openapi.json",
            "api_docs": "/api-docs"
        }
    });
    Json(ApiResponse::success(health_data))
}

/// Comprehensive health check with dependencies
#[utoipa::path(
    get,
    path = "/health/detailed",
    tag = "Health",
    summary = "Detailed health check",
    description = "Returns comprehensive health status including database and dependencies",
    responses(
        (status = 200, description = "All services healthy", body = ApiResponse<DetailedHealthResponse>),
        (status = 503, description = "One or more services unhealthy", body = ApiResponse<DetailedHealthResponse>)
    )
)]
pub async fn detailed_health(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = HashMap::new();

    // Check database connection
    let db_health = check_database(&state).await;
    checks.insert("database".to_string(), db_health);

    // Determine overall status
    let all_healthy = checks.values().all(|check| check.status == "healthy");
    let overall_status = if all_healthy { "healthy" } else { "unhealthy" };

    let health_status = DetailedHealthResponse {
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

/// Kubernetes liveness probe - checks if the application is alive
/// Returns 200 if the basic application is running (minimal checks)
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Health",
    summary = "Liveness probe",
    description = "Kubernetes liveness probe endpoint. Returns 200 if application is running.",
    responses(
        (status = 200, description = "Application is alive", body = ApiResponse<serde_json::Value>)
    )
)]
pub async fn health_live() -> impl IntoResponse {
    // Basic liveness check - just confirm the application is running
    // This should be very lightweight and fast
    Json(ApiResponse::success(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now(),
        "probe": "liveness"
    })))
}

/// Kubernetes readiness probe - checks if the application is ready to serve traffic
/// Returns 200 only if all critical dependencies are available
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "Health",
    summary = "Readiness probe",
    description = "Kubernetes readiness probe endpoint. Returns 200 only if all dependencies are ready.",
    responses(
        (status = 200, description = "Application is ready", body = ApiResponse<serde_json::Value>),
        (status = 503, description = "Application is not ready", body = ApiResponse<serde_json::Value>)
    )
)]
pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = HashMap::new();
    let mut all_ready = true;

    // Check database readiness
    let db_health = check_database(&state).await;
    let db_ready = db_health.status == "healthy";
    checks.insert("database".to_string(), db_health);
    all_ready = all_ready && db_ready;

    // Check application readiness (migrations, etc.)
    let app_health = check_application_readiness(&state).await;
    let app_ready = app_health.status == "healthy";
    checks.insert("application".to_string(), app_health);
    all_ready = all_ready && app_ready;

    let status_code = if all_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let readiness_status = serde_json::json!({
        "status": if all_ready { "ready" } else { "not_ready" },
        "timestamp": Utc::now(),
        "probe": "readiness",
        "checks": checks
    });

    (status_code, Json(ApiResponse::success(readiness_status)))
}

/// Kubernetes startup probe - checks if the application has started successfully
/// Returns 200 when the application has completed initialization
#[utoipa::path(
    get,
    path = "/health/startup",
    tag = "Health",
    summary = "Startup probe",
    description = "Kubernetes startup probe endpoint. Returns 200 when initialization is complete.",
    responses(
        (status = 200, description = "Application has started", body = ApiResponse<serde_json::Value>),
        (status = 503, description = "Application is still starting", body = ApiResponse<serde_json::Value>)
    )
)]
pub async fn health_startup(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = HashMap::new();
    let mut startup_complete = true;

    // Check database connectivity
    let db_health = check_database(&state).await;
    let db_connected = db_health.status == "healthy";
    checks.insert("database".to_string(), db_health);
    startup_complete = startup_complete && db_connected;

    // Check database schema (migrations applied)
    let schema_health = check_database_schema(&state).await;
    let schema_ready = schema_health.status == "healthy";
    checks.insert("schema".to_string(), schema_health);
    startup_complete = startup_complete && schema_ready;

    let status_code = if startup_complete {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let startup_status = serde_json::json!({
        "status": if startup_complete { "started" } else { "starting" },
        "timestamp": Utc::now(),
        "probe": "startup",
        "checks": checks
    });

    (status_code, Json(ApiResponse::success(startup_status)))
}

/// Check database connectivity
async fn check_database(state: &AppState) -> ComponentHealth {
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

/// Check application readiness (beyond just database)
async fn check_application_readiness(state: &AppState) -> ComponentHealth {
    // Check if the application has completed all initialization
    // This could include checking configuration, external services, etc.

    // For now, just verify basic application state
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
async fn check_database_schema(state: &AppState) -> ComponentHealth {
    // Check if essential tables exist (indicating migrations have run)
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
