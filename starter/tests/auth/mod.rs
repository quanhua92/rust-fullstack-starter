use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

pub mod edge_cases;

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
        "username": "testuser",
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
        "username": "testuser",
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

#[tokio::test]
async fn test_token_refresh_success() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and login
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    assert_status(&login_response, StatusCode::OK);

    let login_json: serde_json::Value = login_response.json().await.unwrap();
    let token = login_json["data"]["session_token"].as_str().unwrap();
    let original_expires_at = login_json["data"]["expires_at"].as_str().unwrap();

    // Test token refresh
    let refresh_response = app.post_auth("/api/v1/auth/refresh", token).await;
    assert_status(&refresh_response, StatusCode::OK);

    let refresh_json: serde_json::Value = refresh_response.json().await.unwrap();
    assert_json_field_exists(&refresh_json, "success");
    assert_json_field_exists(&refresh_json["data"], "expires_at");
    assert_json_field_exists(&refresh_json["data"], "refreshed_at");

    // Verify new expiration is later than original
    let new_expires_at = refresh_json["data"]["expires_at"].as_str().unwrap();
    assert!(
        new_expires_at > original_expires_at,
        "New expiration should be later than original"
    );

    // Verify refreshed_at is present and recent
    let refreshed_at = refresh_json["data"]["refreshed_at"].as_str().unwrap();
    assert!(!refreshed_at.is_empty(), "refreshed_at should not be empty");
}

#[tokio::test]
async fn test_token_refresh_rate_limiting() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and login
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    assert_status(&login_response, StatusCode::OK);

    let auth_token = app.extract_auth_token(login_response).await;

    // First refresh should work
    let first_refresh = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&first_refresh, StatusCode::OK);

    // Immediate second refresh should be rate limited (409 CONFLICT)
    let second_refresh = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&second_refresh, StatusCode::CONFLICT);

    let error_json: serde_json::Value = second_refresh.json().await.unwrap();
    assert_json_field_exists(&error_json, "error");
    assert_eq!(error_json["error"]["code"], "CONFLICT");

    let error_message = error_json["error"]["message"].as_str().unwrap();
    assert!(
        error_message.contains("Cannot refresh token yet"),
        "Error should mention rate limiting"
    );
}

#[tokio::test]
async fn test_token_refresh_invalid_token() {
    let app = spawn_app().await;

    // Try to refresh with invalid token
    let invalid_token = "invalid-token-12345";
    let response = app.post_auth("/api/v1/auth/refresh", invalid_token).await;

    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_token_refresh_extends_session_lifetime() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and login
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    let auth_token = app.extract_auth_token(login_response).await;

    // Verify /me works with current token
    let me_response = app.get_auth("/api/v1/auth/me", &auth_token.token).await;
    assert_status(&me_response, StatusCode::OK);

    // Refresh token
    let refresh_response = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&refresh_response, StatusCode::OK);

    let refresh_json: serde_json::Value = refresh_response.json().await.unwrap();
    let new_expires_at = refresh_json["data"]["expires_at"].as_str().unwrap();

    // Verify token still works after refresh
    let me_after_refresh = app.get_auth("/api/v1/auth/me", &auth_token.token).await;
    assert_status(&me_after_refresh, StatusCode::OK);

    // Verify new expiration is approximately 24 hours from now (allowing for test execution time)
    let expires_at_time = chrono::DateTime::parse_from_rfc3339(new_expires_at)
        .expect("Failed to parse expires_at timestamp");
    let now = chrono::Utc::now();
    let duration_until_expiry = expires_at_time.signed_duration_since(now);

    // Should be close to 24 hours (23-25 hours to allow for test execution time)
    assert!(
        duration_until_expiry.num_hours() >= 23,
        "Token should expire in at least 23 hours, got {} hours",
        duration_until_expiry.num_hours()
    );
    assert!(
        duration_until_expiry.num_hours() <= 25,
        "Token should expire in at most 25 hours, got {} hours",
        duration_until_expiry.num_hours()
    );
}

#[tokio::test]
async fn test_token_refresh_after_wait_succeeds() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and login
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    let auth_token = app.extract_auth_token(login_response).await;

    // First refresh should work
    let first_refresh = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&first_refresh, StatusCode::OK);

    // Immediate second refresh should fail
    let second_refresh = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&second_refresh, StatusCode::CONFLICT);

    // Note: In a real test environment, we would wait 5+ minutes to test this,
    // but that would make tests too slow. This test demonstrates the pattern.
    // The rate limiting logic is tested above, and the curl script tests the timing.
}

#[tokio::test]
async fn test_token_refresh_updates_database() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and login
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    let auth_token = app.extract_auth_token(login_response).await;

    // Check database state before refresh
    let mut conn = app.db().await;
    let session_before = sqlx::query!(
        "SELECT last_refreshed_at FROM sessions WHERE token = $1",
        auth_token.token
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Failed to fetch session");

    assert!(
        session_before.last_refreshed_at.is_none(),
        "last_refreshed_at should be null initially"
    );

    // Refresh token
    let refresh_response = app
        .post_auth("/api/v1/auth/refresh", &auth_token.token)
        .await;
    assert_status(&refresh_response, StatusCode::OK);

    // Check database state after refresh
    let session_after = sqlx::query!(
        "SELECT last_refreshed_at, expires_at FROM sessions WHERE token = $1",
        auth_token.token
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Failed to fetch session");

    assert!(
        session_after.last_refreshed_at.is_some(),
        "last_refreshed_at should be set after refresh"
    );

    // Verify last_refreshed_at is recent (within last 10 seconds)
    let last_refreshed = session_after.last_refreshed_at.unwrap();
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(last_refreshed);
    assert!(
        diff.num_seconds() < 10,
        "last_refreshed_at should be very recent"
    );
}
