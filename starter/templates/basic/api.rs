use crate::auth::AuthUser;
use crate::rbac::services as rbac_services;
use crate::__MODULE_NAME_PLURAL__::{
    models::{
        Create__MODULE_STRUCT__Request, Update__MODULE_STRUCT__Request, __MODULE_STRUCT__Response, __MODULE_STRUCT__Stats,
    },
    services as __MODULE_NAME_PLURAL___services,
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

#[derive(Debug, Deserialize)]
pub struct List__MODULE_STRUCT__sQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new __MODULE_NAME__
#[utoipa::path(
    post,
    path = "/__MODULE_NAME_PLURAL__",
    tag = "__MODULE_STRUCT__s",
    summary = "Create __MODULE_NAME__",
    description = "Create a new __MODULE_NAME__",
    request_body = Create__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__ created", body = ApiResponse<__MODULE_STRUCT__Response>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Create__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__Response>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let __MODULE_NAME__ = __MODULE_NAME_PLURAL___services::create___MODULE_NAME__(&mut conn, auth_user.id, request).await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Get __MODULE_NAME__ by ID
#[utoipa::path(
    get,
    path = "/__MODULE_NAME_PLURAL__/{id}",
    tag = "__MODULE_STRUCT__s",
    summary = "Get __MODULE_NAME__ by ID",
    description = "Get a specific __MODULE_NAME__ by its ID",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    responses(
        (status = 200, description = "__MODULE_STRUCT__ found", body = ApiResponse<__MODULE_STRUCT__Response>),
        (status = 404, description = "__MODULE_STRUCT__ not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get___MODULE_NAME___by_id(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__Response>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let __MODULE_NAME__ = __MODULE_NAME_PLURAL___services::get___MODULE_NAME___by_id(&mut conn, id).await?;
    let __MODULE_NAME__ = match __MODULE_NAME__ {
        Some(__MODULE_NAME__) => __MODULE_NAME__,
        None => return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string())),
    };

    // Check if user can access this __MODULE_NAME__ (own resource or admin/moderator)
    match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            // Admin and Moderator can access any resource
        }
        crate::rbac::models::UserRole::User => {
            // Users can only access their own resources
            if __MODULE_NAME__.user_id != auth_user.id {
                return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string()));
            }
        }
    }

    Ok(Json(ApiResponse::success(__MODULE_NAME__.to_response())))
}

/// List __MODULE_NAME_PLURAL__ for current user
#[utoipa::path(
    get,
    path = "/__MODULE_NAME_PLURAL__",
    tag = "__MODULE_STRUCT__s",
    summary = "List __MODULE_NAME_PLURAL__",
    description = "List __MODULE_NAME_PLURAL__ for the current user",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of __MODULE_NAME_PLURAL__ to return"),
        ("offset" = Option<i64>, Query, description = "Number of __MODULE_NAME_PLURAL__ to skip")
    ),
    responses(
        (status = 200, description = "List of __MODULE_NAME_PLURAL__", body = ApiResponse<Vec<__MODULE_STRUCT__Response>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<List__MODULE_STRUCT__sQuery>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__Response>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Regular users can only see their own __MODULE_NAME_PLURAL__, admins/moderators can see all
    let user_id = match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            None // Admin and Moderator see all
        }
        crate::rbac::models::UserRole::User => {
            Some(auth_user.id) // Regular user sees only their own
        }
    };

    let __MODULE_NAME_PLURAL__ = __MODULE_NAME_PLURAL___services::list___MODULE_NAME_PLURAL__(&mut conn, user_id, params.limit, params.offset).await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME_PLURAL__)))
}

/// Update __MODULE_NAME__
#[utoipa::path(
    put,
    path = "/__MODULE_NAME_PLURAL__/{id}",
    tag = "__MODULE_STRUCT__s",
    summary = "Update __MODULE_NAME__",
    description = "Update a __MODULE_NAME__",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    request_body = Update__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__ updated", body = ApiResponse<__MODULE_STRUCT__Response>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "__MODULE_STRUCT__ not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<Update__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__Response>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Check if __MODULE_NAME__ exists and user has permission
    let existing___MODULE_NAME__ = __MODULE_NAME_PLURAL___services::get___MODULE_NAME___by_id(&mut conn, id).await?;
    let existing___MODULE_NAME__ = match existing___MODULE_NAME__ {
        Some(__MODULE_NAME__) => __MODULE_NAME__,
        None => return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string())),
    };

    // Check if user can update this __MODULE_NAME__ (own resource or admin/moderator)
    match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            // Admin and Moderator can update any resource
        }
        crate::rbac::models::UserRole::User => {
            // Users can only update their own resources
            if existing___MODULE_NAME__.user_id != auth_user.id {
                return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string()));
            }
        }
    }

    let __MODULE_NAME__ = __MODULE_NAME_PLURAL___services::update___MODULE_NAME__(&mut conn, id, request).await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Delete __MODULE_NAME__
#[utoipa::path(
    delete,
    path = "/__MODULE_NAME_PLURAL__/{id}",
    tag = "__MODULE_STRUCT__s",
    summary = "Delete __MODULE_NAME__",
    description = "Delete a __MODULE_NAME__ (soft delete)",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    responses(
        (status = 200, description = "__MODULE_STRUCT__ deleted", body = ApiResponse<String>),
        (status = 404, description = "__MODULE_STRUCT__ not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Check if __MODULE_NAME__ exists and user has permission
    let existing___MODULE_NAME__ = __MODULE_NAME_PLURAL___services::get___MODULE_NAME___by_id(&mut conn, id).await?;
    let existing___MODULE_NAME__ = match existing___MODULE_NAME__ {
        Some(__MODULE_NAME__) => __MODULE_NAME__,
        None => return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string())),
    };

    // Check if user can delete this __MODULE_NAME__ (own resource or admin/moderator)
    match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            // Admin and Moderator can delete any resource
        }
        crate::rbac::models::UserRole::User => {
            // Users can only delete their own resources
            if existing___MODULE_NAME__.user_id != auth_user.id {
                return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string()));
            }
        }
    }

    __MODULE_NAME_PLURAL___services::delete___MODULE_NAME__(&mut conn, id).await?;

    Ok(Json(ApiResponse::success_with_message(
        "__MODULE_STRUCT__ deleted successfully".to_string(),
        "__MODULE_STRUCT__ has been deactivated".to_string(),
    )))
}

/// Get __MODULE_NAME_PLURAL__ statistics (Admin only)
#[utoipa::path(
    get,
    path = "/admin/__MODULE_NAME_PLURAL__/stats",
    tag = "Admin",
    summary = "Get __MODULE_NAME_PLURAL__ statistics",
    description = "Get comprehensive __MODULE_NAME_PLURAL__ statistics (Admin only)",
    responses(
        (status = 200, description = "__MODULE_STRUCT__ statistics", body = ApiResponse<__MODULE_STRUCT__Stats>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get___MODULE_NAME_PLURAL___stats(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__Stats>>, Error> {
    // Require admin role
    rbac_services::require_admin(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let stats = __MODULE_NAME_PLURAL___services::get___MODULE_NAME_PLURAL___stats(&mut conn).await?;

    Ok(Json(ApiResponse::success(stats)))
}