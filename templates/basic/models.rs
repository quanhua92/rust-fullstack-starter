//! __MODULE_STRUCT__ data models and request/response types

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
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new __MODULE_NAME__
#[derive(Debug, Deserialize, ToSchema)]
pub struct Create__MODULE_STRUCT__Request {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update an existing __MODULE_NAME__
#[derive(Debug, Deserialize, ToSchema)]
pub struct Update__MODULE_STRUCT__Request {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request for listing __MODULE_NAME_PLURAL__
#[derive(Debug)]
pub struct List__MODULE_STRUCT__Request {
    pub limit: i32,
    pub offset: i32,
    pub search: Option<String>,
}

impl __MODULE_STRUCT__ {
    /// Create a new __MODULE_NAME__ instance
    pub fn new(name: String, description: Option<String>, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
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
            created_by,
        );

        assert_eq!(__MODULE_NAME__.name, "Test __MODULE_STRUCT__");
        assert_eq!(__MODULE_NAME__.description, Some("Test description".to_string()));
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
            created_by,
        );

        let original_created_at = __MODULE_NAME__.created_at;
        let original_updated_at = __MODULE_NAME__.updated_at;

        // Sleep briefly to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let update_request = Update__MODULE_STRUCT__Request {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
        };

        __MODULE_NAME__.update(update_request);

        assert_eq!(__MODULE_NAME__.name, "Updated Name");
        assert_eq!(__MODULE_NAME__.description, Some("Updated description".to_string()));
        assert_eq!(__MODULE_NAME__.created_at, original_created_at);
        assert!(__MODULE_NAME__.updated_at > original_updated_at);
    }
}