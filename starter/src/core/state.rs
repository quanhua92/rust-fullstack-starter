//! Application state management
//!
//! This module defines the application state that is shared across
//! all request handlers and contains configuration, database connections,
//! and other global application context.

use crate::core::{config::AppConfig, database::Database};
use std::time::Instant;

/// Application state shared across all handlers
///
/// This structure contains all the shared state that handlers need access to,
/// including database connections, configuration, and runtime metrics.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool and utilities
    pub database: Database,
    /// Application configuration
    pub config: AppConfig,
    /// Application start time for uptime calculations
    pub start_time: Instant,
}
