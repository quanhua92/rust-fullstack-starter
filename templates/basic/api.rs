//! __MODULE_STRUCT__ API endpoints
//!
//! Provides CRUD operations for __MODULE_NAME__ management.

use crate::{
    auth::AuthUser,
    rbac::services as rbac_services,
    __MODULE_NAME_PLURAL__::{models::*, services::*},
    types::{ApiResponse, Result},
    Database,
};
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{get, post, put, delete},
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
pub fn __MODULE_NAME_PLURAL___routes() -> Router<Database> {
    Router::new()
        .route("/", get(list___MODULE_NAME_PLURAL__).post(create___MODULE_NAME__))
        .route("/:id", get(get___MODULE_NAME__).put(update___MODULE_NAME__).delete(delete___MODULE_NAME__))
}

/// List all __MODULE_NAME_PLURAL__
pub async fn list___MODULE_NAME_PLURAL__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<List__MODULE_STRUCT__Query>,
) -> Result<Json<ApiResponse<Vec<__MODULE_STRUCT__>>>> {
    // Check permissions - users can list __MODULE_NAME_PLURAL__
    // rbac_services::require_user_or_higher(&auth_user)?;

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let __MODULE_NAME_PLURAL__ = list___MODULE_NAME_PLURAL___service(
        &database,
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
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Check permissions - users can get __MODULE_NAME_PLURAL__
    // rbac_services::require_user_or_higher(&auth_user)?;

    let __MODULE_NAME__ = get___MODULE_NAME___service(&database, id).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Create a new __MODULE_NAME__
pub async fn create___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<Create__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Check permissions - users can create __MODULE_NAME_PLURAL__
    // rbac_services::require_user_or_higher(&auth_user)?;

    let __MODULE_NAME__ = create___MODULE_NAME___service(&database, request).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Update an existing __MODULE_NAME__
pub async fn update___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<Update__MODULE_STRUCT__Request>,
) -> Result<Json<ApiResponse<__MODULE_STRUCT__>>> {
    // Check permissions - require moderator or higher for updates
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let __MODULE_NAME__ = update___MODULE_NAME___service(&database, id, request).await?;
    Ok(Json(ApiResponse::success(__MODULE_NAME__)))
}

/// Delete a __MODULE_NAME__
pub async fn delete___MODULE_NAME__(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    // Check permissions - require moderator or higher for deletion
    rbac_services::require_moderator_or_higher(&auth_user)?;

    delete___MODULE_NAME___service(&database, id).await?;
    Ok(Json(ApiResponse::success(())))
}

