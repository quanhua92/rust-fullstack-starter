//! API response types and utilities
//!
//! This module provides standardized response types for the API layer,
//! ensuring consistent response formats across all endpoints.

/// Standard API response wrapper
///
/// All successful API responses should use this structure to ensure
/// consistency across the API surface.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Response data (None for errors)
    pub data: Option<T>,
    /// Optional message for additional context
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    /// Create a successful response with data and message
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
        }
    }
}

/// Standard error response structure
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error details
    pub error: ErrorDetail,
}

/// Error detail information
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorDetail {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
}
