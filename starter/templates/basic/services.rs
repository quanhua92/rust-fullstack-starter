use crate::__MODULE_NAME_PLURAL__::models::{Create__MODULE_STRUCT__Request, Update__MODULE_STRUCT__Request, __MODULE_STRUCT__, __MODULE_STRUCT__Response, __MODULE_STRUCT__Stats, UserStats, Recent__MODULE_STRUCT__s};
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use chrono::Utc;
use sqlx::Acquire;
use uuid::Uuid;

pub async fn create___MODULE_NAME__(
    conn: &mut DbConn,
    user_id: Uuid,
    req: Create__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__Response> {
    req.validate()?;

    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"
        INSERT INTO __MODULE_TABLE__ (title, content, user_id)
        VALUES ($1, $2, $3)
        RETURNING id, title, content, user_id, is_active, created_at, updated_at
        "#,
        req.title,
        req.content,
        user_id
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(__MODULE_NAME__.to_response())
}

pub async fn get___MODULE_NAME___by_id(
    conn: &mut DbConn,
    __MODULE_NAME___id: Uuid,
) -> Result<Option<__MODULE_STRUCT__>> {
    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"
        SELECT id, title, content, user_id, is_active, created_at, updated_at
        FROM __MODULE_TABLE__
        WHERE id = $1 AND is_active = true
        "#,
        __MODULE_NAME___id
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(__MODULE_NAME__)
}

pub async fn list___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Option<Uuid>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    let __MODULE_NAME_PLURAL__ = if let Some(user_id) = user_id {
        sqlx::query_as!(
            __MODULE_STRUCT__,
            r#"
            SELECT id, title, content, user_id, is_active, created_at, updated_at
            FROM __MODULE_TABLE__
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
    } else {
        sqlx::query_as!(
            __MODULE_STRUCT__,
            r#"
            SELECT id, title, content, user_id, is_active, created_at, updated_at
            FROM __MODULE_TABLE__
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
    };

    Ok(__MODULE_NAME_PLURAL__.into_iter().map(|n| n.to_response()).collect())
}

pub async fn update___MODULE_NAME__(
    conn: &mut DbConn,
    __MODULE_NAME___id: Uuid,
    req: Update__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__Response> {
    req.validate()?;

    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"
        UPDATE __MODULE_TABLE__
        SET title = COALESCE($2, title),
            content = COALESCE($3, content),
            updated_at = NOW()
        WHERE id = $1 AND is_active = true
        RETURNING id, title, content, user_id, is_active, created_at, updated_at
        "#,
        __MODULE_NAME___id,
        req.title,
        req.content
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    match __MODULE_NAME__ {
        Some(__MODULE_NAME__) => Ok(__MODULE_NAME__.to_response()),
        None => Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string())),
    }
}

pub async fn delete___MODULE_NAME__(
    conn: &mut DbConn,
    __MODULE_NAME___id: Uuid,
) -> Result<()> {
    let result = sqlx::query!(
        "UPDATE __MODULE_TABLE__ SET is_active = false, updated_at = NOW() WHERE id = $1 AND is_active = true",
        __MODULE_NAME___id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound("__MODULE_STRUCT__ not found".to_string()));
    }

    Ok(())
}

pub async fn get___MODULE_NAME_PLURAL___stats(conn: &mut DbConn) -> Result<__MODULE_STRUCT__Stats> {
    let total___MODULE_NAME_PLURAL__ = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let active___MODULE_NAME_PLURAL__ = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE is_active = true"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let inactive___MODULE_NAME_PLURAL__ = total___MODULE_NAME_PLURAL__ - active___MODULE_NAME_PLURAL__;

    let __MODULE_NAME_PLURAL___by_user = sqlx::query!(
        r#"
        SELECT user_id, COUNT(*) as count
        FROM __MODULE_TABLE__
        WHERE is_active = true
        GROUP BY user_id
        ORDER BY count DESC
        LIMIT 10
        "#
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .into_iter()
    .map(|row| UserStats {
        user_id: row.user_id,
        count: row.count.unwrap_or(0),
    })
    .collect();

    let last_24h = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE created_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let last_7d = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE created_at > NOW() - INTERVAL '7 days'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let last_30d = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE created_at > NOW() - INTERVAL '30 days'"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    Ok(__MODULE_STRUCT__Stats {
        total___MODULE_NAME_PLURAL__,
        active___MODULE_NAME_PLURAL__,
        inactive___MODULE_NAME_PLURAL__,
        __MODULE_NAME_PLURAL___by_user,
        recent___MODULE_NAME_PLURAL__: Recent__MODULE_STRUCT__s {
            last_24h,
            last_7d,
            last_30d,
        },
        last_updated: Utc::now(),
    })
}