use crate::{
    types::{AppState, ApiResponse},
    error::Error,
};
use crate::auth::{models::LoginRequest, services as auth_services, AuthUser};
use crate::users::models::CreateUserRequest;
use axum::{
    extract::{State, Extension},
    response::Json,
};

pub async fn login(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<crate::auth::models::LoginResponse>>, Error> {
    let mut conn = app_state.database.pool.acquire().await.map_err(Error::from_sqlx)?;
    let login_response = auth_services::login(&mut conn, payload).await?;
    Ok(Json(ApiResponse::success(login_response)))
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<crate::users::models::UserProfile>>, Error> {
    let mut conn = app_state.database.pool.acquire().await.map_err(Error::from_sqlx)?;
    let user_profile = auth_services::register(&mut conn, payload).await?;
    Ok(Json(ApiResponse::success(user_profile)))
}

pub async fn logout(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state.database.pool.acquire().await.map_err(Error::from_sqlx)?;
    let sessions_deleted = auth_services::logout_all(&mut conn, auth_user.id).await?;
    
    Ok(Json(ApiResponse::success_with_message(
        "Logged out successfully".to_string(),
        format!("Ended {} session(s)", sessions_deleted)
    )))
}

pub async fn logout_all(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let mut conn = app_state.database.pool.acquire().await.map_err(Error::from_sqlx)?;
    let sessions_deleted = auth_services::logout_all(&mut conn, auth_user.id).await?;
    
    Ok(Json(ApiResponse::success_with_message(
        "Logged out from all devices".to_string(),
        format!("Ended {} session(s)", sessions_deleted)
    )))
}

pub async fn me(
    Extension(auth_user): Extension<AuthUser>,
) -> Json<ApiResponse<AuthUser>> {
    Json(ApiResponse::success(auth_user))
}

pub async fn refresh(
    Extension(_auth_user): Extension<AuthUser>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success_with_message(
        "Token is still valid".to_string(),
        "Current session remains active".to_string()
    ))
}