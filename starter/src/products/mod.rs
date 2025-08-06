//! Products module with advanced features
//!
//! This module provides comprehensive CRUD operations with:
//! - Advanced filtering and searching
//! - Cursor-based and offset-based pagination
//! - Bulk operations (create, update, delete)
//! - Status management and priority handling
//! - Metadata support with JSON storage

pub mod api;
pub mod models;
pub mod services;

pub use api::products_routes;
pub use models::*;
pub use services::*;
