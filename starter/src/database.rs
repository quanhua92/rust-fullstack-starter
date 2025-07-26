use sqlx::{PgPool, postgres::PgPoolOptions, migrate::MigrateDatabase, Postgres};
use crate::{config::AppConfig, types::Result, error::Error};

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Connect to PostgreSQL database with connection pooling
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        let database_url = config.database_url_string();
        
        // Create database if it doesn't exist
        if !Postgres::database_exists(&database_url).await.map_err(Error::Database)? {
            Postgres::create_database(&database_url).await.map_err(Error::Database)?;
            tracing::info!("Created database");
        }
        
        // Create connection pool with configuration
        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(config.connect_timeout())
            .idle_timeout(Some(config.idle_timeout()))
            .max_lifetime(Some(config.max_lifetime()))
            .connect(&database_url)
            .await
            .map_err(Error::Database)?;
        
        tracing::info!("Connected to database with pool size: {}-{}", 
            config.database.min_connections, config.database.max_connections);
        
        Ok(Database { pool })
    }
    
    /// Run database migrations from starter/migrations directory
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(Error::Migration)?;
        tracing::info!("Database migrations completed");
        Ok(())
    }
    
    /// Ensure initial admin user exists if configured
    pub async fn ensure_initial_admin(&self, config: &AppConfig) -> Result<()> {
        use secrecy::ExposeSecret;
        
        // Only create admin if password is configured and no admin exists
        if let Some(admin_password) = &config.initial_admin_password {
            let admin_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM users WHERE role = 'admin'"
            )
            .fetch_one(&self.pool)
            .await
            .map_err(Error::Database)?;
            
            if admin_count == 0 {
                // Hash the password using Argon2
                use argon2::{Argon2, PasswordHasher};
                use argon2::password_hash::{rand_core::OsRng, SaltString};
                
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(admin_password.expose_secret().as_bytes(), &salt)
                    .map_err(|e| Error::internal(&format!("Password hashing failed: {}", e)))?
                    .to_string();
                
                // Create admin user
                sqlx::query(
                    r#"
                    INSERT INTO users (username, email, password_hash, role, is_active, email_verified)
                    VALUES ('admin', 'admin@example.com', $1, 'admin', true, true)
                    "#
                )
                .bind(password_hash)
                .execute(&self.pool)
                .await
                .map_err(Error::Database)?;
                
                tracing::info!("Created initial admin user");
            }
        }
        Ok(())
    }
    
    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}