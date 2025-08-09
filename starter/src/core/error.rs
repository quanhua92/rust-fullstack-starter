use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    // Authentication errors
    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token has expired")]
    TokenExpired,

    // Validation errors
    #[error("Validation failed for {field}: {message}")]
    ValidationError { field: String, message: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // Business logic errors
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Username already exists")]
    UsernameAlreadyExists,

    #[error("Conflict: {0}")]
    Conflict(String),

    // System errors
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Service unavailable")]
    ServiceUnavailable,

    // Task/Worker errors
    #[error("Task not found")]
    TaskNotFound,

    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Worker error: {0}")]
    WorkerError(String),
}

impl Error {
    pub fn validation(field: &str, message: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    pub fn conflict(message: &str) -> Self {
        Self::Conflict(message.to_string())
    }

    pub fn internal(message: &str) -> Self {
        Self::Internal(message.to_string())
    }
}

// Custom From implementations for specific error handling
impl From<argon2::password_hash::Error> for Error {
    fn from(_: argon2::password_hash::Error) -> Self {
        Error::Internal("Password hashing error".to_string())
    }
}

// Manual conversion for specific constraint handling to avoid conflicts with derive
impl Error {
    pub fn from_sqlx(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => Error::UserNotFound,
            sqlx::Error::Database(db_err) => {
                // Handle PostgreSQL specific constraint violations
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => {
                            // unique_violation
                            if db_err.constraint().is_some_and(|c| c.contains("email")) {
                                return Error::EmailAlreadyExists;
                            } else if db_err.constraint().is_some_and(|c| c.contains("username")) {
                                return Error::UsernameAlreadyExists;
                            }
                            Error::UserAlreadyExists
                        }
                        _ => Error::Database(err),
                    }
                } else {
                    Error::Database(err)
                }
            }
            _ => Error::Database(err),
        }
    }
}

// Axum response conversion
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message, error_code) = match &self {
            Error::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred".to_string(),
                "DATABASE_ERROR",
            ),
            Error::Migration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Migration error".to_string(),
                "MIGRATION_ERROR",
            ),
            Error::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized access".to_string(),
                "UNAUTHORIZED",
            ),
            Error::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), "FORBIDDEN"),
            Error::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid credentials".to_string(),
                "INVALID_CREDENTIALS",
            ),
            Error::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "Token has expired".to_string(),
                "TOKEN_EXPIRED",
            ),
            Error::ValidationError { field, message } => (
                StatusCode::BAD_REQUEST,
                format!("Validation failed for {field}: {message}"),
                "VALIDATION_FAILED",
            ),
            Error::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone(), "INVALID_INPUT"),
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), "NOT_FOUND"),
            Error::UserNotFound => (
                StatusCode::NOT_FOUND,
                "User not found".to_string(),
                "USER_NOT_FOUND",
            ),
            Error::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "User already exists".to_string(),
                "USER_ALREADY_EXISTS",
            ),
            Error::EmailAlreadyExists => (
                StatusCode::CONFLICT,
                "Email already exists".to_string(),
                "EMAIL_ALREADY_EXISTS",
            ),
            Error::UsernameAlreadyExists => (
                StatusCode::CONFLICT,
                "Username already exists".to_string(),
                "USERNAME_ALREADY_EXISTS",
            ),
            Error::Conflict(msg) => (StatusCode::CONFLICT, msg.clone(), "CONFLICT"),
            Error::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                "INTERNAL_ERROR",
            ),
            Error::ConfigurationError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
                "CONFIGURATION_ERROR",
            ),
            Error::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service unavailable".to_string(),
                "SERVICE_UNAVAILABLE",
            ),
            Error::TaskNotFound => (
                StatusCode::NOT_FOUND,
                "Task not found".to_string(),
                "TASK_NOT_FOUND",
            ),
            Error::TaskExecutionFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Task execution failed: {msg}"),
                "TASK_EXECUTION_FAILED",
            ),
            Error::WorkerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Worker error: {msg}"),
                "WORKER_ERROR",
            ),
        };

        // Log internal errors for debugging
        if matches!(
            self,
            Error::Database(_) | Error::Internal(_) | Error::ConfigurationError(_)
        ) {
            tracing::error!("Internal error: {}", self);
        }

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": error_message,
            }
        }));

        (status, body).into_response()
    }
}
