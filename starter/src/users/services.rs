use crate::rbac::UserRole;
use crate::users::models::{CreateUserRequest, User, UserProfile};
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use sqlx::Acquire;
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
        req.role.unwrap_or(UserRole::User).to_string()
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

pub async fn list_users(
    conn: &mut DbConn,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<UserProfile>> {
    let limit = limit.unwrap_or(50).min(100); // Default 50, max 100
    let offset = offset.unwrap_or(0);

    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash,
               role, is_active, email_verified,
               created_at, updated_at, last_login_at
        FROM users 
        WHERE is_active = true
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(users.into_iter().map(|u| u.to_profile()).collect())
}

// New service functions for user management

pub async fn update_user_profile(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::UpdateProfileRequest,
) -> Result<UserProfile> {
    req.validate()?;

    // Update user profile
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET username = COALESCE($2, username),
            email = COALESCE($3, email),
            email_verified = CASE 
                WHEN $3 IS NOT NULL AND $3 != email THEN false 
                ELSE email_verified 
            END,
            updated_at = NOW()
        WHERE id = $1 AND is_active = true
        RETURNING id, username, email, password_hash,
                  role, is_active, email_verified,
                  created_at, updated_at, last_login_at
        "#,
        user_id,
        req.username,
        req.email
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    match user {
        Some(user) => Ok(user.to_profile()),
        None => Err(Error::NotFound("User not found".to_string())),
    }
}

pub async fn change_user_password(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::ChangePasswordRequest,
) -> Result<()> {
    req.validate()?;

    // Get current user to verify password
    let user = find_user_by_id(conn, user_id).await?;
    let user = match user {
        Some(user) => user,
        None => return Err(Error::NotFound("User not found".to_string())),
    };

    // Verify current password
    if !verify_password(&req.current_password, &user.password_hash)? {
        return Err(Error::InvalidCredentials);
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password_hash = argon2
        .hash_password(req.new_password.as_bytes(), &salt)?
        .to_string();

    // Update password
    sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
        new_password_hash,
        user_id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn delete_user_account(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::DeleteAccountRequest,
) -> Result<()> {
    req.validate()?;

    // Get current user to verify password
    let user = find_user_by_id(conn, user_id).await?;
    let user = match user {
        Some(user) => user,
        None => return Err(Error::NotFound("User not found".to_string())),
    };

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(Error::validation("password", "Invalid password"));
    }

    let mut tx = conn.begin().await.map_err(Error::from_sqlx)?;

    // Soft delete user (deactivate)
    sqlx::query!(
        "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    // Invalidate all user sessions
    sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    tx.commit().await.map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn update_user_profile_admin(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::UpdateUserProfileRequest,
) -> Result<UserProfile> {
    req.validate()?;

    // Update user profile (admin can update email_verified)
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET username = COALESCE($2, username),
            email = COALESCE($3, email),
            email_verified = COALESCE($4, email_verified),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, username, email, password_hash,
                  role, is_active, email_verified,
                  created_at, updated_at, last_login_at
        "#,
        user_id,
        req.username,
        req.email,
        req.email_verified
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    match user {
        Some(user) => Ok(user.to_profile()),
        None => Err(Error::NotFound("User not found".to_string())),
    }
}

pub async fn update_user_status(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::UpdateUserStatusRequest,
) -> Result<UserProfile> {
    let mut tx = conn.begin().await.map_err(Error::from_sqlx)?;

    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET is_active = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, username, email, password_hash,
                  role, is_active, email_verified,
                  created_at, updated_at, last_login_at
        "#,
        user_id,
        req.is_active
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    match user {
        Some(user) => {
            // If deactivating user, invalidate all sessions
            if !req.is_active {
                sqlx::query!(
                    "UPDATE sessions SET is_active = false WHERE user_id = $1",
                    user_id
                )
                .execute(&mut *tx)
                .await
                .map_err(Error::from_sqlx)?;
            }

            tx.commit().await.map_err(Error::from_sqlx)?;
            Ok(user.to_profile())
        }
        None => {
            tx.rollback().await.ok(); // Explicit rollback, ignore error
            Err(Error::NotFound("User not found".to_string()))
        }
    }
}

pub async fn update_user_role(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::UpdateUserRoleRequest,
) -> Result<UserProfile> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET role = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, username, email, password_hash,
                  role, is_active, email_verified,
                  created_at, updated_at, last_login_at
        "#,
        user_id,
        req.role.to_string()
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    match user {
        Some(user) => Ok(user.to_profile()),
        None => Err(Error::NotFound("User not found".to_string())),
    }
}

pub async fn reset_user_password(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::ResetPasswordRequest,
) -> Result<()> {
    req.validate()?;

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password_hash = argon2
        .hash_password(req.new_password.as_bytes(), &salt)?
        .to_string();

    let mut tx = conn.begin().await.map_err(Error::from_sqlx)?;

    // Update password
    sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
        new_password_hash,
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    // Invalidate all user sessions (force re-login)
    sqlx::query!(
        "UPDATE sessions SET is_active = false WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(Error::from_sqlx)?;

    tx.commit().await.map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn delete_user_admin(
    conn: &mut DbConn,
    user_id: Uuid,
    req: crate::users::models::DeleteUserRequest,
) -> Result<()> {
    let hard_delete = req.hard_delete.unwrap_or(false);

    let mut tx = conn.begin().await.map_err(Error::from_sqlx)?;

    if hard_delete {
        // Hard delete - permanently remove user and all related data
        // First delete sessions
        sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await
            .map_err(Error::from_sqlx)?;

        // Then delete user
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx)
            .await
            .map_err(Error::from_sqlx)?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok(); // Explicit rollback, ignore error
            return Err(Error::NotFound("User not found".to_string()));
        }
    } else {
        // Soft delete - deactivate user
        let result = sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::from_sqlx)?;

        if result.rows_affected() == 0 {
            tx.rollback().await.ok(); // Explicit rollback, ignore error
            return Err(Error::NotFound("User not found".to_string()));
        }

        // Invalidate all user sessions
        sqlx::query!(
            "UPDATE sessions SET is_active = false WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::from_sqlx)?;
    }

    tx.commit().await.map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn get_user_stats(conn: &mut DbConn) -> Result<crate::users::models::UserStats> {
    // Get basic user counts
    let total_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0);

    let active_users = sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE is_active = true")
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0);

    let inactive_users = total_users - active_users;

    let email_verified = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE email_verified = true AND is_active = true"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let email_unverified = active_users - email_verified;

    // Get user counts by role
    let user_count =
        sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE role = 'user' AND is_active = true")
            .fetch_one(&mut **conn)
            .await
            .map_err(Error::from_sqlx)?
            .unwrap_or(0);

    let moderator_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE role = 'moderator' AND is_active = true"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let admin_count =
        sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active = true")
            .fetch_one(&mut **conn)
            .await
            .map_err(Error::from_sqlx)?
            .unwrap_or(0);

    // Get recent registrations
    let last_24h = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let last_7d = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let last_30d = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    Ok(crate::users::models::UserStats {
        total_users,
        active_users,
        inactive_users,
        email_verified,
        email_unverified,
        by_role: crate::users::models::UserRoleStats {
            user: user_count,
            moderator: moderator_count,
            admin: admin_count,
        },
        recent_registrations: crate::users::models::RecentRegistrations {
            last_24h,
            last_7d,
            last_30d,
        },
        last_updated: Utc::now(),
    })
}
