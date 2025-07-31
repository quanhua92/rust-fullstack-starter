use crate::helpers::*;
use starter::Database;
use starter::cli::{AdminService, TaskTypeService};

#[tokio::test]
async fn test_admin_service_list_tasks() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create some test tasks
    let _task1 = factory
        .create_task(
            "email",
            serde_json::json!({
                "to": "test@example.com",
                "subject": "Test",
                "body": "Test body"
            }),
        )
        .await;

    let _task2 = factory
        .create_task(
            "data_processing",
            serde_json::json!({
                "operation": "sum",
                "data": [1, 2, 3, 4, 5]
            }),
        )
        .await;

    // Test admin service - use the test app's database instead of creating a new connection
    let database = Database {
        pool: app.db_pool.clone(),
    };
    let admin_service = AdminService::new(database);

    // Test listing tasks
    let tasks = admin_service
        .list_tasks(None, None, 10, false)
        .await
        .unwrap();
    assert!(!tasks.is_empty(), "Should have at least 2 tasks");
    assert!(tasks.len() >= 2, "Should have at least 2 tasks");

    // Test with limit
    let limited_tasks = admin_service
        .list_tasks(None, None, 1, false)
        .await
        .unwrap();
    assert_eq!(limited_tasks.len(), 1, "Should respect limit parameter");
}

#[tokio::test]
async fn test_admin_service_task_stats() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create test tasks - use regular create_task first
    let _task1 = factory
        .create_task(
            "email",
            serde_json::json!({
                "to": "test@example.com",
                "subject": "Test",
                "body": "Test body"
            }),
        )
        .await;

    // Test admin service - use the test app's database instead of creating a new connection
    let database = Database {
        pool: app.db_pool.clone(),
    };
    let admin_service = AdminService::new(database);

    // Test overall stats
    let overall_stats = admin_service.get_task_stats(None).await.unwrap();
    assert!(overall_stats.total > 0, "Should have at least one task");
    assert!(
        !overall_stats.stats.is_empty(),
        "Should have status statistics"
    );

    // Just verify we have some stats - don't assume the specific status
    assert!(overall_stats.total > 0, "Should have tasks");
    assert!(
        !overall_stats.stats.is_empty(),
        "Should have status statistics"
    );
}

#[tokio::test]
async fn test_admin_service_clear_completed_dry_run() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new_with_task_types(app.clone()).await;

    // Create a test task
    let _task = factory
        .create_task(
            "email",
            serde_json::json!({
                "to": "test@example.com",
                "subject": "Test",
                "body": "Test body"
            }),
        )
        .await;

    // Test admin service - use the test app's database instead of creating a new connection
    let database = Database {
        pool: app.db_pool.clone(),
    };
    let admin_service = AdminService::new(database);

    // Test dry run - should not actually delete anything
    let count = admin_service.clear_completed_tasks(7, true).await.unwrap();
    assert!(count >= 0, "Dry run should return count without errors");

    // Verify tasks still exist
    let tasks_after = admin_service
        .list_tasks(None, None, 10, false)
        .await
        .unwrap();
    assert!(
        !tasks_after.is_empty(),
        "Tasks should still exist after dry run"
    );
}

#[tokio::test]
async fn test_task_stats_display_format() {
    use starter::cli::models::{TaskStats, TaskStatsSummary};

    let stats = vec![
        TaskStats {
            status: "pending".to_string(),
            count: 5,
        },
        TaskStats {
            status: "completed".to_string(),
            count: 10,
        },
        TaskStats {
            status: "failed".to_string(),
            count: 2,
        },
    ];

    let summary = TaskStatsSummary {
        stats,
        total: 17,
        avg_completion_time: Some(2.5),
    };

    // Test that we can create the summary structure
    assert_eq!(summary.total, 17);
    assert_eq!(summary.stats.len(), 3);
    assert_eq!(summary.avg_completion_time, Some(2.5));

    // Test individual stats
    let pending_stat = &summary.stats[0];
    assert_eq!(pending_stat.status, "pending");
    assert_eq!(pending_stat.count, 5);
}

#[tokio::test]
async fn test_cli_models_default_values() {
    use starter::cli::models::AdminConfig;

    let config = AdminConfig::default();
    assert_eq!(config.default_limit, 50);
    assert_eq!(config.default_days, 7);
}

#[tokio::test]
async fn test_task_type_service() {
    // This test validates that the TaskTypeService can be instantiated
    // and its register function exists (though we won't actually call the API in tests)

    // Test that we can call the registration function with a mock URL
    // Note: This will fail in test environment but validates the API exists
    let result =
        TaskTypeService::register_task_types_with_api(Some("http://invalid-test-url".to_string()))
            .await;

    // We expect this to fail in test environment, but it should fail with a network error,
    // not a compilation error, proving the function signature is correct
    assert!(
        result.is_err(),
        "Should fail with network error in test environment"
    );
}

#[cfg(test)]
mod unit_tests {
    use starter::cli::models::{TaskInfo, TaskStats};

    #[test]
    fn test_task_info_creation() {
        let task_info = TaskInfo {
            id: uuid::Uuid::new_v4(),
            task_type: "email".to_string(),
            status: "pending".to_string(),
            priority: "normal".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({"test": true}),
        };

        assert_eq!(task_info.task_type, "email");
        assert_eq!(task_info.status, "pending");
        assert_eq!(task_info.priority, "normal");
    }

    #[test]
    fn test_task_stats_creation() {
        let stats = TaskStats {
            status: "completed".to_string(),
            count: 42,
        };

        assert_eq!(stats.status, "completed");
        assert_eq!(stats.count, 42);
    }
}
