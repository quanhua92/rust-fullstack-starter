//! Health check HTTP handlers
//!
//! This module contains the Axum handlers for various health check endpoints,
//! including basic health, detailed health, and Kubernetes probe endpoints.

use crate::api::response::ApiResponse;
use crate::core::AppState;
use crate::health::{checks, types::*};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use std::collections::HashMap;

/// Basic health check endpoint
///
/// Returns basic application health status with version and uptime.
/// This is a lightweight endpoint suitable for basic monitoring.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    summary = "Basic health check",
    description = "Returns basic application health status with version and uptime",
    responses(
        (status = 200, description = "Application is healthy", body = ApiResponse<HealthResponse>),
        (status = 503, description = "Application is unhealthy")
    )
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let uptime_seconds = state.start_time.elapsed().as_secs_f64();

    let health_response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: uptime_seconds,
        documentation: DocumentationLinks {
            openapi_json: "/api-docs/openapi.json".to_string(),
            api_docs: "/api-docs".to_string(),
        },
    };

    Json(ApiResponse::success(health_response))
}

/// Comprehensive health check with dependencies
///
/// Returns detailed health status including database and other dependencies.
/// Use this endpoint when you need detailed diagnostics.
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
    let mut checks_map = HashMap::new();

    // Check database connection
    let db_health = checks::check_database(&state).await;
    checks_map.insert("database".to_string(), db_health);

    // Determine overall status
    let all_healthy = checks_map.values().all(|check| check.status == "healthy");
    let overall_status = if all_healthy { "healthy" } else { "unhealthy" };

    let health_status = DetailedHealthResponse {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        checks: checks_map,
    };

    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(ApiResponse::success(health_status)))
}

/// Kubernetes liveness probe
///
/// Checks if the application is alive. Returns 200 if the basic
/// application is running. This should be very lightweight and fast.
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
    Json(ApiResponse::success(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now(),
        "probe": "liveness"
    })))
}

/// Kubernetes readiness probe
///
/// Checks if the application is ready to serve traffic.
/// Returns 200 only if all critical dependencies are available.
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
    let mut checks_map = HashMap::new();
    let mut all_ready = true;

    // Check database readiness
    let db_health = checks::check_database(&state).await;
    let db_ready = db_health.status == "healthy";
    checks_map.insert("database".to_string(), db_health);
    all_ready = all_ready && db_ready;

    // Check application readiness (migrations, etc.)
    let app_health = checks::check_application_readiness(&state).await;
    let app_ready = app_health.status == "healthy";
    checks_map.insert("application".to_string(), app_health);
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
        "checks": checks_map
    });

    (status_code, Json(ApiResponse::success(readiness_status)))
}

/// Kubernetes startup probe
///
/// Checks if the application has started successfully.
/// Returns 200 when the application has completed initialization.
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
    let mut checks_map = HashMap::new();
    let mut startup_complete = true;

    // Check database connectivity
    let db_health = checks::check_database(&state).await;
    let db_connected = db_health.status == "healthy";
    checks_map.insert("database".to_string(), db_health);
    startup_complete = startup_complete && db_connected;

    // Check database schema (migrations applied)
    let schema_health = checks::check_database_schema(&state).await;
    let schema_ready = schema_health.status == "healthy";
    checks_map.insert("schema".to_string(), schema_health);
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
        "checks": checks_map
    });

    (status_code, Json(ApiResponse::success(startup_status)))
}
