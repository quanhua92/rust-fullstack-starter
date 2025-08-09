//! Core type definitions for the application
//!
//! This module defines fundamental types used throughout the application,
//! including database connections, result types, and other core abstractions.

use crate::error::Error;

/// Application result type with our custom error
pub type Result<T> = std::result::Result<T, Error>;

/// Database connection pool type
pub type DbPool = sqlx::PgPool;

/// Database connection type - supports both pool connections and transactions
/// Use `conn.as_mut()` for pool connections, `tx.as_mut()` for transactions
pub type DbConn = sqlx::PgConnection;
