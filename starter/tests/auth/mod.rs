use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_user_registration_success() {
    let app = spawn_app().await;

    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "SecurePass123!"
    });

    let response = app.post_json("/api/v1/auth/register", &user_data).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "success");
    assert_json_field_exists(&json, "data");
}

#[tokio::test]
async fn test_login_success() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user
    factory.create_user("testuser").await;

    // Login
    let login_data = json!({
        "username_or_email": "testuser",
        "password": "SecurePass123!"
    });

    let response = app.post_json("/api/v1/auth/login", &login_data).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "session_token");
    assert_json_field_exists(&json["data"], "user");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user
    factory.create_user("testuser").await;

    // Login with wrong password
    let login_data = json!({
        "username_or_email": "testuser",
        "password": "WrongPassword"
    });

    let response = app.post_json("/api/v1/auth/login", &login_data).await;
    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_registration_duplicate_username() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create first user
    factory.create_user("testuser").await;

    // Try to create another user with same username
    let user_data = json!({
        "username": "testuser",
        "email": "different@example.com",
        "password": "SecurePass123!"
    });

    let response = app.post_json("/api/v1/auth/register", &user_data).await;
    assert_status(&response, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_registration_invalid_email() {
    let app = spawn_app().await;

    let user_data = json!({
        "username": "testuser",
        "email": "invalid-email",
        "password": "SecurePass123!"
    });

    let response = app.post_json("/api/v1/auth/register", &user_data).await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_registration_weak_password() {
    let app = spawn_app().await;

    let user_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "weak"
    });

    let response = app.post_json("/api/v1/auth/register", &user_data).await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}
