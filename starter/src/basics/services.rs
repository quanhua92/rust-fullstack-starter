//! Basics business logic and database operations

use crate::types::{DbConn, Result};
use super::models::*;
use uuid::Uuid;

/// List basics with optional filtering
pub async fn list_basics_service(
    conn: &mut DbConn,
    request: ListBasicsRequest,
) -> Result<Vec<Basics>> {
    // Use sqlx! macro for compile-time query validation

    let basics = if let Some(search) = &request.search {
        let search_param = format!("%{}%", search);
        sqlx::query_as!(
            Basics,
            "SELECT id, name, description, created_at, updated_at 
             FROM basics 
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
            search_param,
            request.limit as i64,
            request.offset as i64
        )
        .fetch_all(&mut **conn)
        .await?
    } else {
        sqlx::query_as!(
            Basics,
            "SELECT id, name, description, created_at, updated_at 
             FROM basics 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            request.limit as i64,
            request.offset as i64
        )
        .fetch_all(&mut **conn)
        .await?
    };

    Ok(basics)
}

/// Get a specific basics by ID
pub async fn get_basics_service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<Basics> {
    let basics = sqlx::query_as!(
        Basics,
        "SELECT id, name, description, created_at, updated_at 
         FROM basics 
         WHERE id = $1",
        id
    )
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| crate::error::Error::NotFound(format!("Basics with id {}", id)))?;

    Ok(basics)
}

/// Create a new basics
pub async fn create_basics_service(
    conn: &mut DbConn,
    request: CreateBasicsRequest,
) -> Result<Basics> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(crate::error::Error::validation("name", "Name cannot be empty"));
    }

    let basics = Basics::new(request.name, request.description);

    let created_basics = sqlx::query_as!(
        Basics,
        "INSERT INTO basics (id, name, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, description, created_at, updated_at",
        basics.id,
        basics.name,
        basics.description,
        basics.created_at,
        basics.updated_at
    )
    .fetch_one(&mut **conn)
    .await?;

    Ok(created_basics)
}

/// Update an existing basics
pub async fn update_basics_service(
    conn: &mut DbConn,
    id: Uuid,
    request: UpdateBasicsRequest,
) -> Result<Basics> {
    // Get existing basics
    let mut basics = get_basics_service(conn, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(crate::error::Error::validation("name", "Name cannot be empty"));
        }
    }

    // Update the basics
    basics.update(request);

    let updated_basics = sqlx::query_as!(
        Basics,
        "UPDATE basics 
         SET name = $2, description = $3, updated_at = $4
         WHERE id = $1
         RETURNING id, name, description, created_at, updated_at",
        basics.id,
        basics.name,
        basics.description,
        basics.updated_at
    )
    .fetch_one(&mut **conn)
    .await?;

    Ok(updated_basics)
}

/// Delete a basics
pub async fn delete_basics_service(
    conn: &mut DbConn,
    id: Uuid,
) -> Result<()> {
    let rows_affected = sqlx::query!(
        "DELETE FROM basics WHERE id = $1",
        id
    )
    .execute(&mut **conn)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(crate::error::Error::NotFound(format!("Basics with id {}", id)));
    }

    Ok(())
}

