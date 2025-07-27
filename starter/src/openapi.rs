use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::auth::{
    AuthUser,
    models::{LoginRequest, LoginResponse, RegisterRequest},
};
use crate::tasks::api::{CreateTaskApiRequest, TaskQueryParams};
use crate::tasks::types::{CreateTaskRequest, TaskResponse, TaskStatus};
use crate::types::{DetailedHealthResponse, ErrorResponse, HealthResponse};
use crate::users::models::{User, UserProfile};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust Full-Stack Starter API",
        description = "A production-ready Rust web application starter with authentication, background tasks, and comprehensive API documentation.",
        version = "0.1.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server"),
        (url = "https://api.example.com", description = "Production server")
    ),
    paths(
        // Health endpoints
        crate::api::health::health,
        crate::api::health::detailed_health,
        crate::api::health::health_live,
        crate::api::health::health_ready,
        crate::api::health::health_startup,

        // Auth endpoints
        crate::auth::api::register,
        crate::auth::api::login,
        crate::auth::api::logout,
        crate::auth::api::me,

        // User endpoints
        crate::users::api::get_user_by_id,

        // Task endpoints
        crate::tasks::api::create_task,
        crate::tasks::api::list_tasks,
        crate::tasks::api::get_task,
        crate::tasks::api::cancel_task,
    ),
    components(
        schemas(
            // Auth models
            LoginRequest,
            RegisterRequest,
            LoginResponse,
            AuthUser,

            // User models
            User,
            UserProfile,

            // Task models
            CreateTaskRequest,
            CreateTaskApiRequest,
            TaskResponse,
            TaskStatus,
            TaskQueryParams,

            // Common response types
            ErrorResponse,

            // Health models
            HealthResponse,
            DetailedHealthResponse,
        )
    ),
    tags(
        (name = "Health", description = "Health check and monitoring endpoints"),
        (name = "Authentication", description = "User authentication and session management"),
        (name = "Users", description = "User management operations"),
        (name = "Tasks", description = "Background task management"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;

/// Create Swagger UI service (to be added manually to server)
pub fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())
}

/// Get OpenAPI specification as JSON
pub fn openapi_json() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let openapi = ApiDoc::openapi();

        // Verify basic info
        assert_eq!(openapi.info.title, "Rust Full-Stack Starter API");
        assert_eq!(openapi.info.version, "0.1.0");

        // Verify we have some paths
        assert!(!openapi.paths.paths.is_empty());

        // Verify we have some components
        assert!(openapi.components.is_some());

        // Verify JSON generation works
        let json = openapi_json();
        assert!(!json.is_empty());
        assert!(json.contains("Rust Full-Stack Starter API"));
    }

    #[test]
    fn test_swagger_ui_creation() {
        let _swagger_ui = create_swagger_ui();
        // Just verify it creates without panicking
        // SwaggerUi doesn't implement Debug, so we just test creation
        assert!(true);
    }
}
