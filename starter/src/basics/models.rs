//! Basics data models and request/response types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

/// Basics database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Basics {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new basics
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBasicsRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update an existing basics
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBasicsRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request for listing basics
#[derive(Debug)]
pub struct ListBasicsRequest {
    pub limit: i32,
    pub offset: i32,
    pub search: Option<String>,
}

impl Basics {
    /// Create a new basics instance
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update the basics with new values
    pub fn update(&mut self, request: UpdateBasicsRequest) {
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
    fn test_basics_creation() {
        let basics = Basics::new(
            "Test Basics".to_string(),
            Some("Test description".to_string()),
        );

        assert_eq!(basics.name, "Test Basics");
        assert_eq!(basics.description, Some("Test description".to_string()));
        assert!(basics.created_at <= Utc::now());
        assert!(basics.updated_at <= Utc::now());
    }

    #[test]
    fn test_basics_update() {
        let mut basics = Basics::new(
            "Original Name".to_string(),
            Some("Original description".to_string()),
        );

        let original_created_at = basics.created_at;
        let original_updated_at = basics.updated_at;

        // Sleep briefly to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let update_request = UpdateBasicsRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
        };

        basics.update(update_request);

        assert_eq!(basics.name, "Updated Name");
        assert_eq!(basics.description, Some("Updated description".to_string()));
        assert_eq!(basics.created_at, original_created_at);
        assert!(basics.updated_at > original_updated_at);
    }
}