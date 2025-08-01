use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;

/// Test middleware edge cases that could cause 422 errors
/// These tests ensure robust parsing and error handling in middleware layers

#[tokio::test]
async fn test_authorization_header_edge_cases() {
    let app = spawn_app().await;

    // Test various malformed authorization headers - all should return 401, not 422
    let malformed_headers = vec![
        ("NotBearer invalid-token", "completely malformed header"),
        ("", "empty authorization header"),
        ("Bearer ", "Bearer without token"),
        ("Bearer not-a-valid-uuid-format", "invalid UUID format"),
        (
            "01234567-89ab-cdef-0123-456789abcdef",
            "properly formatted but expired token",
        ),
        ("Bearer", "just 'Bearer' without space or token"),
        ("bearer valid-token", "lowercase 'bearer'"),
        ("BEARER valid-token", "uppercase 'BEARER'"),
        ("Basic valid-token", "wrong auth type"),
    ];

    for (header_value, description) in malformed_headers {
        let response = app
            .client
            .get(format!("{}/api/v1/auth/me", app.address))
            .header("Authorization", header_value)
            .send()
            .await
            .expect("Failed to execute request");

        // All should return 401 Unauthorized, not 422
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Header '{}' ({}): Expected 401, got {}",
            header_value,
            description,
            response.status()
        );
    }
}

#[tokio::test]
async fn test_json_parsing_edge_cases() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;
    let (_user, token) = factory.create_authenticated_user("json_test_user").await;

    // Test malformed JSON with valid auth - should return 400, not 422
    let response = app
        .client
        .put(format!("{}/api/v1/users/me/profile", app.address))
        .header("Authorization", format!("Bearer {}", token.token))
        .header("Content-Type", "application/json")
        .body("{ invalid json without closing brace")
        .send()
        .await
        .expect("Failed to execute request");

    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test JSON body with wrong content type
    let response = app
        .client
        .put(format!("{}/api/v1/users/me/profile", app.address))
        .header("Authorization", format!("Bearer {}", token.token))
        .header("Content-Type", "text/plain")
        .body(r#"{"email": "test@example.com"}"#)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    assert!(
        status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
        "Expected 415 or 400, got {status}"
    );

    // Test extremely large JSON payload
    let large_reason = "x".repeat(10000);
    let large_data = json!({
        "email": "test@example.com",
        "reason": large_reason
    });

    let response = app
        .put_json_auth("/api/v1/users/me/profile", &large_data, &token.token)
        .await;

    let status = response.status();
    assert!(
        status == StatusCode::PAYLOAD_TOO_LARGE
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::OK, // If server accepts large payloads
        "Expected 413, 400, or 200, got {status}"
    );
}

#[tokio::test]
async fn test_role_validation_comprehensive() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("role_test_user").await;
    let (_admin, token) = factory.create_authenticated_admin("role_test_admin").await;

    // Test various invalid role values and null/missing fields
    let invalid_scenarios = vec![
        (json!({"role": "", "reason": "empty string"}), "empty role"),
        (
            json!({"role": "ADMIN", "reason": "uppercase"}),
            "uppercase role",
        ),
        (
            json!({"role": "invalid_role", "reason": "invalid"}),
            "invalid role",
        ),
        (
            json!({"role": "admin ", "reason": "trailing space"}),
            "trailing space",
        ),
        (
            json!({"role": " moderator", "reason": "leading space"}),
            "leading space",
        ),
        (json!({"role": null, "reason": "null role"}), "null role"),
        (json!({"reason": "missing role field"}), "missing role"),
    ];

    for (role_data, description) in invalid_scenarios {
        let response = app
            .put_json_auth(
                &format!("/api/v1/users/{}/role", user.id),
                &role_data,
                &token.token,
            )
            .await;

        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Role validation '{description}': Expected 400 or 422, got {status}"
        );
    }
}

#[tokio::test]
async fn test_unicode_and_concurrent_operations() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Test unicode characters
    let (_user, token) = factory.create_authenticated_user("unicode_test").await;

    let unicode_data = json!({"email": "test🚀@example.com"});
    let response = app
        .put_json_auth("/api/v1/users/me/profile", &unicode_data, &token.token)
        .await;

    let status = response.status();
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::OK,
        "Expected 400 or 200, got {status}"
    );

    // Test extremely long strings
    let long_email = "a".repeat(300) + "@example.com";
    let long_data = json!({"email": long_email});
    let response = app
        .put_json_auth("/api/v1/users/me/profile", &long_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test concurrent role updates
    let user = factory.create_user("concurrent_test_user").await;
    let (_admin1, token1) = factory
        .create_authenticated_admin("concurrent_admin1")
        .await;
    let (_admin2, token2) = factory
        .create_authenticated_admin("concurrent_admin2")
        .await;

    let role_data1 = json!({"role": "moderator", "reason": "First concurrent update"});
    let role_data2 = json!({"role": "admin", "reason": "Second concurrent update"});

    let role_endpoint = format!("/api/v1/users/{}/role", user.id);
    let (response1, response2) = tokio::join!(
        app.put_json_auth(&role_endpoint, &role_data1, &token1.token,),
        app.put_json_auth(&role_endpoint, &role_data2, &token2.token,)
    );

    // Both should succeed or return appropriate errors, but not 422
    for (i, response) in [response1, response2].iter().enumerate() {
        let status = response.status();
        assert!(
            status == StatusCode::OK
                || status == StatusCode::CONFLICT
                || status == StatusCode::BAD_REQUEST,
            "Concurrent request {}: Expected 200, 409, or 400, got {status}",
            i + 1
        );
    }
}
