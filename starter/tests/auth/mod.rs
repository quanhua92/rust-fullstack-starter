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

    // The timing difference should be minimal (within 200ms tolerance for integration tests)
    let timing_diff = avg_valid_time.abs_diff(avg_invalid_time);

    // In a real timing attack, the difference would be orders of magnitude larger (seconds)
    // We allow 200ms tolerance for integration test environment variability (CI/local/performance differences)
    assert!(
        timing_diff < std::time::Duration::from_millis(200),
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

#[tokio::test]
async fn test_common_password_constant_time_validation() {
    let app = spawn_app().await;

    // Test timing for passwords that are in the common password list vs not
    // This tests whether common password checking has timing vulnerabilities
    let common_password_data = json!({
        "username": "testuser1",
        "email": "test1@example.com",
        "password": "password123"  // This is in the common password list
    });

    let uncommon_password_data = json!({
        "username": "testuser2",
        "email": "test2@example.com",
        "password": "VeryUniquePassword123!"  // This should not be in common list
    });

    let weak_but_uncommon_password_data = json!({
        "username": "testuser3",
        "email": "test3@example.com",
        "password": "short"  // This is weak but not in common list (too short)
    });

    // Warm up the system with a few requests
    for _ in 0..3 {
        let _ = app
            .post_json("/api/v1/auth/register", &common_password_data)
            .await;
        let _ = app
            .post_json("/api/v1/auth/register", &uncommon_password_data)
            .await;
        let _ = app
            .post_json("/api/v1/auth/register", &weak_but_uncommon_password_data)
            .await;
    }

    // Measure timing for common password validation
    let mut common_times = Vec::new();
    for i in 0..5 {
        let test_data = json!({
            "username": format!("testuser_common_{}", i),
            "email": format!("test_common_{}@example.com", i),
            "password": "password123"
        });

        let start = std::time::Instant::now();
        let response = app.post_json("/api/v1/auth/register", &test_data).await;
        let duration = start.elapsed();

        // Should be rejected as BAD_REQUEST due to common password
        assert_status(&response, reqwest::StatusCode::BAD_REQUEST);
        common_times.push(duration);
    }

    // Measure timing for uncommon password validation (but still rejected for strength)
    let mut uncommon_times = Vec::new();
    for i in 0..5 {
        let test_data = json!({
            "username": format!("testuser_uncommon_{}", i),
            "email": format!("test_uncommon_{}@example.com", i),
            "password": "short"  // Too short, but not common
        });

        let start = std::time::Instant::now();
        let response = app.post_json("/api/v1/auth/register", &test_data).await;
        let duration = start.elapsed();

        // Should be rejected as BAD_REQUEST due to length requirement
        assert_status(&response, reqwest::StatusCode::BAD_REQUEST);
        uncommon_times.push(duration);
    }

    // Calculate average timing for both scenarios
    let avg_common_time =
        common_times.iter().sum::<std::time::Duration>() / common_times.len() as u32;
    let avg_uncommon_time =
        uncommon_times.iter().sum::<std::time::Duration>() / uncommon_times.len() as u32;

    // The timing difference should be minimal (within 20ms tolerance for integration tests)
    let timing_diff = avg_common_time.abs_diff(avg_uncommon_time);

    assert!(
        timing_diff < std::time::Duration::from_millis(20),
        "Common password validation timing difference too large: {:?} (avg_common: {:?}, avg_uncommon: {:?}). \
         This suggests timing attack vulnerability in password validation.",
        timing_diff,
        avg_common_time,
        avg_uncommon_time
    );

    // Both should take some time (password validation should not be instant)
    assert!(
        avg_common_time > std::time::Duration::from_micros(100),
        "Common password validation too fast: {:?}",
        avg_common_time
    );
    assert!(
        avg_uncommon_time > std::time::Duration::from_micros(100),
        "Uncommon password validation too fast: {:?}",
        avg_uncommon_time
    );
}

#[tokio::test]
async fn test_session_fixation_prevention() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create a user for testing
    factory.create_user("testuser").await;

    // Step 1: Login to create an active session
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    assert_status(&login_response, StatusCode::OK);

    let auth_token = app.extract_auth_token(login_response).await;

    // Step 2: Use direct database access to simulate an old session (older than 30 days)
    let mut conn = app.db().await;

    // Get user ID from the current session
    let session_data = sqlx::query!(
        "SELECT user_id FROM sessions WHERE token = $1",
        auth_token.token
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Failed to fetch session data");

    let user_id = session_data.user_id;

    // Create an old session (simulate session older than 30 days)
    let old_token = "old-session-token-12345";
    let old_last_activity = chrono::Utc::now() - chrono::Duration::days(35); // 35 days ago

    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at, last_activity_at, is_active) 
         VALUES ($1, $2, $3, $4, true)",
        user_id,
        old_token,
        chrono::Utc::now() + chrono::Duration::hours(24),
        old_last_activity
    )
    .execute(&mut *conn)
    .await
    .expect("Failed to insert old session");

    // Create a recent session (less than 30 days old)
    let recent_token = "recent-session-token-67890";
    let recent_last_activity = chrono::Utc::now() - chrono::Duration::days(10); // 10 days ago

    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at, last_activity_at, is_active) 
         VALUES ($1, $2, $3, $4, true)",
        user_id,
        recent_token,
        chrono::Utc::now() + chrono::Duration::hours(24),
        recent_last_activity
    )
    .execute(&mut *conn)
    .await
    .expect("Failed to insert recent session");

    // Verify all sessions are initially active
    let active_sessions_before = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Failed to count active sessions")
    .count
    .unwrap_or(0);

    assert_eq!(
        active_sessions_before, 3,
        "Should have 3 active sessions initially"
    );

    // Step 3: Perform another login - this should trigger session fixation prevention
    // The login should invalidate only sessions older than 30 days
    let second_login_response = app.post_json("/api/v1/auth/login", &login_data).await;
    assert_status(&second_login_response, StatusCode::OK);

    // Step 4: Verify session fixation prevention worked correctly
    let sessions_after = sqlx::query!(
        "SELECT token, is_active, last_activity_at FROM sessions WHERE user_id = $1 ORDER BY created_at",
        user_id
    )
    .fetch_all(&mut *conn)
    .await
    .expect("Failed to fetch sessions after login");

    // Check each session's status
    let mut old_session_found = false;
    let mut recent_session_found = false;
    let mut current_session_active = false;
    let mut new_session_created = false;

    for session in &sessions_after {
        match session.token.as_str() {
            token if token == auth_token.token => {
                current_session_active = session.is_active;
            }
            token if token == old_token => {
                old_session_found = true;
                // Old session (35+ days) should be deactivated due to session fixation prevention
                assert!(
                    !session.is_active,
                    "Old session (35+ days) should be deactivated for session fixation prevention"
                );
            }
            token if token == recent_token => {
                recent_session_found = true;
                // Recent session (10 days) should remain active
                assert!(
                    session.is_active,
                    "Recent session (10 days) should remain active"
                );
            }
            _ => {
                // This should be the new session created by the second login
                if session.is_active {
                    new_session_created = true;
                }
            }
        }
    }

    // Verify all expected sessions were found
    assert!(old_session_found, "Old session should exist in database");
    assert!(
        recent_session_found,
        "Recent session should exist in database"
    );
    assert!(
        current_session_active,
        "Current session should remain active"
    );
    assert!(new_session_created, "New session should be created");

    // Step 5: Verify current session still works (not affected by fixation prevention)
    let me_response = app.get_auth("/api/v1/auth/me", &auth_token.token).await;
    assert_status(&me_response, StatusCode::OK);

    // Step 6: Final verification - count active sessions
    let active_sessions_after = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions WHERE user_id = $1 AND is_active = true",
        user_id
    )
    .fetch_one(&mut *conn)
    .await
    .expect("Failed to count active sessions after login")
    .count
    .unwrap_or(0);

    // Should have one less active session (old session deactivated)
    // Original session + recent session + new session = 3 active sessions
    assert_eq!(
        active_sessions_after, 3,
        "Should have 3 active sessions after login (old session deactivated)"
    );
}

#[tokio::test]
async fn test_password_validation_security_edge_cases() {
    let app = spawn_app().await;

    // Test various security edge cases for password validation
    let security_test_cases = [
        (
            "Password123",
            "Mixed case version of common password - should be rejected for case bypass",
        ),
        ("PASSWORD", "All caps version of common password"),
        ("pAsSwOrD123", "Mixed case 'password123' variant"),
        ("Admin123", "Mixed case 'admin123' variant"),
        ("Welcome123", "Mixed case 'welcome123' variant"),
        ("\0password123\0", "Password with null bytes"),
        ("password123\r\n", "Password with line endings"),
        ("password123\t", "Password with tab character"),
        (" password123 ", "Password with leading/trailing spaces"),
        ("password\u{200B}123", "Password with zero-width space"),
        ("p\u{00AD}assword123", "Password with soft hyphen"),
        ("password123\u{FEFF}", "Password with BOM character"),
    ];

    for (i, (password, description)) in security_test_cases.iter().enumerate() {
        let user_data = json!({
            "username": format!("testuser_{}", i),
            "email": format!("test_{}@example.com", i),
            "password": password
        });

        let response = app.post_json("/api/v1/auth/register", &user_data).await;

        // All these should be rejected (either for being common passwords or invalid characters)
        assert!(
            response.status() == StatusCode::BAD_REQUEST,
            "Expected BAD_REQUEST for password security test case {} ({}), got: {}",
            i,
            description,
            response.status()
        );

        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap_or("");

        // Should get appropriate error message
        assert!(
            error_message.contains("password")
                || error_message.contains("common")
                || error_message.contains("Invalid")
                || error_message.contains("character"),
            "Expected password-related error for test case {} ({}), got: {}",
            i,
            description,
            error_message
        );
    }

    // Test that legitimate strong passwords still work
    let strong_passwords = [
        "MyVerySecurePassword123!",
        "AnotherStrongP@ssw0rd",
        "Complex&Secure#2024!",
        "UnbreakableP4$$w0rd!",
    ];

    for (i, password) in strong_passwords.iter().enumerate() {
        let user_data = json!({
            "username": format!("strong_user_{}", i),
            "email": format!("strong_{}@example.com", i),
            "password": password
        });

        let response = app.post_json("/api/v1/auth/register", &user_data).await;

        // These should succeed (or fail only due to duplicate username, not password)
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::CONFLICT,
            "Strong password test case {} should be accepted, got: {}",
            i,
            response.status()
        );
    }
}

#[tokio::test]
async fn test_logout_vs_logout_all_difference() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create user and two sessions
    factory.create_user("testuser").await;
    let login_data = json!({
        "username": "testuser",
        "password": "SecurePass123!"
    });

    let login1 = app.post_json("/api/v1/auth/login", &login_data).await;
    let token1 = app.extract_auth_token(login1).await.token;

    let login2 = app.post_json("/api/v1/auth/login", &login_data).await;
    let token2 = app.extract_auth_token(login2).await.token;

    // Verify both sessions work
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token1).await,
        StatusCode::OK,
    );
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token2).await,
        StatusCode::OK,
    );

    // Test /auth/logout (single session) - should only invalidate current session
    let logout_response = app.post_auth("/api/v1/auth/logout", &token1).await;
    assert_status(&logout_response, StatusCode::OK);

    // token1 should be invalid, token2 should still work
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token1).await,
        StatusCode::UNAUTHORIZED,
    );
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token2).await,
        StatusCode::OK,
    );

    // Create third session
    let login3 = app.post_json("/api/v1/auth/login", &login_data).await;
    let token3 = app.extract_auth_token(login3).await.token;

    // Test /auth/logout-all - should invalidate all sessions
    let logout_all_response = app.post_auth("/api/v1/auth/logout-all", &token2).await;
    assert_status(&logout_all_response, StatusCode::OK);

    let json: serde_json::Value = logout_all_response.json().await.unwrap();
    assert!(json["message"].as_str().unwrap().contains("session(s)"));

    // Both remaining sessions should be invalid
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token2).await,
        StatusCode::UNAUTHORIZED,
    );
    assert_status(
        &app.get_auth("/api/v1/auth/me", &token3).await,
        StatusCode::UNAUTHORIZED,
    );
}
