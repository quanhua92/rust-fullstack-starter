//! __MODULE_STRUCT__ business logic and database operations

use super::models::*;
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use uuid::Uuid;

/// List __MODULE_NAME_PLURAL__ with optional filtering
pub async fn list___MODULE_NAME_PLURAL___service(
    conn: &mut DbConn,
    request: List__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__ListResponse> {
    let limit = request.limit.unwrap_or(20).clamp(1, 100) as i64;
    let offset = request.offset.unwrap_or(0) as i64;

    let __MODULE_NAME_PLURAL__ = if let Some(search) = &request.search {
        let search_param = format!("%{search}%");
        sqlx::query_as!(
            __MODULE_STRUCT__,
            r#"SELECT id, name, description, status as "status: __MODULE_STRUCT__Status", priority, metadata, created_by, created_at, updated_at 
               FROM __MODULE_TABLE__ 
               WHERE name ILIKE $1 OR description ILIKE $1
               ORDER BY created_at DESC 
               LIMIT $2 OFFSET $3"#,
            search_param,
            limit,
            offset
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(Error::from_sqlx)?
    } else {
        sqlx::query_as!(
            __MODULE_STRUCT__,
            r#"SELECT id, name, description, status as "status: __MODULE_STRUCT__Status", priority, metadata, created_by, created_at, updated_at 
               FROM __MODULE_TABLE__ 
               ORDER BY created_at DESC 
               LIMIT $1 OFFSET $2"#,
            limit,
            offset
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(Error::from_sqlx)?
    };

    // Apply the same filters to count query as main query
    let total_count = if let Some(search) = &request.search {
        let search_param = format!("%{search}%");
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__ WHERE name ILIKE $1 OR description ILIKE $1",
            search_param
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    } else {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM __MODULE_TABLE__"
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0)
    };

    let pagination = PaginationInfo {
        total_count,
        page_count: (total_count + limit - 1) / limit,
        current_page: (offset / limit) + 1,
        per_page: limit as i32,
        has_next: (offset + limit) < total_count,
        has_prev: offset > 0,
        next_cursor: None,
        prev_cursor: None,
    };

    Ok(__MODULE_STRUCT__ListResponse {
        items: __MODULE_NAME_PLURAL__,
        pagination,
    })
}

/// Get a specific __MODULE_NAME__ by ID
pub async fn get___MODULE_NAME___service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<__MODULE_STRUCT__> {
    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"SELECT id, name, description, status as "status: __MODULE_STRUCT__Status", priority, metadata, created_by, created_at, updated_at 
           FROM __MODULE_TABLE__ 
           WHERE id = $1"#,
        id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?
    .ok_or_else(|| Error::NotFound(format!("__MODULE_STRUCT__ with id {id}")))?;

    Ok(__MODULE_NAME__)
}

/// Create a new __MODULE_NAME__
pub async fn create___MODULE_NAME___service(
    conn: &mut DbConn,
    request: Create__MODULE_STRUCT__Request,
    created_by: Uuid,
) -> Result<__MODULE_STRUCT__> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(Error::validation("name", "Name cannot be empty"));
    }

    let __MODULE_NAME__ = __MODULE_STRUCT__::new(
        request.name,
        request.description,
        request.status,
        request.priority,
        request.metadata,
        created_by,
    );

    let created___MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"INSERT INTO __MODULE_TABLE__ (id, name, description, status, priority, metadata, created_by, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, name, description, status as "status: __MODULE_STRUCT__Status", priority, metadata, created_by, created_at, updated_at"#,
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
        __MODULE_NAME__.status as __MODULE_STRUCT__Status,
        __MODULE_NAME__.priority,
        __MODULE_NAME__.metadata,
        __MODULE_NAME__.created_by,
        __MODULE_NAME__.created_at,
        __MODULE_NAME__.updated_at
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(created___MODULE_NAME__)
}

/// Update an existing __MODULE_NAME__
pub async fn update___MODULE_NAME___service(
    conn: &mut DbConn,
    id: Uuid,
    request: Update__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__> {
    // Get existing __MODULE_NAME__
    let mut __MODULE_NAME__ = get___MODULE_NAME___service(conn, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(Error::validation("name", "Name cannot be empty"));
        }
    }

    // Update the __MODULE_NAME__
    __MODULE_NAME__.update(request);

    let updated___MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        r#"UPDATE __MODULE_TABLE__ 
           SET name = $2, description = $3, status = $4, priority = $5, metadata = $6, updated_at = $7
           WHERE id = $1
           RETURNING id, name, description, status as "status: __MODULE_STRUCT__Status", priority, metadata, created_by, created_at, updated_at"#,
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
        __MODULE_NAME__.status as __MODULE_STRUCT__Status,
        __MODULE_NAME__.priority,
        __MODULE_NAME__.metadata,
        __MODULE_NAME__.updated_at
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(updated___MODULE_NAME__)
}

/// Delete a __MODULE_NAME__
pub async fn delete___MODULE_NAME___service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<()> {
    let rows_affected = sqlx::query!(
        "DELETE FROM __MODULE_TABLE__ WHERE id = $1",
        id
    )
    .execute(&mut *conn)
    .await
    .map_err(Error::from_sqlx)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(Error::NotFound(format!("__MODULE_STRUCT__ with id {id}")));
    }

    Ok(())
}

/// Bulk create __MODULE_NAME_PLURAL__
pub async fn bulk_create___MODULE_NAME_PLURAL___service(
    conn: &mut DbConn,
    request: Bulk__MODULE_STRUCT__CreateRequest,
    created_by: Uuid,
) -> Result<BulkOperationResponse<__MODULE_STRUCT__>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item) in request.items.into_iter().enumerate() {
        match create___MODULE_NAME___service(conn, item, created_by).await {
            Ok(__MODULE_NAME__) => results.push(__MODULE_NAME__),
            Err(error) => {
                errors.push(BulkOperationError {
                    index,
                    id: None,
                    error: error.to_string(),
                });
                if !skip_errors {
                    break;
                }
            }
        }
    }

    Ok(BulkOperationResponse {
        success_count: results.len(),
        error_count: errors.len(),
        errors,
        results,
    })
}

/// Bulk update __MODULE_NAME_PLURAL__
pub async fn bulk_update___MODULE_NAME_PLURAL___service(
    conn: &mut DbConn,
    request: Bulk__MODULE_STRUCT__UpdateRequest,
) -> Result<BulkOperationResponse<__MODULE_STRUCT__>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item) in request.items.into_iter().enumerate() {
        match update___MODULE_NAME___service(conn, item.id, item.data).await {
            Ok(__MODULE_NAME__) => results.push(__MODULE_NAME__),
            Err(error) => {
                errors.push(BulkOperationError {
                    index,
                    id: Some(item.id),
                    error: error.to_string(),
                });
                if !skip_errors {
                    break;
                }
            }
        }
    }

    Ok(BulkOperationResponse {
        success_count: results.len(),
        error_count: errors.len(),
        errors,
        results,
    })
}

/// Bulk delete __MODULE_NAME_PLURAL__
pub async fn bulk_delete___MODULE_NAME_PLURAL___service(
    conn: &mut DbConn,
    request: Bulk__MODULE_STRUCT__DeleteRequest,
) -> Result<BulkOperationResponse<Uuid>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, id) in request.ids.into_iter().enumerate() {
        match delete___MODULE_NAME___service(conn, id).await {
            Ok(()) => results.push(id),
            Err(error) => {
                errors.push(BulkOperationError {
                    index,
                    id: Some(id),
                    error: error.to_string(),
                });
                if !skip_errors {
                    break;
                }
            }
        }
    }

    Ok(BulkOperationResponse {
        success_count: results.len(),
        error_count: errors.len(),
        errors,
        results,
    })
}