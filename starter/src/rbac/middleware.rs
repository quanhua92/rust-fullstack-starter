use crate::auth::AuthUser;
use crate::error::Error;
use crate::rbac::models::{Permission, Resource, UserRole};
use crate::rbac::services;
use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware that requires a specific role or higher
pub async fn require_role(
    required_role: UserRole,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Error>> + Send>>
+ Clone {
    move |req: Request, next: Next| {
        let required_role = required_role;
        Box::pin(async move {
            // Get authenticated user from request extensions (set by auth_middleware)
            let auth_user = req
                .extensions()
                .get::<AuthUser>()
                .ok_or(Error::Unauthorized)?;

            // Check if user has required role or higher
            if !services::has_role_or_higher(auth_user, required_role) {
                return Err(Error::Forbidden(format!(
                    "Access denied: {required_role} role or higher required"
                )));
            }

            Ok(next.run(req).await)
        })
    }
}

/// Middleware that requires a specific role or higher (convenience function)
pub async fn require_role_or_higher(
    req: Request,
    next: Next,
    required_role: UserRole,
) -> Result<Response, Error> {
    // Get authenticated user from request extensions
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(Error::Unauthorized)?;

    // Check if user has required role or higher
    if !services::has_role_or_higher(auth_user, required_role) {
        return Err(Error::Forbidden(format!(
            "Access denied: {required_role} role or higher required"
        )));
    }

    Ok(next.run(req).await)
}

/// Middleware that requires a specific permission on a resource
pub async fn require_permission(
    resource: Resource,
    permission: Permission,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Error>> + Send>>
+ Clone {
    move |req: Request, next: Next| {
        let resource = resource;
        let permission = permission;
        Box::pin(async move {
            // Get authenticated user from request extensions
            let auth_user = req
                .extensions()
                .get::<AuthUser>()
                .ok_or(Error::Unauthorized)?;

            // Check if user has required permission
            services::check_permission(auth_user, resource, permission)?;

            Ok(next.run(req).await)
        })
    }
}

/// Convenience middleware functions for common role requirements
/// Require admin role
pub async fn require_admin_role(req: Request, next: Next) -> Result<Response, Error> {
    require_role_or_higher(req, next, UserRole::Admin).await
}

/// Require moderator role or higher
pub async fn require_moderator_role(req: Request, next: Next) -> Result<Response, Error> {
    require_role_or_higher(req, next, UserRole::Moderator).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request as HttpRequest, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn test_handler() -> &'static str {
        "success"
    }

    fn create_test_user(role: &str) -> AuthUser {
        AuthUser {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: role.to_string().into(),
        }
    }

    #[tokio::test]
    async fn test_require_admin_middleware() {
        let app = Router::new()
            .route("/admin", get(test_handler))
            .layer(middleware::from_fn(require_admin_role));

        // Test with admin user
        let mut admin_req = HttpRequest::builder()
            .uri("/admin")
            .body(Body::empty())
            .unwrap();
        admin_req.extensions_mut().insert(create_test_user("admin"));

        let admin_response = app.clone().oneshot(admin_req).await.unwrap();
        assert_eq!(admin_response.status(), StatusCode::OK);

        // Test with regular user (should fail)
        let mut user_req = HttpRequest::builder()
            .uri("/admin")
            .body(Body::empty())
            .unwrap();
        user_req.extensions_mut().insert(create_test_user("user"));

        let user_response = app.oneshot(user_req).await.unwrap();
        assert_eq!(user_response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_require_moderator_middleware() {
        let app = Router::new()
            .route("/moderator", get(test_handler))
            .layer(middleware::from_fn(require_moderator_role));

        // Test with admin user (should work)
        let mut admin_req = HttpRequest::builder()
            .uri("/moderator")
            .body(Body::empty())
            .unwrap();
        admin_req.extensions_mut().insert(create_test_user("admin"));

        let admin_response = app.clone().oneshot(admin_req).await.unwrap();
        assert_eq!(admin_response.status(), StatusCode::OK);

        // Test with moderator user (should work)
        let mut mod_req = HttpRequest::builder()
            .uri("/moderator")
            .body(Body::empty())
            .unwrap();
        mod_req
            .extensions_mut()
            .insert(create_test_user("moderator"));

        let mod_response = app.clone().oneshot(mod_req).await.unwrap();
        assert_eq!(mod_response.status(), StatusCode::OK);

        // Test with regular user (should fail)
        let mut user_req = HttpRequest::builder()
            .uri("/moderator")
            .body(Body::empty())
            .unwrap();
        user_req.extensions_mut().insert(create_test_user("user"));

        let user_response = app.oneshot(user_req).await.unwrap();
        assert_eq!(user_response.status(), StatusCode::FORBIDDEN);
    }
}
