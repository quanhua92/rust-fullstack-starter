use crate::{
    types::{DbConn, Result},
    error::Error,
};
use crate::auth::models::{Session, LoginRequest, LoginResponse};
use crate::users::{models::UserProfile, services as user_services};
use uuid::Uuid;
use chrono::{Utc, Duration};

fn generate_session_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..64)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn create_session(
    conn: &mut DbConn,
    user_id: Uuid,
    user_agent: Option<&str>,
) -> Result<Session> {
    let token = generate_session_token();
    let expires_at = Utc::now() + Duration::hours(24);
    
    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at, user_agent)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, token, expires_at, created_at, updated_at,
                  last_activity_at, user_agent, is_active
        "#,
        user_id,
        token,
        expires_at,
        user_agent
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    let session = Session {
        id: session.id,
        user_id: session.user_id,
        token: session.token,
        expires_at: session.expires_at,
        created_at: session.created_at,
        updated_at: session.updated_at,
        last_activity_at: Some(session.last_activity_at),
        user_agent: session.user_agent,
        is_active: session.is_active,
    };
    
    Ok(session)
}

pub async fn find_session_by_token(
    conn: &mut DbConn,
    token: &str,
) -> Result<Option<Session>> {
    let session = sqlx::query!(
        r#"
        SELECT id, user_id, token, expires_at, created_at, updated_at,
               last_activity_at, user_agent, is_active
        FROM sessions 
        WHERE token = $1 AND is_active = true
        "#,
        token
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    let session = session.map(|s| Session {
        id: s.id,
        user_id: s.user_id,
        token: s.token,
        expires_at: s.expires_at,
        created_at: s.created_at,
        updated_at: s.updated_at,
        last_activity_at: Some(s.last_activity_at),
        user_agent: s.user_agent,
        is_active: s.is_active,
    });
    
    Ok(session)
}

pub async fn update_session_activity(
    conn: &mut DbConn,
    session_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        "UPDATE sessions SET last_activity_at = NOW() WHERE id = $1",
        session_id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    Ok(())
}

pub async fn delete_session(
    conn: &mut DbConn,
    token: &str,
) -> Result<()> {
    sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE token = $1",
        token
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    Ok(())
}

pub async fn delete_all_user_sessions(
    conn: &mut DbConn,
    user_id: Uuid,
) -> Result<u64> {
    let result = sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    Ok(result.rows_affected())
}

pub async fn cleanup_expired_sessions(
    conn: &mut DbConn,
) -> Result<u64> {
    let result = sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE expires_at < NOW() AND is_active = true"
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;
    
    Ok(result.rows_affected())
}

pub async fn validate_session_with_user(
    conn: &mut DbConn,
    token: &str,
) -> Result<Option<crate::users::models::User>> {
    let session = find_session_by_token(conn, token).await?;
    
    if let Some(session) = session {
        if !session.is_expired() {
            update_session_activity(conn, session.id).await?;
            let user = user_services::find_user_by_id(conn, session.user_id).await?;
            return Ok(user);
        }
    }
    
    Ok(None)
}

pub async fn login(
    conn: &mut DbConn,
    req: LoginRequest,
) -> Result<LoginResponse> {
    req.validate()?;
    
    let user = if req.username_or_email.contains('@') {
        user_services::find_user_by_email(conn, &req.username_or_email).await?
    } else {
        user_services::find_user_by_username(conn, &req.username_or_email).await?
    };

    let user = user.ok_or(Error::InvalidCredentials)?;

    if !user_services::verify_password(&req.password, &user.password_hash)? {
        return Err(Error::InvalidCredentials);
    }

    if !user.is_active {
        return Err(Error::InvalidCredentials);
    }

    let session = create_session(
        conn,
        user.id,
        req.user_agent.as_deref(),
    ).await?;
    
    user_services::update_last_login(conn, user.id).await?;
    
    Ok(LoginResponse {
        session_token: session.token,
        expires_at: session.expires_at,
        user: user.to_profile(),
    })
}

pub async fn logout(
    conn: &mut DbConn,
    token: &str,
) -> Result<()> {
    delete_session(conn, token).await
}

pub async fn logout_all(
    conn: &mut DbConn,
    user_id: Uuid,
) -> Result<u64> {
    delete_all_user_sessions(conn, user_id).await
}

pub async fn get_current_user(
    conn: &mut DbConn,
    token: &str,
) -> Result<Option<UserProfile>> {
    let user = validate_session_with_user(conn, token).await?;
    Ok(user.map(|u| u.to_profile()))
}

pub async fn register(
    conn: &mut DbConn,
    req: crate::users::models::CreateUserRequest,
) -> Result<UserProfile> {
    user_services::create_user(conn, req).await
}