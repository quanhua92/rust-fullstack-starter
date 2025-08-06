//! Integration tests for __MODULE_NAME_PLURAL__ module with advanced features

use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test___MODULE_NAME___crud_workflow() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user
    let (_user, token) = factory.create_authenticated_user("testuser").await;

    // Test CREATE
    let create_data = json!({
        "name": "Test __MODULE_STRUCT__",
        "description": "Test description",
        "status": "active",
        "priority": 10,
        "metadata": {"key": "value"}
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &create_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let created: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&created, "data");
    let __MODULE_NAME___id = created["data"]["id"].as_str().unwrap();

    // Test READ (get single)
    let response = app
        .get_auth(
            &format!("/api/v1/__MODULE_NAME_PLURAL__/{__MODULE_NAME___id}"),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let retrieved: serde_json::Value = response.json().await.unwrap();
    assert_eq!(retrieved["data"]["name"], "Test __MODULE_STRUCT__");

    // Test READ (list)
    let response = app.get_auth("/api/v1/__MODULE_NAME_PLURAL__", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    assert!(!list["data"]["items"].as_array().unwrap().is_empty());

    // Test UPDATE
    let update_data = json!({
        "name": "Updated __MODULE_STRUCT__",
        "description": "Updated description",
        "status": "inactive",
        "priority": 20
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/__MODULE_NAME_PLURAL__/{__MODULE_NAME___id}"),
            &update_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let updated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated["data"]["name"], "Updated __MODULE_STRUCT__");

    // Test DELETE
    let response = app
        .delete_auth(
            &format!("/api/v1/__MODULE_NAME_PLURAL__/{__MODULE_NAME___id}"),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);

    // Verify deletion
    let response = app
        .get_auth(
            &format!("/api/v1/__MODULE_NAME_PLURAL__/{__MODULE_NAME___id}"),
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test___MODULE_NAME___advanced_filtering() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_user("filteruser").await;

    // Create test items with different statuses
    for (name, status, priority) in [
        ("Alpha Item", "active", 10),
        ("Beta Item", "inactive", 20),
        ("Gamma Item", "active", 15),
    ] {
        let create_data = json!({
            "name": name,
            "description": format!("Description for {name}"),
            "status": status,
            "priority": priority,
            "metadata": {"category": "test"}
        });

        let response = app
            .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &create_data, &token.token)
            .await;

        assert_status(&response, StatusCode::OK);
    }

    // Test status filtering
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?status=active", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert!(items.len() >= 2); // Alpha and Gamma

    // Test search functionality
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?search=Alpha", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert!(!items.is_empty());
    assert!(items[0]["name"].as_str().unwrap().contains("Alpha"));
}

#[tokio::test]
async fn test___MODULE_NAME___validation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_user("validationuser").await;

    // Test empty name
    let create_data = json!({
        "name": "",
        "description": "Test description"
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &create_data, &token.token)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test___MODULE_NAME___bulk_operations() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_moderator("bulkuser").await;

    // Test bulk create
    let bulk_data = json!({
        "items": [
            {
                "name": "Bulk Item 1",
                "description": "Bulk description 1",
                "status": "active",
                "priority": 1
            },
            {
                "name": "Bulk Item 2", 
                "description": "Bulk description 2",
                "status": "inactive",
                "priority": 2
            }
        ],
        "skip_errors": true
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let result: serde_json::Value = response.json().await.unwrap();
    assert_eq!(result["data"]["success_count"], 2);
    assert_eq!(result["data"]["error_count"], 0);
    
    // Store IDs for bulk update/delete tests
    let created_items = result["data"]["results"].as_array().unwrap();
    let item1_id = created_items[0]["id"].as_str().unwrap();
    let item2_id = created_items[1]["id"].as_str().unwrap();

    // Test bulk update
    let bulk_update_data = json!({
        "items": [
            {
                "id": item1_id,
                "data": {
                    "name": "Updated Bulk Item 1",
                    "status": "pending",
                    "priority": 15
                }
            },
            {
                "id": item2_id,
                "data": {
                    "name": "Updated Bulk Item 2",
                    "status": "archived", 
                    "priority": 25
                }
            }
        ],
        "skip_errors": false
    });

    let response = app
        .put_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_update_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let update_result: serde_json::Value = response.json().await.unwrap();
    assert_eq!(update_result["data"]["success_count"], 2);
    assert_eq!(update_result["data"]["error_count"], 0);

    // Verify updates worked
    let response = app
        .get_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{item1_id}"), &token.token)
        .await;
    
    assert_status(&response, StatusCode::OK);
    let updated_item: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated_item["data"]["name"], "Updated Bulk Item 1");
    assert_eq!(updated_item["data"]["status"], "pending");

    // Test bulk delete
    let bulk_delete_data = json!({
        "ids": [item1_id, item2_id],
        "skip_errors": false
    });

    let response = app
        .delete_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_delete_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let delete_result: serde_json::Value = response.json().await.unwrap();
    assert_eq!(delete_result["data"]["success_count"], 2);
    assert_eq!(delete_result["data"]["error_count"], 0);

    // Verify deletions worked
    let response = app
        .get_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{item1_id}"), &token.token)
        .await;
    
    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test___MODULE_NAME___bulk_operations_transaction_safety() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_moderator("bulktxuser").await;

    // Test bulk create with one invalid item (without skip_errors)
    let bulk_data = json!({
        "items": [
            {
                "name": "Valid Item",
                "description": "Valid description",
                "status": "active",
                "priority": 10
            },
            {
                "name": "", // Invalid - empty name should fail
                "description": "Invalid item",
                "status": "active",
                "priority": 5
            }
        ],
        "skip_errors": false
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_data, &token.token)
        .await;

    // Should fail because of transaction rollback
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Verify no items were created (transaction was rolled back)
    let response = app.get_auth("/api/v1/__MODULE_NAME_PLURAL__?search=Valid Item", &token.token).await;
    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert!(items.is_empty(), "Transaction should have rolled back all items");
}

#[tokio::test]
async fn test___MODULE_NAME___rbac_permissions() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create regular user (not moderator/admin)
    let (_user, token) = factory.create_authenticated_user("regularuser").await;

    // Test that bulk operations require moderator permissions
    let bulk_data = json!({
        "items": [{"name": "Test Item", "status": "active", "priority": 1}],
        "skip_errors": true
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_data, &token.token)
        .await;

    // Should be forbidden for regular users
    assert_status(&response, StatusCode::FORBIDDEN);

    // Test bulk update permission
    let bulk_update_data = json!({
        "items": [{"id": "550e8400-e29b-41d4-a716-446655440000", "data": {"name": "Updated"}}],
        "skip_errors": true
    });

    let response = app
        .put_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_update_data, &token.token)
        .await;

    assert_status(&response, StatusCode::FORBIDDEN);

    // Test bulk delete permission
    let bulk_delete_data = json!({
        "ids": ["550e8400-e29b-41d4-a716-446655440000"],
        "skip_errors": true
    });

    let response = app
        .delete_json_auth("/api/v1/__MODULE_NAME_PLURAL__/bulk", &bulk_delete_data, &token.token)
        .await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test___MODULE_NAME___pagination_and_filtering() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_user("paginationuser").await;

    // Create test items with different priorities and creation dates
    for i in 1..=25 {
        let create_data = json!({
            "name": format!("Item {:02}", i),
            "description": format!("Description for item {i}"),
            "status": if i % 2 == 0 { "active" } else { "inactive" },
            "priority": i,
            "metadata": {"index": i}
        });

        let response = app
            .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &create_data, &token.token)
            .await;
        
        assert_status(&response, StatusCode::OK);
    }

    // Test pagination with limit
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?limit=10&offset=0", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 10);

    // Test priority range filtering
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?min_priority=10&max_priority=15", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert!(items.len() >= 6); // Items 10-15

    // Test multiple status filtering
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?status=active,inactive&limit=5", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 5);
}