pub mod health;
pub mod types;

// Re-export only public-facing types to avoid namespace pollution
pub use health::{HealthResponse, DetailedHealthResponse, HealthStatus, ComponentHealth};
pub use types::{PaginationParams, PaginatedResponse, PaginationInfo, ApiResponse, ErrorResponse, ErrorDetail};
