use crate::{
    api::health,
    auth::{api as auth_api, middleware::{auth_middleware, admin_middleware}},
    tasks::api as tasks_api,
    error::Error,
    types::{AppState, Result},
    config::AppConfig,
    database::Database,
};
use axum::{
    routing::{get, post},
    Router,
    middleware,
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
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/health/detailed", get(health::health_detailed))
        .route("/auth/login", post(auth_api::login))
        .route("/auth/register", post(auth_api::register));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/auth/logout", post(auth_api::logout))
        .route("/auth/logout-all", post(auth_api::logout_all))
        .route("/auth/me", get(auth_api::me))
        .route("/auth/refresh", post(auth_api::refresh))
        // Task management routes
        .route("/tasks", post(tasks_api::create_task))
        .route("/tasks", get(tasks_api::list_tasks))
        .route("/tasks/stats", get(tasks_api::get_stats))
        .route("/tasks/{id}", get(tasks_api::get_task))
        .route("/tasks/{id}/cancel", post(tasks_api::cancel_task))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin routes (admin role required)
    let admin_routes = Router::new()
        .route("/admin/health", get(health::health_detailed))
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
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