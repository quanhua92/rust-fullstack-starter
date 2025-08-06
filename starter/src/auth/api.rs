use crate::auth::{
    AuthUser,
    models::{LoginRequest, LoginResponse, RefreshResponse, RegisterRequest},
    services as auth_services,
};
use crate::users::models::UserProfile;
use crate::{
    error::Error,
    types::{ApiResponse, AppState, ErrorResponse},
};
use axum::{
    extract::{Extension, Request, State},
    response::Json,
};

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Authentication",
    summary = "User login",
    description = "Authenticate user with username/email and password",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<crate::auth::models::LoginResponse>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let login_response = auth_services::login(conn.as_mut(), payload).await?;
    Ok(Json(ApiResponse::success(login_response)))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Authentication",
    summary = "User registration",
    description = "Register a new user account",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = ApiResponse<UserProfile>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse)
    )
)]
pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<crate::users::models::UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let user_profile = auth_services::register(conn.as_mut(), payload).await?;
    Ok(Json(ApiResponse::success(user_profile)))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Authentication",
    summary = "User logout",
    description = "Logout current user and end all sessions",
    responses(
        (status = 200, description = "Logout successful", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let sessions_deleted = auth_services::logout_all(conn.as_mut(), auth_user.id).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Logged out successfully".to_string(),
        format!("Ended {sessions_deleted} session(s)"),
    )))
}

#[utoipa::path(
    post,
    path = "/auth/logout-all",
    tag = "Authentication",
    summary = "Logout from all devices",
    description = "Logout current user from all devices and end all sessions",
    responses(
        (status = 200, description = "Logout successful", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout_all(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let sessions_deleted = auth_services::logout_all(conn.as_mut(), auth_user.id).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Logged out from all devices".to_string(),
        format!("Ended {sessions_deleted} session(s)"),
    )))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Authentication",
    summary = "Get current user",
    description = "Get current authenticated user information",
    responses(
        (status = 200, description = "Current user information", body = ApiResponse<AuthUser>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn me(Extension(auth_user): Extension<AuthUser>) -> Json<ApiResponse<AuthUser>> {
    Json(ApiResponse::success(auth_user))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Authentication",
    summary = "Refresh token",
    description = "Refresh session token by extending its expiration time",
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<RefreshResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Cannot refresh yet", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn refresh(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    req: Request,
) -> Result<Json<ApiResponse<crate::auth::models::RefreshResponse>>, Error> {
    // Extract token from Authorization header
    let token = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_header| auth_header.strip_prefix("Bearer "))
        .ok_or(Error::Unauthorized)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let refreshed_session = auth_services::refresh_session_token(
        conn.as_mut(),
        token,
        Some(app_state.config.refresh_extend_hours()),
        Some(app_state.config.refresh_min_interval_minutes()),
    )
    .await?;

    match refreshed_session {
        Some(session) => {
            let refresh_response = crate::auth::models::RefreshResponse {
                expires_at: session.expires_at,
                refreshed_at: session.last_refreshed_at.unwrap_or(session.updated_at),
            };
            Ok(Json(ApiResponse::success(refresh_response)))
        }
        None => Err(Error::conflict(
            "Cannot refresh token yet. Please wait before requesting another refresh.",
        )),
    }
}
