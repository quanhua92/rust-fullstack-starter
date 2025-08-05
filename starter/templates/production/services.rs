use crate::__MODULE_NAME_PLURAL__::models::{Create__MODULE_STRUCT__Request, Update__MODULE_STRUCT__Request, __MODULE_STRUCT__, __MODULE_STRUCT__Response, __MODULE_STRUCT__Stats, UserStats, Recent__MODULE_STRUCT__s};
use crate::__MODULE_NAME_PLURAL__::api::{List__MODULE_STRUCT__sQuery, __MODULE_STRUCT__CountResponse, Search__MODULE_STRUCT__sQuery, Filter__MODULE_STRUCT__sQuery};
use crate::rbac::models::UserRole;
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use chrono::{DateTime, Utc};
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
    params: List__MODULE_STRUCT__sQuery,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Build dynamic query based on search and filter parameters
    let mut query = "SELECT id, title, content, user_id, is_active, created_at, updated_at FROM __MODULE_TABLE__ WHERE 1=1".to_string();
    let mut args: Vec<Box<dyn sqlx::postgres::PgArgument + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    // Add user filter
    if let Some(user_id) = user_id {
        param_count += 1;
        query.push_str(&format!(" AND user_id = ${}", param_count));
        // Note: This is simplified - in real production code you'd use proper query building
    }

    // Add active filter
    if let Some(is_active) = params.is_active {
        param_count += 1;
        query.push_str(&format!(" AND is_active = ${}", param_count));
    } else {
        query.push_str(" AND is_active = true");
    }

    // Add search filter (simplified)
    if let Some(search) = &params.search {
        param_count += 1;
        query.push_str(&format!(" AND (title ILIKE ${} OR content ILIKE ${})", param_count, param_count));
    }

    // Add sorting
    let sort_field = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = if params.sort_order.as_deref() == Some("asc") { "ASC" } else { "DESC" };
    
    match sort_field {
        "title" | "created_at" | "updated_at" => {
            query.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));
        }
        _ => {
            query.push_str(" ORDER BY created_at DESC");
        }
    }

    param_count += 1;
    query.push_str(&format!(" LIMIT ${}", param_count));
    param_count += 1;
    query.push_str(&format!(" OFFSET ${}", param_count));

    // For now, use simple queries for basic functionality
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

    Ok(__MODULE_NAME_PLURAL__.into_iter().map(|__MODULE_NAME__| __MODULE_NAME__.to_response()).collect())
}

pub async fn count___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Option<Uuid>,
    params: List__MODULE_STRUCT__sQuery,
) -> Result<__MODULE_STRUCT__CountResponse> {
    // Simplified count implementation
    let total = if let Some(user_id) = user_id {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE user_id = $1",
            user_id
        )
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__"
        )
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    };

    let active = if let Some(user_id) = user_id {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE is_active = true"
        )
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    };

    let inactive = total - active;

    Ok(__MODULE_STRUCT__CountResponse {
        total,
        active,
        inactive,
    })
}

pub async fn bulk_create___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Uuid,
    requests: Vec<Create__MODULE_STRUCT__Request>,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let mut results = Vec::new();
    
    // Simple implementation - create one by one
    // Production code would use batch inserts
    for req in requests {
        let result = create___MODULE_NAME__(conn, user_id, req).await?;
        results.push(result);
    }
    
    Ok(results)
}

pub async fn bulk_update___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Uuid,
    user_role: UserRole,
    items: Vec<crate::__MODULE_NAME_PLURAL__::api::BulkUpdate__MODULE_STRUCT__Item>,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let mut results = Vec::new();
    
    for item in items {
        // Check permissions for each item
        let existing = get___MODULE_NAME___by_id(conn, item.id).await?;
        let existing = match existing {
            Some(item) => item,
            None => continue, // Skip items that don't exist
        };

        // Check ownership for regular users
        match user_role {
            UserRole::Admin | UserRole::Moderator => {
                // Admin and Moderator can update any resource
            }
            UserRole::User => {
                // Users can only update their own resources
                if existing.user_id != user_id {
                    continue; // Skip items user doesn't own
                }
            }
        }

        let result = update___MODULE_NAME__(conn, item.id, item.data).await?;
        results.push(result);
    }
    
    Ok(results)
}

pub async fn bulk_delete___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Uuid,
    user_role: UserRole,
    ids: Vec<Uuid>,
) -> Result<i64> {
    let mut deleted_count = 0;
    
    for id in ids {
        // Check permissions for each item
        let existing = get___MODULE_NAME___by_id(conn, id).await?;
        let existing = match existing {
            Some(item) => item,
            None => continue, // Skip items that don't exist
        };

        // Check ownership for regular users
        match user_role {
            UserRole::Admin | UserRole::Moderator => {
                // Admin and Moderator can delete any resource
            }
            UserRole::User => {
                // Users can only delete their own resources
                if existing.user_id != user_id {
                    continue; // Skip items user doesn't own
                }
            }
        }

        delete___MODULE_NAME__(conn, id).await?;
        deleted_count += 1;
    }
    
    Ok(deleted_count)
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
        SET title = $1, content = $2, updated_at = NOW()
        WHERE id = $3 AND is_active = true
        RETURNING id, title, content, user_id, is_active, created_at, updated_at
        "#,
        req.title,
        req.content,
        __MODULE_NAME___id
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(__MODULE_NAME__.to_response())
}

pub async fn delete___MODULE_NAME__(
    conn: &mut DbConn,
    __MODULE_NAME___id: Uuid,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE __MODULE_TABLE__
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        "#,
        __MODULE_NAME___id
    )
    .execute(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(())
}

pub async fn get___MODULE_NAME_PLURAL___stats(
    conn: &mut DbConn,
) -> Result<__MODULE_STRUCT__Stats> {
    let total_active = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE is_active = true"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let total_inactive = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE is_active = false"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let recent_activity = sqlx::query_as!(
        Recent__MODULE_STRUCT__s,
        r#"
        SELECT 
            DATE(created_at) as date,
            COUNT(*) as count
        FROM __MODULE_TABLE__ 
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY DATE(created_at)
        ORDER BY date DESC
        LIMIT 30
        "#
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let user_distribution = sqlx::query_as!(
        UserStats,
        r#"
        SELECT 
            user_id,
            COUNT(*) as count
        FROM __MODULE_TABLE__ 
        WHERE is_active = true
        GROUP BY user_id
        ORDER BY count DESC
        LIMIT 10
        "#
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(__MODULE_STRUCT__Stats {
        total_active,
        total_inactive,
        recent_activity,
        user_distribution,
    })
}

pub async fn search___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Option<Uuid>,
    params: Search__MODULE_STRUCT__sQuery,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    
    // Build search query with PostgreSQL full-text search
    let search_term = format!("%{}%", params.q);
    
    let __MODULE_NAME_PLURAL__ = if let Some(user_id) = user_id {
        sqlx::query_as!(
            __MODULE_STRUCT__,
            r#"
            SELECT id, title, content, user_id, is_active, created_at, updated_at
            FROM __MODULE_TABLE__
            WHERE user_id = $1 
              AND is_active = true
              AND (title ILIKE $2 OR content ILIKE $2)
            ORDER BY 
              CASE 
                WHEN $3 = 'title' THEN title
                WHEN $3 = 'created_at' THEN created_at::text
                WHEN $3 = 'updated_at' THEN updated_at::text
                ELSE created_at::text
              END
              CASE WHEN $4 = 'asc' THEN ASC ELSE DESC END
            LIMIT $5 OFFSET $6
            "#,
            user_id,
            search_term,
            params.sort_by.as_deref().unwrap_or("created_at"),
            params.sort_order.as_deref().unwrap_or("desc"),
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
              AND (title ILIKE $1 OR content ILIKE $1)
            ORDER BY 
              CASE 
                WHEN $2 = 'title' THEN title
                WHEN $2 = 'created_at' THEN created_at::text
                WHEN $2 = 'updated_at' THEN updated_at::text
                ELSE created_at::text
              END
              CASE WHEN $3 = 'asc' THEN ASC ELSE DESC END
            LIMIT $4 OFFSET $5
            "#,
            search_term,
            params.sort_by.as_deref().unwrap_or("created_at"),
            params.sort_order.as_deref().unwrap_or("desc"),
            limit,
            offset
        )
        .fetch_all(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
    };

    Ok(__MODULE_NAME_PLURAL__.into_iter().map(|__MODULE_NAME__| __MODULE_NAME__.to_response()).collect())
}

pub async fn filter___MODULE_NAME_PLURAL__(
    conn: &mut DbConn,
    user_id: Option<Uuid>,
    params: Filter__MODULE_STRUCT__sQuery,
) -> Result<Vec<__MODULE_STRUCT__Response>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Build dynamic WHERE clause based on filters
    let mut where_conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();
    
    // Base conditions
    where_conditions.push("1=1".to_string());
    
    if let Some(user_id) = user_id {
        where_conditions.push("user_id = $1".to_string());
        bind_values.push(user_id.to_string());
    }
    
    if let Some(is_active) = params.is_active {
        let param_num = bind_values.len() + 1;
        where_conditions.push(format!("is_active = ${}", param_num));
        bind_values.push(is_active.to_string());
    } else {
        where_conditions.push("is_active = true".to_string());
    }
    
    if let Some(title_contains) = &params.title_contains {
        let param_num = bind_values.len() + 1;
        where_conditions.push(format!("title ILIKE ${}", param_num));
        bind_values.push(format!("%{}%", title_contains));
    }

    // For now, use a simplified approach (production code would use query builder)
    let __MODULE_NAME_PLURAL__ = if let Some(user_id) = user_id {
        let mut base_query = sqlx::query_as!(
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
        );
        
        if let Some(title_contains) = &params.title_contains {
            base_query = sqlx::query_as!(
                __MODULE_STRUCT__,
                r#"
                SELECT id, title, content, user_id, is_active, created_at, updated_at
                FROM __MODULE_TABLE__
                WHERE user_id = $1 AND is_active = true AND title ILIKE $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                user_id,
                format!("%{}%", title_contains),
                limit,
                offset
            );
        }
        
        base_query.fetch_all(&mut **conn).await.map_err(Error::from_sqlx)?
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

    Ok(__MODULE_NAME_PLURAL__.into_iter().map(|__MODULE_NAME__| __MODULE_NAME__.to_response()).collect())
}