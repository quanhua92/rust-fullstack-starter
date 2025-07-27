use crate::users::models::{CreateUserRequest, User, UserProfile};
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

pub async fn find_user_by_email(conn: &mut DbConn, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, 
               role, is_active, email_verified,
               created_at, updated_at, last_login_at
        FROM users 
        WHERE email = $1 AND is_active = true
        "#,
        email
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(user)
}

pub async fn find_user_by_username(conn: &mut DbConn, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role, is_active, email_verified,
               created_at, updated_at, last_login_at
        FROM users 
        WHERE username = $1 AND is_active = true
        "#,
        username
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(user)
}

pub async fn find_user_by_id(conn: &mut DbConn, user_id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role, is_active, email_verified,
               created_at, updated_at, last_login_at
        FROM users 
        WHERE id = $1 AND is_active = true
        "#,
        user_id
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(user)
}

pub async fn create_user(conn: &mut DbConn, req: CreateUserRequest) -> Result<UserProfile> {
    req.validate()?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)?
        .to_string();

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, password_hash,
                  role, is_active, email_verified,
                  created_at, updated_at, last_login_at
        "#,
        req.username,
        req.email,
        password_hash,
        req.role.unwrap_or_else(|| "user".to_string())
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(user.to_profile())
}

pub async fn update_last_login(conn: &mut DbConn, user_id: Uuid) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET last_login_at = NOW() WHERE id = $1",
        user_id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn get_user_profile(conn: &mut DbConn, user_id: Uuid) -> Result<Option<UserProfile>> {
    let user = find_user_by_id(conn, user_id).await?;
    Ok(user.map(|u| u.to_profile()))
}

pub async fn is_email_available(conn: &mut DbConn, email: &str) -> Result<bool> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE email = $1", email)
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(count.unwrap_or(0) == 0)
}

pub async fn is_username_available(conn: &mut DbConn, username: &str) -> Result<bool> {
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE username = $1", username)
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(count.unwrap_or(0) == 0)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| Error::Internal("Invalid password hash".to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
