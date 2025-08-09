//! Health check system
//!
//! This module provides comprehensive health checking functionality,
//! including handlers for different types of health checks (liveness,
//! readiness, startup) and the underlying health check implementations.

pub mod checks;
pub mod handlers;
pub mod types;

// Re-export commonly used types
pub use handlers::{detailed_health, health, health_live, health_ready, health_startup};
pub use types::{ComponentHealth, DetailedHealthResponse, HealthResponse, HealthStatus};
