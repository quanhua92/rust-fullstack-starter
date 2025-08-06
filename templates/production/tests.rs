//! Comprehensive tests for __MODULE_STRUCT__ module with advanced features

use uuid::Uuid;
use serde_json::json;

use crate::{
    __MODULE_NAME_PLURAL__::{models::*, services::*},
    helpers::db::*,
};

#[tokio::test]
async fn test___MODULE_NAME___lifecycle() {
    let database = create_test_database().await;

    // Test creation with all fields
    let create_request = Create__MODULE_STRUCT__Request {
        name: "Test __MODULE_STRUCT__".to_string(),
        description: Some("Test description".to_string()),
        status: Some(__MODULE_STRUCT__Status::Active),
        priority: Some(10),
        metadata: Some(json!({"key": "value", "number": 42})),
    };

    let created = create___MODULE_NAME___service(&database, create_request).await.unwrap();
    assert_eq!(created.name, "Test __MODULE_STRUCT__");
    assert_eq!(created.priority, 10);
    assert!(matches!(created.status, __MODULE_STRUCT__Status::Active));
    assert_eq!(created.metadata["key"], "value");

    // Test retrieval
    let retrieved = get___MODULE_NAME___service(&database, created.id).await.unwrap();
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.metadata["number"], 42);

    // Test update
    let update_request = Update__MODULE_STRUCT__Request {
        name: Some("Updated Name".to_string()),
        description: Some("Updated description".to_string()),
        status: Some(__MODULE_STRUCT__Status::Inactive),
        priority: Some(20),
        metadata: Some(json!({"updated": true})),
    };

    let updated = update___MODULE_NAME___service(&database, created.id, update_request).await.unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.priority, 20);
    assert!(matches!(updated.status, __MODULE_STRUCT__Status::Inactive));
    assert_eq!(updated.metadata["updated"], true);
    assert!(updated.updated_at > created.updated_at);

    // Test deletion
    delete___MODULE_NAME___service(&database, created.id).await.unwrap();
    let result = get___MODULE_NAME___service(&database, created.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_advanced_filtering() {
    let database = create_test_database().await;

    // Create test data with different statuses and priorities
    let test_data = vec![
        ("Alpha Item", __MODULE_STRUCT__Status::Active, 10),
        ("Beta Item", __MODULE_STRUCT__Status::Inactive, 20),
        ("Gamma Item", __MODULE_STRUCT__Status::Active, 15),
        ("Delta Item", __MODULE_STRUCT__Status::Pending, 5),
        ("Epsilon Item", __MODULE_STRUCT__Status::Archived, 25),
    ];

    for (name, status, priority) in test_data {
        let request = Create__MODULE_STRUCT__Request {
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            status: Some(status),
            priority: Some(priority),
            metadata: Some(json!({"category": "test"})),
        };
        create___MODULE_NAME___service(&database, request).await.unwrap();
    }

    // Test status filtering
    let request = List__MODULE_STRUCT__Request {
        status: Some(vec![__MODULE_STRUCT__Status::Active]),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    assert_eq!(response.items.len(), 2); // Alpha and Gamma

    // Test priority range filtering
    let request = List__MODULE_STRUCT__Request {
        min_priority: Some(10),
        max_priority: Some(20),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    assert_eq!(response.items.len(), 3); // Alpha, Beta, Gamma

    // Test text search
    let request = List__MODULE_STRUCT__Request {
        search: Some("Alpha".to_string()),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    assert_eq!(response.items.len(), 1);
    assert!(response.items[0].name.contains("Alpha"));

    // Test combined filters
    let request = List__MODULE_STRUCT__Request {
        status: Some(vec![__MODULE_STRUCT__Status::Active, __MODULE_STRUCT__Status::Inactive]),
        min_priority: Some(15),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    assert_eq!(response.items.len(), 2); // Beta and Gamma
}

#[tokio::test]
async fn test_pagination() {
    let database = create_test_database().await;

    // Create 25 test items
    for i in 1..=25 {
        let request = Create__MODULE_STRUCT__Request {
            name: format!("Item {:02}", i),
            description: Some(format!("Description {}", i)),
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(i),
            metadata: Some(json!({"index": i})),
        };
        create___MODULE_NAME___service(&database, request).await.unwrap();
    }

    // Test first page
    let request = List__MODULE_STRUCT__Request {
        limit: Some(10),
        offset: Some(0),
        sort_by: Some(__MODULE_STRUCT__SortField::Priority),
        sort_order: Some(SortOrder::Asc),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.items.len(), 10);
    assert_eq!(response.pagination.total_count, 25);
    assert_eq!(response.pagination.current_page, 1);
    assert_eq!(response.pagination.page_count, 3);
    assert!(!response.pagination.has_prev);
    assert!(response.pagination.has_next);
    assert!(response.pagination.next_cursor.is_some());

    // Test second page
    let request = List__MODULE_STRUCT__Request {
        limit: Some(10),
        offset: Some(10),
        sort_by: Some(__MODULE_STRUCT__SortField::Priority),
        sort_order: Some(SortOrder::Asc),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.items.len(), 10);
    assert_eq!(response.pagination.current_page, 2);
    assert!(response.pagination.has_prev);
    assert!(response.pagination.has_next);

    // Test last page
    let request = List__MODULE_STRUCT__Request {
        limit: Some(10),
        offset: Some(20),
        sort_by: Some(__MODULE_STRUCT__SortField::Priority),
        sort_order: Some(SortOrder::Asc),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.items.len(), 5);
    assert_eq!(response.pagination.current_page, 3);
    assert!(response.pagination.has_prev);
    assert!(!response.pagination.has_next);
    assert!(response.pagination.next_cursor.is_none());
}

#[tokio::test]
async fn test_sorting() {
    let database = create_test_database().await;

    // Create test items with different priorities
    let priorities = vec![30, 10, 20];
    for priority in priorities {
        let request = Create__MODULE_STRUCT__Request {
            name: format!("Priority {}", priority),
            description: None,
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(priority),
            metadata: None,
        };
        create___MODULE_NAME___service(&database, request).await.unwrap();
    }

    // Test ascending sort
    let request = List__MODULE_STRUCT__Request {
        sort_by: Some(__MODULE_STRUCT__SortField::Priority),
        sort_order: Some(SortOrder::Asc),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.items.len(), 3);
    assert_eq!(response.items[0].priority, 10);
    assert_eq!(response.items[1].priority, 20);
    assert_eq!(response.items[2].priority, 30);

    // Test descending sort
    let request = List__MODULE_STRUCT__Request {
        sort_by: Some(__MODULE_STRUCT__SortField::Priority),
        sort_order: Some(SortOrder::Desc),
        ..Default::default()
    };
    let response = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.items[0].priority, 30);
    assert_eq!(response.items[1].priority, 20);
    assert_eq!(response.items[2].priority, 10);
}

#[tokio::test]
async fn test_bulk_create() {
    let database = create_test_database().await;

    let request = Bulk__MODULE_STRUCT__CreateRequest {
        items: vec![
            Create__MODULE_STRUCT__Request {
                name: "Bulk Item 1".to_string(),
                description: Some("Bulk description 1".to_string()),
                status: Some(__MODULE_STRUCT__Status::Active),
                priority: Some(1),
                metadata: Some(json!({"bulk": true})),
            },
            Create__MODULE_STRUCT__Request {
                name: "Bulk Item 2".to_string(),
                description: Some("Bulk description 2".to_string()),
                status: Some(__MODULE_STRUCT__Status::Inactive),
                priority: Some(2),
                metadata: Some(json!({"bulk": true})),
            },
            Create__MODULE_STRUCT__Request {
                name: "".to_string(), // This should fail validation
                description: None,
                status: Some(__MODULE_STRUCT__Status::Active),
                priority: Some(3),
                metadata: None,
            },
        ],
        skip_errors: Some(true),
    };

    let response = bulk_create___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.success_count, 2);
    assert_eq!(response.error_count, 1);
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].index, 2);
}

#[tokio::test]
async fn test_bulk_update() {
    let database = create_test_database().await;

    // Create test items first
    let mut ids = Vec::new();
    for i in 1..=3 {
        let request = Create__MODULE_STRUCT__Request {
            name: format!("Item {}", i),
            description: Some(format!("Description {}", i)),
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(i),
            metadata: None,
        };
        let created = create___MODULE_NAME___service(&database, request).await.unwrap();
        ids.push(created.id);
    }

    // Bulk update
    let request = Bulk__MODULE_STRUCT__UpdateRequest {
        items: vec![
            BulkUpdateItem {
                id: ids[0],
                data: Update__MODULE_STRUCT__Request {
                    name: Some("Updated Item 1".to_string()),
                    status: Some(__MODULE_STRUCT__Status::Inactive),
                    priority: Some(10),
                    description: None,
                    metadata: None,
                },
            },
            BulkUpdateItem {
                id: ids[1],
                data: Update__MODULE_STRUCT__Request {
                    name: Some("Updated Item 2".to_string()),
                    status: Some(__MODULE_STRUCT__Status::Pending),
                    priority: Some(20),
                    description: None,
                    metadata: None,
                },
            },
            BulkUpdateItem {
                id: Uuid::new_v4(), // Non-existent ID
                data: Update__MODULE_STRUCT__Request {
                    name: Some("Should Fail".to_string()),
                    status: None,
                    priority: None,
                    description: None,
                    metadata: None,
                },
            },
        ],
        skip_errors: Some(true),
    };

    let response = bulk_update___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.success_count, 2);
    assert_eq!(response.error_count, 1);
    assert_eq!(response.results.len(), 2);
    
    // Verify updates
    let updated1 = get___MODULE_NAME___service(&database, ids[0]).await.unwrap();
    assert_eq!(updated1.name, "Updated Item 1");
    assert!(matches!(updated1.status, __MODULE_STRUCT__Status::Inactive));
}

#[tokio::test]
async fn test_bulk_delete() {
    let database = create_test_database().await;

    // Create test items
    let mut ids = Vec::new();
    for i in 1..=3 {
        let request = Create__MODULE_STRUCT__Request {
            name: format!("Item {}", i),
            description: None,
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(i),
            metadata: None,
        };
        let created = create___MODULE_NAME___service(&database, request).await.unwrap();
        ids.push(created.id);
    }

    // Add non-existent ID to test error handling
    ids.push(Uuid::new_v4());

    let request = Bulk__MODULE_STRUCT__DeleteRequest {
        ids,
        skip_errors: Some(true),
    };

    let response = bulk_delete___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(response.success_count, 3);
    assert_eq!(response.error_count, 1);
    assert_eq!(response.results.len(), 3);
}

#[tokio::test]
async fn test_cursor_pagination() {
    let database = create_test_database().await;

    // Create test items
    for i in 1..=15 {
        let request = Create__MODULE_STRUCT__Request {
            name: format!("Item {:02}", i),
            description: None,
            status: Some(__MODULE_STRUCT__Status::Active),
            priority: Some(i),
            metadata: None,
        };
        create___MODULE_NAME___service(&database, request).await.unwrap();
    }

    // Get first page
    let request = List__MODULE_STRUCT__Request {
        limit: Some(5),
        offset: Some(0),
        ..Default::default()
    };
    let first_page = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(first_page.items.len(), 5);
    assert!(first_page.pagination.next_cursor.is_some());

    // Use cursor for next page
    let cursor = first_page.pagination.next_cursor.unwrap();
    let parsed_offset = parse_cursor(&cursor).unwrap();
    assert_eq!(parsed_offset, 5);

    let request = List__MODULE_STRUCT__Request {
        limit: Some(5),
        cursor: Some(cursor),
        ..Default::default()
    };
    let second_page = list___MODULE_NAME_PLURAL___service(&database, request).await.unwrap();
    
    assert_eq!(second_page.items.len(), 5);
    assert_eq!(second_page.pagination.current_page, 2);
}

#[tokio::test]
async fn test_validation_errors() {
    let database = create_test_database().await;

    // Test empty name validation
    let request = Create__MODULE_STRUCT__Request {
        name: "".to_string(),
        description: None,
        status: None,
        priority: None,
        metadata: None,
    };

    let result = create___MODULE_NAME___service(&database, request).await;
    assert!(result.is_err());

    // Test whitespace-only name validation
    let request = Create__MODULE_STRUCT__Request {
        name: "   ".to_string(),
        description: None,
        status: None,
        priority: None,
        metadata: None,
    };

    let result = create___MODULE_NAME___service(&database, request).await;
    assert!(result.is_err());
}

#[tokio::test] 
async fn test_default_values() {
    let database = create_test_database().await;

    // Test creation with minimal data (using defaults)
    let request = Create__MODULE_STRUCT__Request {
        name: "Minimal Item".to_string(),
        description: None,
        status: None,
        priority: None,
        metadata: None,
    };

    let created = create___MODULE_NAME___service(&database, request).await.unwrap();
    
    assert_eq!(created.name, "Minimal Item");
    assert!(matches!(created.status, __MODULE_STRUCT__Status::Active));
    assert_eq!(created.priority, 0);
    assert_eq!(created.metadata, json!({}));
    assert!(created.description.is_none());
}