//! __MODULE_STRUCT__ module with advanced features
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

pub use api::__MODULE_NAME_PLURAL___routes;
pub use models::*;
pub use services::*;