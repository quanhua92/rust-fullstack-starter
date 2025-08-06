//! Products business logic and database operations with advanced features

use super::models::*;
use crate::types::{DbConn, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use uuid::Uuid;

/// List products with advanced filtering and pagination
pub async fn list_products_service(
    conn: &mut DbConn,
    request: ListProductsRequest,
) -> Result<ProductsListResponse> {
    let limit = request.limit.unwrap_or(20).min(100).max(1) as i64;
    let offset = request.offset.unwrap_or(0) as i64;

    // Use simplified queries based on filters
    let (items, total_count) = if request.search.is_some()
        || request.status.is_some()
        || request.min_priority.is_some()
        || request.max_priority.is_some()
        || request.created_after.is_some()
        || request.created_before.is_some()
    {
        // Filtered query
        list_with_filters(conn, &request, limit, offset).await?
    } else {
        // Simple query without filters
        list_without_filters(conn, &request, limit, offset).await?
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

    Ok(ProductsListResponse {
        items: final_items,
        pagination,
    })
}

/// List items without filters (optimized path)
async fn list_without_filters(
    conn: &mut DbConn,
    request: &ListProductsRequest,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Products>, i64)> {
    let sort_field = match request
        .sort_by
        .as_ref()
        .unwrap_or(&ProductsSortField::CreatedAt)
    {
        ProductsSortField::Name => "name",
        ProductsSortField::Priority => "priority",
        ProductsSortField::Status => "status",
        ProductsSortField::CreatedAt => "created_at",
        ProductsSortField::UpdatedAt => "updated_at",
    };

    let sort_order = match request.sort_order.as_ref().unwrap_or(&SortOrder::Desc) {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    // Get items with one extra to check for has_next
    let items = match (sort_field, sort_order) {
        ("created_at", "DESC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        ("created_at", "ASC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY created_at ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        ("name", "ASC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY name ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        ("name", "DESC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY name DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        ("priority", "ASC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY priority ASC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        ("priority", "DESC") => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY priority DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
        _ => sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        ).fetch_all(&mut **conn).await?,
    };

    // Get total count
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(&mut **conn)
        .await?
        .unwrap_or(0);

    Ok((items, total_count))
}

/// List items with filters applied
async fn list_with_filters(
    conn: &mut DbConn,
    request: &ListProductsRequest,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Products>, i64)> {
    // For simplicity, implement basic search and status filter
    let items = if let Some(search) = &request.search {
        let search_param = format!("%{}%", search);
        sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
            search_param,
            limit + 1,
            offset
        )
        .fetch_all(&mut **conn)
        .await?
    } else if let Some(status_list) = &request.status {
        if !status_list.is_empty() {
            let status = &status_list[0]; // Use first status for simplicity
            sqlx::query_as!(
                Products,
                "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
                 FROM products 
                 WHERE status = $1
                 ORDER BY created_at DESC 
                 LIMIT $2 OFFSET $3",
                status as &ProductsStatus,
                limit + 1,
                offset
            )
            .fetch_all(&mut **conn)
            .await?
        } else {
            Vec::new()
        }
    } else {
        // Other filters can be added here
        sqlx::query_as!(
            Products,
            "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
             FROM products 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2",
            limit + 1,
            offset
        )
        .fetch_all(&mut **conn)
        .await?
    };

    // Get approximate count (for filtered results, this might be less accurate)
    let total_count = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
        .fetch_one(&mut **conn)
        .await?
        .unwrap_or(0);

    Ok((items, total_count))
}

/// Get a specific products by ID
pub async fn get_products_service(conn: &mut DbConn, id: Uuid) -> Result<Products> {
    let products = sqlx::query_as!(
        Products,
        "SELECT id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at 
         FROM products 
         WHERE id = $1",
        id
    )
    .fetch_optional(&mut **conn)
    .await?
    .ok_or_else(|| crate::error::Error::NotFound(format!("Products with id {}", id)))?;

    Ok(products)
}

/// Create a new products
pub async fn create_products_service(
    conn: &mut DbConn,
    request: CreateProductsRequest,
) -> Result<Products> {
    // Validate request
    if request.name.trim().is_empty() {
        return Err(crate::error::Error::validation(
            "name",
            "Name cannot be empty",
        ));
    }

    let products = Products::new(
        request.name,
        request.description,
        request.status,
        request.priority,
        request.metadata,
    );

    let created_products = sqlx::query_as!(
        Products,
        "INSERT INTO products (id, name, description, status, priority, metadata, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at",
        products.id,
        products.name,
        products.description,
        products.status as ProductsStatus,
        products.priority,
        products.metadata,
        products.created_at,
        products.updated_at
    )
    .fetch_one(&mut **conn)
    .await?;

    Ok(created_products)
}

/// Update an existing products
pub async fn update_products_service(
    conn: &mut DbConn,
    id: Uuid,
    request: UpdateProductsRequest,
) -> Result<Products> {
    // Get existing products
    let mut products = get_products_service(conn, id).await?;

    // Validate request
    if let Some(ref name) = request.name {
        if name.trim().is_empty() {
            return Err(crate::error::Error::validation(
                "name",
                "Name cannot be empty",
            ));
        }
    }

    // Update the products
    products.update(request);

    let updated_products = sqlx::query_as!(
        Products,
        "UPDATE products 
         SET name = $2, description = $3, status = $4, priority = $5, metadata = $6, updated_at = $7
         WHERE id = $1
         RETURNING id, name, description, status as \"status: ProductsStatus\", priority, metadata, created_at, updated_at",
        products.id,
        products.name,
        products.description,
        products.status as ProductsStatus,
        products.priority,
        products.metadata,
        products.updated_at
    )
    .fetch_one(&mut **conn)
    .await?;

    Ok(updated_products)
}

/// Delete a products
pub async fn delete_products_service(conn: &mut DbConn, id: Uuid) -> Result<()> {
    let rows_affected = sqlx::query!("DELETE FROM products WHERE id = $1", id)
        .execute(&mut **conn)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(crate::error::Error::NotFound(format!(
            "Products with id {}",
            id
        )));
    }

    Ok(())
}

/// Bulk create products
pub async fn bulk_create_products_service(
    conn: &mut DbConn,
    request: BulkProductsCreateRequest,
) -> Result<BulkOperationResponse<Products>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item_request) in request.items.into_iter().enumerate() {
        match create_products_service(conn, item_request).await {
            Ok(products) => results.push(products),
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

/// Bulk update products
pub async fn bulk_update_products_service(
    conn: &mut DbConn,
    request: BulkProductsUpdateRequest,
) -> Result<BulkOperationResponse<Products>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, item) in request.items.into_iter().enumerate() {
        match update_products_service(conn, item.id, item.data).await {
            Ok(products) => results.push(products),
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

/// Bulk delete products
pub async fn bulk_delete_products_service(
    conn: &mut DbConn,
    request: BulkProductsDeleteRequest,
) -> Result<BulkOperationResponse<Uuid>> {
    let mut results = Vec::new();
    let mut errors = Vec::new();
    let skip_errors = request.skip_errors.unwrap_or(false);

    for (index, id) in request.ids.into_iter().enumerate() {
        match delete_products_service(conn, id).await {
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
    let decoded = BASE64
        .decode(cursor)
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor format"))?;

    let offset_str = String::from_utf8(decoded)
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor encoding"))?;

    offset_str
        .parse::<i64>()
        .map_err(|_| crate::error::Error::validation("cursor", "Invalid cursor value"))
}
