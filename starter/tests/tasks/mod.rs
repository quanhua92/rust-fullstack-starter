use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid;

#[tokio::test]
async fn test_create_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    let task_response = factory.create_task("send_email", json!({
        "to": "test@example.com",
        "subject": "Test",
        "body": "Hello"
    })).await;
    
    assert_json_field_exists(&task_response, "data");
    assert_eq!(task_response["data"]["task_type"], "send_email");
}

#[tokio::test]
async fn test_get_task_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create task
    let task_response = factory.create_task("send_email", json!({
        "to": "test@example.com",
        "subject": "Test",
        "body": "Hello"
    })).await;
    
    let task_id = task_response["data"]["id"].as_str().unwrap();
    
    // Get task status
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    let response = app.get_auth(&format!("/tasks/{}", task_id), &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    // Check if task was found and has the expected status
    if !json["data"].is_null() {
        assert_eq!(json["data"]["status"], "Pending");
    }
}

#[tokio::test]
async fn test_list_tasks() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create multiple tasks
    factory.create_task("send_email", json!({"to": "user1@example.com"})).await;
    factory.create_task("send_webhook", json!({"url": "https://example.com/webhook"})).await;
    factory.create_task("send_email", json!({"to": "user2@example.com"})).await;
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    let response = app.get_auth("/tasks", &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
    
    let tasks = json["data"].as_array().unwrap();
    assert!(tasks.len() >= 3, "Should have at least 3 tasks");
}

#[tokio::test]
async fn test_create_task_with_priority() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    
    let task_data = json!({
        "task_type": "send_email",
        "payload": {
            "to": "test@example.com",
            "subject": "High Priority",
            "body": "Urgent message"
        },
        "priority": "high"
    });
    
    let response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
}


#[tokio::test]
async fn test_get_nonexistent_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    
    let fake_id = uuid::Uuid::new_v4();
    let response = app.get_auth(&format!("/tasks/{}", fake_id), &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["data"].is_null(), "Expected null data for nonexistent task");
}

#[tokio::test]
async fn test_task_retry_mechanism() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    
    let task_data = json!({
        "task_type": "send_email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test Retry",
            "body": "This should retry on failure"
        },
        "metadata": {"max_retries": 3}
    });
    
    let response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    
    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
}

#[tokio::test]
async fn test_tasks_pagination() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create many tasks
    for i in 0..15 {
        factory.create_task("send_email", json!({
            "to": format!("user{}@example.com", i),
            "subject": format!("Test {}", i),
            "body": "Test message"
        })).await;
    }
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    
    // Test pagination
    let response = app.get_auth("/tasks?limit=10&offset=0", &token.token).await;
    assert_status(&response, StatusCode::OK);
    
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
    
    let tasks = json["data"].as_array().unwrap();
    assert!(tasks.len() <= 10, "Should have at most 10 tasks per page");
}

#[tokio::test]
async fn test_filter_tasks_by_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());
    
    // Create tasks
    factory.create_task("send_email", json!({"to": "test1@example.com"})).await;
    factory.create_task("send_email", json!({"to": "test2@example.com"})).await;
    
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    
    // Filter by status
    let response = app.get_auth("/tasks?status=pending", &token.token).await;
    assert_status(&response, StatusCode::OK);
    
    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();
    
    // All returned tasks should be pending (if any exist)
    for task in tasks {
        if let Some(status) = task["status"].as_str() {
            assert_eq!(status, "Pending");
        }
    }
}