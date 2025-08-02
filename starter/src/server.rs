use crate::{
    api::health,
    auth::{
        api as auth_api,
        middleware::{admin_middleware, auth_middleware},
    },
    config::AppConfig,
    database::Database,
    error::Error,
    openapi,
    rbac::middleware::require_moderator_role,
    tasks::api as tasks_api,
    types::{AppState, Result},
    users::api as users_api,
};
use axum::{
    Json, Router, middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::path::Path;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::info;
use utoipa::OpenApi;

/// Handle 404 Not Found errors
async fn not_found_handler() -> impl IntoResponse {
    Error::NotFound("The requested resource was not found".to_string())
}

/// Serve OpenAPI JSON specification
async fn openapi_json() -> impl IntoResponse {
    Json(openapi::ApiDoc::openapi())
}

/// API documentation page with links
async fn api_docs() -> impl IntoResponse {
    axum::response::Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>API Documentation - Rust Full-Stack Starter</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
                .container { max-width: 800px; margin: 0 auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
                h1 { color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }
                .links { margin: 20px 0; }
                .links a { display: inline-block; margin: 10px 15px 10px 0; padding: 10px 20px; background: #007acc; color: white; text-decoration: none; border-radius: 4px; }
                .links a:hover { background: #005999; }
                .info { background: #e3f2fd; padding: 15px; border-radius: 4px; margin: 20px 0; }
                pre { background: #f4f4f4; padding: 15px; border-radius: 4px; overflow-x: auto; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🦀 Rust Full-Stack Starter API Documentation</h1>
                
                <div class="info">
                    <p><strong>API Version:</strong> 0.1.0</p>
                    <p><strong>Base URL:</strong> <code>http://localhost:3000/api/v1</code></p>
                </div>

                <h2>📋 Available Documentation</h2>
                <div class="links">
                    <a href="/api-docs/openapi.json" target="_blank">📄 OpenAPI JSON Schema</a>
                    <a href="https://petstore.swagger.io/?url=https://raw.githubusercontent.com/quanhua92/rust-fullstack-starter/refs/heads/main/docs/openapi.json" target="_blank">🔧 Swagger UI (External)</a>
                </div>

                <h2>🚀 Quick Start</h2>
                <h3>Test the API:</h3>
                <pre>curl http://localhost:3000/api/v1/health</pre>
                
                <h3>Register a new user:</h3>
                <pre>curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "password123"}'</pre>

                <h3>Login:</h3>
                <pre>curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "password123"}'</pre>

                <h2>📖 API Endpoints</h2>
                <ul>
                    <li><strong>Health:</strong> <code>GET /api/v1/health</code> - Basic health check</li>
                    <li><strong>Auth:</strong> <code>POST /api/v1/auth/register</code>, <code>POST /api/v1/auth/login</code></li>
                    <li><strong>Users:</strong> <code>GET /api/v1/users/{id}</code> - Get user by ID</li>
                    <li><strong>Tasks:</strong> <code>POST /api/v1/tasks</code>, <code>GET /api/v1/tasks</code>, <code>GET /api/v1/tasks/{id}</code>, <code>GET /api/v1/tasks/dead-letter</code>, <code>POST /api/v1/tasks/{id}/retry</code>, <code>DELETE /api/v1/tasks/{id}</code></li>
                </ul>

                <p><em>For complete API documentation, see the OpenAPI JSON schema above.</em></p>
            </div>
        </body>
        </html>
        "#,
    )
}

/// Create the application router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health::health))
        .route("/health/detailed", get(health::detailed_health))
        .route("/health/live", get(health::health_live))
        .route("/health/ready", get(health::health_ready))
        .route("/health/startup", get(health::health_startup))
        .route("/auth/login", post(auth_api::login))
        .route("/auth/register", post(auth_api::register))
        // Task type registration (public for workers)
        .route("/tasks/types", post(tasks_api::register_task_type))
        .route("/tasks/types", get(tasks_api::list_task_types));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/auth/logout", post(auth_api::logout))
        .route("/auth/logout-all", post(auth_api::logout_all))
        .route("/auth/me", get(auth_api::me))
        .route("/auth/refresh", post(auth_api::refresh))
        // User management routes
        .route("/users/{id}", get(users_api::get_user_by_id))
        .route("/users/me/profile", put(users_api::update_own_profile))
        .route("/users/me/password", put(users_api::change_own_password))
        .route("/users/me", delete(users_api::delete_own_account))
        // Task management routes
        .route("/tasks", post(tasks_api::create_task))
        .route("/tasks", get(tasks_api::list_tasks))
        .route("/tasks/stats", get(tasks_api::get_stats))
        .route("/tasks/dead-letter", get(tasks_api::get_dead_letter_queue))
        // Individual task routes
        .route("/tasks/{id}", get(tasks_api::get_task))
        .route("/tasks/{id}", delete(tasks_api::delete_task))
        .route("/tasks/{id}/cancel", post(tasks_api::cancel_task))
        .route("/tasks/{id}/retry", post(tasks_api::retry_task))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Moderator routes (moderator role or higher required)
    let moderator_routes = Router::new()
        .route("/users", get(users_api::list_users))
        .route("/users/{id}/status", put(users_api::update_user_status))
        .route(
            "/users/{id}/reset-password",
            post(users_api::reset_user_password),
        )
        .layer(middleware::from_fn(require_moderator_role))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Admin routes (admin role required)
    let admin_routes = Router::new()
        .route("/admin/health", get(health::detailed_health))
        .route("/admin/users/stats", get(users_api::get_user_stats))
        .route("/users", post(users_api::create_user))
        .route("/users/{id}/profile", put(users_api::update_user_profile))
        .route("/users/{id}/role", put(users_api::update_user_role))
        .route("/users/{id}", delete(users_api::delete_user))
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(moderator_routes)
        .merge(admin_routes)
        .fallback(not_found_handler)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                        axum::http::header::HeaderName::from_static("x-request-id"),
                        axum::http::HeaderValue::from_static("test-request-id"),
                    ),
                )
                .layer(
                    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                        axum::http::header::X_CONTENT_TYPE_OPTIONS,
                        axum::http::HeaderValue::from_static("nosniff"),
                    ),
                )
                .layer(
                    tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                        axum::http::header::X_FRAME_OPTIONS,
                        axum::http::HeaderValue::from_static("DENY"),
                    ),
                )
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
}

/// Start the HTTP server
pub async fn start_server(config: AppConfig, database: Database) -> Result<()> {
    let state = AppState {
        config: config.clone(),
        database,
    };
    let api_router = create_router(state);

    // Setup static file serving for web frontend
    let web_build_path = &config.server.web_build_path;
    let index_path = Path::new(web_build_path).join("index.html");

    let static_files_service =
        ServeDir::new(web_build_path).not_found_service(ServeFile::new(index_path));

    let app = Router::new()
        .nest("/api/v1", api_router)
        // Keep documentation routes at root level
        .route("/api-docs", get(api_docs))
        .route("/api-docs/openapi.json", get(openapi_json))
        // Serve static files with SPA fallback
        .fallback_service(static_files_service);

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| Error::Internal(format!("Failed to bind to {bind_addr}: {e}")))?;

    info!("Server starting on {}", bind_addr);
    info!(
        "Serving static files from: {}",
        config.server.web_build_path
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Internal(format!("Server error: {e}")))?;

    Ok(())
}
