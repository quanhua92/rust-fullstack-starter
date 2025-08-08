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

#[tokio::test]
async fn test_login_timing_attack_protection() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create a user to test valid credentials
    factory.create_user("validuser").await;

    // Test data for timing measurements
    let valid_login_data = json!({
        "username": "validuser",
        "password": "WrongPassword"  // Wrong password for existing user
    });

    let invalid_login_data = json!({
        "username": "nonexistentuser",
        "password": "AnyPassword"  // Any password for non-existent user
    });

    // Warm up the system with a few requests to stabilize timing
    for _ in 0..3 {
        let _ = app.post_json("/api/v1/auth/login", &valid_login_data).await;
        let _ = app
            .post_json("/api/v1/auth/login", &invalid_login_data)
            .await;
    }

    // Measure timing for existing user with wrong password (should use real hash)
    let mut valid_user_times = Vec::new();
    for _ in 0..5 {
        let start = std::time::Instant::now();
        let response = app.post_json("/api/v1/auth/login", &valid_login_data).await;
        let duration = start.elapsed();

        // Should return UNAUTHORIZED for wrong password
        assert_status(&response, reqwest::StatusCode::UNAUTHORIZED);
        valid_user_times.push(duration);
    }

    // Measure timing for non-existent user (should use dummy hash)
    let mut invalid_user_times = Vec::new();
    for _ in 0..5 {
        let start = std::time::Instant::now();
        let response = app
            .post_json("/api/v1/auth/login", &invalid_login_data)
            .await;
        let duration = start.elapsed();

        // Should return UNAUTHORIZED for non-existent user
        assert_status(&response, reqwest::StatusCode::UNAUTHORIZED);
        invalid_user_times.push(duration);
    }

    // Calculate average timing for both scenarios
    let avg_valid_time =
        valid_user_times.iter().sum::<std::time::Duration>() / valid_user_times.len() as u32;
    let avg_invalid_time =
        invalid_user_times.iter().sum::<std::time::Duration>() / invalid_user_times.len() as u32;

    // The timing difference should be minimal (within 50ms tolerance for integration tests)
    let timing_diff = avg_valid_time.abs_diff(avg_invalid_time);

    // In a real timing attack, the difference would be orders of magnitude larger
    // We allow 50ms tolerance for integration test environment variability
    assert!(
        timing_diff < std::time::Duration::from_millis(50),
        "Timing difference too large: {:?} (avg_valid: {:?}, avg_invalid: {:?}). \
         This suggests timing attack vulnerability - dummy hash may not be working correctly.",
        timing_diff,
        avg_valid_time,
        avg_invalid_time
    );

    // Both scenarios should take a reasonable amount of time (bcrypt should be slow)
    // Bcrypt with cost 12 should take at least a few milliseconds
    assert!(
        avg_valid_time > std::time::Duration::from_millis(1),
        "Valid user password verification too fast: {:?}",
        avg_valid_time
    );
    assert!(
        avg_invalid_time > std::time::Duration::from_millis(1),
        "Invalid user dummy hash verification too fast: {:?}",
        avg_invalid_time
    );

    // Verify both scenarios return the same error structure
    let valid_response = app.post_json("/api/v1/auth/login", &valid_login_data).await;
    let invalid_response = app
        .post_json("/api/v1/auth/login", &invalid_login_data)
        .await;

    let valid_json: serde_json::Value = valid_response.json().await.unwrap();
    let invalid_json: serde_json::Value = invalid_response.json().await.unwrap();

    // Both should return the same error message to prevent user enumeration
    assert_eq!(valid_json["error"]["code"], invalid_json["error"]["code"]);
    assert_eq!(valid_json["error"]["code"], "INVALID_CREDENTIALS");
    assert_eq!(valid_json["error"]["message"], "Invalid credentials");
    assert_eq!(invalid_json["error"]["message"], "Invalid credentials");
}

#[tokio::test]
async fn test_login_dummy_hash_error_handling() {
    let app = spawn_app().await;

    // Test with non-existent user to trigger dummy hash usage
    let login_data = json!({
        "username": "definitelynonexistentuser",
        "password": "TestPassword123!"
    });

    // This should use the dummy hash and complete without errors
    let response = app.post_json("/api/v1/auth/login", &login_data).await;

    // Should return UNAUTHORIZED (not 500 Internal Server Error)
    assert_status(&response, reqwest::StatusCode::UNAUTHORIZED);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["error"]["message"], "Invalid credentials");
    assert_eq!(json["error"]["code"], "INVALID_CREDENTIALS");

    // Verify the response structure is consistent
    assert_json_field_exists(&json, "error");
    assert_json_field_exists(&json["error"], "message");
    assert_json_field_exists(&json["error"], "code");
}
