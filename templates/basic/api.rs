//! __MODULE_STRUCT__ API endpoints
//!
//! Provides CRUD operations for __MODULE_NAME__ management.
//!
//! ## Pattern Note
//! Each handler function acquires a database connection using the same pattern:
//! ```rust,ignore
//! let mut conn = app_state.database.pool.acquire().await?;
//! ```
//! In a larger application, you might want to extract this into a helper function
//! or middleware to reduce boilerplate code.

use crate::{
    auth::AuthUser,
    rbac::services as rbac_services,
    __MODULE_NAME_PLURAL__::{models::*, services::*},
    types::{ApiResponse, AppState, Result},
};
#[allow(unused_imports)] // These are used in the routes function but compiler can't detect it
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for listing __MODULE_NAME_PLURAL__
#[derive(Deserialize)]
pub struct List__MODULE_STRUCT__Query {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub search: Option<String>,
}

/// API endpoints for __MODULE_NAME__ management
pub fn __MODULE_NAME_PLURAL___routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list___MODULE_NAME_PLURAL__).post(create___MODULE_NAME__))
        .route("/{id}", get(get___MODULE_NAME__).put(update___MODULE_NAME__).delete(delete___MODULE_NAME__))
}

/// List all __MODULE_NAME_PLURAL__
pub async fn list___MODULE_NAME_PLURAL__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<List__MODULE_STRUCT__Query>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__>>>> {
    // Basic authenticated access - user just needs to be logged in
    // The auth_user parameter ensures user is authenticated
    let _ = &auth_user; // Explicitly acknowledge the auth requirement

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let __MODULE_NAME_PLURAL__ = list___MODULE_NAME_PLURAL___service(
        conn.as_mut(),
        List__MODULE_STRUCT__Request {
            limit,
            offset,
            search: query.search,
        },
    )
    .await?;

    Ok(Json(ApiResponse::success(__MODULE_NAME_PLURAL__)))
}

/// Get a specific __MODULE_NAME__
pub async fn get___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Basic authenticated access - user just needs to be logged in
    // The auth_user parameter ensures user is authenticated
    let _ = &auth_user; // Explicitly acknowledge the auth requirement

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let __MODULE_NAME__ = get___MODULE_NAME___service(conn.as_mut(), id).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Create a new __MODULE_NAME__
pub async fn create___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Create__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Basic authenticated access - user just needs to be logged in
    // The auth_user parameter ensures user is authenticated
    let _ = &auth_user; // Explicitly acknowledge the auth requirement

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let __MODULE_NAME__ = create___MODULE_NAME___service(conn.as_mut(), request, auth_user.id).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Update an existing __MODULE_NAME__
pub async fn update___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<Update__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    let mut tx = app_state
        .database
        .pool
        .begin()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    // First get the item to check ownership
    let existing_item = get___MODULE_NAME___service(&mut tx, id).await?;
    
    // Check RBAC authorization - Admin/Moderator can update any item, users only their own
    rbac_services::can_access_own_resource(&auth_user, existing_item.created_by)?;

    let __MODULE_NAME__ = update___MODULE_NAME___service(&mut tx, id, request).await?;
    
    tx.commit()
        .await
        .map_err(crate::error::Error::from_sqlx)?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Delete a __MODULE_NAME__
pub async fn delete___MODULE_NAME__(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let mut tx = app_state
        .database
        .pool
        .begin()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    // First get the item to check ownership
    let existing_item = get___MODULE_NAME___service(&mut tx, id).await?;
    
    // Check RBAC authorization - Admin/Moderator can delete any item, users only their own
    rbac_services::can_access_own_resource(&auth_user, existing_item.created_by)?;

    delete___MODULE_NAME___service(&mut tx, id).await?;
    
    tx.commit()
        .await
        .map_err(crate::error::Error::from_sqlx)?;
    Ok(Json(ApiResponse::success(())))
}

