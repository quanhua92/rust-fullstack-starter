//! Integration tests for testbasics module

use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_testbasic_crud_workflow() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user
    let (_user, token) = factory.create_authenticated_user("testuser").await;

    // Test CREATE
    let create_data = json!({
        "name": "Test Testbasic",
        "description": "Test description"
    });

    let response = app
        .post_auth("/api/v1/testbasics", &token.token, &create_data)
        .await;

    assert_status(&response, StatusCode::OK);
    let created: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&created, "data");
    let testbasic_id = created["data"]["id"].as_str().unwrap();

    // Test READ (get single)
    let response = app
        .get_auth(&format!("/api/v1/testbasics/{}", testbasic_id), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let retrieved: serde_json::Value = response.json().await.unwrap();
    assert_eq!(retrieved["data"]["name"], "Test Testbasic");

    // Test READ (list)
    let response = app.get_auth("/api/v1/testbasics", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let list: serde_json::Value = response.json().await.unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Test UPDATE
    let update_data = json!({
        "name": "Updated Testbasic",
        "description": "Updated description"
    });

    let response = app
        .put_auth(
            &format!("/api/v1/testbasics/{}", testbasic_id),
            &token.token,
            &update_data,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let updated: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated["data"]["name"], "Updated Testbasic");

    // Test DELETE
    let response = app
        .delete_auth(&format!("/api/v1/testbasics/{}", testbasic_id), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);

    // Verify deletion
    let response = app
        .get_auth(&format!("/api/v1/testbasics/{}", testbasic_id), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_testbasic_list_with_search() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_user("searchuser").await;

    // Create test testbasics
    for i in 1..=3 {
        let create_data = json!({
            "name": format!("Test Testbasic {}", i),
            "description": format!("Description {}", i)
        });

        let response = app
            .post_auth("/api/v1/testbasics", &token.token, &create_data)
            .await;

        assert_status(&response, StatusCode::OK);
    }

    // Test search
    let response = app
        .get_auth("/api/v1/testbasics?search=Test", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let results: serde_json::Value = response.json().await.unwrap();
    assert!(results["data"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test_testbasic_access_control() {
    let app = spawn_app().await;

    // Test without authentication
    let response = app.client.get(&format!("{}/api/v1/testbasics", &app.address)).send().await.unwrap();
    assert_status(&response, StatusCode::UNAUTHORIZED);

    // Test with invalid token
    let response = app.client
        .get(&format!("{}/api/v1/testbasics", &app.address))
        .header("Authorization", "Bearer invalid_token")
        .send()
        .await
        .unwrap();
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_testbasic_validation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let (_user, token) = factory.create_authenticated_user("validationuser").await;

    // Test empty name
    let create_data = json!({
        "name": "",
        "description": "Test description"
    });

    let response = app
        .post_auth("/api/v1/testbasics", &token.token, &create_data)
        .await;

    assert_status(&response, StatusCode::BAD_REQUEST);
}