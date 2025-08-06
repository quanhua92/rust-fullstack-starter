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
    assert!(!list["data"].as_array().unwrap().is_empty());

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
    let items = list["data"].as_array().unwrap();
    assert!(items.len() >= 2); // Alpha and Gamma

    // Test search functionality
    let response = app
        .get_auth("/api/v1/__MODULE_NAME_PLURAL__?search=Alpha", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    let items = list["data"].as_array().unwrap();
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

    let (_user, token) = factory.create_authenticated_user("bulkuser").await;

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
}