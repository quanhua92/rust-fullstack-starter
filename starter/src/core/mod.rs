//! Core application types and functionality
//!
//! This module contains the fundamental types and structures that form
//! the backbone of the application, including application state, database
//! types, and result types.

pub mod state;
pub mod types;

// Re-export commonly used types
pub use state::AppState;
pub use types::{DbConn, DbPool, Result};
