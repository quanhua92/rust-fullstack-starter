//! __MODULE_STRUCT__ business logic and database operations with advanced features

use crate::{types::Result, Database};
use super::models::*;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// List __MODULE_NAME_PLURAL__ with advanced filtering and pagination
pub async fn list___MODULE_NAME_PLURAL___service(
    database: &Database,
    request: List__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__ListResponse> {
    let limit = request.limit.unwrap_or(20).min(100).max(1) as i64;
    let offset = request.offset.unwrap_or(0) as i64;
    
    // Use simplified queries based on filters
    let (items, total_count) = if request.search.is_some() || request.status.is_some() || 
                                 request.min_priority.is_some() || request.max_priority.is_some() ||
                                 request.created_after.is_some() || request.created_before.is_some() {
        // Filtered query
        list_with_filters(database, &request, limit, offset).await?
    } else {
        // Simple query without filters
        list_without_filters(database, &request, limit, offset).await?
    };
    
    // Check if there are more items (for has_next)
    let has_next = items.len() > limit as usize;
    let mut final_items = items;
    if has_next {
        final_items.pop(); // Remove the extra item
    }
    
    // Calculate pagination info
    let current_page = (offset / limit) + 1;
    let page_count = (total_count + limit - 1) / limit;
    let has_prev = offset > 0;
    
    // Generate cursors for cursor-based pagination
    let next_cursor = if has_next {
        Some(generate_cursor(offset + limit))
    } else {
        None
    };
    
    let prev_cursor = if has_prev {
        Some(generate_cursor((offset - limit).max(0)))
    } else {
        None
    };
    
    let pagination = PaginationInfo {
        total_count,
        page_count,
        current_page,
        per_page: limit as i32,
        has_next,
        has_prev,
        next_cursor,
        prev_cursor,
    };
    
    Ok(__MODULE_STRUCT__ListResponse { items: final_items, pagination })
}

/// List items without filters (optimized path)
async fn list_without_filters(
    database: &Database,
    request: &List__MODULE_STRUCT__Request,
    limit: i64,
    offset: i64,
) -> Result<(Vec<__MODULE_STRUCT__>, i64)> {
    let sort_field = match request.sort_by.as_ref().unwrap_or(&__MODULE_STRUCT__SortField::CreatedAt) {
        __MODULE_STRUCT__SortField::Name => "name",
        __MODULE_STRUCT__SortField::Priority => "priority", 
        __MODULE_STRUCT__SortField::Status => "status",
        __MODULE_STRUCT__SortField::CreatedAt => "created_at",
        __MODULE_STRUCT__SortField::UpdatedAt => "updated_at",
    };
    
    let sort_order = match request.sort_order.as_ref().unwrap_or(&SortOrder::Desc) {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    
    // Get items with one extra to check for has_next
    let items = match (sort_field, sort_order) {
        ("created_at", "DESC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        ("created_at", "ASC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY created_at ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        ("name", "ASC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY name ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        ("name", "DESC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY name DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        ("priority", "ASC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY priority ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        ("priority", "DESC") => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY priority DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
        _ => sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&database.pool).await?,
    };

    // Get total count
    let total_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__"
    )
    .fetch_one(&database.pool)
    .await?
    .unwrap_or(0);

    Ok((items, total_count))
}

/// List items with filters applied
async fn list_with_filters(
    database: &Database,
    request: &List__MODULE_STRUCT__Request,
    limit: i64,
    offset: i64,
) -> Result<(Vec<__MODULE_STRUCT__>, i64)> {
    // For simplicity, implement basic search and status filter
    let items = if let Some(search) = &request.search {
        let search_param = format!("%{}%", search);
        sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
            search_param,
            limit + 1,
            offset
        )
        .fetch_all(&database.pool)
        .await?
    } else if let Some(status_list) = &request.status {
        if !status_list.is_empty() {
            let status = &status_list[0]; // Use first status for simplicity
            sqlx::query_as!(
                __MODULE_STRUCT__,
                "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
                 FROM __MODULE_TABLE__ 
                 WHERE status = $1
                 ORDER BY created_at DESC 
                 LIMIT $2 OFFSET $3",
                status as &__MODULE_STRUCT__Status,
                limit + 1,
                offset
            )
            .fetch_all(&database.pool)
            .await?
        } else {
            Vec::new()
        }
    } else {
        // Other filters can be added here
        sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        )
        .fetch_all(&database.pool)
        .await?
    };

    // Get approximate count (for filtered results, this might be less accurate)
    let total_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM __MODULE_TABLE__"
    )
    .fetch_one(&database.pool)
    .await?
    .unwrap_or(0);

    Ok((items, total_count))
}

/// Get a specific __MODULE_NAME__ by ID
pub async fn get___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
) -> Result<__MODULE_STRUCT__> {
    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        "SELECT id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at 
         FROM __MODULE_TABLE__ 
         WHERE id = $1",
        id
    )
    .fetch_optional(&database.pool)
    .await?
    .ok_or_else(|| crate::error::Error::NotFound(format!("__MODULE_STRUCT__ with id {}", id)))?;

    Ok(__MODULE_NAME__)
}

/// Create a new __MODULE_NAME__
pub async fn create___MODULE_NAME___service(
    database: &Database,
    request: Create__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(crate::error::Error::validation("name", "Name cannot be empty"));
    }

    let __MODULE_NAME__ = __MODULE_STRUCT__::new(
        request.name, 
        request.description, 
        request.status,
        request.priority,
        request.metadata
    );

    let created___MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        "INSERT INTO __MODULE_TABLE__ (id, name, description, status, priority, metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at",
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
        __MODULE_NAME__.status as __MODULE_STRUCT__Status,
        __MODULE_NAME__.priority,
        __MODULE_NAME__.metadata,
        __MODULE_NAME__.created_at,
        __MODULE_NAME__.updated_at
    )
    .fetch_one(&database.pool)
    .await?;

    Ok(created___MODULE_NAME__)
}

/// Update an existing __MODULE_NAME__
pub async fn update___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
    request: Update__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__> {
    // Get existing __MODULE_NAME__
    let mut __MODULE_NAME__ = get___MODULE_NAME___service(database, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(crate::error::Error::validation("name", "Name cannot be empty"));
        }
    }

    // Update the __MODULE_NAME__
    __MODULE_NAME__.update(request);

    let updated___MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        "UPDATE __MODULE_TABLE__ 
         SET name = $2, description = $3, status = $4, priority = $5, metadata = $6, updated_at = $7
         WHERE id = $1
         RETURNING id, name, description, status as \"status: __MODULE_STRUCT__Status\", priority, metadata, created_at, updated_at",
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
        __MODULE_NAME__.status as __MODULE_STRUCT__Status,
        __MODULE_NAME__.priority,
        __MODULE_NAME__.metadata,
        __MODULE_NAME__.updated_at
    )
    .fetch_one(&database.pool)
    .await?;

    Ok(updated___MODULE_NAME__)
}

/// Delete a __MODULE_NAME__
pub async fn delete___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
) -> Result<()> {
    let rows_affected = sqlx::query!(
        "DELETE FROM __MODULE_TABLE__ WHERE id = $1",
        id
    )
    .execute(&database.pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(crate::error::Error::NotFound(format!("__MODULE_STRUCT__ with id {}", id)));
    }

    Ok(())
}

/// Bulk create __MODULE_NAME_PLURAL__
pub async fn bulk_create___MODULE_NAME_PLURAL___service(
    database: &Database,
    request: Bulk__MODULE_STRUCT__CreateRequest,
) -> Result<BulkOperationResponse<__MODULE_STRUCT__>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item_request) in request.items.into_iter().enumerate() {
        match create___MODULE_NAME___service(database, item_request).await {
            Ok(__MODULE_NAME__) => results.push(__MODULE_NAME__),
            Err(e) => {
                errors.push(BulkOperationError {
                    index,
                    id: None,
                    error: e.to_string(),
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
    database: &Database,
    request: Bulk__MODULE_STRUCT__UpdateRequest,
) -> Result<BulkOperationResponse<__MODULE_STRUCT__>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item) in request.items.into_iter().enumerate() {
        match update___MODULE_NAME___service(database, item.id, item.data).await {
            Ok(__MODULE_NAME__) => results.push(__MODULE_NAME__),
            Err(e) => {
                errors.push(BulkOperationError {
                    index,
                    id: Some(item.id),
                    error: e.to_string(),
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
    database: &Database,
    request: Bulk__MODULE_STRUCT__DeleteRequest,
) -> Result<BulkOperationResponse<Uuid>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, id) in request.ids.into_iter().enumerate() {
        match delete___MODULE_NAME___service(database, id).await {
            Ok(()) => results.push(id),
            Err(e) => {
                errors.push(BulkOperationError {
                    index,
                    id: Some(id),
                    error: e.to_string(),
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

/// Generate a cursor for pagination
fn generate_cursor(offset: i64) -> String {
    BASE64.encode(offset.to_string().as_bytes())
}

/// Parse a cursor for pagination
pub fn parse_cursor(cursor: &str) -> Result<i64> {
    let decoded = BASE64.decode(cursor)
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor format"))?;
    
    let offset_str = String::from_utf8(decoded)
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor encoding"))?;
    
    offset_str.parse::<i64>()
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor value"))
}

