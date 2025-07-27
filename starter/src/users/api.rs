use crate::auth::AuthUser;
use crate::users::{models::UserProfile, services as user_services};
use crate::{
    error::Error,
    types::{ApiResponse, AppState},
};
use axum::{
    extract::{Extension, Path, State},
    response::Json,
};
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

pub async fn get_user_by_id(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserProfile>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;
    let profile = user_services::get_user_profile(&mut conn, user_id).await?;

    match profile {
        Some(profile) => Ok(Json(ApiResponse::success(profile))),
        None => Err(Error::NotFound("User not found".to_string())),
    }
}
