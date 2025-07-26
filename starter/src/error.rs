use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Authorization failed")]
    AuthorizationFailed,
    
    #[error("Validation error: {field}: {message}")]
    ValidationError { field: String, message: String },
    
    #[error("Not found")]
    NotFound,
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
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