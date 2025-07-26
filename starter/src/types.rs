use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type DbPool = sqlx::PgPool;
pub type DbConn = sqlx::pool::PoolConnection<sqlx::Postgres>;