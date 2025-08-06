pub mod health;
pub mod types;

// Re-export only public-facing types to avoid namespace pollution
pub use health::{ComponentHealth, DetailedHealthResponse, HealthResponse, HealthStatus};
pub use types::{
    ApiResponse, ErrorDetail, ErrorResponse, PaginatedResponse, PaginationInfo, PaginationParams,
};
