//! Public type re-exports for backward compatibility
//!
//! This module maintains backward compatibility by re-exporting types
//! from their new organized locations. New code should import directly
//! from the specific modules instead of using these re-exports.

// Re-export core types (most commonly used)
pub use crate::core::{AppState, DbConn, DbPool, Result};

// Re-export API types for backward compatibility
pub use crate::api::{
    ApiResponse, ErrorDetail, ErrorResponse, PaginatedResponse, PaginationInfo, PaginationParams,
};

// Re-export health types for backward compatibility
pub use crate::health::{ComponentHealth, DetailedHealthResponse, HealthResponse, HealthStatus};
