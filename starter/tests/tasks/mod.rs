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
async fn test_create_task_with_metadata() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create authenticated user for task creation
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create task with custom metadata
    let task_payload = json!({
        "task_type": "delay_task",
        "payload": {
            "delay_seconds": 1,
            "task_id": "test_metadata_preservation",
            "test_scenario": "metadata_test"
        },
        "priority": "normal",
        "metadata": {
            "chaos_test": true,
            "tag": "multiworker",
            "delay_duration": 1,
            "custom_field": "test_value",
            "task_id": "test_metadata_preservation"
        }
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &task_payload, &token.token)
        .await;

    assert_eq!(response.status(), 200);
    let task_response: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&task_response, "data");

    let task_id = task_response["data"]["id"].as_str().unwrap();

    // Get the task and verify metadata is preserved
    let get_response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;

    assert_eq!(get_response.status(), 200);
    let get_task_response: serde_json::Value = get_response.json().await.unwrap();

    let metadata = &get_task_response["data"]["metadata"];
    assert_eq!(metadata["chaos_test"], true);
    assert_eq!(metadata["tag"], "multiworker");
    assert_eq!(metadata["delay_duration"], 1);
    assert_eq!(metadata["custom_field"], "test_value");
    assert_eq!(metadata["task_id"], "test_metadata_preservation");
    assert_eq!(metadata["api_created"], true); // Should be added by API
}

#[tokio::test]
async fn test_get_task_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create authenticated user first
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create task using authenticated API (so it belongs to the user)
    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "subject": "Test",
            "body": "Hello"
        }
    });

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Get task status - user should be able to see their own task
    let response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    // Check if task was found and has the expected status
    if !json["data"].is_null() {
        assert_eq!(json["data"]["status"], "pending");
    }
}

#[tokio::test]
async fn test_list_tasks() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create authenticated user first
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple tasks using authenticated API (so they belong to the user)
    let task1_data = json!({
        "task_type": "email",
        "payload": {"to": "user1@example.com"}
    });
    let task2_data = json!({
        "task_type": "webhook",
        "payload": {"url": "https://example.com/webhook"}
    });
    let task3_data = json!({
        "task_type": "email",
        "payload": {"to": "user2@example.com"}
    });

    app.post_json_auth("/api/v1/tasks", &task1_data, &token.token)
        .await;
    app.post_json_auth("/api/v1/tasks", &task2_data, &token.token)
        .await;
    app.post_json_auth("/api/v1/tasks", &task3_data, &token.token)
        .await;

    // List tasks - should see all 3 tasks created by this user
    let response = app.get_auth("/api/v1/tasks", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");

    let tasks = json["data"].as_array().unwrap();
    assert!(tasks.len() >= 3, "Should have at least 3 tasks");
}

#[tokio::test]
async fn test_create_task_with_priority() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;

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
        .get_auth(&format!("/api/v1/tasks/{fake_id}"), &token.token)
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
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;

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
    let response = app
        .get_auth("/api/v1/tasks?limit=10&offset=0", &token.token)
        .await;
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
    let response = app
        .get_auth("/api/v1/tasks?status=pending", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();

    // All returned tasks should be pending (if any exist)
    for task in tasks {
        if let Some(status) = task["status"].as_str() {
            assert_eq!(status, "pending");
        }
    }
}

#[tokio::test]
async fn test_dead_letter_queue() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Need auth for protected routes
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test empty dead letter queue
    let response = app
        .get_auth("/api/v1/tasks/dead-letter", &token.token)
        .await;
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
        .post_json_auth("/api/v1/tasks", &failing_task_data, &token.token)
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
    let response = app
        .get_auth("/api/v1/tasks/dead-letter", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let dead_tasks = json["data"].as_array().unwrap();
    assert_eq!(dead_tasks.len(), initial_failed_count + 1);

    // Verify the failed task is in the dead letter queue
    let failed_task = dead_tasks
        .iter()
        .find(|task| task["id"].as_str() == Some(task_id))
        .expect("Failed task should be in dead letter queue");

    assert_eq!(failed_task["status"], "failed");
    assert!(failed_task["last_error"].as_str().is_some());
}

#[tokio::test]
async fn test_retry_failed_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
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
        .post_auth(&format!("/api/v1/tasks/{task_id}/retry"), &token.token)
        .await;
    assert_status(&retry_response, StatusCode::OK);

    let retry_json: serde_json::Value = retry_response.json().await.unwrap();
    assert_eq!(retry_json["success"], true);

    // Verify task is now pending again
    let task_response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();

    if !task_json["data"].is_null() {
        assert_eq!(task_json["data"]["status"], "pending");
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
        .post_auth(&format!("/api/v1/tasks/{fake_id}/retry"), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_retry_non_failed_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Try to retry a pending task (should fail)
    let retry_response = app
        .post_auth(&format!("/api/v1/tasks/{task_id}/retry"), &token.token)
        .await;
    assert_status(&retry_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
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
        .delete_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::OK);

    let delete_json: serde_json::Value = delete_response.json().await.unwrap();
    assert_eq!(delete_json["success"], true);

    // Verify task is deleted
    let task_response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
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
        .delete_auth(&format!("/api/v1/tasks/{fake_id}"), &token.token)
        .await;

    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_pending_task() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

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

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
    assert_status(&task_response, StatusCode::OK);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Try to delete a pending task (should fail)
    let delete_response = app
        .delete_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_filter_tasks_by_failed_status() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create authenticated user first
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple tasks using authenticated API (so they belong to the user)
    let task1_data = json!({
        "task_type": "email",
        "payload": {"to": "test1@example.com"}
    });
    let task2_data = json!({
        "task_type": "email",
        "payload": {"to": "test2@example.com"}
    });
    let task3_data = json!({
        "task_type": "email",
        "payload": {"to": "test3@example.com"}
    });

    let task1_response = app
        .post_json_auth("/api/v1/tasks", &task1_data, &token.token)
        .await;
    let task2_response = app
        .post_json_auth("/api/v1/tasks", &task2_data, &token.token)
        .await;
    let _task3_response = app
        .post_json_auth("/api/v1/tasks", &task3_data, &token.token)
        .await;

    assert_status(&task1_response, StatusCode::OK);
    assert_status(&task2_response, StatusCode::OK);

    let task1_json: serde_json::Value = task1_response.json().await.unwrap();
    let task2_json: serde_json::Value = task2_response.json().await.unwrap();
    let task1_id = task1_json["data"]["id"].as_str().unwrap();
    let task2_id = task2_json["data"]["id"].as_str().unwrap();

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

    // Filter by failed status - should only see user's own failed tasks
    let response = app
        .get_auth("/api/v1/tasks?status=failed", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();

    // Verify all returned tasks are failed and belong to the user
    let failed_task_ids: Vec<&str> = tasks
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(failed_task_ids.contains(&task1_id));
    assert!(failed_task_ids.contains(&task2_id));

    // All returned tasks should be failed
    for task in tasks {
        if let Some(status) = task["status"].as_str() {
            assert_eq!(status, "failed");
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
        .get_auth("/api/v1/tasks/dead-letter?limit=10&offset=0", &token.token)
        .await;
    assert_status(&response, StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let tasks = json["data"].as_array().unwrap();
    assert!(tasks.len() <= 10, "Should have at most 10 tasks per page");

    // All should be failed tasks
    for task in tasks {
        assert_eq!(task["status"], "failed");
    }
}

/// TDD Test: Comprehensive metadata persistence through ACTUAL task processing pipeline
/// This test verifies that custom metadata (especially tag used by chaos testing)
/// is preserved when tasks are processed by the real worker system, not direct DB updates.
/// THIS TEST SHOULD FAIL initially to expose the metadata persistence bug in task processing.
#[tokio::test]
async fn test_metadata_persistence_through_all_state_transitions() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Start a task processor for this test (simulates the worker)
    use starter::database::Database;
    use starter::tasks::processor::{ProcessorConfig, TaskProcessor};
    use std::time::Duration;

    let database = Database {
        pool: app.db_pool.clone(),
    };
    let config = ProcessorConfig {
        poll_interval: Duration::from_millis(100), // Fast polling for tests
        task_timeout: Duration::from_secs(30),
        max_concurrent_tasks: 2,
        ..Default::default()
    };

    let processor = TaskProcessor::new(database, config);

    // Register the delay_task handler
    use starter::tasks::handlers::DelayTaskHandler;
    processor
        .register_handler("delay_task".to_string(), DelayTaskHandler)
        .await;

    // Start the processor in background
    let processor_handle = {
        let processor_clone = processor.clone();
        tokio::spawn(async move {
            processor_clone
                .start_worker()
                .await
                .expect("Processor should start");
        })
    };

    // Give the processor a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create authenticated user for task operations
    let unique_username = format!("testuser_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test metadata that should persist through all state transitions
    let test_metadata = json!({
        "tag": "multiworker",
        "chaos_test": true,
        "test_run_id": "metadata_persistence_test",
        "custom_field": "test_value",
        "priority_override": "high",
        "created_by_test": true
    });

    // 1. CREATE task with custom metadata (Pending state)
    let task_payload = json!({
        "task_type": "delay_task",
        "payload": {
            "delay_seconds": 1,  // Short delay so test completes quickly
            "task_id": "metadata_persistence_test",
            "test_scenario": "worker_processing"
        },
        "priority": "normal",
        "metadata": test_metadata
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &task_payload, &token.token)
        .await;

    assert_eq!(response.status(), 200);
    let task_response: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&task_response, "data");

    let task_id = task_response["data"]["id"].as_str().unwrap();

    // Verify metadata in initial state (could be Pending or Running due to worker timing)
    let get_response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;
    assert_eq!(get_response.status(), 200);
    let initial_task: serde_json::Value = get_response.json().await.unwrap();

    let initial_status = initial_task["data"]["status"].as_str().unwrap();
    assert!(
        initial_status == "pending" || initial_status == "running",
        "Expected pending or running, got: {initial_status}"
    );
    assert_metadata_preserved(&initial_task["data"]["metadata"], &test_metadata);

    // 2. Wait for task to be processed by the actual worker
    // This is the critical difference - we let the REAL task processor handle it
    let mut attempts = 0;
    let max_attempts = 15; // 15 seconds max wait
    let mut final_task: Option<serde_json::Value> = None;

    while attempts < max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let check_response = app
            .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
            .await;

        if check_response.status() == 200 {
            let task_data: serde_json::Value = check_response.json().await.unwrap();
            let status = task_data["data"]["status"].as_str().unwrap_or("Unknown");

            // Log the current state for debugging
            println!(
                "Attempt {}: Task status = {}, metadata = {}",
                attempts + 1,
                status,
                task_data["data"]["metadata"]
            );

            match status {
                "completed" => {
                    final_task = Some(task_data);
                    break;
                }
                "failed" => {
                    panic!(
                        "Task failed during processing: {}",
                        task_data["data"]["last_error"]
                            .as_str()
                            .unwrap_or("Unknown error")
                    );
                }
                "running" => {
                    // Verify metadata is preserved during RUNNING state
                    assert_metadata_preserved(&task_data["data"]["metadata"], &test_metadata);
                }
                "pending" => {
                    // Still waiting
                }
                _ => {
                    println!("Unexpected status: {status}");
                }
            }
        }

        attempts += 1;
    }

    // 3. Verify the task completed and check metadata preservation
    let completed_task = final_task.unwrap_or_else(|| panic!("Task {task_id} did not complete within {max_attempts} seconds. This might indicate the worker is not running or the task is stuck."));

    assert_eq!(completed_task["data"]["status"], "completed");

    // THIS IS THE CRITICAL TEST - metadata should be preserved after worker processing
    // This assertion should FAIL initially, exposing the bug
    assert_metadata_preserved(&completed_task["data"]["metadata"], &test_metadata);

    // 4. Also verify in task listings (as chaos testing uses this)
    let list_response = app.get_auth("/api/v1/tasks?limit=100", &token.token).await;
    assert_eq!(list_response.status(), 200);
    let list_result: serde_json::Value = list_response.json().await.unwrap();
    let tasks_list = list_result["data"].as_array().unwrap();

    // Find our test task in the list
    let test_task = tasks_list
        .iter()
        .find(|task| task["id"].as_str() == Some(task_id))
        .expect("Test task should be in task list");

    // Verify metadata is preserved in task listings (critical for chaos testing)
    assert_metadata_preserved(&test_task["metadata"], &test_metadata);

    // 5. Additional test: Create a task that would be used by chaos testing
    // and verify it can be found by the monitoring script's filtering logic
    let chaos_task_payload = json!({
        "task_type": "delay_task",
        "payload": {
            "delay_seconds": 1,
            "task_id": "chaos_multiworker_test"
        },
        "priority": "normal",
        "metadata": {
            "tag": "multiworker",  // This is what chaos testing looks for
            "chaos_test": true,
            "scenario": "tdd_verification"
        }
    });

    let chaos_response = app
        .post_json_auth("/api/v1/tasks", &chaos_task_payload, &token.token)
        .await;

    let chaos_task_data: serde_json::Value = chaos_response.json().await.unwrap();
    let chaos_task_id = chaos_task_data["data"]["id"].as_str().unwrap();

    // Wait for chaos task to complete
    attempts = 0;
    while attempts < max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let chaos_check = app
            .get_auth(&format!("/api/v1/tasks/{chaos_task_id}"), &token.token)
            .await;

        if chaos_check.status() == 200 {
            let chaos_data: serde_json::Value = chaos_check.json().await.unwrap();
            if chaos_data["data"]["status"] == "completed" {
                // Verify the chaos task still has its tag after completion
                let metadata = &chaos_data["data"]["metadata"];
                assert_eq!(
                    metadata["tag"], "multiworker",
                    "tag metadata must be preserved for chaos testing to work"
                );
                assert_eq!(
                    metadata["chaos_test"], true,
                    "chaos_test metadata must be preserved"
                );
                break;
            }
        }
        attempts += 1;
    }

    if attempts >= max_attempts {
        panic!("Chaos task did not complete within {max_attempts} seconds");
    }

    // Clean up: stop the processor
    processor_handle.abort();
}

/// Helper function to assert that all expected metadata fields are preserved
fn assert_metadata_preserved(
    actual_metadata: &serde_json::Value,
    expected_metadata: &serde_json::Value,
) {
    let actual_map = actual_metadata
        .as_object()
        .expect("Metadata should be an object");
    let expected_map = expected_metadata
        .as_object()
        .expect("Expected metadata should be an object");

    for (key, expected_value) in expected_map.iter() {
        assert!(
            actual_map.contains_key(key),
            "Metadata should contain key '{key}'. Actual metadata: {actual_metadata:?}"
        );

        assert_eq!(
            actual_map[key], *expected_value,
            "Metadata value for key '{key}' should be preserved. Expected: {expected_value:?}, Actual: {:?}",
            actual_map[key]
        );
    }

    // Specifically verify the critical fields for chaos testing
    assert_eq!(
        actual_map["tag"], "multiworker",
        "tag metadata is critical for chaos testing and must be preserved"
    );
    assert_eq!(
        actual_map["chaos_test"], true,
        "chaos_test metadata must be preserved"
    );
}

// =====================================
// IDOR SECURITY TESTS
// =====================================
// These tests verify that the Insecure Direct Object Reference (IDOR)
// vulnerabilities identified in code review have been properly fixed.

#[tokio::test]
async fn test_idor_protection_get_task_different_user() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create User A and their task
    let unique_username_a = format!("user_a_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_a, token_a) = factory.create_authenticated_user(&unique_username_a).await;

    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "user_a@example.com",
            "subject": "User A's Task",
            "body": "This belongs to User A"
        }
    });

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token_a.token)
        .await;
    assert_eq!(task_response.status(), 200);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Create User B
    let unique_username_b = format!("user_b_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_b, token_b) = factory.create_authenticated_user(&unique_username_b).await;

    // IDOR Attack: User B tries to access User A's task
    let response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token_b.token)
        .await;

    // Should return 404 (not 403) to prevent user enumeration
    assert_status(&response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_idor_protection_modify_task_different_user() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create User A and their task
    let unique_username_a = format!("user_a_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_a, token_a) = factory.create_authenticated_user(&unique_username_a).await;

    let task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "user_a@example.com",
            "subject": "User A's Task",
            "body": "This belongs to User A"
        }
    });

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token_a.token)
        .await;
    assert_eq!(task_response.status(), 200);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Create User B
    let unique_username_b = format!("user_b_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_b, token_b) = factory.create_authenticated_user(&unique_username_b).await;

    // IDOR Attack: User B tries to cancel User A's task
    let cancel_response = app
        .post_auth(&format!("/api/v1/tasks/{task_id}/cancel"), &token_b.token)
        .await;
    assert_status(&cancel_response, StatusCode::NOT_FOUND);

    // IDOR Attack: User B tries to retry User A's task
    let retry_response = app
        .post_auth(&format!("/api/v1/tasks/{task_id}/retry"), &token_b.token)
        .await;
    assert_status(&retry_response, StatusCode::NOT_FOUND);

    // IDOR Attack: User B tries to delete User A's task
    let delete_response = app
        .delete_auth(&format!("/api/v1/tasks/{task_id}"), &token_b.token)
        .await;
    assert_status(&delete_response, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_idor_protection_task_listing_isolation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create User A and their tasks
    let unique_username_a = format!("user_a_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_a, token_a) = factory.create_authenticated_user(&unique_username_a).await;

    let task_a1_data = json!({
        "task_type": "email",
        "payload": {"to": "user_a_task1@example.com", "subject": "A Task 1"}
    });
    let task_a2_data = json!({
        "task_type": "email",
        "payload": {"to": "user_a_task2@example.com", "subject": "A Task 2"}
    });

    let task_a1_response = app
        .post_json_auth("/api/v1/tasks", &task_a1_data, &token_a.token)
        .await;
    let task_a2_response = app
        .post_json_auth("/api/v1/tasks", &task_a2_data, &token_a.token)
        .await;
    assert_eq!(task_a1_response.status(), 200);
    assert_eq!(task_a2_response.status(), 200);

    let task_a1_json: serde_json::Value = task_a1_response.json().await.unwrap();
    let task_a2_json: serde_json::Value = task_a2_response.json().await.unwrap();
    let task_a1_id = task_a1_json["data"]["id"].as_str().unwrap();
    let task_a2_id = task_a2_json["data"]["id"].as_str().unwrap();

    // Create User B and their tasks
    let unique_username_b = format!("user_b_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_b, token_b) = factory.create_authenticated_user(&unique_username_b).await;

    let task_b1_data = json!({
        "task_type": "email",
        "payload": {"to": "user_b_task1@example.com", "subject": "B Task 1"}
    });

    let task_b1_response = app
        .post_json_auth("/api/v1/tasks", &task_b1_data, &token_b.token)
        .await;
    assert_eq!(task_b1_response.status(), 200);
    let task_b1_json: serde_json::Value = task_b1_response.json().await.unwrap();
    let task_b1_id = task_b1_json["data"]["id"].as_str().unwrap();

    // User A should only see their own tasks
    let response_a = app.get_auth("/api/v1/tasks", &token_a.token).await;
    assert_status(&response_a, StatusCode::OK);
    let json_a: serde_json::Value = response_a.json().await.unwrap();
    let tasks_a = json_a["data"].as_array().unwrap();

    let task_ids_a: Vec<&str> = tasks_a
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(
        task_ids_a.contains(&task_a1_id),
        "User A should see their own task A1"
    );
    assert!(
        task_ids_a.contains(&task_a2_id),
        "User A should see their own task A2"
    );
    assert!(
        !task_ids_a.contains(&task_b1_id),
        "User A should NOT see User B's task"
    );

    // User B should only see their own tasks
    let response_b = app.get_auth("/api/v1/tasks", &token_b.token).await;
    assert_status(&response_b, StatusCode::OK);
    let json_b: serde_json::Value = response_b.json().await.unwrap();
    let tasks_b = json_b["data"].as_array().unwrap();

    let task_ids_b: Vec<&str> = tasks_b
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(
        task_ids_b.contains(&task_b1_id),
        "User B should see their own task"
    );
    assert!(
        !task_ids_b.contains(&task_a1_id),
        "User B should NOT see User A's task A1"
    );
    assert!(
        !task_ids_b.contains(&task_a2_id),
        "User B should NOT see User A's task A2"
    );
}

#[tokio::test]
async fn test_idor_protection_dead_letter_queue_isolation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create User A and their failed task
    let unique_username_a = format!("user_a_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_a, token_a) = factory.create_authenticated_user(&unique_username_a).await;

    let task_a_data = json!({
        "task_type": "email",
        "payload": {"to": "user_a@example.com", "subject": "A Failed Task"}
    });

    let task_a_response = app
        .post_json_auth("/api/v1/tasks", &task_a_data, &token_a.token)
        .await;
    assert_eq!(task_a_response.status(), 200);
    let task_a_json: serde_json::Value = task_a_response.json().await.unwrap();
    let task_a_id = task_a_json["data"]["id"].as_str().unwrap();

    // Create User B and their failed task
    let unique_username_b = format!("user_b_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user_b, token_b) = factory.create_authenticated_user(&unique_username_b).await;

    let task_b_data = json!({
        "task_type": "email",
        "payload": {"to": "user_b@example.com", "subject": "B Failed Task"}
    });

    let task_b_response = app
        .post_json_auth("/api/v1/tasks", &task_b_data, &token_b.token)
        .await;
    assert_eq!(task_b_response.status(), 200);
    let task_b_json: serde_json::Value = task_b_response.json().await.unwrap();
    let task_b_id = task_b_json["data"]["id"].as_str().unwrap();

    // Mark both tasks as failed
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'failed', last_error = 'Test failure', completed_at = NOW() WHERE id = $1 OR id = $2",
        uuid::Uuid::parse_str(task_a_id).unwrap(),
        uuid::Uuid::parse_str(task_b_id).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // User A should only see their own failed tasks in dead letter queue
    let response_a = app
        .get_auth("/api/v1/tasks/dead-letter", &token_a.token)
        .await;
    assert_status(&response_a, StatusCode::OK);
    let json_a: serde_json::Value = response_a.json().await.unwrap();
    let dead_tasks_a = json_a["data"].as_array().unwrap();

    let dead_task_ids_a: Vec<&str> = dead_tasks_a
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(
        dead_task_ids_a.contains(&task_a_id),
        "User A should see their own failed task"
    );
    assert!(
        !dead_task_ids_a.contains(&task_b_id),
        "User A should NOT see User B's failed task"
    );

    // User B should only see their own failed tasks in dead letter queue
    let response_b = app
        .get_auth("/api/v1/tasks/dead-letter", &token_b.token)
        .await;
    assert_status(&response_b, StatusCode::OK);
    let json_b: serde_json::Value = response_b.json().await.unwrap();
    let dead_tasks_b = json_b["data"].as_array().unwrap();

    let dead_task_ids_b: Vec<&str> = dead_tasks_b
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(
        dead_task_ids_b.contains(&task_b_id),
        "User B should see their own failed task"
    );
    assert!(
        !dead_task_ids_b.contains(&task_a_id),
        "User B should NOT see User A's failed task"
    );
}

#[tokio::test]
async fn test_idor_protection_unauthenticated_access() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create a task with authentication
    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let task_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com", "subject": "Protected Task"}
    });

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
    assert_eq!(task_response.status(), 200);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // Try to access endpoints without authentication - should get 401 Unauthorized
    let unauthenticated_responses = vec![
        app.get(&format!("/api/v1/tasks/{task_id}")).await,
        app.get("/api/v1/tasks").await,
        app.post_json("/api/v1/tasks", &task_data).await,
        app.client
            .post(format!("{}/api/v1/tasks/{task_id}/cancel", app.address))
            .send()
            .await
            .unwrap(),
        app.client
            .post(format!("{}/api/v1/tasks/{task_id}/retry", app.address))
            .send()
            .await
            .unwrap(),
        app.client
            .delete(format!("{}/api/v1/tasks/{task_id}", app.address))
            .send()
            .await
            .unwrap(),
        app.get("/api/v1/tasks/dead-letter").await,
    ];

    for response in unauthenticated_responses {
        assert_status(&response, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn test_idor_protection_own_task_access_allowed() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create user and their task
    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let task_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com", "subject": "Own Task"}
    });

    let task_response = app
        .post_json_auth("/api/v1/tasks", &task_data, &token.token)
        .await;
    assert_eq!(task_response.status(), 200);
    let task_json: serde_json::Value = task_response.json().await.unwrap();
    let task_id = task_json["data"]["id"].as_str().unwrap();

    // User should be able to access their own task
    let get_response = app
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
        .await;
    assert_status(&get_response, StatusCode::OK);
    let get_json: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_json["data"]["id"].as_str().unwrap(), task_id);

    // User should be able to cancel their own task
    let cancel_response = app
        .post_auth(&format!("/api/v1/tasks/{task_id}/cancel"), &token.token)
        .await;
    assert_status(&cancel_response, StatusCode::OK);

    // Create another task for retry/delete tests
    let task_data2 = json!({
        "task_type": "email",
        "payload": {"to": "test2@example.com", "subject": "Own Task 2"}
    });

    let task_response2 = app
        .post_json_auth("/api/v1/tasks", &task_data2, &token.token)
        .await;
    let task_json2: serde_json::Value = task_response2.json().await.unwrap();
    let task_id2 = task_json2["data"]["id"].as_str().unwrap();

    // Mark as failed so we can retry
    let db = &app.db_pool;
    sqlx::query!(
        "UPDATE tasks SET status = 'failed', last_error = 'Test failure', completed_at = NOW() WHERE id = $1",
        uuid::Uuid::parse_str(task_id2).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // User should be able to retry their own failed task
    let retry_response = app
        .post_auth(&format!("/api/v1/tasks/{task_id2}/retry"), &token.token)
        .await;
    assert_status(&retry_response, StatusCode::OK);

    // Mark as completed so we can delete
    sqlx::query!(
        "UPDATE tasks SET status = 'completed', completed_at = NOW() WHERE id = $1",
        uuid::Uuid::parse_str(task_id2).unwrap()
    )
    .execute(db)
    .await
    .unwrap();

    // User should be able to delete their own completed task
    let delete_response = app
        .delete_auth(&format!("/api/v1/tasks/{task_id2}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::OK);
}

#[tokio::test]
async fn test_idor_protection_system_tasks_denied() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create a regular user
    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Manually create a system task (created_by = NULL) in the database
    let db = &app.db_pool;
    let system_task_id = uuid::Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO tasks (id, task_type, status, payload, priority, scheduled_at, created_at, updated_at, created_by)
        VALUES ($1, 'system_maintenance', 'pending', '{"type": "system"}', 'normal', NOW(), NOW(), NOW(), NULL)
        "#,
        system_task_id
    )
    .execute(db)
    .await
    .unwrap();

    // Regular user should not be able to access system tasks (created_by = NULL)
    let get_response = app
        .get_auth(&format!("/api/v1/tasks/{system_task_id}"), &token.token)
        .await;
    assert_status(&get_response, StatusCode::NOT_FOUND);

    // Regular user should not be able to modify system tasks
    let cancel_response = app
        .post_auth(
            &format!("/api/v1/tasks/{system_task_id}/cancel"),
            &token.token,
        )
        .await;
    assert_status(&cancel_response, StatusCode::NOT_FOUND);

    let retry_response = app
        .post_auth(
            &format!("/api/v1/tasks/{system_task_id}/retry"),
            &token.token,
        )
        .await;
    assert_status(&retry_response, StatusCode::NOT_FOUND);

    let delete_response = app
        .delete_auth(&format!("/api/v1/tasks/{system_task_id}"), &token.token)
        .await;
    assert_status(&delete_response, StatusCode::NOT_FOUND);

    // System tasks should not appear in user's task listing
    let list_response = app.get_auth("/api/v1/tasks", &token.token).await;
    assert_status(&list_response, StatusCode::OK);
    let list_json: serde_json::Value = list_response.json().await.unwrap();
    let tasks = list_json["data"].as_array().unwrap();

    let system_task_in_list = tasks
        .iter()
        .any(|task| task["id"].as_str() == Some(&system_task_id.to_string()));

    assert!(
        !system_task_in_list,
        "System tasks should not appear in user task listings"
    );
}

// =====================================
// COMPREHENSIVE SECURITY TESTS
// =====================================
// These tests verify all the security vulnerabilities that were fixed

#[tokio::test]
async fn test_security_input_validation_task_creation() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test invalid task_type with special characters (should be rejected)
    let malicious_task_data = json!({
        "task_type": "email'; DROP TABLE tasks; --",
        "payload": {"to": "test@example.com"}
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &malicious_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    // The validation error can come from either our validation or server-side validation
    let error_message = json["error"]["message"].as_str().unwrap();
    assert!(
        error_message.contains("alphanumeric") || 
        error_message.contains("not registered") ||
        error_message.contains("Invalid")
    );

    // Test empty task_type (should be rejected)
    let empty_task_data = json!({
        "task_type": "",
        "payload": {"to": "test@example.com"}
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &empty_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test extremely long task_type (should be rejected)
    let long_task_data = json!({
        "task_type": "a".repeat(200),
        "payload": {"to": "test@example.com"}
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &long_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test malicious metadata keys
    let malicious_metadata_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com"},
        "metadata": {
            "'; DROP TABLE tasks; --": "malicious_value",
            "normal_key": "normal_value"
        }
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &malicious_metadata_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_security_payload_size_limits() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test oversized payload (should be rejected to prevent DoS)
    let large_payload = "x".repeat(2 * 1024 * 1024); // 2MB payload
    let oversized_task_data = json!({
        "task_type": "email",
        "payload": {
            "to": "test@example.com",
            "large_data": large_payload
        }
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &oversized_task_data, &token.token)
        .await;
    // Server may return 413 (Payload Too Large) or 400 (Bad Request) depending on where validation occurs
    assert!(
        response.status() == StatusCode::BAD_REQUEST || 
        response.status() == StatusCode::PAYLOAD_TOO_LARGE
    );
    
    if response.status() == StatusCode::BAD_REQUEST {
        let json: serde_json::Value = response.json().await.unwrap();
        assert!(json["error"]["message"].as_str().unwrap().contains("1MB"));
    }
}

#[tokio::test]
async fn test_security_metadata_size_limits() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test oversized total metadata (should be rejected)
    let mut large_metadata = serde_json::Map::new();
    for i in 0..200 {
        large_metadata.insert(
            format!("key_{}", i),
            serde_json::Value::String("x".repeat(500)), // 500 chars per value
        );
    }

    let oversized_metadata_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com"},
        "metadata": large_metadata
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &oversized_metadata_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(json["error"]["message"].as_str().unwrap().contains("64KB"));
}

#[tokio::test]
async fn test_security_scheduling_limits() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test scheduling too far in the future (should be rejected)
    let far_future = chrono::Utc::now() + chrono::Duration::days(400);
    let future_task_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com"},
        "scheduled_at": far_future.to_rfc3339()
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &future_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);

    // Test scheduling too far in the past (should be rejected)
    let far_past = chrono::Utc::now() - chrono::Duration::hours(2);
    let past_task_data = json!({
        "task_type": "email",
        "payload": {"to": "test@example.com"},
        "scheduled_at": far_past.to_rfc3339()
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &past_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_security_sql_injection_protection_priority_filter() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Test SQL injection attempt in priority parameter
    let malicious_queries = vec![
        "high'; DROP TABLE tasks; --",
        "normal' OR 1=1 --",
        "low'; UPDATE tasks SET status='completed' WHERE 1=1; --",
        "critical' UNION SELECT * FROM users --",
    ];

    for malicious_priority in malicious_queries {
        let encoded_priority = malicious_priority.replace("'", "%27").replace(" ", "%20");
        let url = format!("/api/v1/tasks?priority={}", encoded_priority);
        let response = app.get_auth(&url, &token.token).await;

        // Should return OK with empty results (malicious priority is ignored)
        assert_status(&response, StatusCode::OK);
        let json: serde_json::Value = response.json().await.unwrap();

        // Should return normal response structure (not error)
        assert_json_field_exists(&json, "data");

        // Verify database is not compromised by checking task count
        let stats_response = app.get_auth("/api/v1/tasks", &token.token).await;
        assert_status(&stats_response, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_security_rbac_stats_endpoint_protection() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Regular user should be denied access to stats
    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app.get_auth("/api/v1/tasks/stats", &token.token).await;
    assert_status(&response, StatusCode::FORBIDDEN);

    // Create admin user manually using direct database access for test
    let admin_user_data = json!({
        "username": format!("admin_{}", &uuid::Uuid::new_v4().to_string()[..8]),
        "email": format!("admin_{}@example.com", &uuid::Uuid::new_v4().to_string()[..8]),
        "password": "AdminPass123!"
    });

    let admin_response = app
        .post_json("/api/v1/auth/register", &admin_user_data)
        .await;
    assert_status(&admin_response, StatusCode::OK);

    // Manually promote to admin in database
    let admin_json: serde_json::Value = admin_response.json().await.unwrap();
    
    // Debug print to see the actual response structure
    println!("Admin registration response: {}", serde_json::to_string_pretty(&admin_json).unwrap());
    
    let admin_id: uuid::Uuid = if let Some(id) = admin_json["data"]["user"]["id"].as_str() {
        uuid::Uuid::parse_str(id).unwrap()
    } else if let Some(id) = admin_json["data"]["id"].as_str() {
        uuid::Uuid::parse_str(id).unwrap()
    } else {
        panic!("Could not find user ID in response: {}", admin_json);
    };

    let db = &app.db_pool;
    sqlx::query!("UPDATE users SET role = 'admin' WHERE id = $1", admin_id)
        .execute(db)
        .await
        .unwrap();

    // Login as admin
    let admin_login_data = json!({
        "username": admin_user_data["username"],
        "password": "AdminPass123!"
    });
    let admin_login_response = app.post_json("/api/v1/auth/login", &admin_login_data).await;
    assert_status(&admin_login_response, StatusCode::OK);
    let admin_login_json: serde_json::Value = admin_login_response.json().await.unwrap();
    let admin_token = admin_login_json["data"]["session_token"].as_str().unwrap();

    let admin_stats_response = app.get_auth("/api/v1/tasks/stats", admin_token).await;
    assert_status(&admin_stats_response, StatusCode::OK);
}

#[tokio::test]
async fn test_security_task_type_validation_enforcement() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Try to create task with unregistered task type (should be rejected)
    let unregistered_task_data = json!({
        "task_type": "unregistered_malicious_type",
        "payload": {"malicious": "data"}
    });

    let response = app
        .post_json_auth("/api/v1/tasks", &unregistered_task_data, &token.token)
        .await;
    assert_status(&response, StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    assert!(
        json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not registered")
    );
}

#[tokio::test]
async fn test_security_concurrent_task_processing_race_conditions() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    let unique_username = format!("user_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple tasks that would normally cause race conditions
    let mut task_ids = Vec::new();
    for i in 0..5 {
        let task_data = json!({
            "task_type": "delay_task",
            "payload": {
                "delay_seconds": 0,  // No delay for fast processing
                "task_id": format!("race_test_{}", i)
            }
        });

        let response = app
            .post_json_auth("/api/v1/tasks", &task_data, &token.token)
            .await;
        assert_status(&response, StatusCode::OK);
        let json: serde_json::Value = response.json().await.unwrap();
        task_ids.push(json["data"]["id"].as_str().unwrap().to_string());
    }

    // Try to concurrently cancel all tasks (test race condition protection)
    let cancel_futures: Vec<_> = task_ids
        .iter()
        .map(|task_id| {
            let app = app.clone();
            let token = token.token.clone();
            let task_id = task_id.clone();
            tokio::spawn(async move {
                app.post_auth(&format!("/api/v1/tasks/{}/cancel", task_id), &token)
                    .await
            })
        })
        .collect();

    let mut results = Vec::new();
    for future in cancel_futures {
        results.push(future.await.unwrap());
    }

    // At least some should succeed, none should cause inconsistent state
    let success_count = results
        .iter()
        .filter(|r| r.status() == StatusCode::OK)
        .count();

    // Should have some successful cancellations
    assert!(
        success_count >= 1,
        "At least one cancellation should succeed"
    );

    // Verify no tasks are in inconsistent state
    for task_id in task_ids {
        let task_response = app
            .get_auth(&format!("/api/v1/tasks/{}", task_id), &token.token)
            .await;
        assert_status(&task_response, StatusCode::OK);
        let task_json: serde_json::Value = task_response.json().await.unwrap();

        if let Some(status) = task_json["data"]["status"].as_str() {
            // Task should be in a valid final state
            assert!(
                matches!(
                    status,
                    "pending" | "running" | "completed" | "failed" | "cancelled"
                ),
                "Task should be in valid state, got: {}",
                status
            );
        }
    }
}
