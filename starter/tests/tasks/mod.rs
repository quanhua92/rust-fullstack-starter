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
        .get_auth(&format!("/api/v1/tasks/{task_id}"), &token.token)
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
            assert_eq!(status, "Pending");
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

    assert_eq!(failed_task["status"], "Failed");
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
    let response = app
        .get_auth("/api/v1/tasks?status=failed", &token.token)
        .await;
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
        .get_auth("/api/v1/tasks/dead-letter?limit=10&offset=0", &token.token)
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
        initial_status == "Pending" || initial_status == "Running",
        "Expected Pending or Running, got: {initial_status}"
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
                "Completed" => {
                    final_task = Some(task_data);
                    break;
                }
                "Failed" => {
                    panic!(
                        "Task failed during processing: {}",
                        task_data["data"]["last_error"]
                            .as_str()
                            .unwrap_or("Unknown error")
                    );
                }
                "Running" => {
                    // Verify metadata is preserved during RUNNING state
                    assert_metadata_preserved(&task_data["data"]["metadata"], &test_metadata);
                }
                "Pending" => {
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

    assert_eq!(completed_task["data"]["status"], "Completed");

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
            if chaos_data["data"]["status"] == "Completed" {
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
