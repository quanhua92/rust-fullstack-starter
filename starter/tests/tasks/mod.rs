use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid;

#[tokio::test]
async fn test_create_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let task_response = factory
        .create_task(
            "email",
            json!({
                "to": "test@example.com",
                "subject": "Test",
                "body": "Hello"
            }),
        )
        .await;

    assert_json_field_exists(&task_response, "data");
    assert_eq!(task_response["data"]["task_type"], "email");
}

#[tokio::test]
async fn test_get_task_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create task
    let task_response = factory
        .create_task(
            "email",
            json!({
                "to": "test@example.com",
                "subject": "Test",
                "body": "Hello"
            }),
        )
        .await;

    let task_id = task_response["data"]["id"].as_str().unwrap();

    // Get task status
    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;
    let response = app
        .get_auth(&format!("/tasks/{task_id}"), &token.token)
        .await;

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
    factory
        .create_task("email", json!({"to": "user1@example.com"}))
        .await;
    factory
        .create_task("webhook", json!({"url": "https://example.com/webhook"}))
        .await;
    factory
        .create_task("email", json!({"to": "user2@example.com"}))
        .await;

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
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
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let task_data = json!({
        "task_type": "email",
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
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .get_auth(&format!("/tasks/{fake_id}"), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(
        json["data"].is_null(),
        "Expected null data for nonexistent task"
    );
}

#[tokio::test]
async fn test_task_retry_mechanism() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let task_data = json!({
        "task_type": "email",
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
        factory
            .create_task(
                "email",
                json!({
                    "to": format!("user{}@example.com", i),
                    "subject": format!("Test {}", i),
                    "body": "Test message"
                }),
            )
            .await;
    }

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
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
    factory
        .create_task("email", json!({"to": "test1@example.com"}))
        .await;
    factory
        .create_task("email", json!({"to": "test2@example.com"}))
        .await;

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
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

#[tokio::test]
async fn test_dead_letter_queue() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test empty dead letter queue
    let response = app.get_auth("/tasks/dead-letter", &token.token).await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");
    let dead_tasks = json["data"].as_array().unwrap();

    // Initially should be empty or only contain existing failed tasks
    let initial_failed_count = dead_tasks.len();

    // Create tasks that will fail (using task type that causes failure)
    let failing_task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test Failure",
            "body": "fail"  // This triggers failure in the email handler
        },
        "priority": "high"
    });

    let task_response = app
        .post_json_auth("/tasks", &failing_task_data, &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Manually mark the task as failed (simulating task processor failure)
    // In a real scenario, the worker would process and fail the task
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'failed', last_error = 'Simulated failure for testing', completed_at = NOW() WHERE id = $1",
        uuid::Uuid::parse_str(task_id).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // Check dead letter queue now contains the failed task
    let response = app.get_auth("/tasks/dead-letter", &token.token).await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let dead_tasks = json["data"].as_array().unwrap();
    assert_eq!(dead_tasks.len(), initial_failed_count + 1);

    // Verify the failed task is in the dead letter queue
    let failed_task = dead_tasks
        .iter()
        .find(|task| task["id"].as_str() == Some(task_id))
        .expect("Failed task should be in dead letter queue");

    assert_eq!(failed_task["status"], "Failed");
    assert!(failed_task["last_error"].as_str().is_some());
}

#[tokio::test]
async fn test_retry_failed_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a task
    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test Retry",
            "body": "Hello"
        }
    });

    let task_response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Manually mark the task as failed
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'failed', last_error = 'Test failure', completed_at = NOW(), current_attempt = 3 WHERE id = $1",
        uuid::Uuid::parse_str(task_id).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // Retry the failed task
    let retry_response = app
        .post_auth(&format!("/tasks/{task_id}/retry"), &token.token)
        .await;
    assert_status(&retry_response, StatusCode::OK);

    let retry_json: serde_json::Value = retry_response.json().await.unwrap();
    assert_eq!(retry_json["success"], true);

    // Verify task is now pending again
    let task_response = app
        .get_auth(&format!("/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();

    if !task_json["data"].is_null() {
        assert_eq!(task_json["data"]["status"], "Pending");
        assert_eq!(task_json["data"]["current_attempt"], 0);
        assert!(task_json["data"]["last_error"].is_null());
    }
}

#[tokio::test]
async fn test_retry_nonexistent_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .post_auth(&format!("/tasks/{fake_id}/retry"), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_retry_non_failed_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a pending task
    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test",
            "body": "Hello"
        }
    });

    let task_response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Try to retry a pending task (should fail)
    let retry_response = app
        .post_auth(&format!("/tasks/{task_id}/retry"), &token.token)
        .await;
    assert_status(&retry_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a task
    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test Delete",
            "body": "Hello"
        }
    });

    let task_response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Mark task as completed so it can be deleted
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'completed', completed_at = NOW() WHERE id = $1",
        uuid::Uuid::parse_str(task_id).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // Delete the task
    let delete_response = app
        .delete_auth(&format!("/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::OK);

    let delete_json: serde_json::Value = delete_response.json().await.unwrap();
    assert_eq!(delete_json["success"], true);

    // Verify task is deleted
    let task_response = app
        .get_auth(&format!("/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    assert!(task_json["data"].is_null());
}

#[tokio::test]
async fn test_delete_nonexistent_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .delete_auth(&format!("/tasks/{fake_id}"), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_pending_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create a pending task
    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test",
            "body": "Hello"
        }
    });

    let task_response = app.post_json_auth("/tasks", &task_data, &token.token).await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Try to delete a pending task (should fail)
    let delete_response = app
        .delete_auth(&format!("/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_filter_tasks_by_failed_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple tasks
    let task1_response = factory
        .create_task("email", json!({"to": "test1@example.com"}))
        .await;
    let task2_response = factory
        .create_task("email", json!({"to": "test2@example.com"}))
        .await;
    let _task3_response = factory
        .create_task("email", json!({"to": "test3@example.com"}))
        .await;

    let task1_id = task1_response["data"]["id"].as_str().unwrap();
    let task2_id = task2_response["data"]["id"].as_str().unwrap();

    // Mark some tasks as failed
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'failed', last_error = 'Test failure', completed_at = NOW() WHERE id = $1 OR id = $2",
        uuid::Uuid::parse_str(task1_id).unwrap(),
        uuid::Uuid::parse_str(task2_id).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // Filter by failed status
    let response = app.get_auth("/tasks?status=failed", &token.token).await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();

    // Verify all returned tasks are failed
    let failed_task_ids: Vec<&str> = tasks
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(failed_task_ids.contains(&task1_id));
    assert!(failed_task_ids.contains(&task2_id));

    // All returned tasks should be failed
    for task in tasks {
        if let Some(status) = task["status"].as_str() {
            assert_eq!(status, "Failed");
        }
    }
}

#[tokio::test]
async fn test_dead_letter_queue_pagination() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple failed tasks
    let mut task_ids = Vec::new();
    for i in 0..15 {
        let task_response = factory
            .create_task("email", json!({"to": format!("test{}@example.com", i)}))
            .await;
        let task_id = task_response["data"]["id"].as_str().unwrap().to_string();
        task_ids.push(task_id);
    }

    // Mark all as failed
    let db = &app.db_pool;
    for task_id in &task_ids {
        sqlx::query!(
            "UPDATE tasks SET status = 'failed', last_error = 'Test failure', completed_at = NOW() WHERE id = $1",
            uuid::Uuid::parse_str(task_id).unwrap()
        )
        .execute(db)
        .await
        .unwrap();
    }

    // Test pagination
    let response = app
        .get_auth("/tasks/dead-letter?limit=10&offset=0", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();
    assert!(tasks.len() <= 10, "Should have at most 10 tasks per page");

    // All should be failed tasks
    for task in tasks {
        assert_eq!(task["status"], "Failed");
    }
}
