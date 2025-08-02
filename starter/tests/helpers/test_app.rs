use once_cell::sync::Lazy;
use reqwest::redirect::Policy;
use sqlx::PgPool;
use starter::{AppConfig, Database, server};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub config: AppConfig,
    pub db_pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
}

// Global tracing setup - only initialize once
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_test_writer()
            .init();
    }
});

pub async fn spawn_app() -> TestApp {
    // Initialize tracing once across all tests
    Lazy::force(&TRACING);

    // Load environment variables
    dotenvy::dotenv().ok();

    // Create test configuration
    let mut config = AppConfig::load().expect("Failed to load config");

    // Create isolated test database
    let test_db = super::db::create_test_db()
        .await
        .expect("Failed to create test database");

    // Update config to use test database - we can't directly set URL, so we'll update the database name
    config.database.database = test_db.name.clone();
    config.database.max_connections = 5;
    config.database.min_connections = 1;

    // Create database instance with test pool
    let database = Database {
        pool: test_db.pool.clone(),
    };

    // Build application with state
    let state = starter::types::AppState {
        config: config.clone(),
        database,
        start_time: std::time::Instant::now(),
    };
    let api_router = server::create_router(state);
    let app = axum::Router::new().nest("/api/v1", api_router);

    // Bind to random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    // Start server in background
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Give server time to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Create HTTP client with persistent cookies
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    TestApp {
        address,
        client,
        config,
        db_pool: test_db.pool.clone(),
    }
}

impl TestApp {
    // GET request
    pub async fn get(&self, path: &str) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .get(url)
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    // GET request with auth token
    pub async fn get_auth(&self, path: &str, token: &str) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("Failed to execute GET request")
    }

    // POST with JSON body
    pub async fn post_json<T: serde::Serialize>(&self, path: &str, json: &T) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .post(url)
            .json(json)
            .send()
            .await
            .expect("Failed to execute POST request")
    }

    // POST with auth token (no body)
    pub async fn post_auth(&self, path: &str, token: &str) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("Failed to execute POST request")
    }

    // POST with JSON body and auth token
    pub async fn post_json_auth<T: serde::Serialize>(
        &self,
        path: &str,
        json: &T,
        token: &str,
    ) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {token}"))
            .json(json)
            .send()
            .await
            .expect("Failed to execute POST request")
    }

    // PUT with JSON body and auth token
    pub async fn put_json_auth<T: serde::Serialize>(
        &self,
        path: &str,
        json: &T,
        token: &str,
    ) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .put(url)
            .header("Authorization", format!("Bearer {token}"))
            .json(json)
            .send()
            .await
            .expect("Failed to execute PUT request")
    }

    // DELETE request with auth token
    pub async fn delete_auth(&self, path: &str, token: &str) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .delete(url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("Failed to execute DELETE request")
    }

    // DELETE with JSON body and auth token
    pub async fn delete_json_auth<T: serde::Serialize>(
        &self,
        path: &str,
        json: &T,
        token: &str,
    ) -> reqwest::Response {
        let url = format!("{}{}", self.address, path);
        self.client
            .delete(url)
            .header("Authorization", format!("Bearer {token}"))
            .json(json)
            .send()
            .await
            .expect("Failed to execute DELETE request")
    }

    // Extract auth token from login response
    pub async fn extract_auth_token(&self, response: reqwest::Response) -> AuthToken {
        let json: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse response as JSON");

        // API response format: { "success": true, "data": { "session_token": "...", ... } }
        let token = json["data"]["session_token"]
            .as_str()
            .expect("No session_token in response")
            .to_string();

        AuthToken { token }
    }

    // Helper to get database connection
    pub async fn db(&self) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
        self.db_pool
            .acquire()
            .await
            .expect("Failed to acquire db connection")
    }
}
