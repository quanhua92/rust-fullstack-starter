use crate::helpers::*;
use reqwest::StatusCode;
use uuid;

#[tokio::test]
#[ignore] // Route not implemented yet
async fn test_get_users_list() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create test users
    factory.create_multiple_users(3).await;

    let response = app.get("/api/v1/users/").await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "users");
}

#[tokio::test]
async fn test_get_user_by_id() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user with unique name
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let user = factory.create_user(&unique_username).await;

    // Need admin auth to access another user's profile
    let (_auth_user, token) = factory
        .create_authenticated_admin(&format!("admin_{}", &unique_username))
        .await;
    let response = app
        .get_auth(&format!("/api/v1/users/{}", user.id), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["username"], unique_username);
}

#[tokio::test]
async fn test_get_user_profile_authenticated() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user with unique name
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app.get_auth("/api/v1/auth/me", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["id"], user.id.to_string());
    assert_eq!(json["data"]["username"], unique_username);
}

#[tokio::test]
async fn test_get_user_profile_unauthenticated() {
    let app = spawn_app().await;

    let response = app.get("/api/v1/auth/me").await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore] // Route not implemented yet
async fn test_update_user_profile() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user with unique name
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let update_data = serde_json::json!({
        "email": "newemail@example.com"
    });

    let response = app
        .put_json_auth("/api/v1/auth/me", &update_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field(
        &json["user"]["email"],
        "",
        &serde_json::Value::String("newemail@example.com".to_string()),
    );
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth to access user endpoint
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .get_auth(&format!("/api/v1/users/{fake_id}"), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore] // Route not implemented yet
async fn test_users_pagination() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create many users
    factory.create_multiple_users(25).await;

    // Test first page
    let response = app.get("/api/v1/users/?page=1&limit=10").await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "users");
    assert_json_field_exists(&json, "pagination");

    // Verify pagination metadata
    let pagination = &json["pagination"];
    assert_json_field(pagination, "page", &serde_json::Value::Number(1.into()));
    assert_json_field(pagination, "limit", &serde_json::Value::Number(10.into()));
    assert_json_field_exists(pagination, "total");
    assert_json_field_exists(pagination, "total_pages");
}
