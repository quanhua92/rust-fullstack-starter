//! __MODULE_STRUCT__ data models and request/response types with advanced features

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

/// __MODULE_STRUCT__ database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct __MODULE_STRUCT__ {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: __MODULE_STRUCT__Status,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// __MODULE_STRUCT__ status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "__MODULE_NAME___status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum __MODULE_STRUCT__Status {
    Active,
    Inactive,
    Pending,
    Archived,
}

/// Request to create a new __MODULE_NAME__
#[derive(Debug, Deserialize, ToSchema)]
pub struct Create__MODULE_STRUCT__Request {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<__MODULE_STRUCT__Status>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Request to update an existing __MODULE_NAME__
#[derive(Debug, Deserialize, ToSchema)]
pub struct Update__MODULE_STRUCT__Request {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<__MODULE_STRUCT__Status>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

/// Advanced filtering and pagination request
#[derive(Debug, Deserialize, ToSchema)]
pub struct List__MODULE_STRUCT__Request {
    /// Number of items per page (max 100)
    pub limit: Option<i32>,
    /// Page offset (0-based)
    pub offset: Option<i32>,
    /// Cursor for pagination (alternative to offset)
    pub cursor: Option<String>,
    /// Text search in name and description
    pub search: Option<String>,
    /// Filter by status
    pub status: Option<Vec<__MODULE_STRUCT__Status>>,
    /// Filter by priority range
    pub min_priority: Option<i32>,
    pub max_priority: Option<i32>,
    /// Filter by creation date range
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    /// Sort field and direction
    pub sort_by: Option<__MODULE_STRUCT__SortField>,
    pub sort_order: Option<SortOrder>,
}

/// Sort fields for __MODULE_STRUCT__
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum __MODULE_STRUCT__SortField {
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

/// Paginated response for listing __MODULE_NAME_PLURAL__
#[derive(Debug, Serialize, ToSchema)]
pub struct __MODULE_STRUCT__ListResponse {
    pub items: Vec<__MODULE_STRUCT__>,
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
pub struct Bulk__MODULE_STRUCT__CreateRequest {
    pub items: Vec<Create__MODULE_STRUCT__Request>,
    pub skip_errors: Option<bool>,
}

/// Bulk update request
#[derive(Debug, Deserialize, ToSchema)]
pub struct Bulk__MODULE_STRUCT__UpdateRequest {
    pub items: Vec<BulkUpdateItem>,
    pub skip_errors: Option<bool>,
}

/// Single item in bulk update
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkUpdateItem {
    pub id: Uuid,
    pub data: Update__MODULE_STRUCT__Request,
}

/// Bulk delete request
#[derive(Debug, Deserialize, ToSchema)]
pub struct Bulk__MODULE_STRUCT__DeleteRequest {
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

impl Default for __MODULE_STRUCT__Status {
    fn default() -> Self {
        Self::Active
    }
}

impl Default for List__MODULE_STRUCT__Request {
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
            sort_by: Some(__MODULE_STRUCT__SortField::CreatedAt),
            sort_order: Some(SortOrder::Desc),
        }
    }
}

impl __MODULE_STRUCT__ {
    /// Create a new __MODULE_NAME__ instance
    pub fn new(
        name: String,
        description: Option<String>,
        status: Option<__MODULE_STRUCT__Status>,
        priority: Option<i32>,
        metadata: Option<serde_json::Value>,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: status.unwrap_or_default(),
            priority: priority.unwrap_or(0),
            metadata: metadata.unwrap_or_else(|| serde_json::json!({})),
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update the __MODULE_NAME__ with new values
    pub fn update(&mut self, request: Update__MODULE_STRUCT__Request) {
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
    fn test___MODULE_NAME___creation() {
        let created_by = Uuid::new_v4();
        let __MODULE_NAME__ = __MODULE_STRUCT__::new(
            "Test __MODULE_STRUCT__".to_string(),
            Some("Test description".to_string()),
            Some(__MODULE_STRUCT__Status::Active),
            Some(10),
            Some(serde_json::json!({"key": "value"})),
            created_by,
        );

        assert_eq!(__MODULE_NAME__.name, "Test __MODULE_STRUCT__");
        assert_eq!(__MODULE_NAME__.description, Some("Test description".to_string()));
        assert!(matches!(__MODULE_NAME__.status, __MODULE_STRUCT__Status::Active));
        assert_eq!(__MODULE_NAME__.priority, 10);
        assert_eq!(__MODULE_NAME__.metadata["key"], "value");
        assert_eq!(__MODULE_NAME__.created_by, created_by);
        assert!(__MODULE_NAME__.created_at <= Utc::now());
        assert!(__MODULE_NAME__.updated_at <= Utc::now());
    }

    #[test]
    fn test___MODULE_NAME___update() {
        let created_by = Uuid::new_v4();
        let mut __MODULE_NAME__ = __MODULE_STRUCT__::new(
            "Original Name".to_string(),
            Some("Original description".to_string()),
            Some(__MODULE_STRUCT__Status::Pending),
            Some(5),
            None,
            created_by,
        );

        let update_request = Update__MODULE_STRUCT__Request {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(15),
            metadata: Some(serde_json::json!({"updated": true})),
        };

        __MODULE_NAME__.update(update_request);

        assert_eq!(__MODULE_NAME__.name, "Updated Name");
        assert_eq!(__MODULE_NAME__.description, Some("Updated description".to_string()));
        assert!(matches!(__MODULE_NAME__.status, __MODULE_STRUCT__Status::Active));
        assert_eq!(__MODULE_NAME__.priority, 15);
        assert_eq!(__MODULE_NAME__.metadata["updated"], true);
        assert_eq!(__MODULE_NAME__.created_by, created_by); // Should remain unchanged
        assert!(__MODULE_NAME__.updated_at > __MODULE_NAME__.created_at);
    }

    #[test]
    fn test_list_request_defaults() {
        let request = List__MODULE_STRUCT__Request::default();
        assert_eq!(request.limit, Some(20));
        assert_eq!(request.offset, Some(0));
        assert!(request.sort_by.is_some());
        assert!(request.sort_order.is_some());
    }
}