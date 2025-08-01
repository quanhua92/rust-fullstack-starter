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
