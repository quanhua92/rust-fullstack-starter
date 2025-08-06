//! Products REST API endpoints with advanced features

#[allow(unused_imports)] // These are used in the routes function but compiler can't detect it
use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    products::{models::*, services::*},
    types::{ApiResponse, AppState, Result},
};

/// Create products router with all endpoints
pub fn products_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products))
        .route("/", post(create_products))
        .route("/bulk", post(bulk_create_products))
        .route("/bulk", put(bulk_update_products))
        .route("/bulk", delete(bulk_delete_products))
        .route("/{id}", get(get_products))
        .route("/{id}", put(update_products))
        .route("/{id}", delete(delete_products))
}

/// Query parameters for listing products
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListProductsQueryParams {
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

/// List products with advanced filtering and pagination
#[utoipa::path(
    get,
    path = "/api/v1/products",
    params(ListProductsQueryParams),
    responses(
        (status = 200, description = "List of products", body = ProductsListResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn list_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ListProductsQueryParams>,
) -> Result<Json<ApiResponse<ProductsListResponse>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    // Parse status filter
    let status = if let Some(status_str) = params.status {
        Some(
            status_str
                .split(',')
                .filter_map(|s| match s.trim().to_lowercase().as_str() {
                    "active" => Some(ProductsStatus::Active),
                    "inactive" => Some(ProductsStatus::Inactive),
                    "pending" => Some(ProductsStatus::Pending),
                    "archived" => Some(ProductsStatus::Archived),
                    _ => None,
                })
                .collect(),
        )
    } else {
        None
    };

    // Parse date filters
    let created_after =
        if let Some(date_str) = params.created_after {
            Some(date_str.parse().map_err(|_| {
                crate::error::Error::validation("created_after", "Invalid date format")
            })?)
        } else {
            None
        };

    let created_before = if let Some(date_str) = params.created_before {
        Some(date_str.parse().map_err(|_| {
            crate::error::Error::validation("created_before", "Invalid date format")
        })?)
    } else {
        None
    };

    // Parse sort parameters
    let sort_by = params
        .sort_by
        .and_then(|s| match s.to_lowercase().as_str() {
            "name" => Some(ProductsSortField::Name),
            "priority" => Some(ProductsSortField::Priority),
            "status" => Some(ProductsSortField::Status),
            "created_at" => Some(ProductsSortField::CreatedAt),
            "updated_at" => Some(ProductsSortField::UpdatedAt),
            _ => None,
        });

    let sort_order = params
        .sort_order
        .and_then(|s| match s.to_lowercase().as_str() {
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

    let request = ListProductsRequest {
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

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let response = list_products_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Get a specific products by ID
#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    params(
        ("id" = Uuid, Path, description = "Products ID")
    ),
    responses(
        (status = 200, description = "Products found", body = Products),
        (status = 404, description = "Products not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn get_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Products>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let products = get_products_service(&mut conn, id).await?;
    Ok(Json(ApiResponse::success(products)))
}

/// Create a new products
#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductsRequest,
    responses(
        (status = 201, description = "Products created", body = Products),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn create_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateProductsRequest>,
) -> Result<Json<ApiResponse<Products>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let products = create_products_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(products)))
}

/// Update an existing products
#[utoipa::path(
    put,
    path = "/api/v1/products/{id}",
    params(
        ("id" = Uuid, Path, description = "Products ID")
    ),
    request_body = UpdateProductsRequest,
    responses(
        (status = 200, description = "Products updated", body = Products),
        (status = 404, description = "Products not found"),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn update_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProductsRequest>,
) -> Result<Json<ApiResponse<Products>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let products = update_products_service(&mut conn, id, request).await?;
    Ok(Json(ApiResponse::success(products)))
}

/// Delete a products
#[utoipa::path(
    delete,
    path = "/api/v1/products/{id}",
    params(
        ("id" = Uuid, Path, description = "Products ID")
    ),
    responses(
        (status = 204, description = "Products deleted"),
        (status = 404, description = "Products not found"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn delete_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    delete_products_service(&mut conn, id).await?;
    Ok(Json(ApiResponse::success(())))
}

/// Bulk create products
#[utoipa::path(
    post,
    path = "/api/v1/products/bulk",
    request_body = BulkProductsCreateRequest,
    responses(
        (status = 200, description = "Bulk create results", body = BulkOperationResponse<Products>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn bulk_create_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkProductsCreateRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<Products>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let response = bulk_create_products_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Bulk update products
#[utoipa::path(
    put,
    path = "/api/v1/products/bulk",
    request_body = BulkProductsUpdateRequest,
    responses(
        (status = 200, description = "Bulk update results", body = BulkOperationResponse<Products>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn bulk_update_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkProductsUpdateRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<Products>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let response = bulk_update_products_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(response)))
}

/// Bulk delete products
#[utoipa::path(
    delete,
    path = "/api/v1/products/bulk",
    request_body = BulkProductsDeleteRequest,
    responses(
        (status = 200, description = "Bulk delete results", body = BulkOperationResponse<Uuid>),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "products"
)]
pub async fn bulk_delete_products(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<BulkProductsDeleteRequest>,
) -> Result<Json<ApiResponse<BulkOperationResponse<Uuid>>>> {
    // Authenticated access required - user must be logged in
    let _ = &auth_user; // Explicitly acknowledge the auth requirement
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let response = bulk_delete_products_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(response)))
}
