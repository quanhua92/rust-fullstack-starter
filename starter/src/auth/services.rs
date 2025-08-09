use crate::auth::models::{LoginRequest, LoginResponse, RegisterRequest, Session};
use crate::users::{models::UserProfile, services as user_services};
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use chrono::{Duration, Utc};
use sqlx::Acquire;
use uuid::Uuid;

// Dummy hash with valid Argon2 format for timing attack protection.
// Using a validly formatted hash is crucial to prevent the password verification
// function from returning early due to a parsing error, which would reintroduce
// a timing vulnerability. This is a pre-computed Argon2 hash that will always fail verification.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$L2QVZ8LBhz/3BLvW+hBf1e4NkLYBu+GeBxdJJ1+BW5Q";

fn generate_session_token() -> String {
    use base64::Engine;
    use rand::Rng;

    // Use cryptographically secure random number generator
    // Generate 48 random bytes for 384 bits of entropy
    let mut rng = rand::rng();
    let bytes: [u8; 48] = rng.random();

    // Use URL-safe base64 encoding (no padding, URL safe)
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
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
                  last_activity_at, last_refreshed_at, user_agent, is_active
        "#,
        user_id,
        token,
        expires_at,
        user_agent
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    let session = Session {
        id: session.id,
        user_id: session.user_id,
        token: session.token,
        expires_at: session.expires_at,
        created_at: session.created_at,
        updated_at: session.updated_at,
        last_activity_at: session.last_activity_at,
        last_refreshed_at: session.last_refreshed_at,
        user_agent: session.user_agent,
        is_active: session.is_active,
    };

    Ok(session)
}

pub async fn find_session_by_token(conn: &mut DbConn, token: &str) -> Result<Option<Session>> {
    let session = sqlx::query!(
        r#"
        SELECT id, user_id, token, expires_at, created_at, updated_at,
               last_activity_at, last_refreshed_at, user_agent, is_active
        FROM sessions 
        WHERE token = $1 AND is_active = true
        "#,
        token
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    let session = session.map(|s| Session {
        id: s.id,
        user_id: s.user_id,
        token: s.token,
        expires_at: s.expires_at,
        created_at: s.created_at,
        updated_at: s.updated_at,
        last_activity_at: s.last_activity_at,
        last_refreshed_at: s.last_refreshed_at,
        user_agent: s.user_agent,
        is_active: s.is_active,
    });

    Ok(session)
}

pub async fn update_session_activity(conn: &mut DbConn, session_id: Uuid) -> Result<()> {
    sqlx::query!(
        "UPDATE sessions SET last_activity_at = NOW() WHERE id = $1",
        session_id
    )
    .execute(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn delete_session(conn: &mut DbConn, token: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE token = $1",
        token
    )
    .execute(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn delete_all_user_sessions(conn: &mut DbConn, user_id: Uuid) -> Result<u64> {
    let result = sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .execute(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(result.rows_affected())
}

pub async fn cleanup_expired_sessions(conn: &mut DbConn) -> Result<u64> {
    let result = sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE expires_at < NOW() AND is_active = true"
    )
    .execute(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(result.rows_affected())
}

pub async fn validate_session_with_user(
    conn: &mut DbConn,
    token: &str,
) -> Result<Option<crate::users::models::User>> {
    let session = find_session_by_token(conn, token).await?;

    if let Some(session) = session
        && !session.is_expired()
    {
        update_session_activity(conn, session.id).await?;
        let user = user_services::find_user_by_id(conn, session.user_id).await?;
        return Ok(user);
    }

    Ok(None)
}

pub async fn login(conn: &mut DbConn, req: LoginRequest) -> Result<LoginResponse> {
    req.validate()?;

    // Always perform the same amount of work regardless of user existence to prevent timing attacks
    let user_option = match (&req.username, &req.email) {
        (Some(username), None) => user_services::find_user_by_username(conn, username).await?,
        (None, Some(email)) => user_services::find_user_by_email(conn, email).await?,
        _ => {
            // This should never happen due to validation, but handle it gracefully
            return Err(Error::InvalidCredentials);
        }
    };

    // Always verify password, even if user doesn't exist (using dummy hash)
    let password_valid = match &user_option {
        Some(user) => user_services::verify_password(&req.password, &user.password_hash)?,
        None => {
            // Perform dummy verification to maintain constant timing
            // We must handle the result to ensure the computation actually happens
            match user_services::verify_password(&req.password, DUMMY_HASH) {
                Ok(_) => false,  // Always false for non-existent users
                Err(_) => false, // Even if dummy hash fails to parse, return false
            }
        }
    };

    // Check all conditions after password verification
    let user = user_option.ok_or(Error::InvalidCredentials)?;
    if !password_valid || !user.is_active {
        return Err(Error::InvalidCredentials);
    }

    let mut tx = conn.begin().await.map_err(Error::from_sqlx)?;

    // Fix session fixation: Only invalidate sessions older than 30 days
    sqlx::query!(
        "UPDATE sessions SET is_active = false
         WHERE user_id = $1
         AND is_active = true
         AND last_activity_at < NOW() - INTERVAL '30 days'",
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    // Create session within transaction
    let token = generate_session_token();
    let expires_at = Utc::now() + Duration::hours(24);

    let session = sqlx::query!(
        r#"
        INSERT INTO sessions (user_id, token, expires_at, user_agent)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, token, expires_at, created_at, updated_at,
                  last_activity_at, last_refreshed_at, user_agent, is_active
        "#,
        user.id,
        token,
        expires_at,
        req.user_agent.as_deref()
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    // Update last login within transaction
    sqlx::query!(
        "UPDATE users SET last_login_at = NOW() WHERE id = $1",
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    tx.commit().await.map_err(Error::from_sqlx)?;

    let session = Session {
        id: session.id,
        user_id: session.user_id,
        token: session.token,
        expires_at: session.expires_at,
        created_at: session.created_at,
        updated_at: session.updated_at,
        last_activity_at: session.last_activity_at,
        last_refreshed_at: session.last_refreshed_at,
        user_agent: session.user_agent,
        is_active: session.is_active,
    };

    Ok(LoginResponse {
        session_token: session.token,
        expires_at: session.expires_at,
        user: user.to_profile(),
    })
}

pub async fn logout(conn: &mut DbConn, token: &str) -> Result<()> {
    delete_session(conn, token).await
}

pub async fn logout_all(conn: &mut DbConn, user_id: Uuid) -> Result<u64> {
    delete_all_user_sessions(conn, user_id).await
}

pub async fn get_current_user(conn: &mut DbConn, token: &str) -> Result<Option<UserProfile>> {
    let user = validate_session_with_user(conn, token).await?;
    Ok(user.map(|u| u.to_profile()))
}

/// Refresh a session token by extending its expiration time
pub async fn refresh_session_token(
    conn: &mut DbConn,
    token: &str,
    extend_hours: Option<i64>,
    min_refresh_interval_minutes: Option<i64>,
) -> Result<Option<Session>> {
    let extend_hours = extend_hours.unwrap_or(24); // Default 24 hours extension
    let min_refresh_interval = min_refresh_interval_minutes.unwrap_or(5); // Default 5 minutes minimum between refreshes

    // Find the current session
    let session = find_session_by_token(conn, token).await?;

    if let Some(session) = session {
        // Check if session can be refreshed
        if !session.can_refresh(min_refresh_interval) {
            return Ok(None); // Cannot refresh yet
        }

        let new_expires_at = session.calculate_refresh_expiration(extend_hours);
        let now = Utc::now();

        // Update the session with new expiration and refresh timestamp
        let updated_session = sqlx::query!(
            r#"
            UPDATE sessions 
            SET expires_at = $1, last_refreshed_at = $2, updated_at = $2
            WHERE token = $3 AND is_active = true
            RETURNING id, user_id, token, expires_at, created_at, updated_at,
                      last_activity_at, last_refreshed_at, user_agent, is_active
            "#,
            new_expires_at,
            now,
            token
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(Error::from_sqlx)?;

        if let Some(s) = updated_session {
            let refreshed_session = Session {
                id: s.id,
                user_id: s.user_id,
                token: s.token,
                expires_at: s.expires_at,
                created_at: s.created_at,
                updated_at: s.updated_at,
                last_activity_at: s.last_activity_at,
                last_refreshed_at: s.last_refreshed_at,
                user_agent: s.user_agent,
                is_active: s.is_active,
            };
            return Ok(Some(refreshed_session));
        }
    }

    Ok(None)
}

pub async fn register(conn: &mut DbConn, req: RegisterRequest) -> Result<UserProfile> {
    req.validate()?;

    // Convert RegisterRequest to CreateUserRequest
    let create_req = crate::users::models::CreateUserRequest {
        username: req.username,
        email: req.email,
        password: req.password,
        role: None, // Default role
    };

    user_services::create_user(conn, create_req).await
}
