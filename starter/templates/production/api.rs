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
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct List__MODULE_STRUCT__sQuery {
    // Pagination
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    
    // Search
    pub search: Option<String>,        // Search in title and content
    pub title: Option<String>,         // Filter by exact title match
    
    // Filtering
    pub user_id: Option<Uuid>,         // Filter by user (admin/moderator only)
    pub is_active: Option<bool>,       // Filter by active status
    pub created_after: Option<String>, // ISO date string
    pub created_before: Option<String>,// ISO date string
    
    // Sorting
    pub sort_by: Option<String>,       // "created_at", "updated_at", "title"
    pub sort_order: Option<String>,    // "asc", "desc"
}

#[derive(Debug, Deserialize)]
pub struct BulkCreate__MODULE_STRUCT__Request {
    pub items: Vec<Create__MODULE_STRUCT__Request>,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdate__MODULE_STRUCT__Request {
    pub items: Vec<BulkUpdate__MODULE_STRUCT__Item>,
}

#[derive(Debug, Deserialize)]
pub struct BulkUpdate__MODULE_STRUCT__Item {
    pub id: Uuid,
    pub data: Update__MODULE_STRUCT__Request,
}

#[derive(Debug, Deserialize)]
pub struct BulkDelete__MODULE_STRUCT__Request {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct __MODULE_STRUCT__CountResponse {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
}

#[derive(Debug, Deserialize)]
pub struct Search__MODULE_STRUCT__sQuery {
    pub q: String,                     // Search query (required)
    pub limit: Option<i64>,            // Max results
    pub offset: Option<i64>,           // Pagination offset
    pub sort_by: Option<String>,       // Sort field
    pub sort_order: Option<String>,    // Sort direction
}

#[derive(Debug, Deserialize)]
pub struct Filter__MODULE_STRUCT__sQuery {
    // Advanced filtering
    pub user_id: Option<Uuid>,         // Filter by user
    pub is_active: Option<bool>,       // Filter by status
    pub created_after: Option<String>, // Date range start
    pub created_before: Option<String>,// Date range end
    pub title_contains: Option<String>,// Title contains text
    
    // Pagination and sorting
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
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
        ("limit" = Option<i64>, Query, description = "Maximum number of __MODULE_NAME_PLURAL__ to return (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of __MODULE_NAME_PLURAL__ to skip for pagination"),
        ("search" = Option<String>, Query, description = "Search in title and content fields"),
        ("title" = Option<String>, Query, description = "Filter by exact title match"),
        ("user_id" = Option<Uuid>, Query, description = "Filter by user ID (admin/moderator only)"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("created_after" = Option<String>, Query, description = "Filter by creation date (ISO format)"),
        ("created_before" = Option<String>, Query, description = "Filter by creation date (ISO format)"),
        ("sort_by" = Option<String>, Query, description = "Sort by field: 'created_at', 'updated_at', 'title'"),
        ("sort_order" = Option<String>, Query, description = "Sort order: 'asc' or 'desc'")
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

    let __MODULE_NAME_PLURAL__ = __MODULE_NAME_PLURAL___services::list___MODULE_NAME_PLURAL__(&mut conn, user_id, params).await?;

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

/// Get __MODULE_NAME_PLURAL__ count
#[utoipa::path(
    get,
    path = "/__MODULE_NAME_PLURAL__/count",
    tag = "__MODULE_STRUCT__s",
    summary = "Get __MODULE_NAME_PLURAL__ count",
    description = "Get total count of __MODULE_NAME_PLURAL__ with optional filtering",
    params(
        ("search" = Option<String>, Query, description = "Search in title and content fields"),
        ("title" = Option<String>, Query, description = "Filter by exact title match"),
        ("user_id" = Option<Uuid>, Query, description = "Filter by user ID (admin/moderator only)"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("created_after" = Option<String>, Query, description = "Filter by creation date (ISO format)"),
        ("created_before" = Option<String>, Query, description = "Filter by creation date (ISO format)")
    ),
    responses(
        (status = 200, description = "__MODULE_STRUCT__ count", body = ApiResponse<__MODULE_STRUCT__CountResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get___MODULE_NAME_PLURAL___count(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<List__MODULE_STRUCT__sQuery>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__CountResponse>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Regular users can only count their own __MODULE_NAME_PLURAL__, admins/moderators can count all
    let user_id = match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            params.user_id // Admin and Moderator can filter by any user
        }
        crate::rbac::models::UserRole::User => {
            Some(auth_user.id) // Regular user sees only their own
        }
    };

    let count = __MODULE_NAME_PLURAL___services::count___MODULE_NAME_PLURAL__(&mut conn, user_id, params).await?;

    Ok(Json(ApiResponse::success(count)))
}

/// Bulk create __MODULE_NAME_PLURAL__
#[utoipa::path(
    post,
    path = "/__MODULE_NAME_PLURAL__/bulk",
    tag = "__MODULE_STRUCT__s",
    summary = "Bulk create __MODULE_NAME_PLURAL__",
    description = "Create multiple __MODULE_NAME_PLURAL__ in a single request",
    request_body = BulkCreate__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__s created", body = ApiResponse<Vec<__MODULE_STRUCT__Response>>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_create___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkCreate__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__Response>>>, Error> {
    if request.items.is_empty() {
        return Err(Error::BadRequest("No items provided for bulk creation".to_string()));
    }

    if request.items.len() > 100 {
        return Err(Error::BadRequest("Too many items for bulk creation (max: 100)".to_string()));
    }

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let __MODULE_NAME_PLURAL__ = __MODULE_NAME_PLURAL___services::bulk_create___MODULE_NAME_PLURAL__(&mut conn, auth_user.id, request.items).await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME_PLURAL__)))
}

/// Bulk update __MODULE_NAME_PLURAL__
#[utoipa::path(
    put,
    path = "/__MODULE_NAME_PLURAL__/bulk",
    tag = "__MODULE_STRUCT__s",
    summary = "Bulk update __MODULE_NAME_PLURAL__",
    description = "Update multiple __MODULE_NAME_PLURAL__ in a single request",
    request_body = BulkUpdate__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__s updated", body = ApiResponse<Vec<__MODULE_STRUCT__Response>>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_update___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkUpdate__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__Response>>>, Error> {
    if request.items.is_empty() {
        return Err(Error::BadRequest("No items provided for bulk update".to_string()));
    }

    if request.items.len() > 100 {
        return Err(Error::BadRequest("Too many items for bulk update (max: 100)".to_string()));
    }

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let __MODULE_NAME_PLURAL__ = __MODULE_NAME_PLURAL___services::bulk_update___MODULE_NAME_PLURAL__(&mut conn, auth_user.id, auth_user.role, request.items).await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME_PLURAL__)))
}

/// Bulk delete __MODULE_NAME_PLURAL__
#[utoipa::path(
    delete,
    path = "/__MODULE_NAME_PLURAL__/bulk",
    tag = "__MODULE_STRUCT__s",
    summary = "Bulk delete __MODULE_NAME_PLURAL__",
    description = "Delete multiple __MODULE_NAME_PLURAL__ in a single request (soft delete)",
    request_body = BulkDelete__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__s deleted", body = ApiResponse<String>),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn bulk_delete___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkDelete__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<String>>, Error> {
    if request.ids.is_empty() {
        return Err(Error::BadRequest("No IDs provided for bulk deletion".to_string()));
    }

    if request.ids.len() > 100 {
        return Err(Error::BadRequest("Too many items for bulk deletion (max: 100)".to_string()));
    }

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let deleted_count = __MODULE_NAME_PLURAL___services::bulk_delete___MODULE_NAME_PLURAL__(&mut conn, auth_user.id, auth_user.role, request.ids).await?;

    Ok(Json(ApiResponse::success_with_message(
        format!("Bulk deletion completed"),
        format!("{} __MODULE_NAME_PLURAL__ have been deactivated", deleted_count),
    )))
}

/// Search __MODULE_NAME_PLURAL__
#[utoipa::path(
    get,
    path = "/__MODULE_NAME_PLURAL__/search",
    tag = "__MODULE_STRUCT__s",
    summary = "Search __MODULE_NAME_PLURAL__",
    description = "Full-text search across __MODULE_NAME_PLURAL__ with advanced options",
    params(
        ("q" = String, Query, description = "Search query (required)"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip for pagination"),
        ("sort_by" = Option<String>, Query, description = "Sort by field: 'created_at', 'updated_at', 'title', 'relevance'"),
        ("sort_order" = Option<String>, Query, description = "Sort order: 'asc' or 'desc' (default: 'desc' for dates, 'desc' for relevance)")
    ),
    responses(
        (status = 200, description = "Search results", body = ApiResponse<Vec<__MODULE_STRUCT__Response>>),
        (status = 400, description = "Invalid search query", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn search___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<Search__MODULE_STRUCT__sQuery>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__Response>>>, Error> {
    if params.q.trim().is_empty() {
        return Err(Error::BadRequest("Search query cannot be empty".to_string()));
    }

    if params.q.len() < 2 {
        return Err(Error::BadRequest("Search query must be at least 2 characters".to_string()));
    }

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Regular users can only search their own __MODULE_NAME_PLURAL__, admins/moderators can search all
    let user_id = match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            None // Admin and Moderator search all
        }
        crate::rbac::models::UserRole::User => {
            Some(auth_user.id) // Regular user searches only their own
        }
    };

    let results = __MODULE_NAME_PLURAL___services::search___MODULE_NAME_PLURAL__(&mut conn, user_id, params).await?;

    Ok(Json(ApiResponse::success(results)))
}

/// Filter __MODULE_NAME_PLURAL__ with advanced criteria
#[utoipa::path(
    get,
    path = "/__MODULE_NAME_PLURAL__/filter",
    tag = "__MODULE_STRUCT__s",
    summary = "Filter __MODULE_NAME_PLURAL__",
    description = "Advanced filtering with multiple criteria",
    params(
        ("user_id" = Option<Uuid>, Query, description = "Filter by user ID (admin/moderator only)"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("created_after" = Option<String>, Query, description = "Filter by creation date (ISO format)"),
        ("created_before" = Option<String>, Query, description = "Filter by creation date (ISO format)"),
        ("title_contains" = Option<String>, Query, description = "Filter by title containing text"),
        ("limit" = Option<i64>, Query, description = "Maximum number of results (default: 50, max: 100)"),
        ("offset" = Option<i64>, Query, description = "Number of results to skip for pagination"),
        ("sort_by" = Option<String>, Query, description = "Sort by field: 'created_at', 'updated_at', 'title'"),
        ("sort_order" = Option<String>, Query, description = "Sort order: 'asc' or 'desc'")
    ),
    responses(
        (status = 200, description = "Filtered results", body = ApiResponse<Vec<__MODULE_STRUCT__Response>>),
        (status = 400, description = "Invalid filter criteria", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required for user_id filter", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn filter___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<Filter__MODULE_STRUCT__sQuery>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__Response>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Check permissions for user_id filter
    let filter_user_id = match auth_user.role {
        crate::rbac::models::UserRole::Admin | crate::rbac::models::UserRole::Moderator => {
            params.user_id // Admin and Moderator can filter by any user
        }
        crate::rbac::models::UserRole::User => {
            if params.user_id.is_some() && params.user_id != Some(auth_user.id) {
                return Err(Error::Forbidden("Cannot filter by other users".to_string()));
            }
            Some(auth_user.id) // Regular user can only filter their own
        }
    };

    let results = __MODULE_NAME_PLURAL___services::filter___MODULE_NAME_PLURAL__(&mut conn, filter_user_id, params).await?;

    Ok(Json(ApiResponse::success(results)))
}