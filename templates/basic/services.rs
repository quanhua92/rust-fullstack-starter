//! __MODULE_STRUCT__ business logic and database operations

use crate::{error::Error, Database};
use super::models::*;
use uuid::Uuid;

/// List __MODULE_NAME_PLURAL__ with optional filtering
pub async fn list___MODULE_NAME_PLURAL___service(
    database: &Database,
    request: List__MODULE_STRUCT__Request,
) -> Result<Vec<__MODULE_STRUCT__>, Error> {
    let mut query = format!(
        "SELECT id, name, description, created_at, updated_at 
         FROM __MODULE_TABLE__ 
         WHERE 1=1"
    );
    
    let mut params = Vec::new();
    let mut param_count = 0;

    // Add search filter if provided
    if let Some(search) = &request.search {
        param_count += 1;
        query.push_str(&format!(" AND (name ILIKE ${} OR description ILIKE ${})", param_count, param_count));
        params.push(format!("%{}%", search));
    }

    query.push_str(" ORDER BY created_at DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", request.limit, request.offset));

    let mut sqlx_query = sqlx::query_as::<_, __MODULE_STRUCT__>(&query);
    
    // Bind parameters
    for param in params {
        sqlx_query = sqlx_query.bind(param);
    }

    let __MODULE_NAME_PLURAL__ = sqlx_query
        .fetch_all(&database.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to list __MODULE_NAME_PLURAL__: {}", e)))?;

    Ok(__MODULE_NAME_PLURAL__)
}

/// Get a specific __MODULE_NAME__ by ID
pub async fn get___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
) -> Result<__MODULE_STRUCT__, Error> {
    let __MODULE_NAME__ = sqlx::query_as::<_, __MODULE_STRUCT__>(
        "SELECT id, name, description, created_at, updated_at 
         FROM __MODULE_TABLE__ 
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&database.pool)
    .await
    .map_err(|e| Error::Database(format!("Failed to get __MODULE_NAME__: {}", e)))?
    .ok_or_else(|| Error::NotFound("__MODULE_STRUCT__".to_string(), id.to_string()))?;

    Ok(__MODULE_NAME__)
}

/// Create a new __MODULE_NAME__
pub async fn create___MODULE_NAME___service(
    database: &Database,
    request: Create__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__, Error> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(Error::validation("name", "Name cannot be empty"));
    }

    let __MODULE_NAME__ = __MODULE_STRUCT__::new(request.name, request.description);

    let created___MODULE_NAME__ = sqlx::query_as::<_, __MODULE_STRUCT__>(
        "INSERT INTO __MODULE_TABLE__ (id, name, description, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, description, created_at, updated_at"
    )
    .bind(__MODULE_NAME__.id)
    .bind(__MODULE_NAME__.name)
    .bind(__MODULE_NAME__.description)
    .bind(__MODULE_NAME__.created_at)
    .bind(__MODULE_NAME__.updated_at)
    .fetch_one(&database.pool)
    .await
    .map_err(|e| Error::Database(format!("Failed to create __MODULE_NAME__: {}", e)))?;

    Ok(created___MODULE_NAME__)
}

/// Update an existing __MODULE_NAME__
pub async fn update___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
    request: Update__MODULE_STRUCT__Request,
) -> Result<__MODULE_STRUCT__, Error> {
    // Get existing __MODULE_NAME__
    let mut __MODULE_NAME__ = get___MODULE_NAME___service(database, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(Error::validation("name", "Name cannot be empty"));
        }
    }

    // Update the __MODULE_NAME__
    __MODULE_NAME__.update(request);

    let updated___MODULE_NAME__ = sqlx::query_as::<_, __MODULE_STRUCT__>(
        "UPDATE __MODULE_TABLE__ 
         SET name = $2, description = $3, updated_at = $4
         WHERE id = $1
         RETURNING id, name, description, created_at, updated_at"
    )
    .bind(__MODULE_NAME__.id)
    .bind(__MODULE_NAME__.name)
    .bind(__MODULE_NAME__.description)
    .bind(__MODULE_NAME__.updated_at)
    .fetch_one(&database.pool)
    .await
    .map_err(|e| Error::Database(format!("Failed to update __MODULE_NAME__: {}", e)))?;

    Ok(updated___MODULE_NAME__)
}

/// Delete a __MODULE_NAME__
pub async fn delete___MODULE_NAME___service(
    database: &Database,
    id: Uuid,
) -> Result<(), Error> {
    let rows_affected = sqlx::query("DELETE FROM __MODULE_TABLE__ WHERE id = $1")
        .bind(id)
        .execute(&database.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete __MODULE_NAME__: {}", e)))?
        .rows_affected();

    if rows_affected == 0 {
        return Err(Error::NotFound("__MODULE_STRUCT__".to_string(), id.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::db::*;

    #[tokio::test]
    async fn test_create_and_get___MODULE_NAME__() {
        let database = create_test_database().await;

        let request = Create__MODULE_STRUCT__Request {
            name: "Test __MODULE_STRUCT__".to_string(),
            description: Some("Test description".to_string()),
        };

        let created = create___MODULE_NAME___service(&database, request).await.unwrap();
        let retrieved = get___MODULE_NAME___service(&database, created.id).await.unwrap();

        assert_eq!(created.id, retrieved.id);
        assert_eq!(created.name, retrieved.name);
        assert_eq!(created.description, retrieved.description);
    }

    #[tokio::test]
    async fn test_list___MODULE_NAME_PLURAL__() {
        let database = create_test_database().await;

        // Create test __MODULE_NAME_PLURAL__
        for i in 1..=3 {
            let request = Create__MODULE_STRUCT__Request {
                name: format!("Test __MODULE_STRUCT__ {}", i),
                description: Some(format!("Description {}", i)),
            };
            create___MODULE_NAME___service(&database, request).await.unwrap();
        }

        let list_request = List__MODULE_STRUCT__Request {
            limit: 10,
            offset: 0,
            search: None,
        };

        let __MODULE_NAME_PLURAL__ = list___MODULE_NAME_PLURAL___service(&database, list_request).await.unwrap();
        assert_eq!(__MODULE_NAME_PLURAL__.len(), 3);
    }

    #[tokio::test]
    async fn test_update___MODULE_NAME__() {
        let database = create_test_database().await;

        let create_request = Create__MODULE_STRUCT__Request {
            name: "Original Name".to_string(),
            description: Some("Original description".to_string()),
        };

        let created = create___MODULE_NAME___service(&database, create_request).await.unwrap();

        let update_request = Update__MODULE_STRUCT__Request {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
        };

        let updated = update___MODULE_NAME___service(&database, created.id, update_request).await.unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert!(updated.updated_at > created.updated_at);
    }

    #[tokio::test]
    async fn test_delete___MODULE_NAME__() {
        let database = create_test_database().await;

        let request = Create__MODULE_STRUCT__Request {
            name: "Test __MODULE_STRUCT__".to_string(),
            description: Some("Test description".to_string()),
        };

        let created = create___MODULE_NAME___service(&database, request).await.unwrap();
        delete___MODULE_NAME___service(&database, created.id).await.unwrap();

        let result = get___MODULE_NAME___service(&database, created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let database = create_test_database().await;

        // Test empty name
        let request = Create__MODULE_STRUCT__Request {
            name: "".to_string(),
            description: None,
        };

        let result = create___MODULE_NAME___service(&database, request).await;
        assert!(result.is_err());
    }
}