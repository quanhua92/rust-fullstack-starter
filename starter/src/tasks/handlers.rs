use async_trait::async_trait;
use std::collections::HashMap;

use crate::tasks::types::{TaskContext, TaskError, TaskResult};

/// Trait that all task handlers must implement
#[async_trait]
pub trait TaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError>;
}

/// Example: Email notification task handler
pub struct EmailTaskHandler;

#[async_trait]
impl TaskHandler for EmailTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        // Extract email data from payload
        let to = context
            .payload
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'to' field in email payload".to_string())
            })?;

        let subject = context
            .payload
            .get("subject")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'subject' field in email payload".to_string())
            })?;

        let body = context
            .payload
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'body' field in email payload".to_string())
            })?;

        // Simulate email sending (replace with actual email service)
        tracing::info!("Sending email to: {}, subject: {}", to, subject);

        // Simulate some processing time
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Simulate occasional failures for testing
        if body.contains("fail") {
            return Err(TaskError::Execution(
                "Email service temporarily unavailable".to_string(),
            ));
        }

        // Return success with metadata
        let mut metadata = HashMap::new();
        metadata.insert("recipient".to_string(), serde_json::json!(to));
        metadata.insert("sent_at".to_string(), serde_json::json!(chrono::Utc::now()));

        Ok(TaskResult::success_empty().with_metadata("email_sent", serde_json::json!(true)))
    }
}

/// Example: Data processing task handler
pub struct DataProcessingTaskHandler;

#[async_trait]
impl TaskHandler for DataProcessingTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let data = context
            .payload
            .get("data")
            .ok_or_else(|| TaskError::Execution("Missing 'data' field in payload".to_string()))?;

        let operation = context
            .payload
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("process");

        tracing::info!("Processing data with operation: {}", operation);

        // Simulate data processing
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let result = match operation {
            "count" => {
                if let Some(array) = data.as_array() {
                    serde_json::json!({ "count": array.len() })
                } else {
                    return Err(TaskError::Execution(
                        "Data is not an array for count operation".to_string(),
                    ));
                }
            }
            "sum" => {
                if let Some(array) = data.as_array() {
                    let sum: f64 = array.iter().filter_map(|v| v.as_f64()).sum();
                    serde_json::json!({ "sum": sum })
                } else {
                    return Err(TaskError::Execution(
                        "Data is not an array for sum operation".to_string(),
                    ));
                }
            }
            "process" => {
                serde_json::json!({ "processed": true, "timestamp": chrono::Utc::now() })
            }
            _ => {
                return Err(TaskError::Execution(format!(
                    "Unknown operation: {operation}"
                )));
            }
        };

        Ok(TaskResult::success(result))
    }
}

/// Example: File cleanup task handler
pub struct FileCleanupTaskHandler;

#[async_trait]
impl TaskHandler for FileCleanupTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let file_path = context
            .payload
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'file_path' field in payload".to_string())
            })?;

        let max_age_hours = context
            .payload
            .get("max_age_hours")
            .and_then(|v| v.as_u64())
            .unwrap_or(24);

        tracing::info!(
            "Cleaning up files in path: {}, max age: {} hours",
            file_path,
            max_age_hours
        );

        // Simulate file cleanup operation
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        // In a real implementation, you would:
        // 1. List files in the directory
        // 2. Check file ages
        // 3. Delete files older than max_age_hours
        // 4. Return statistics

        let files_deleted = 3; // Simulated result
        let bytes_freed = 1024 * 1024; // Simulated result

        let result = serde_json::json!({
            "files_deleted": files_deleted,
            "bytes_freed": bytes_freed,
            "cleanup_path": file_path
        });

        Ok(TaskResult::success(result))
    }
}

/// Example: Report generation task handler
pub struct ReportGenerationTaskHandler;

#[async_trait]
impl TaskHandler for ReportGenerationTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let report_type = context
            .payload
            .get("report_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'report_type' field in payload".to_string())
            })?;

        let start_date = context
            .payload
            .get("start_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'start_date' field in payload".to_string())
            })?;

        let end_date = context
            .payload
            .get("end_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TaskError::Execution("Missing 'end_date' field in payload".to_string())
            })?;

        tracing::info!(
            "Generating {} report from {} to {}",
            report_type,
            start_date,
            end_date
        );

        // Simulate report generation (this could be quite long for real reports)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let report_url =
            format!("/reports/{report_type}/{report_type}-{start_date}-{end_date}.pdf");

        let result = serde_json::json!({
            "report_type": report_type,
            "start_date": start_date,
            "end_date": end_date,
            "report_url": report_url,
            "generated_at": chrono::Utc::now(),
            "status": "completed"
        });

        Ok(TaskResult::success(result))
    }
}

/// Example: Webhook notification task handler
pub struct WebhookTaskHandler;

#[async_trait]
impl TaskHandler for WebhookTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let url = context
            .payload
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TaskError::Execution("Missing 'url' field in payload".to_string()))?;

        let _payload = context
            .payload
            .get("payload")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let method = context
            .payload
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST");

        tracing::info!("Sending webhook {} to: {}", method, url);

        // Simulate HTTP request (replace with actual HTTP client)
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        // Simulate response
        let response_status = 200;
        let response_body = serde_json::json!({ "success": true });

        if url.contains("fail") {
            return Err(TaskError::Execution(
                "Webhook endpoint returned 500".to_string(),
            ));
        }

        let result = serde_json::json!({
            "url": url,
            "method": method,
            "response_status": response_status,
            "response_body": response_body,
            "sent_at": chrono::Utc::now()
        });

        Ok(TaskResult::success(result))
    }
}

/// Example: Delay task handler for chaos testing
pub struct DelayTaskHandler;

#[async_trait]
impl TaskHandler for DelayTaskHandler {
    async fn handle(&self, context: TaskContext) -> Result<TaskResult, TaskError> {
        let delay_seconds = context
            .payload
            .get("delay_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        let task_id = context
            .payload
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let test_scenario = context
            .payload
            .get("test_scenario")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        // Check if deadline is specified in payload
        let deadline_str = context.payload.get("deadline").and_then(|v| v.as_str());

        tracing::info!(
            "Processing delay task: {} (scenario: {}, delay: {}s, attempt: {})",
            task_id,
            test_scenario,
            delay_seconds,
            context.attempt
        );

        // Check deadline before starting work
        if let Some(deadline) = deadline_str {
            if let Ok(deadline_time) = chrono::DateTime::parse_from_rfc3339(deadline) {
                let deadline_utc = deadline_time.with_timezone(&chrono::Utc);
                let now = chrono::Utc::now();
                if now >= deadline_utc {
                    tracing::warn!(
                        "Task {} already past deadline: {} vs {}",
                        task_id,
                        now,
                        deadline_utc
                    );
                    return Err(TaskError::Execution(format!(
                        "Task {task_id} missed deadline: {now} vs {deadline_utc}"
                    )));
                }

                let time_remaining = (deadline_utc - now).num_seconds();
                if time_remaining < delay_seconds as i64 {
                    tracing::warn!(
                        "Task {} cannot complete within deadline: {}s remaining, {}s needed",
                        task_id,
                        time_remaining,
                        delay_seconds
                    );
                    return Err(TaskError::Execution(format!(
                        "Task {task_id} insufficient time: {time_remaining}s remaining, {delay_seconds}s needed"
                    )));
                }
            }
        }

        // Simulate the delay work
        tracing::info!("Task {} starting {}s delay work...", task_id, delay_seconds);
        tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;

        // Check deadline again after work
        if let Some(deadline) = deadline_str {
            if let Ok(deadline_time) = chrono::DateTime::parse_from_rfc3339(deadline) {
                let deadline_utc = deadline_time.with_timezone(&chrono::Utc);
                let now = chrono::Utc::now();
                if now >= deadline_utc {
                    tracing::warn!(
                        "Task {} completed but past deadline: {} vs {}",
                        task_id,
                        now,
                        deadline_utc
                    );
                    return Err(TaskError::Execution(format!(
                        "Task {task_id} completed past deadline: {now} vs {deadline_utc}"
                    )));
                }
            }
        }

        tracing::info!(
            "Task {} completed successfully after {}s",
            task_id,
            delay_seconds
        );

        let result = serde_json::json!({
            "task_id": task_id,
            "delay_seconds": delay_seconds,
            "test_scenario": test_scenario,
            "attempt": context.attempt,
            "completed_at": chrono::Utc::now(),
            "status": "completed"
        });

        Ok(TaskResult::success(result))
    }
}

/// Helper function to register all example handlers
pub async fn register_example_handlers(processor: &crate::tasks::processor::TaskProcessor) {
    processor
        .register_handler("email".to_string(), EmailTaskHandler)
        .await;
    processor
        .register_handler("data_processing".to_string(), DataProcessingTaskHandler)
        .await;
    processor
        .register_handler("file_cleanup".to_string(), FileCleanupTaskHandler)
        .await;
    processor
        .register_handler("report_generation".to_string(), ReportGenerationTaskHandler)
        .await;
    processor
        .register_handler("webhook".to_string(), WebhookTaskHandler)
        .await;
    processor
        .register_handler("delay_task".to_string(), DelayTaskHandler)
        .await;

    tracing::info!("Registered all example task handlers including delay_task");
}
