use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    types::{AppState, ApiResponse},
    error::Error,
    tasks::{
        types::{CreateTaskRequest, TaskFilter, TaskStats},
        processor::TaskProcessor,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskApiRequest {
    pub task_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub priority: Option<String>, // "low", "normal", "high", "critical"
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQueryParams {
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new background task
pub async fn create_task(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTaskApiRequest>,
) -> Result<Json<ApiResponse<crate::tasks::types::Task>>, Error> {
    use crate::tasks::types::TaskPriority;
    
    let priority = match payload.priority.as_deref() {
        Some("low") => TaskPriority::Low,
        Some("high") => TaskPriority::High,
        Some("critical") => TaskPriority::Critical,
        _ => TaskPriority::Normal,
    };

    let request = CreateTaskRequest::new(payload.task_type, payload.payload)
        .with_priority(priority)
        .with_scheduled_at(payload.scheduled_at.unwrap_or_else(chrono::Utc::now))
        .with_metadata("api_created", serde_json::json!(true));

    // Create a temporary processor to create the task
    // In a real application, you might want to have a shared processor instance
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let task = processor.create_task(request).await
        .map_err(|e| Error::Internal(format!("Failed to create task: {}", e)))?;

    Ok(Json(ApiResponse::success(task)))
}

/// Get a task by ID
pub async fn get_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Option<crate::tasks::types::Task>>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let task = processor.get_task(task_id).await
        .map_err(|e| Error::Internal(format!("Failed to get task: {}", e)))?;

    Ok(Json(ApiResponse::success(task)))
}

/// List tasks with optional filtering
pub async fn list_tasks(
    State(app_state): State<AppState>,
    Query(params): Query<TaskQueryParams>,
) -> Result<Json<ApiResponse<Vec<crate::tasks::types::Task>>>, Error> {
    let filter = TaskFilter {
        task_type: params.task_type,
        status: None, // TODO: Parse status string
        priority: None, // TODO: Parse priority string
        created_by: None,
        created_after: None,
        created_before: None,
        limit: params.limit,
        offset: params.offset,
    };

    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let tasks = processor.list_tasks(filter).await
        .map_err(|e| Error::Internal(format!("Failed to list tasks: {}", e)))?;

    Ok(Json(ApiResponse::success(tasks)))
}

/// Get task statistics
pub async fn get_stats(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<TaskStats>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let stats = processor.get_stats().await
        .map_err(|e| Error::Internal(format!("Failed to get stats: {}", e)))?;

    Ok(Json(ApiResponse::success(stats)))
}

/// Cancel a task
pub async fn cancel_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    processor.cancel_task(task_id).await
        .map_err(|e| Error::Internal(format!("Failed to cancel task: {}", e)))?;

    Ok(Json(ApiResponse::success_with_message(
        "Task cancelled successfully".to_string(),
        format!("Task {} has been cancelled", task_id)
    )))
}