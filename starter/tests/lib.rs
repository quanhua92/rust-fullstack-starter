//! Integration Tests for Rust React Starter
//! 
//! This test suite follows integration testing best practices:
//! - Each test gets an isolated database
//! - Tests use real HTTP requests via TestApp
//! - Template database pattern for fast setup
//! - Comprehensive test data factories

pub mod helpers;
pub mod auth;
pub mod users;
pub mod tasks;
pub mod health;
pub mod api;

// Re-export common test utilities
pub use helpers::*;