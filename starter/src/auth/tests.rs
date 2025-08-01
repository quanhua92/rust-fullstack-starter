#[cfg(test)]
mod auth_tests {
    use crate::{config::AppConfig, database::Database, types::AppState};
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        // This would need proper test database setup
        // For now, just showing the structure
        let config = AppConfig::load().expect("Failed to load config");
        let database = Database::connect(&config)
            .await
            .expect("Failed to connect to database");
        let state = AppState { config, database };

        crate::server::create_router(state)
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires database setup
    async fn test_register_and_login() {
        let app = setup_test_app().await;

        // Test user registration
        let register_payload = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test login
        let login_payload = json!({
            "email": "test@example.com",
            "password": "password123"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires database setup
    async fn test_protected_route_without_auth() {
        let app = setup_test_app().await;

        let request = Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
