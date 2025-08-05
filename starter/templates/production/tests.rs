use crate::helpers::*;
use reqwest::StatusCode;
use uuid;

#[tokio::test]
async fn test_create___MODULE_NAME__() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let __MODULE_NAME___data = serde_json::json!({
        "title": "Test __MODULE_STRUCT__",
        "content": "This is a test __MODULE_NAME__"
    });

    let response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
    assert_eq!(json["data"]["title"], "Test __MODULE_STRUCT__");
}

#[tokio::test]
async fn test_get___MODULE_NAME___by_id() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a __MODULE_NAME__ first
    let __MODULE_NAME___data = serde_json::json!({
        "title": "Test __MODULE_STRUCT__",
        "content": "This is a test __MODULE_NAME__"
    });

    let create_response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token.token)
        .await;
    
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let __MODULE_NAME___id = create_json["data"]["id"].as_str().unwrap();

    // Get the __MODULE_NAME__ by ID
    let response = app
        .get_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["title"], "Test __MODULE_STRUCT__");
}

#[tokio::test]
async fn test_list___MODULE_NAME_PLURAL__() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a few __MODULE_NAME_PLURAL__
    for i in 1..=3 {
        let __MODULE_NAME___data = serde_json::json!({
            "title": format!("Test __MODULE_STRUCT__ {}", i),
            "content": format!("Content for __MODULE_NAME__ {}", i)
        });

        app.post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token.token)
            .await;
    }

    let response = app.get_auth("/api/v1/__MODULE_NAME_PLURAL__", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
    let __MODULE_NAME_PLURAL__ = json["data"].as_array().unwrap();
    assert!((__MODULE_NAME_PLURAL__.len() >= 3));
}

#[tokio::test]
async fn test_update___MODULE_NAME__() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a __MODULE_NAME__ first
    let __MODULE_NAME___data = serde_json::json!({
        "title": "Original Title",
        "content": "Original content"
    });

    let create_response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token.token)
        .await;
    
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let __MODULE_NAME___id = create_json["data"]["id"].as_str().unwrap();

    // Update the __MODULE_NAME__
    let update_data = serde_json::json!({
        "title": "Updated Title",
        "content": "Updated content"
    });

    let response = app
        .put_json_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id), &update_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["title"], "Updated Title");
    assert_eq!(json["data"]["content"], "Updated content");
}

#[tokio::test]
async fn test_delete___MODULE_NAME__() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a __MODULE_NAME__ first
    let __MODULE_NAME___data = serde_json::json!({
        "title": "To be deleted",
        "content": "This will be deleted"
    });

    let create_response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token.token)
        .await;
    
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let __MODULE_NAME___id = create_json["data"]["id"].as_str().unwrap();

    // Delete the __MODULE_NAME__
    let response = app
        .delete_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);

    // Verify it's deleted (should return 404)
    let get_response = app
        .get_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id), &token.token)
        .await;
    assert_status(&get_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_access_control() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create two users
    let user1_name = format!("user1_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let user2_name = format!("user2_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user1, token1) = factory.create_authenticated_user(&user1_name).await;
    let (_user2, token2) = factory.create_authenticated_user(&user2_name).await;

    // User1 creates a __MODULE_NAME__
    let __MODULE_NAME___data = serde_json::json!({
        "title": "User1's __MODULE_STRUCT__",
        "content": "Private content"
    });

    let create_response = app
        .post_json_auth("/api/v1/__MODULE_NAME_PLURAL__", &__MODULE_NAME___data, &token1.token)
        .await;
    
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let __MODULE_NAME___id = create_json["data"]["id"].as_str().unwrap();

    // User2 should not be able to access User1's __MODULE_NAME__
    let response = app
        .get_auth(&format!("/api/v1/__MODULE_NAME_PLURAL__/{}", __MODULE_NAME___id), &token2.token)
        .await;
    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get___MODULE_NAME_PLURAL___stats_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("admin_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_admin, token) = factory.create_authenticated_admin(&unique_username).await;

    let response = app.get_auth("/api/v1/admin/__MODULE_NAME_PLURAL__/stats", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
}