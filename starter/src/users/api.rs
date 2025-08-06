use crate::auth::AuthUser;
use crate::rbac::services as rbac_services;
use crate::users::{
    models::{
        ChangePasswordRequest, CreateUserRequest, DeleteAccountRequest, DeleteUserRequest,
        ResetPasswordRequest, UpdateProfileRequest, UpdateUserProfileRequest,
        UpdateUserRoleRequest, UpdateUserStatusRequest, UserProfile, UserStats,
    },
    services as user_services,
};
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

#[utoipa::path(
    get,
    path = "/users/me/profile",
    tag = "Users",
    summary = "Get own profile",
    description = "Get current user's profile information",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ApiResponse<UserProfile>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User profile not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
    let profile = user_services::get_user_profile(conn.as_mut(), auth_user.id).await?;

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
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Get target user to access their role for authorization
    let target_user = user_services::find_user_by_id(conn.as_mut(), id).await?;
    let target_user = match target_user {
        Some(user) => user,
        None => return Err(Error::NotFound("User not found".to_string())),
    };

    // Check RBAC authorization with target user's role
    rbac_services::can_access_user_profile(&auth_user, id, target_user.role)?;

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

    let users = user_services::list_users(conn.as_mut(), params.limit, params.offset).await?;

    Ok(Json(ApiResponse::success(users)))
}

/// Create a new user (Admin only)
#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    summary = "Create user",
    description = "Create a new user account (Admin only)",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created", body = ApiResponse<UserProfile>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Username or email already exists", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_user(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let user = user_services::create_user(conn.as_mut(), request).await?;

    Ok(Json(ApiResponse::success(user)))
}

/// Update own profile
#[utoipa::path(
    put,
    path = "/users/me/profile",
    tag = "Users",
    summary = "Update own profile",
    description = "Update own user profile (username, email)",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ApiResponse<UserProfile>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Username or email already exists", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_own_profile(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let user = user_services::update_user_profile(conn.as_mut(), auth_user.id, request).await?;

    Ok(Json(ApiResponse::success(user)))
}

/// Change own password
#[utoipa::path(
    put,
    path = "/users/me/password",
    tag = "Users",
    summary = "Change own password",
    description = "Change own password with current password verification",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed", body = ApiResponse<String>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized or invalid current password", body = ErrorResponse),
        (status = 422, description = "New password same as current", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_own_password(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    user_services::change_user_password(conn.as_mut(), auth_user.id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Password updated successfully".to_string(),
        "Password has been changed. All existing sessions remain active.".to_string(),
    )))
}

/// Delete own account
#[utoipa::path(
    delete,
    path = "/users/me",
    tag = "Users",
    summary = "Delete own account",
    description = "Delete own user account (soft delete)",
    request_body = DeleteAccountRequest,
    responses(
        (status = 200, description = "Account deleted", body = ApiResponse<String>),
        (status = 400, description = "Missing confirmation or incorrect password", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_own_account(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    user_services::delete_user_account(conn.as_mut(), auth_user.id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Account deleted successfully".to_string(),
        "Your account has been deactivated. All data will be retained for 30 days.".to_string(),
    )))
}

/// Update any user's profile (Admin only)
#[utoipa::path(
    put,
    path = "/users/{id}/profile",
    tag = "Users",
    summary = "Update user profile",
    description = "Update any user's profile (Admin only)",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = ApiResponse<UserProfile>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 409, description = "Username or email already exists", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_profile(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let user = user_services::update_user_profile_admin(conn.as_mut(), id, request).await?;

    Ok(Json(ApiResponse::success(user)))
}

/// Update user status (Moderator/Admin)
#[utoipa::path(
    put,
    path = "/users/{id}/status",
    tag = "Users",
    summary = "Update user status",
    description = "Activate or deactivate a user account (Moderator/Admin)",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserStatusRequest,
    responses(
        (status = 200, description = "User status updated", body = ApiResponse<UserProfile>),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Moderator access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_status(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Require moderator or higher role
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let user = user_services::update_user_status(conn.as_mut(), id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        user,
        "User account status updated".to_string(),
    )))
}

/// Update user role (Admin only)
#[utoipa::path(
    put,
    path = "/users/{id}/role",
    tag = "Users",
    summary = "Update user role",
    description = "Change a user's role (Admin only)",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated", body = ApiResponse<UserProfile>),
        (status = 400, description = "Invalid role value", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_role(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let user = user_services::update_user_role(conn.as_mut(), id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        user,
        "User role updated successfully".to_string(),
    )))
}

/// Reset user password (Moderator/Admin)
#[utoipa::path(
    post,
    path = "/users/{id}/reset-password",
    tag = "Users",
    summary = "Reset user password",
    description = "Force password reset for a user (Moderator/Admin)",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset", body = ApiResponse<String>),
        (status = 400, description = "Invalid password", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Moderator access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reset_user_password(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<String>>, Error> {
    // Require moderator or higher role
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    user_services::reset_user_password(conn.as_mut(), id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        "Password reset successfully".to_string(),
        "User's password has been updated. All existing sessions have been invalidated."
            .to_string(),
    )))
}

/// Delete user account (Admin only)
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    summary = "Delete user account",
    description = "Delete a user account (Admin only)",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = DeleteUserRequest,
    responses(
        (status = 200, description = "User deleted", body = ApiResponse<String>),
        (status = 400, description = "Cannot delete own account via this endpoint", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<DeleteUserRequest>,
) -> Result<Json<ApiResponse<String>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    // Prevent admin from deleting their own account via this endpoint
    if auth_user.id == id {
        return Err(Error::validation(
            "id",
            "Cannot delete own account via this endpoint",
        ));
    }

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    user_services::delete_user_admin(conn.as_mut(), id, request).await?;

    Ok(Json(ApiResponse::success_with_message(
        "User account deleted successfully".to_string(),
        "User account has been deactivated. Data retained for 30 days for recovery.".to_string(),
    )))
}

/// Get user statistics (Admin only)
#[utoipa::path(
    get,
    path = "/admin/users/stats",
    tag = "Admin",
    summary = "Get user statistics",
    description = "Get comprehensive user statistics (Admin only)",
    responses(
        (status = 200, description = "User statistics", body = ApiResponse<UserStats>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_stats(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<UserStats>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let stats = user_services::get_user_stats(conn.as_mut()).await?;

    Ok(Json(ApiResponse::success(stats)))
}
