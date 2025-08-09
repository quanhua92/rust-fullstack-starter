//! Health check response types
//!
//! This module defines the response structures used by health check endpoints.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Overall health status with component details
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthStatus {
    /// Overall status (healthy/unhealthy)
    pub status: String,
    /// Timestamp when the check was performed
    pub timestamp: DateTime<Utc>,
    /// Individual component health checks
    pub checks: HashMap<String, ComponentHealth>,
}

/// Health status of an individual component
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ComponentHealth {
    /// Component status (healthy/unhealthy)
    pub status: String,
    /// Optional human-readable message
    pub message: Option<String>,
    /// Optional additional details
    pub details: Option<serde_json::Value>,
}

/// Basic health response for simple health checks
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Application version
    pub version: String,
    /// Uptime in seconds
    pub uptime: f64,
}

/// Detailed health response with component breakdown
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DetailedHealthResponse {
    /// Overall status
    pub status: String,
    /// Timestamp when the check was performed
    pub timestamp: DateTime<Utc>,
    /// Individual component health checks
    pub checks: HashMap<String, ComponentHealth>,
}
