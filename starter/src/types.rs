use crate::error::Error;
use crate::{config::AppConfig, database::Database};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = sqlx::PgPool;
pub type DbConn = sqlx::pool::PoolConnection<sqlx::Postgres>;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: AppConfig,
}

// Request/Response types for API layer
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
}

// API Response wrapper
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
        }
    }
}

// Health check types
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub checks: HashMap<String, ComponentHealth>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

// Basic health response
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: f64,
}

// Detailed health response
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub checks: HashMap<String, ComponentHealth>,
}

// Error response structure
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
