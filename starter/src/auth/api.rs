use crate::auth::{
    AuthUser,
    models::{LoginRequest, LoginResponse, RegisterRequest},
    services as auth_services,
};
use crate::users::models::UserProfile;
use crate::{
    error::Error,
    types::{ApiResponse, AppState, ErrorResponse},
};
use axum::{
    extract::{Extension, State},
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
    let login_response = auth_services::login(&mut conn, payload).await?;
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
    let user_profile = auth_services::register(&mut conn, payload).await?;
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
    let sessions_deleted = auth_services::logout_all(&mut conn, auth_user.id).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Logged out successfully".to_string(),
        format!("Ended {sessions_deleted} session(s)"),
    )))
}

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
    let sessions_deleted = auth_services::logout_all(&mut conn, auth_user.id).await?;

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

pub async fn refresh(Extension(_auth_user): Extension<AuthUser>) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success_with_message(
        "Token is still valid".to_string(),
        "Current session remains active".to_string(),
    ))
}
