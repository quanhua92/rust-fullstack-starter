//! Testbasic business logic and database operations

use crate::{types::Result, Database};
use super::models::*;
use uuid::Uuid;

/// List testbasics with optional filtering
pub async fn list_testbasics_service(
    database: &Database,
    request: ListTestbasicRequest,
) -> Result<Vec<Testbasic>> {
    // Use sqlx! macro for compile-time query validation

    let testbasics = if let Some(search) = &request.search {
        let search_param = format!("%{}%", search);
        sqlx::query_as!(
            Testbasic,
            "SELECT id, name, description, created_at, updated_at 
             FROM testbasics 
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
            Testbasic,
            "SELECT id, name, description, created_at, updated_at 
             FROM testbasics 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            request.limit as i64,
            request.offset as i64
        )
        .fetch_all(&database.pool)
        .await?
    };

    Ok(testbasics)
}

/// Get a specific testbasic by ID
pub async fn get_testbasic_service(
    database: &Database,
    id: Uuid,
) -> Result<Testbasic> {
    let testbasic = sqlx::query_as!(
        Testbasic,
        "SELECT id, name, description, created_at, updated_at 
         FROM testbasics 
         WHERE id = $1",
        id
    )
    .fetch_optional(&database.pool)
    .await?
    .ok_or_else(|| crate::error::Error::NotFound(format!("Testbasic with id {}", id)))?;

    Ok(testbasic)
}

/// Create a new testbasic
pub async fn create_testbasic_service(
    database: &Database,
    request: CreateTestbasicRequest,
) -> Result<Testbasic> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(crate::error::Error::validation("name", "Name cannot be empty"));
    }

    let testbasic = Testbasic::new(request.name, request.description);

    let created_testbasic = sqlx::query_as!(
        Testbasic,
        "INSERT INTO testbasics (id, name, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, description, created_at, updated_at",
        testbasic.id,
        testbasic.name,
        testbasic.description,
        testbasic.created_at,
        testbasic.updated_at
    )
    .fetch_one(&database.pool)
    .await?;

    Ok(created_testbasic)
}

/// Update an existing testbasic
pub async fn update_testbasic_service(
    database: &Database,
    id: Uuid,
    request: UpdateTestbasicRequest,
) -> Result<Testbasic> {
    // Get existing testbasic
    let mut testbasic = get_testbasic_service(database, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(crate::error::Error::validation("name", "Name cannot be empty"));
        }
    }

    // Update the testbasic
    testbasic.update(request);

    let updated_testbasic = sqlx::query_as!(
        Testbasic,
        "UPDATE testbasics 
         SET name = $2, description = $3, updated_at = $4
         WHERE id = $1
         RETURNING id, name, description, created_at, updated_at",
        testbasic.id,
        testbasic.name,
        testbasic.description,
        testbasic.updated_at
    )
    .fetch_one(&database.pool)
    .await?;

    Ok(updated_testbasic)
}

/// Delete a testbasic
pub async fn delete_testbasic_service(
    database: &Database,
    id: Uuid,
) -> Result<()> {
    let rows_affected = sqlx::query!(
        "DELETE FROM testbasics WHERE id = $1",
        id
    )
    .execute(&database.pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(crate::error::Error::NotFound(format!("Testbasic with id {}", id)));
    }

    Ok(())
}

