//! API layer types and utilities
//!
//! This module contains types and utilities specific to the HTTP API layer,
//! including response formats, pagination, and request handling utilities.

pub mod pagination;
pub mod response;

// Re-export commonly used API types
pub use pagination::{PaginatedResponse, PaginationInfo, PaginationParams};
pub use response::{ApiResponse, ErrorDetail, ErrorResponse};
