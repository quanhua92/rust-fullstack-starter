//! Integration Tests for Rust React Starter
//!
//! This test suite follows integration testing best practices:
//! - Each test gets an isolated database
//! - Tests use real HTTP requests via TestApp
//! - Template database pattern for fast setup
//! - Comprehensive test data factories

pub mod api;
pub mod auth;
pub mod cli;
pub mod health;
pub mod helpers;
pub mod middleware;
pub mod monitoring;
pub mod tasks;
pub mod users;

// Re-export common test utilities
pub use helpers::*;
