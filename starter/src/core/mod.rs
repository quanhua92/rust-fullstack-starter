//! Core application infrastructure
//!
//! This module contains the fundamental infrastructure components that form
//! the backbone of the application, including configuration, database,
//! error handling, application state, server setup, and OpenAPI documentation.

pub mod config;
pub mod database;
pub mod error;
pub mod openapi;
pub mod server;
pub mod state;
pub mod types;

// Re-export commonly used types for convenience
pub use config::AppConfig;
pub use database::Database;
pub use error::Error;
pub use server::{create_router, start_server};
pub use state::AppState;
pub use types::{DbConn, DbPool, Result};
