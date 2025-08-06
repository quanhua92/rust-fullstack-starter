//! Basics API endpoints
//!
//! Provides CRUD operations for basics management.

use crate::{
    auth::AuthUser,
    rbac::services as rbac_services,
    basics::{models::*, services::*},
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

/// Query parameters for listing basics
#[derive(Deserialize)]
pub struct ListBasicsQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub search: Option<String>,
}

/// API endpoints for basics management
pub fn basics_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_basics).post(create_basics))
        .route("/{id}", get(get_basics).put(update_basics).delete(delete_basics))
}

/// List all basics
pub async fn list_basics(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListBasicsQuery>,
) -> Result<Json<ApiResponse<Vec<Basics>>>> {
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

    let basics = list_basics_service(
        &mut conn,
        ListBasicsRequest {
            limit,
            offset,
            search: query.search,
        },
    )
    .await?;

    Ok(Json(ApiResponse::success(basics)))
}

/// Get a specific basics
pub async fn get_basics(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Basics>>> {
    // Basic authenticated access - user just needs to be logged in
    // The auth_user parameter ensures user is authenticated
    let _ = &auth_user; // Explicitly acknowledge the auth requirement

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let basics = get_basics_service(&mut conn, id).await?;
    Ok(Json(ApiResponse::success(basics)))
}

/// Create a new basics
pub async fn create_basics(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateBasicsRequest>,
) -> Result<Json<ApiResponse<Basics>>> {
    // Basic authenticated access - user just needs to be logged in
    // The auth_user parameter ensures user is authenticated
    let _ = &auth_user; // Explicitly acknowledge the auth requirement

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let basics = create_basics_service(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(basics)))
}

/// Update an existing basics
pub async fn update_basics(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateBasicsRequest>,
) -> Result<Json<ApiResponse<Basics>>> {
    // Check permissions - require moderator or higher for updates
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    let basics = update_basics_service(&mut conn, id, request).await?;
    Ok(Json(ApiResponse::success(basics)))
}

/// Delete a basics
pub async fn delete_basics(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    // Check permissions - require moderator or higher for deletion
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(crate::error::Error::from_sqlx)?;

    delete_basics_service(&mut conn, id).await?;
    Ok(Json(ApiResponse::success(())))
}

