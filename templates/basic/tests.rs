//! Integration tests for __MODULE_NAME_PLURAL__ module

use crate::helpers::{db::*, test_app::*, test_data::*};
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test___MODULE_NAME___crud_workflow() {
    let app = create_test_app().await;
    let server = TestServer::new(app.into_make_service()).unwrap();

    // Create test user and get auth token
    let auth_user = create_test_user(&server).await;
    let token = get_auth_token(&server, &auth_user).await;

    // Test CREATE
    let create_request = json!({
        "name": "Test __MODULE_STRUCT__",
        "description": "Test description"
    });

    let create_response = server
        .post("/api/v1/__MODULE_NAME_PLURAL__")
        .add_header("authorization", format!("Bearer {}", token))
        .json(&create_request)
        .await;

    create_response.assert_status_ok();
    let created___MODULE_NAME__: serde_json::Value = create_response.json();
    let __MODULE_NAME___id = created___MODULE_NAME__["data"]["id"].as_str().unwrap();

    // Test READ (get single)
    let get_response = server
        .get(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id))
        .add_header("authorization", format!("Bearer {}", token))
        .await;

    get_response.assert_status_ok();
    let retrieved___MODULE_NAME__: serde_json::Value = get_response.json();
    assert_eq!(retrieved___MODULE_NAME__["data"]["name"], "Test __MODULE_STRUCT__");

    // Test READ (list)
    let list_response = server
        .get("/api/v1/__MODULE_NAME_PLURAL__")
        .add_header("authorization", format!("Bearer {}", token))
        .await;

    list_response.assert_status_ok();
    let __MODULE_NAME_PLURAL___list: serde_json::Value = list_response.json();
    assert!(
        __MODULE_NAME_PLURAL___list["data"].as_array().unwrap().len() >= 1
    );

    // Test UPDATE
    let update_request = json!({
        "name": "Updated __MODULE_STRUCT__",
        "description": "Updated description"
    });

    let update_response = server
        .put(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id))
        .add_header("authorization", format!("Bearer {}", token))
        .json(&update_request)
        .await;

    update_response.assert_status_ok();
    let updated___MODULE_NAME__: serde_json::Value = update_response.json();
    assert_eq!(updated___MODULE_NAME__["data"]["name"], "Updated __MODULE_STRUCT__");

    // Test DELETE
    let delete_response = server
        .delete(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id))
        .add_header("authorization", format!("Bearer {}", token))
        .await;

    delete_response.assert_status_ok();

    // Verify deletion
    let get_deleted_response = server
        .get(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id))
        .add_header("authorization", format!("Bearer {}", token))
        .await;

    get_deleted_response.assert_status(404);
}

#[tokio::test]
async fn test___MODULE_NAME___list_with_search() {
    let app = create_test_app().await;
    let server = TestServer::new(app.into_make_service()).unwrap();

    let auth_user = create_test_user(&server).await;
    let token = get_auth_token(&server, &auth_user).await;

    // Create test __MODULE_NAME_PLURAL__
    for i in 1..=3 {
        let create_request = json!({
            "name": format!("Test __MODULE_STRUCT__ {}", i),
            "description": format!("Description {}", i)
        });

        server
            .post("/api/v1/__MODULE_NAME_PLURAL__")
            .add_header("authorization", format!("Bearer {}", token))
            .json(&create_request)
            .await
            .assert_status_ok();
    }

    // Test search
    let search_response = server
        .get("/api/v1/__MODULE_NAME_PLURAL__?search=Test")
        .add_header("authorization", format!("Bearer {}", token))
        .await;

    search_response.assert_status_ok();
    let search_results: serde_json::Value = search_response.json();
    assert!(search_results["data"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test___MODULE_NAME___access_control() {
    let app = create_test_app().await;
    let server = TestServer::new(app.into_make_service()).unwrap();

    // Test without authentication
    let unauth_response = server.get("/api/v1/__MODULE_NAME_PLURAL__").await;
    unauth_response.assert_status(401);

    // Test with invalid token
    let invalid_response = server
        .get("/api/v1/__MODULE_NAME_PLURAL__")
        .add_header("authorization", "Bearer invalid_token")
        .await;
    invalid_response.assert_status(401);
}