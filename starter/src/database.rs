use crate::{config::AppConfig, error::Error, types::Result};
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Connect to PostgreSQL database with connection pooling
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        let database_url = config.database_url_string();

        // Create database if it doesn't exist
        if !Postgres::database_exists(&database_url)
            .await
            .map_err(Error::Database)?
        {
            Postgres::create_database(&database_url)
                .await
                .map_err(Error::Database)?;
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

        tracing::info!(
            "Connected to database with pool size: {}-{}",
            config.database.min_connections,
            config.database.max_connections
        );

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
        use crate::rbac::UserRole;
        use secrecy::ExposeSecret;

        // Check if any admin users exist
        let admin_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'admin'")
                .fetch_one(&self.pool)
                .await
                .map_err(Error::Database)?;

        if let Some(admin_password) = &config.initial_admin_password {
            // Admin password is configured
            if admin_count == 0 {
                // Hash the password using Argon2
                use argon2::password_hash::{SaltString, rand_core::OsRng};
                use argon2::{Argon2, PasswordHasher};

                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2
                    .hash_password(admin_password.expose_secret().as_bytes(), &salt)
                    .map_err(|e| Error::internal(&format!("Password hashing failed: {e}")))?
                    .to_string();

                // Create admin user with UserRole::Admin
                let admin_role = UserRole::Admin;
                sqlx::query(
                    r#"
                    INSERT INTO users (username, email, password_hash, role, is_active, email_verified)
                    VALUES ('admin', 'admin@example.com', $1, $2, true, true)
                    "#
                )
                .bind(password_hash)
                .bind(admin_role)
                .execute(&self.pool)
                .await
                .map_err(Error::Database)?;

                tracing::info!("✅ Created initial admin user (username: admin)");
            } else {
                tracing::info!("Admin user already exists, skipping creation");
            }
        } else {
            // No admin password configured - warn only if no admin exists
            if admin_count == 0 {
                tracing::warn!("═══════════════════════════════════════════════════════════════");
                tracing::warn!("⚠️  NO INITIAL ADMIN USER CREATED");
                tracing::warn!("   No STARTER__INITIAL_ADMIN_PASSWORD environment variable set");
                tracing::warn!("   To create an admin user:");
                tracing::warn!("   1. Set STARTER__INITIAL_ADMIN_PASSWORD in .env");
                tracing::warn!("   2. Restart the server");
                tracing::warn!("   3. Remove the password from .env after first startup");
                tracing::warn!("═══════════════════════════════════════════════════════════════");
            } else {
                tracing::info!("Admin user exists, no initial admin password needed");
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
