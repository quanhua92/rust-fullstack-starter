//! __MODULE_STRUCT__ REST API endpoints with advanced features

#[allow(unused_imports)] // These are used in the routes function but compiler can't detect it
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::Deserialize;
use uuid::Uuid;
use utoipa::IntoParams;

use crate::{
    auth::AuthUser,
    __MODULE_NAME_PLURAL__::{models::*, services::*},
    types::{ApiResponse, Result},
    Database,
};

/// Create __MODULE_NAME_PLURAL__ router with all endpoints
pub fn __MODULE_NAME_PLURAL___routes() -> Router<Database> {
    Router::new()
        .route("/", get(list___MODULE_NAME_PLURAL__))
        .route("/", post(create___MODULE_NAME__))
        .route("/bulk", post(bulk_create___MODULE_NAME_PLURAL__))
        .route("/bulk", put(bulk_update___MODULE_NAME_PLURAL__))
        .route("/bulk", delete(bulk_delete___MODULE_NAME_PLURAL__))
        .route("/:id", get(get___MODULE_NAME__))
        .route("/:id", put(update___MODULE_NAME__))
        .route("/:id", delete(delete___MODULE_NAME__))
}

/// Query parameters for listing __MODULE_NAME_PLURAL__
#[derive(Debug, Deserialize, IntoParams)]
pub struct List__MODULE_STRUCT__QueryParams {
    /// Number of items per page (max 100)
    pub limit: Option<i32>,
    /// Page offset (0-based)
    pub offset: Option<i32>,
    /// Cursor for pagination (alternative to offset)
    pub cursor: Option<String>,
    /// Text search in name and description
    pub search: Option<String>,
    /// Filter by status (comma-separated)
    pub status: Option<String>,
    /// Filter by priority range
    pub min_priority: Option<i32>,
    pub max_priority: Option<i32>,
    /// Filter by creation date range (ISO 8601)
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    /// Sort field and direction
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// List __MODULE_NAME_PLURAL__ with advanced filtering and pagination
#[utoipa::path(
    get,
    path = "/api/v1/__MODULE_NAME_PLURAL__",
    params(List__MODULE_STRUCT__QueryParams),
    responses(
        (status = 200, description = "List of __MODULE_NAME_PLURAL__", body = __MODULE_STRUCT__ListResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn list___MODULE_NAME_PLURAL__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<List__MODULE_STRUCT__QueryParams>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__ListResponse>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    // Parse status filter
    let status = if let Some(status_str) = params.status {
        Some(
            status_str
                .split(',')
                .filter_map(|s| match s.trim().to_lowercase().as_str() {
                    "active" => Some(__MODULE_STRUCT__Status::Active),
                    "inactive" => Some(__MODULE_STRUCT__Status::Inactive),
                    "pending" => Some(__MODULE_STRUCT__Status::Pending),
                    "archived" => Some(__MODULE_STRUCT__Status::Archived),
                    _ => None,
                })
                .collect(),
        )
    } else {
        None
    };

    // Parse date filters
    let created_after = if let Some(date_str) = params.created_after {
        Some(
            date_str
                .parse()
                .map_err(|_| crate::error::Error::validation("created_after", "Invalid date format"))?,
        )
    } else {
        None
    };

    let created_before = if let Some(date_str) = params.created_before {
        Some(
            date_str
                .parse()
                .map_err(|_| crate::error::Error::validation("created_before", "Invalid date format"))?,
        )
    } else {
        None
    };

    // Parse sort parameters
    let sort_by = params.sort_by.and_then(|s| match s.to_lowercase().as_str() {
        "name" => Some(__MODULE_STRUCT__SortField::Name),
        "priority" => Some(__MODULE_STRUCT__SortField::Priority),
        "status" => Some(__MODULE_STRUCT__SortField::Status),
        "created_at" => Some(__MODULE_STRUCT__SortField::CreatedAt),
        "updated_at" => Some(__MODULE_STRUCT__SortField::UpdatedAt),
        _ => None,
    });

    let sort_order = params.sort_order.and_then(|s| match s.to_lowercase().as_str() {
        "asc" => Some(SortOrder::Asc),
        "desc" => Some(SortOrder::Desc),
        _ => None,
    });

    // Handle cursor-based pagination
    let (offset, cursor) = if let Some(cursor_str) = params.cursor.clone() {
        (Some(parse_cursor(&cursor_str)? as i32), Some(cursor_str))
    } else {
        (params.offset, None)
    };

    let request = List__MODULE_STRUCT__Request {
        limit: params.limit,
        offset,
        cursor,
        search: params.search,
        status,
        min_priority: params.min_priority,
        max_priority: params.max_priority,
        created_after,
        created_before,
        sort_by,
        sort_order,
    };

    let response = list___MODULE_NAME_PLURAL___service(&database, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Get a specific __MODULE_NAME__ by ID
#[utoipa::path(
    get,
    path = "/api/v1/__MODULE_NAME_PLURAL__/{id}",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    responses(
        (status = 200, description = "__MODULE_STRUCT__ found", body = __MODULE_STRUCT__),
        (status = 404, description = "__MODULE_STRUCT__ not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn get___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let __MODULE_NAME__ = get___MODULE_NAME___service(&database, id).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Create a new __MODULE_NAME__
#[utoipa::path(
    post,
    path = "/api/v1/__MODULE_NAME_PLURAL__",
    request_body = Create__MODULE_STRUCT__Request,
    responses(
        (status = 201, description = "__MODULE_STRUCT__ created", body = __MODULE_STRUCT__),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn create___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Create__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let __MODULE_NAME__ = create___MODULE_NAME___service(&database, request).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Update an existing __MODULE_NAME__
#[utoipa::path(
    put,
    path = "/api/v1/__MODULE_NAME_PLURAL__/{id}",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    request_body = Update__MODULE_STRUCT__Request,
    responses(
        (status = 200, description = "__MODULE_STRUCT__ updated", body = __MODULE_STRUCT__),
        (status = 404, description = "__MODULE_STRUCT__ not found"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn update___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<Update__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let __MODULE_NAME__ = update___MODULE_NAME___service(&database, id, request).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Delete a __MODULE_NAME__
#[utoipa::path(
    delete,
    path = "/api/v1/__MODULE_NAME_PLURAL__/{id}",
    params(
        ("id" = Uuid, Path, description = "__MODULE_STRUCT__ ID")
    ),
    responses(
        (status = 204, description = "__MODULE_STRUCT__ deleted"),
        (status = 404, description = "__MODULE_STRUCT__ not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn delete___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    delete___MODULE_NAME___service(&database, id).await?;
    Ok(Json(ApiResponse::success(())))
}

/// Bulk create __MODULE_NAME_PLURAL__
#[utoipa::path(
    post,
    path = "/api/v1/__MODULE_NAME_PLURAL__/bulk",
    request_body = Bulk__MODULE_STRUCT__CreateRequest,
    responses(
        (status = 200, description = "Bulk create results", body = BulkOperationResponse<__MODULE_STRUCT__>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn bulk_create___MODULE_NAME_PLURAL__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Bulk__MODULE_STRUCT__CreateRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<__MODULE_STRUCT__>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let response = bulk_create___MODULE_NAME_PLURAL___service(&database, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Bulk update __MODULE_NAME_PLURAL__
#[utoipa::path(
    put,
    path = "/api/v1/__MODULE_NAME_PLURAL__/bulk",
    request_body = Bulk__MODULE_STRUCT__UpdateRequest,
    responses(
        (status = 200, description = "Bulk update results", body = BulkOperationResponse<__MODULE_STRUCT__>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn bulk_update___MODULE_NAME_PLURAL__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Bulk__MODULE_STRUCT__UpdateRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<__MODULE_STRUCT__>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let response = bulk_update___MODULE_NAME_PLURAL___service(&database, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Bulk delete __MODULE_NAME_PLURAL__
#[utoipa::path(
    delete,
    path = "/api/v1/__MODULE_NAME_PLURAL__/bulk",
    request_body = Bulk__MODULE_STRUCT__DeleteRequest,
    responses(
        (status = 200, description = "Bulk delete results", body = BulkOperationResponse<Uuid>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "__MODULE_NAME_PLURAL__"
)]
pub async fn bulk_delete___MODULE_NAME_PLURAL__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Bulk__MODULE_STRUCT__DeleteRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<Uuid>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let response = bulk_delete___MODULE_NAME_PLURAL___service(&database, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

