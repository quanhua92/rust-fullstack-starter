use once_cell::sync::Lazy;
use sqlx::{Connection, PgConnection, PgPool};
use tokio::sync::{OnceCell, Semaphore};
use uuid::Uuid;

// Template database optimization with OnceCell
static TEMPLATE_DB_NAME: &str = "starter_test_template";
static TEMPLATE_INITIALIZED: OnceCell<()> = OnceCell::const_new();

// Semaphore to limit concurrent database operations
static DB_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(3));

#[derive(Clone)]
pub struct TestDatabase {
    pub name: String,
    pub url: String,
    pub pool: PgPool,
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        tracing::debug!("TestDatabase dropped: {}", self.name);
    }
}

/// Get database configuration from environment variables
fn get_db_config() -> starter::AppConfig {
    starter::AppConfig::load().expect("Failed to load config")
}

/// Ensures template database exists with all migrations applied
/// OnceCell automatically handles synchronization across multiple tests
async fn ensure_template_db() -> Result<(), Box<dyn std::error::Error>> {
    TEMPLATE_INITIALIZED
        .get_or_init(|| async {
            tracing::debug!("Creating template database");
            create_template_database()
                .await
                .expect("Failed to create template database");
            tracing::info!("Template database ready: {}", TEMPLATE_DB_NAME);
        })
        .await;

    Ok(())
}

/// Creates the template database with migrations
async fn create_template_database() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_db_config();
    let admin_url = config
        .database_url_string()
        .replace(&config.database.database, "postgres");

    // Create template database
    let mut admin_conn = PgConnection::connect(&admin_url).await?;

    let result = sqlx::query(&format!("CREATE DATABASE \"{TEMPLATE_DB_NAME}\""))
        .execute(&mut admin_conn)
        .await;

    admin_conn.close().await?;

    // Handle the case where database already exists
    match result {
        Ok(_) => tracing::debug!("Created template database"),
        Err(sqlx::Error::Database(db_err)) if db_err.code() == Some("42P04".into()) => {
            tracing::debug!("Template database already exists");
        }
        Err(sqlx::Error::Database(db_err)) if db_err.code() == Some("23505".into()) => {
            tracing::debug!("Template database already exists (duplicate key)");
        }
        Err(e) => return Err(Box::new(e)),
    }

    // Connect to template database and run migrations
    let template_url = config
        .database_url_string()
        .replace(&config.database.database, TEMPLATE_DB_NAME);
    let mut template_conn = PgConnection::connect(&template_url).await?;

    sqlx::migrate!("./migrations")
        .run(&mut template_conn)
        .await?;

    template_conn.close().await?;
    Ok(())
}

/// Creates test database by cloning from template (10x faster than migrations)
pub async fn create_test_db() -> Result<TestDatabase, Box<dyn std::error::Error>> {
    // Acquire semaphore for rate limiting
    let _permit = DB_SEMAPHORE.acquire().await?;

    // Ensure template database exists
    ensure_template_db().await?;

    // Create unique test database name
    let uuid = Uuid::now_v7();
    let db_name = format!("test_{}", uuid.simple());

    let config = get_db_config();
    let admin_url = config
        .database_url_string()
        .replace(&config.database.database, "postgres");

    // Create database by cloning from template
    let mut admin_conn = PgConnection::connect(&admin_url).await?;

    sqlx::query(&format!(
        "CREATE DATABASE \"{db_name}\" WITH TEMPLATE \"{TEMPLATE_DB_NAME}\""
    ))
    .execute(&mut admin_conn)
    .await?;

    admin_conn.close().await?;

    // Create connection URL for the new test database
    let test_url = config
        .database_url_string()
        .replace(&config.database.database, &db_name);

    // Create connection pool for test database
    let test_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(10))
        .connect(&test_url)
        .await?;

    tracing::info!("Created test database from template: {}", db_name);

    Ok(TestDatabase {
        name: db_name,
        url: test_url,
        pool: test_pool,
    })
}

/// Helper to clean up test database
pub async fn cleanup_test_db(db_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = get_db_config();
    let admin_url = config
        .database_url_string()
        .replace(&config.database.database, "postgres");

    let mut admin_conn = PgConnection::connect(&admin_url).await?;

    // Force close all connections to the test database
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(&mut admin_conn)
    .await?;

    // Drop the test database
    sqlx::query(&format!("DROP DATABASE IF EXISTS \"{db_name}\""))
        .execute(&mut admin_conn)
        .await?;

    admin_conn.close().await?;
    tracing::debug!("Cleaned up test database: {}", db_name);
    Ok(())
}
