use crate::{
    api::health,
    error::Error,
    types::{AppState, Result},
    config::AppConfig,
    database::Database,
};
use axum::{
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

/// Create the application router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health endpoints
        .route("/health", get(health::health_check))
        .route("/health/detailed", get(health::health_detailed))
        // Add state and middleware
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                ),
        )
}

/// Start the HTTP server
pub async fn start_server(config: AppConfig, database: Database) -> Result<()> {
    let state = AppState { config: config.clone(), database };
    let app = create_router(state);

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| Error::Internal(format!("Failed to bind to {}: {}", bind_addr, e)))?;

    info!("Server starting on {}", bind_addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}