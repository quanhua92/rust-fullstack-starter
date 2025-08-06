//! Testbasic API endpoints
//!
//! Provides CRUD operations for testbasic management.

use crate::{
    auth::AuthUser,
    rbac::services as rbac_services,
    testbasics::{models::*, services::*},
    types::{ApiResponse, Result},
    Database,
};
use axum::{
    extract::{Extension, Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for listing testbasics
#[derive(Deserialize)]
pub struct ListTestbasicQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub search: Option<String>,
}

/// API endpoints for testbasic management
pub fn testbasics_routes() -> Router<Database> {
    Router::new()
        .route("/", get(list_testbasics).post(create_testbasic))
        .route("/:id", get(get_testbasic).put(update_testbasic).delete(delete_testbasic))
}

/// List all testbasics
pub async fn list_testbasics(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListTestbasicQuery>,
) -> Result<Json<ApiResponse<Vec<Testbasic>>>> {
    // Basic authenticated access - user just needs to be logged in
    // Additional permissions can be added based on your business logic

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let testbasics = list_testbasics_service(
        &database,
        ListTestbasicRequest {
            limit,
            offset,
            search: query.search,
        },
    )
    .await?;

    Ok(Json(ApiResponse::success(testbasics)))
}

/// Get a specific testbasic
pub async fn get_testbasic(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Testbasic>>> {
    // Basic authenticated access - user just needs to be logged in
    // Additional permissions can be added based on your business logic

    let testbasic = get_testbasic_service(&database, id).await?;
    Ok(Json(ApiResponse::success(testbasic)))
}

/// Create a new testbasic
pub async fn create_testbasic(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateTestbasicRequest>,
) -> Result<Json<ApiResponse<Testbasic>>> {
    // Basic authenticated access - user just needs to be logged in
    // Additional permissions can be added based on your business logic

    let testbasic = create_testbasic_service(&database, request).await?;
    Ok(Json(ApiResponse::success(testbasic)))
}

/// Update an existing testbasic
pub async fn update_testbasic(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTestbasicRequest>,
) -> Result<Json<ApiResponse<Testbasic>>> {
    // Check permissions - require moderator or higher for updates
    rbac_services::require_moderator_or_higher(&auth_user)?;

    let testbasic = update_testbasic_service(&database, id, request).await?;
    Ok(Json(ApiResponse::success(testbasic)))
}

/// Delete a testbasic
pub async fn delete_testbasic(
    State(database): State<Database>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    // Check permissions - require moderator or higher for deletion
    rbac_services::require_moderator_or_higher(&auth_user)?;

    delete_testbasic_service(&database, id).await?;
    Ok(Json(ApiResponse::success(())))
}

