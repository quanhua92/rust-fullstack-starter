//! Pagination types and utilities
//!
//! This module provides reusable pagination functionality that can be
//! used across different API endpoints.

/// Request parameters for pagination
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Number of items per page
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

impl PaginationParams {
    /// Get the page number (defaults to 1)
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    /// Get the limit (defaults to 20)
    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20)
    }

    /// Calculate the offset for database queries
    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.limit()
    }
}

/// Paginated response wrapper
#[derive(Debug, serde::Serialize)]
pub struct PaginatedResponse<T> {
    /// The actual data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationInfo,
}

/// Pagination metadata
#[derive(Debug, serde::Serialize)]
pub struct PaginationInfo {
    /// Current page number
    pub page: u32,
    /// Items per page
    pub limit: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub total_pages: u32,
}

impl PaginationInfo {
    /// Create pagination info from parameters and total count
    pub fn new(params: &PaginationParams, total: u64) -> Self {
        let limit = params.limit();
        let total_pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as u32
        } else {
            1
        };

        Self {
            page: params.page(),
            limit,
            total,
            total_pages,
        }
    }
}
