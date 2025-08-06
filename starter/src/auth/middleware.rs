use crate::auth::services;
use crate::error::Error;
use crate::rbac::UserRole;
use crate::types::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(req: &Request) -> Option<String> {
    req.headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_header| {
            auth_header
                .strip_prefix("Bearer ")
                .map(|token| token.to_string())
        })
}

/// Session-based authentication middleware
pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Error> {
    // Extract token from Authorization header
    let token = match extract_bearer_token(&req) {
        Some(token) => token,
        None => return Err(Error::Unauthorized),
    };

    // Get database connection
    let mut conn = match app_state.database.pool.acquire().await {
        Ok(conn) => conn,
        Err(_) => {
            tracing::error!("Failed to acquire database connection");
            return Err(Error::Internal("Database connection failed".to_string()));
        }
    };

    // Validate session and get user
    let user = match services::validate_session_with_user(conn.as_mut(), &token).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::debug!("Invalid or expired session token");
            return Err(Error::Unauthorized);
        }
        Err(e) => {
            tracing::error!("Error validating session: {}", e);
            return Err(Error::Internal("Session validation failed".to_string()));
        }
    };

    // Check if user is active
    if !user.is_active {
        tracing::debug!("User {} is not active", user.id);
        return Err(Error::Unauthorized);
    }

    // Add user info to request extensions
    req.extensions_mut().insert(AuthUser {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
    });

    Ok(next.run(req).await)
}

/// Optional authentication middleware - sets user if token is valid but doesn't require auth
pub async fn optional_auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // Try to extract token
    if let Some(token) = extract_bearer_token(&req) {
        // Try to get database connection
        if let Ok(mut conn) = app_state.database.pool.acquire().await {
            // Try to validate session
            if let Ok(Some(user)) = services::validate_session_with_user(conn.as_mut(), &token).await {
                if user.is_active {
                    // Add user info to request extensions
                    req.extensions_mut().insert(AuthUser {
                        id: user.id,
                        username: user.username,
                        email: user.email,
                        role: user.role,
                    });
                }
            }
        }
    }

    next.run(req).await
}

/// Admin-only middleware (requires auth_middleware to run first)
pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, Error> {
    // Get authenticated user from request extensions
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(Error::Unauthorized)?;

    // Check if user is admin
    if auth_user.role != UserRole::Admin {
        tracing::debug!("User {} attempted to access admin endpoint", auth_user.id);
        return Err(Error::Forbidden("Admin access required".to_string()));
    }

    Ok(next.run(req).await)
}
