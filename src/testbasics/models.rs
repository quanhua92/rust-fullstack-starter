//! Testbasic data models and request/response types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use utoipa::ToSchema;

/// Testbasic database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Testbasic {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new testbasic
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTestbasicRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update an existing testbasic
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTestbasicRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Request for listing testbasics
#[derive(Debug)]
pub struct ListTestbasicRequest {
    pub limit: i32,
    pub offset: i32,
    pub search: Option<String>,
}

impl Testbasic {
    /// Create a new testbasic instance
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update the testbasic with new values
    pub fn update(&mut self, request: UpdateTestbasicRequest) {
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
    fn test_testbasic_creation() {
        let testbasic = Testbasic::new(
            "Test Testbasic".to_string(),
            Some("Test description".to_string()),
        );

        assert_eq!(testbasic.name, "Test Testbasic");
        assert_eq!(testbasic.description, Some("Test description".to_string()));
        assert!(testbasic.created_at <= Utc::now());
        assert!(testbasic.updated_at <= Utc::now());
    }

    #[test]
    fn test_testbasic_update() {
        let mut testbasic = Testbasic::new(
            "Original Name".to_string(),
            Some("Original description".to_string()),
        );

        let original_created_at = testbasic.created_at;
        let original_updated_at = testbasic.updated_at;

        // Sleep briefly to ensure updated_at changes
        std::thread::sleep(std::time::Duration::from_millis(1));

        let update_request = UpdateTestbasicRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
        };

        testbasic.update(update_request);

        assert_eq!(testbasic.name, "Updated Name");
        assert_eq!(testbasic.description, Some("Updated description".to_string()));
        assert_eq!(testbasic.created_at, original_created_at);
        assert!(testbasic.updated_at > original_updated_at);
    }
}