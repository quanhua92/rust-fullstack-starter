pub mod config;
pub mod error;
pub mod types;
pub mod database;
pub mod models;
pub mod api;
pub mod server;

// Re-export common types
pub use config::AppConfig;
pub use error::Error;
pub use types::{Result, DbPool, DbConn};
pub use database::Database;