//! Products data models and request/response types with advanced features

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Products database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Products {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ProductsStatus,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Products status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "products_status", rename_all = "lowercase")]
pub enum ProductsStatus {
    Active,
    Inactive,
    Pending,
    Archived,
}

/// Request to create a new products
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProductsRequest {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<ProductsStatus>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Request to update an existing products
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProductsRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProductsStatus>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Advanced filtering and pagination request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListProductsRequest {
    /// Number of items per page (max 100)
    pub limit: Option<i32>,
    /// Page offset (0-based)
    pub offset: Option<i32>,
    /// Cursor for pagination (alternative to offset)
    pub cursor: Option<String>,
    /// Text search in name and description
    pub search: Option<String>,
    /// Filter by status
    pub status: Option<Vec<ProductsStatus>>,
    /// Filter by priority range
    pub min_priority: Option<i32>,
    pub max_priority: Option<i32>,
    /// Filter by creation date range
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    /// Sort field and direction
    pub sort_by: Option<ProductsSortField>,
    pub sort_order: Option<SortOrder>,
}

/// Sort fields for Products
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductsSortField {
    Name,
    Priority,
    Status,
    CreatedAt,
    UpdatedAt,
}

/// Sort order
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Paginated response for listing products
#[derive(Debug, Serialize, ToSchema)]
pub struct ProductsListResponse {
    pub items: Vec<Products>,
    pub pagination: PaginationInfo,
}

/// Pagination metadata
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationInfo {
    pub total_count: i64,
    pub page_count: i64,
    pub current_page: i64,
    pub per_page: i32,
    pub has_next: bool,
    pub has_prev: bool,
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

/// Bulk create request
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkProductsCreateRequest {
    pub items: Vec<CreateProductsRequest>,
    pub skip_errors: Option<bool>,
}

/// Bulk update request
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkProductsUpdateRequest {
    pub items: Vec<BulkUpdateItem>,
    pub skip_errors: Option<bool>,
}

/// Single item in bulk update
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkUpdateItem {
    pub id: Uuid,
    pub data: UpdateProductsRequest,
}

/// Bulk delete request
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkProductsDeleteRequest {
    pub ids: Vec<Uuid>,
    pub skip_errors: Option<bool>,
}

/// Bulk operation response
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkOperationResponse<T> {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<BulkOperationError>,
    pub results: Vec<T>,
}

/// Bulk operation error
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkOperationError {
    pub index: usize,
    pub id: Option<Uuid>,
    pub error: String,
}

impl Default for ProductsStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl Default for ListProductsRequest {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            cursor: None,
            search: None,
            status: None,
            min_priority: None,
            max_priority: None,
            created_after: None,
            created_before: None,
            sort_by: Some(ProductsSortField::CreatedAt),
            sort_order: Some(SortOrder::Desc),
        }
    }
}

impl Products {
    /// Create a new products instance
    pub fn new(
        name: String,
        description: Option<String>,
        status: Option<ProductsStatus>,
        priority: Option<i32>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: status.unwrap_or_default(),
            priority: priority.unwrap_or(0),
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update the products with new values
    pub fn update(&mut self, request: UpdateProductsRequest) {
        if let Some(name) = request.name {
            self.name = name;
        }
        if request.description.is_some() {
            self.description = request.description;
        }
        if let Some(status) = request.status {
            self.status = status;
        }
        if let Some(priority) = request.priority {
            self.priority = priority;
        }
        if let Some(metadata) = request.metadata {
            self.metadata = metadata;
        }
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_products_creation() {
        let products = Products::new(
            "Test Products".to_string(),
            Some("Test description".to_string()),
            Some(ProductsStatus::Active),
            Some(10),
            Some(serde_json::json!({"key": "value"})),
        );

        assert_eq!(products.name, "Test Products");
        assert_eq!(products.description, Some("Test description".to_string()));
        assert!(matches!(products.status, ProductsStatus::Active));
        assert_eq!(products.priority, 10);
        assert_eq!(products.metadata["key"], "value");
    }

    #[test]
    fn test_products_update() {
        let mut products = Products::new(
            "Original Name".to_string(),
            Some("Original description".to_string()),
            Some(ProductsStatus::Pending),
            Some(5),
            None,
        );

        let update_request = UpdateProductsRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
            status: Some(ProductsStatus::Active),
            priority: Some(15),
            metadata: Some(serde_json::json!({"updated": true})),
        };

        products.update(update_request);

        assert_eq!(products.name, "Updated Name");
        assert_eq!(
            products.description,
            Some("Updated description".to_string())
        );
        assert!(matches!(products.status, ProductsStatus::Active));
        assert_eq!(products.priority, 15);
        assert_eq!(products.metadata["updated"], true);
    }

    #[test]
    fn test_list_request_defaults() {
        let request = ListProductsRequest::default();
        assert_eq!(request.limit, Some(20));
        assert_eq!(request.offset, Some(0));
        assert!(request.sort_by.is_some());
        assert!(request.sort_order.is_some());
    }
}
