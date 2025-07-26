use serde::{Deserialize, Serialize};
use secrecy::SecretString;
use std::time::Duration;
use crate::types::Result;
use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub worker: WorkerConfig,
    #[serde(skip)]
    pub initial_admin_password: Option<SecretString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub session_duration_hours: u64,
    pub cleanup_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct WorkerConfig {
    pub concurrency: usize,
    pub poll_interval_secs: u64,
    pub max_retries: u32,
    pub retry_backoff_base_secs: u64,
}

impl AppConfig {
    /// Load configuration from environment variables only
    pub fn load() -> Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            // Start with defaults
            .add_source(config::Config::try_from(&Self::default())?)
            // Override with environment variables using __ separator
            .add_source(
                config::Environment::with_prefix("STARTER")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;

        let mut app_config: AppConfig = config.try_deserialize()
            .map_err(|e| Error::ConfigurationError(format!("Failed to parse config: {}", e)))?;

        // Handle initial admin password separately since it's skipped in serde
        if let Ok(password) = std::env::var("STARTER__INITIAL_ADMIN_PASSWORD") {
            app_config.initial_admin_password = Some(SecretString::new(password.into()));
        }

        app_config.validate()?;
        Ok(app_config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate database components
        if self.database.user.is_empty() {
            return Err(Error::ConfigurationError("Database user cannot be empty".to_string()));
        }
        if self.database.password.is_empty() {
            return Err(Error::ConfigurationError("Database password cannot be empty".to_string()));
        }
        if self.database.host.is_empty() {
            return Err(Error::ConfigurationError("Database host cannot be empty".to_string()));
        }
        if self.database.database.is_empty() {
            return Err(Error::ConfigurationError("Database name cannot be empty".to_string()));
        }

        // Validate server port
        if self.server.port == 0 {
            return Err(Error::ConfigurationError("Server port must be specified".to_string()));
        }

        // Validate connection pool settings
        if self.database.max_connections < self.database.min_connections {
            return Err(Error::ConfigurationError("max_connections must be >= min_connections".to_string()));
        }

        if self.database.min_connections == 0 {
            return Err(Error::ConfigurationError("min_connections must be > 0".to_string()));
        }

        // Validate worker settings
        if self.worker.concurrency == 0 {
            return Err(Error::ConfigurationError("Worker concurrency must be > 0".to_string()));
        }

        Ok(())
    }

    /// Get server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get database URL as SecretString
    pub fn database_url(&self) -> SecretString {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        );
        SecretString::from(url)
    }
    
    /// Get database URL as plain string (for internal use)
    pub fn database_url_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }

    /// Get server request timeout
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_secs)
    }

    /// Get database connection timeout
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.database.connect_timeout_secs)
    }

    /// Get database idle timeout
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.database.idle_timeout_secs)
    }

    /// Get database max lifetime
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.database.max_lifetime_secs)
    }

    /// Get auth session duration
    pub fn session_duration(&self) -> chrono::Duration {
        chrono::Duration::hours(self.auth.session_duration_hours as i64)
    }

    /// Get auth cleanup interval
    pub fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.auth.cleanup_interval_secs)
    }

    /// Get worker poll interval
    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.worker.poll_interval_secs)
    }

    /// Get worker retry backoff base
    pub fn retry_backoff_base(&self) -> Duration {
        Duration::from_secs(self.worker.retry_backoff_base_secs)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_origins: vec!["http://localhost:5173".to_string()],
                request_timeout_secs: 30,
            },
            database: DatabaseConfig {
                user: "starter_user".to_string(),
                password: "starter_pass".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                database: "starter_db".to_string(),
                max_connections: 10,
                min_connections: 2,
                connect_timeout_secs: 30,
                idle_timeout_secs: 300,
                max_lifetime_secs: 600,
            },
            auth: AuthConfig {
                session_duration_hours: 24,
                cleanup_interval_secs: 3600, // 1 hour
            },
            worker: WorkerConfig {
                concurrency: 4,
                poll_interval_secs: 5,
                max_retries: 3,
                retry_backoff_base_secs: 2,
            },
            initial_admin_password: None,
        }
    }
}


// Conversion from config::ConfigError to our Error type
impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Error::ConfigurationError(err.to_string())
    }
}