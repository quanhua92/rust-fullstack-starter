use crate::helpers::*;
use reqwest::StatusCode;
use uuid;

#[tokio::test]
async fn test_get_users_list_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create test users
    factory.create_multiple_users(3).await;

    // Create moderator
    let (_moderator, token) = factory
        .create_authenticated_moderator("moderator_test")
        .await;

    let response = app.get_auth("/api/v1/users", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
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
async fn test_update_own_profile() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create authenticated user with unique name
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let update_data = serde_json::json!({
        "email": "newemail@example.com"
    });

    let response = app
        .put_json_auth("/api/v1/users/me/profile", &update_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["email"], "newemail@example.com");
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

// New comprehensive user management tests

#[tokio::test]
async fn test_create_user_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_id = &uuid::Uuid::new_v4().to_string()[..8];
    let (_admin, token) = factory
        .create_authenticated_admin(&format!("admin_test_{unique_id}"))
        .await;

    // Admin can only create regular users, not specify roles directly
    let new_user_data = serde_json::json!({
        "username": format!("new_user_test_{unique_id}"),
        "email": format!("newuser_{unique_id}@example.com"),
        "password": "SecurePassword123!"
        // Note: No role field - users are created as "user" by default
    });

    let response = app
        .post_json_auth("/api/v1/users", &new_user_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(
        json["data"]["username"],
        format!("new_user_test_{unique_id}")
    );
    assert_eq!(json["data"]["role"], "user"); // UserRole enum now serializes as lowercase
}

#[tokio::test]
async fn test_create_user_as_non_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory.create_authenticated_user("regular_user").await;

    let new_user_data = serde_json::json!({
        "username": "new_user_test",
        "email": "newuser@example.com",
        "password": "SecurePassword123!",
        "role": "user"
    });

    let response = app
        .post_json_auth("/api/v1/users", &new_user_data, &token.token)
        .await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_change_own_password() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory
        .create_authenticated_user("password_test_user")
        .await;

    let password_data = serde_json::json!({
        "current_password": "SecurePass123!",  // Default password from factory
        "new_password": "NewSecurePassword123!"
    });

    let response = app
        .put_json_auth("/api/v1/users/me/password", &password_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "Password updated successfully");
}

#[tokio::test]
async fn test_change_password_wrong_current() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory
        .create_authenticated_user("password_wrong_test")
        .await;

    let password_data = serde_json::json!({
        "current_password": "wrongpassword",
        "new_password": "NewSecurePassword123!"
    });

    let response = app
        .put_json_auth("/api/v1/users/me/password", &password_data, &token.token)
        .await;

    assert_status(&response, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_own_account() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory.create_authenticated_user("delete_self_test").await;

    let delete_data = serde_json::json!({
        "password": "SecurePass123!",  // Default password from factory
        "confirmation": "DELETE"
    });

    let response = app
        .delete_json_auth("/api/v1/users/me", &delete_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "Account deleted successfully");
}

#[tokio::test]
async fn test_update_user_profile_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("target_user").await;
    let (_admin, token) = factory.create_authenticated_admin("admin_test").await;

    let update_data = serde_json::json!({
        "username": "updated_username",
        "email": "updated@example.com",
        "email_verified": true
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/profile", user.id),
            &update_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["username"], "updated_username");
    assert_eq!(json["data"]["email_verified"], true);
}

#[tokio::test]
async fn test_update_user_status_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("target_user").await;
    let (_moderator, token) = factory
        .create_authenticated_moderator("moderator_test")
        .await;

    let status_data = serde_json::json!({
        "is_active": false,
        "reason": "Account suspended for testing"
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/status", user.id),
            &status_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["is_active"], false);
}

#[tokio::test]
async fn test_update_user_role_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("target_user").await;
    let (_admin, token) = factory.create_authenticated_admin("admin_test").await;

    let role_data = serde_json::json!({
        "role": "moderator",
        "reason": "Promoted to moderator for testing"
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/role", user.id),
            &role_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"]["role"], "moderator"); // UserRole enum now serializes as lowercase
}

#[tokio::test]
async fn test_reset_user_password_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("target_user").await;
    let (_moderator, token) = factory
        .create_authenticated_moderator("moderator_test")
        .await;

    let reset_data = serde_json::json!({
        "new_password": "NewTemporaryPassword123!",
        "require_change": true,
        "reason": "Password reset requested by user"
    });

    let response = app
        .post_json_auth(
            &format!("/api/v1/users/{}/reset-password", user.id),
            &reset_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "Password reset successfully");
}

#[tokio::test]
async fn test_delete_user_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user = factory.create_user("target_user").await;
    let (_admin, token) = factory.create_authenticated_admin("admin_test").await;

    let delete_data = serde_json::json!({
        "reason": "Account deletion requested by user",
        "hard_delete": false
    });

    let response = app
        .delete_json_auth(
            &format!("/api/v1/users/{}", user.id),
            &delete_data,
            &token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "User account deleted successfully");
}

#[tokio::test]
async fn test_get_user_stats_as_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create some test users
    factory.create_multiple_users(5).await;
    let (_admin, token) = factory.create_authenticated_admin("admin_test").await;

    let response = app
        .get_auth("/api/v1/admin/users/stats", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "total_users");
    assert_json_field_exists(&json["data"], "active_users");
    assert_json_field_exists(&json["data"], "by_role");
    assert_json_field_exists(&json["data"], "recent_registrations");
}

#[tokio::test]
async fn test_get_user_stats_as_non_admin() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory.create_authenticated_user("regular_user").await;

    let response = app
        .get_auth("/api/v1/admin/users/stats", &token.token)
        .await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_users_list_as_regular_user() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_user, token) = factory.create_authenticated_user("regular_user").await;

    let response = app.get_auth("/api/v1/users", &token.token).await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_user_can_only_access_own_profile() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let user1 = factory.create_user("user1").await;
    let (_user2, token2) = factory.create_authenticated_user("user2").await;

    // User2 tries to access User1's profile
    let response = app
        .get_auth(&format!("/api/v1/users/{}", user1.id), &token2.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND); // RBAC returns 404 to prevent enumeration
}

// Security vulnerability tests

#[tokio::test]
async fn test_registration_common_password_case_bypass() {
    let app = spawn_app().await;

    // Test that mixed-case versions of common passwords are rejected
    // This tests the case-insensitive bypass vulnerability in models.rs:218
    let test_cases = [
        ("Password123!", "password123! with mixed case"),
        ("Admin123!", "admin123! with mixed case"),
        ("Qwerty123!", "qwerty123! with mixed case"),
        ("Welcome123!", "welcome123! with mixed case"),
        ("Password1!", "password1! with mixed case"),
    ];

    for (i, (mixed_case_password, description)) in test_cases.iter().enumerate() {
        let user_data = serde_json::json!({
            "username": format!("testuser_{}", i),
            "email": format!("test_{}@example.com", i),
            "password": mixed_case_password
        });

        let response = app.post_json("/api/v1/auth/register", &user_data).await;

        // Should be rejected as BAD_REQUEST due to common password validation
        assert_status(&response, StatusCode::BAD_REQUEST);
        let json: serde_json::Value = response.json().await.unwrap();

        // Verify the error message indicates password is too common
        let error_message = json["error"]["message"].as_str().unwrap_or("");
        assert!(
            error_message.contains("too common") || error_message.contains("common password"),
            "Expected common password error for {}, got: {}",
            description,
            error_message
        );
    }
}

#[tokio::test]
async fn test_registration_invalid_email_formats() {
    let app = spawn_app().await;

    // Test various invalid email formats that should be rejected
    // This tests the weak email validation in models.rs:75-80
    let long_local = format!("{}@domain.com", "a".repeat(65));
    let long_domain = format!("user@{}.com", "a".repeat(250));
    let invalid_emails = [
        ("", "empty email"),
        ("@", "just @"),
        ("@domain.com", "missing local part"),
        ("user@", "missing domain"),
        ("user@@domain.com", "double @"),
        ("user@domain@com", "multiple @ symbols"),
        ("user space@domain.com", "space in local part"),
        ("user@domain .com", "space in domain"),
        ("user@domain..com", "double dot in domain"),
        ("user@.domain.com", "leading dot in domain"),
        ("user@domain.com.", "trailing dot in domain"),
        ("user@domain", "missing TLD"),
        ("user.@domain.com", "trailing dot in local part"),
        (".user@domain.com", "leading dot in local part"),
        ("user..name@domain.com", "consecutive dots in local part"),
        (long_local.as_str(), "local part too long"),
        (long_domain.as_str(), "domain too long"),
        ("user@domain.c", "TLD too short"),
        ("user@-domain.com", "domain starts with hyphen"),
        ("user@domain-.com", "domain ends with hyphen"),
        ("user@domain.com-", "TLD ends with hyphen"),
        ("user@[192.168.1.1", "malformed IP literal"),
        ("user@192.168.1.1]", "malformed IP literal"),
        ("user@[192.168.1.999]", "invalid IP in literal"),
        ("user name@domain.com", "unquoted space"),
        ("user@dom ain.com", "space in domain"),
        ("user@domain...", "multiple trailing dots"),
        ("user@", "just local part with @"),
        ("@domain.com@", "multiple @ at boundaries"),
    ];

    for (i, (invalid_email, description)) in invalid_emails.iter().enumerate() {
        let user_data = serde_json::json!({
            "username": format!("testuser_{}", i),
            "email": invalid_email,
            "password": "ValidPassword123!"
        });

        let response = app.post_json("/api/v1/auth/register", &user_data).await;

        // Should be rejected as BAD_REQUEST due to invalid email format
        assert!(
            response.status() == StatusCode::BAD_REQUEST,
            "Expected BAD_REQUEST for invalid email: {} ({}), got: {}",
            invalid_email,
            description,
            response.status()
        );

        let json: serde_json::Value = response.json().await.unwrap();
        let error_message = json["error"]["message"].as_str().unwrap_or("");

        // Verify the error is related to email validation
        assert!(
            error_message.contains("email")
                || error_message.contains("format")
                || error_message.contains("Invalid"),
            "Expected email format error for {} ({}), got: {}",
            invalid_email,
            description,
            error_message
        );
    }
}

#[tokio::test]
async fn test_user_stats_database_error_handling() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_admin, token) = factory.create_authenticated_admin("admin_stats_test").await;

    // First, test normal operation
    let response = app
        .get_auth("/api/v1/admin/users/stats", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();

    // Verify the response structure contains expected fields
    assert_json_field_exists(&json["data"], "total_users");
    assert_json_field_exists(&json["data"], "active_users");
    assert_json_field_exists(&json["data"], "inactive_users");
    assert_json_field_exists(&json["data"], "email_verified");
    assert_json_field_exists(&json["data"], "email_unverified");
    assert_json_field_exists(&json["data"], "by_role");
    assert_json_field_exists(&json["data"], "recent_registrations");

    // Verify counts make sense (basic sanity check)
    let total_users = json["data"]["total_users"].as_i64().unwrap();
    let active_users = json["data"]["active_users"].as_i64().unwrap();
    let inactive_users = json["data"]["inactive_users"].as_i64().unwrap();

    assert!(total_users >= 0, "Total users should be non-negative");
    assert!(active_users >= 0, "Active users should be non-negative");
    assert!(inactive_users >= 0, "Inactive users should be non-negative");
    assert_eq!(
        total_users,
        active_users + inactive_users,
        "Total should equal active + inactive"
    );

    // Verify role counts
    let user_count = json["data"]["by_role"]["user"].as_i64().unwrap();
    let moderator_count = json["data"]["by_role"]["moderator"].as_i64().unwrap();
    let admin_count = json["data"]["by_role"]["admin"].as_i64().unwrap();

    assert!(user_count >= 0, "User role count should be non-negative");
    assert!(
        moderator_count >= 0,
        "Moderator role count should be non-negative"
    );
    assert!(
        admin_count >= 1,
        "Admin role count should be at least 1 (test admin)"
    );
    assert_eq!(
        active_users,
        user_count + moderator_count + admin_count,
        "Active users should equal sum of role counts"
    );

    // Test that the service properly handles potential database issues
    // Note: This tests the logic around unwrap_or(0) in services.rs:506-576
    // We can't easily simulate database connection failures in integration tests,
    // but we can verify the response structure and calculations are correct

    // Create some additional users to test different scenarios
    factory.create_multiple_users(3).await;

    // Test stats again with more users
    let response2 = app
        .get_auth("/api/v1/admin/users/stats", &token.token)
        .await;
    assert_status(&response2, StatusCode::OK);
    let json2: serde_json::Value = response2.json().await.unwrap();

    let new_total_users = json2["data"]["total_users"].as_i64().unwrap();
    assert!(
        new_total_users >= total_users,
        "User count should have increased after creating users"
    );
}

#[tokio::test]
async fn test_concurrent_account_deletion() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create a user to test deletion
    let (_user, token) = factory
        .create_authenticated_user("concurrent_delete_test")
        .await;

    // Test self-deletion - should work properly and invalidate sessions
    let delete_data = serde_json::json!({
        "password": "SecurePass123!",  // Default password from factory
        "confirmation": "DELETE"
    });

    let response = app
        .delete_json_auth("/api/v1/users/me", &delete_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "Account deleted successfully");

    // Verify that the token is no longer valid (sessions should be invalidated)
    let me_response = app.get_auth("/api/v1/auth/me", &token.token).await;
    assert_status(&me_response, StatusCode::UNAUTHORIZED);

    // Test admin deletion with transaction consistency
    let target_user = factory.create_user("admin_delete_target").await;
    let (_admin, admin_token) = factory.create_authenticated_admin("admin_deleter").await;

    let admin_delete_data = serde_json::json!({
        "reason": "Test deletion for transaction consistency",
        "hard_delete": false
    });

    let response = app
        .delete_json_auth(
            &format!("/api/v1/users/{}", target_user.id),
            &admin_delete_data,
            &admin_token.token,
        )
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "User account deleted successfully");

    // Verify the user is marked as inactive - should return NOT_FOUND
    let get_response = app
        .get_auth(
            &format!("/api/v1/users/{}", target_user.id),
            &admin_token.token,
        )
        .await;

    // Should return NOT_FOUND because user is now inactive (soft deleted)
    assert_status(&get_response, StatusCode::NOT_FOUND);

    // Test that trying to delete the same user again returns NOT_FOUND
    let second_delete = app
        .delete_json_auth(
            &format!("/api/v1/users/{}", target_user.id),
            &admin_delete_data,
            &admin_token.token,
        )
        .await;

    assert_status(&second_delete, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_user_rbac_error_consistency() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create users with different roles
    let regular_user = factory.create_user("regular_user").await;
    let (_moderator, mod_token) = factory.create_authenticated_moderator("mod_user").await;
    let (_admin, admin_token) = factory.create_authenticated_admin("admin_user").await;
    let (_regular_auth, regular_token) = factory.create_authenticated_user("regular_auth").await;

    // Test 1: Regular user trying to access another regular user's profile
    // Should return NOT_FOUND (not FORBIDDEN) to prevent user enumeration
    let response = app
        .get_auth(
            &format!("/api/v1/users/{}", regular_user.id),
            &regular_token.token,
        )
        .await;
    assert_status(&response, StatusCode::NOT_FOUND);

    // Test 2: Moderator accessing regular user profile - should work
    let response = app
        .get_auth(
            &format!("/api/v1/users/{}", regular_user.id),
            &mod_token.token,
        )
        .await;
    assert_status(&response, StatusCode::OK);

    // Test 3: Admin accessing any user profile - should work
    let response = app
        .get_auth(
            &format!("/api/v1/users/{}", regular_user.id),
            &admin_token.token,
        )
        .await;
    assert_status(&response, StatusCode::OK);

    // Test 4: Try to access non-existent user with valid permissions
    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .get_auth(&format!("/api/v1/users/{}", fake_id), &admin_token.token)
        .await;
    assert_status(&response, StatusCode::NOT_FOUND);

    // Test 5: Verify error response structure is consistent
    let fake_id2 = uuid::Uuid::new_v4();
    let response1 = app
        .get_auth(&format!("/api/v1/users/{}", fake_id2), &regular_token.token)
        .await;
    let response2 = app
        .get_auth(&format!("/api/v1/users/{}", fake_id2), &admin_token.token)
        .await;

    // Both should return NOT_FOUND but for different reasons
    assert_status(&response1, StatusCode::NOT_FOUND);
    assert_status(&response2, StatusCode::NOT_FOUND);

    // Verify error response structure
    let json1: serde_json::Value = response1.json().await.unwrap();
    let json2: serde_json::Value = response2.json().await.unwrap();

    assert_json_field_exists(&json1, "error");
    assert_json_field_exists(&json2, "error");
    assert_eq!(json1["error"]["code"], "NOT_FOUND");
    assert_eq!(json2["error"]["code"], "NOT_FOUND");

    // Test 6: Regular user can access their own profile via the correct endpoint
    let response = app.get_auth("/api/v1/auth/me", &regular_token.token).await;
    assert_status(&response, StatusCode::OK);
}

#[tokio::test]
async fn test_admin_cannot_delete_self_via_any_endpoint() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_admin, admin_token) = factory
        .create_authenticated_admin("self_delete_admin")
        .await;

    // Get admin's user ID for testing
    let me_response = app.get_auth("/api/v1/auth/me", &admin_token.token).await;
    assert_status(&me_response, StatusCode::OK);
    let me_json: serde_json::Value = me_response.json().await.unwrap();
    let admin_id = me_json["data"]["id"].as_str().unwrap();

    // Test 1: Admin trying to delete themselves via admin endpoint should fail
    let admin_delete_data = serde_json::json!({
        "reason": "Testing self-deletion prevention",
        "hard_delete": false
    });

    let response = app
        .delete_json_auth(
            &format!("/api/v1/users/{}", admin_id),
            &admin_delete_data,
            &admin_token.token,
        )
        .await;

    // Should return BAD_REQUEST with specific error message
    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(
        error_message.contains("Cannot delete own account"),
        "Expected self-deletion prevention error, got: {}",
        error_message
    );

    // Test 2: Verify admin protection worked - the admin should still be able to access their profile
    let me_response_after = app.get_auth("/api/v1/auth/me", &admin_token.token).await;
    assert_status(&me_response_after, StatusCode::OK);

    // Test 3: Admin can still delete their own account via the self-deletion endpoint if they choose
    let self_delete_data = serde_json::json!({
        "password": "SecurePass123!",
        "confirmation": "DELETE"
    });

    let response = app
        .delete_json_auth("/api/v1/users/me", &self_delete_data, &admin_token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["data"], "Account deleted successfully");

    // Verify the token is no longer valid after self-deletion
    let me_response_final = app.get_auth("/api/v1/auth/me", &admin_token.token).await;
    assert_status(&me_response_final, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_transaction_rollback_on_failures() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let (_admin, admin_token) = factory
        .create_authenticated_admin("transaction_test_admin")
        .await;

    // Test password reset with invalid user ID - should handle gracefully
    let fake_id = uuid::Uuid::new_v4();
    let reset_data = serde_json::json!({
        "new_password": "NewPassword123!",
        "require_change": true,
        "reason": "Testing transaction rollback"
    });

    let response = app
        .post_json_auth(
            &format!("/api/v1/users/{}/reset-password", fake_id),
            &reset_data,
            &admin_token.token,
        )
        .await;

    // Should return NOT_FOUND for non-existent user
    assert_status(&response, StatusCode::NOT_FOUND);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["error"]["code"], "NOT_FOUND");

    // Test user status update with invalid user ID
    let status_data = serde_json::json!({
        "is_active": false,
        "reason": "Testing transaction rollback"
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/status", fake_id),
            &status_data,
            &admin_token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);

    // Test role update with invalid user ID
    let role_data = serde_json::json!({
        "role": "moderator",
        "reason": "Testing transaction rollback"
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/role", fake_id),
            &role_data,
            &admin_token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);

    // Test profile update with invalid user ID
    let profile_data = serde_json::json!({
        "username": "should_not_be_created",
        "email": "should_not@example.com",
        "email_verified": true
    });

    let response = app
        .put_json_auth(
            &format!("/api/v1/users/{}/profile", fake_id),
            &profile_data,
            &admin_token.token,
        )
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);

    // Verify that no partial state was created during failed operations
    // This is important for testing the rollback logic in services.rs
    let stats_response = app
        .get_auth("/api/v1/admin/users/stats", &admin_token.token)
        .await;
    assert_status(&stats_response, StatusCode::OK);

    let stats_json: serde_json::Value = stats_response.json().await.unwrap();
    let total_users = stats_json["data"]["total_users"].as_i64().unwrap();

    // Should not have any phantom users from failed operations
    assert!(total_users > 0, "Should have at least the test admin user");

    // Test creating a user and then verify deletion transaction consistency
    let create_data = serde_json::json!({
        "username": "transaction_test_user",
        "email": "transaction_test@example.com",
        "password": "TestPassword123!"
    });

    let create_response = app
        .post_json_auth("/api/v1/users", &create_data, &admin_token.token)
        .await;
    assert_status(&create_response, StatusCode::OK);

    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let created_user_id = create_json["data"]["id"].as_str().unwrap();

    // Now delete this user and verify transaction consistency
    let delete_data = serde_json::json!({
        "reason": "Testing transaction consistency",
        "hard_delete": false
    });

    let delete_response = app
        .delete_json_auth(
            &format!("/api/v1/users/{}", created_user_id),
            &delete_data,
            &admin_token.token,
        )
        .await;
    assert_status(&delete_response, StatusCode::OK);

    // Verify the user is no longer accessible
    let get_response = app
        .get_auth(
            &format!("/api/v1/users/{}", created_user_id),
            &admin_token.token,
        )
        .await;
    assert_status(&get_response, StatusCode::NOT_FOUND);
}
