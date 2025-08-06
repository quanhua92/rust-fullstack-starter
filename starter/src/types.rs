use crate::error::Error;
use crate::{config::AppConfig, database::Database};
use std::time::Instant;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = sqlx::PgPool;
pub type DbConn = sqlx::PgConnection;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: AppConfig,
    pub start_time: Instant,
}

// Re-export API types for backward compatibility
pub use crate::api::types::{
    ApiResponse, ErrorDetail, ErrorResponse, PaginatedResponse, PaginationInfo, PaginationParams,
};

// Re-export health types for backward compatibility
pub use crate::api::health::{
    ComponentHealth, DetailedHealthResponse, HealthResponse, HealthStatus,
};
