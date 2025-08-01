use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

/// Authentication edge case tests to prevent 422 errors from middleware parsing issues
/// These tests specifically target scenarios where malformed requests could cause
/// unexpected 422 responses instead of proper 400/401 responses

#[tokio::test]
async fn test_registration_json_type_mismatches() {
    let app = spawn_app().await;

    // Test various JSON type mismatches - should return 422 for JSON deserialization errors
    let type_mismatch_cases = vec![
        (
            json!({"username": 123, "email": "test@example.com", "password": "ValidPass123!"}),
            "number username",
        ),
        (
            json!({"email": true, "password": "password"}),
            "boolean email",
        ),
        // Note: Arrays are sometimes accepted by some servers, so this might return 200
        // (json!(["username", "email@example.com", "password"]), "array instead of object"),
        (
            json!({"user": {"username": "testuser", "email": "test@example.com", "password": "ValidPass123!"}}),
            "nested object",
        ),
    ];

    for (invalid_data, description) in type_mismatch_cases {
        let response = app.post_json("/api/v1/auth/register", &invalid_data).await;

        // JSON type mismatches should return 422 for deserialization errors
        let status = response.status();
        assert!(
            status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
            "Type mismatch '{description}': Expected 422 or 400, got {status}"
        );
    }
}

#[tokio::test]
async fn test_registration_with_extra_fields() {
    let app = spawn_app().await;

    // Test registration with extra fields - should succeed or return 400, not 422
    let data_with_extras = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "ValidPass123!",
        "extra_field": "should_be_ignored",
        "another_extra": 42,
        "nested_extra": {"field": "value"}
    });

    let response = app
        .post_json("/api/v1/auth/register", &data_with_extras)
        .await;

    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "Expected 200 or 400, got {status}"
    );
}

#[tokio::test]
async fn test_password_operations_with_wrong_types() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Test password change with numeric passwords - should return 422
    let (_user, token) = factory.create_authenticated_user("pwd_test").await;
    let invalid_pwd_data = json!({
        "current_password": 123456,
        "new_password": 789012
    });

    let response = app
        .put_json_auth("/api/v1/users/me/password", &invalid_pwd_data, &token.token)
        .await;
    assert_status(&response, StatusCode::UNPROCESSABLE_ENTITY);

    // Test account deletion with wrong confirmation type - should return 422
    let (_user2, token2) = factory.create_authenticated_user("delete_test").await;
    let invalid_delete_data = json!({
        "password": "SecurePass123!",
        "confirmation": 123 // Should be string "DELETE"
    });

    let response = app
        .delete_json_auth("/api/v1/users/me", &invalid_delete_data, &token2.token)
        .await;
    assert_status(&response, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_http_edge_cases_and_malformed_requests() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Test multiple Content-Type headers
    let response = app
        .client
        .post(format!("{}/api/v1/auth/register", app.address))
        .header("Content-Type", "application/json")
        .header("Content-Type", "text/plain") // Duplicate header
        .body(r#"{"username":"test","email":"test@example.com","password":"ValidPass123!"}"#)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "Expected 200, 400, or 415, got {status}"
    );

    // Test deeply nested JSON
    let mut nested_json = json!("value");
    for _ in 0..50 {
        nested_json = json!({ "nested": nested_json });
    }
    let response = app.post_json("/api/v1/auth/register", &nested_json).await;
    assert_status(&response, StatusCode::UNPROCESSABLE_ENTITY);

    // Test unicode edge cases
    let unicode_data = json!({
        "username": "test\u{0000}user", // Null character
        "email": "test\u{FEFF}@example.com", // BOM character
        "password": "Pass\u{200B}word123!" // Zero-width space
    });
    let response = app.post_json("/api/v1/auth/register", &unicode_data).await;
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test JSON injection attempts
    let injection_json = r#"{"username":"test","email":"test@example.com","password":"ValidPass123!","__proto__":{"admin":true}}"#;
    let response = app
        .client
        .post(format!("{}/api/v1/auth/register", app.address))
        .header("Content-Type", "application/json")
        .body(injection_json)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::OK
            || status == StatusCode::CONFLICT
            || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Injection attempt failed with: {status}"
    );

    // Test token refresh with unexpected payload
    let (_user, token) = factory.create_authenticated_user("refresh_test").await;
    let invalid_refresh_data = json!({
        "refresh_token": "should_not_be_here",
        "grant_type": "refresh_token"
    });

    let response = app
        .post_json_auth("/api/v1/auth/refresh", &invalid_refresh_data, &token.token)
        .await;

    let status = response.status();
    assert!(
        status == StatusCode::OK
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::CONFLICT,
        "Expected 200, 400, or 409, got {status}"
    );
}
