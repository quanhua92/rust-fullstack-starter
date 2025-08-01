use crate::auth::AuthUser;
use crate::rbac::services as rbac_services;
use crate::users::{models::UserProfile, services as user_services};
use crate::{
    error::Error,
    types::{ApiResponse, AppState, ErrorResponse},
};
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

pub async fn get_profile(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let profile = user_services::get_user_profile(&mut conn, auth_user.id).await?;

    match profile {
        Some(profile) => Ok(Json(ApiResponse::success(profile))),
        None => Err(Error::NotFound("User profile not found".to_string())),
    }
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    summary = "Get user by ID",
    description = "Get user information by user ID",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = ApiResponse<UserProfile>),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Get target user to access their role for authorization
    let target_user = user_services::find_user_by_id(&mut conn, user_id).await?;
    let target_user = match target_user {
        Some(user) => user,
        None => return Err(Error::NotFound("User not found".to_string())),
    };

    // Check RBAC authorization with target user's role
    rbac_services::can_access_user_profile(&auth_user, user_id, target_user.role)?;

    // Return user profile
    Ok(Json(ApiResponse::success(target_user.to_profile())))
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List all users (Admin/Moderator only)
#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    summary = "List users",
    description = "List all users in the system (Admin/Moderator only)",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of users to return"),
        ("offset" = Option<i64>, Query, description = "Number of users to skip")
    ),
    responses(
        (status = 200, description = "List of users", body = ApiResponse<Vec<UserProfile>>),
        (status = 403, description = "Forbidden - Moderator access required", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserProfile>>>, Error> {
    // Require moderator or higher role
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let users = user_services::list_users(&mut conn, params.limit, params.offset).await?;

    Ok(Json(ApiResponse::success(users)))
}
