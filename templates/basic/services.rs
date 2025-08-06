//! __MODULE_STRUCT__ business logic and database operations

use crate::{types::Result, Database};
use super::models::*;
use uuid::Uuid;

/// List __MODULE_NAME_PLURAL__ with optional filtering
pub async fn list___MODULE_NAME_PLURAL___service(
    database: &Database,
    request: List__MODULE_STRUCT__Request,
) -> Result<Vec<__MODULE_STRUCT__>> {
    // Use sqlx! macro for compile-time query validation

    let __MODULE_NAME_PLURAL__ = if let Some(search) = &request.search {
        let search_param = format!("%{}%", search);
        sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
            search_param,
            request.limit as i64,
            request.offset as i64
        )
        .fetch_all(&database.pool)
        .await?
    } else {
        sqlx::query_as!(
            __MODULE_STRUCT__,
            "SELECT id, name, description, created_at, updated_at 
             FROM __MODULE_TABLE__ 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            request.limit as i64,
            request.offset as i64
        )
        .fetch_all(&database.pool)
        .await?
    };

    Ok(__MODULE_NAME_PLURAL__)
}

/// Get a specific __MODULE_NAME__ by ID
pub async fn get___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
) -> Result<__MODULE_STRUCT__> {
    let __MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        "SELECT id, name, description, created_at, updated_at 
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

    let __MODULE_NAME__ = __MODULE_STRUCT__::new(request.name, request.description);

    let created___MODULE_NAME__ = sqlx::query_as!(
        __MODULE_STRUCT__,
        "INSERT INTO __MODULE_TABLE__ (id, name, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, description, created_at, updated_at",
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
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
         SET name = $2, description = $3, updated_at = $4
         WHERE id = $1
         RETURNING id, name, description, created_at, updated_at",
        __MODULE_NAME__.id,
        __MODULE_NAME__.name,
        __MODULE_NAME__.description,
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

