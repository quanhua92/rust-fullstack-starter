use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::Error,
    tasks::{
        processor::TaskProcessor,
        types::{CreateTaskRequest, TaskFilter, TaskResponse, TaskStats, TaskStatus},
    },
    types::{ApiResponse, AppState, ErrorResponse},
};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateTaskApiRequest {
    pub task_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub priority: Option<String>, // "low", "normal", "high", "critical"
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TaskQueryParams {
    pub task_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RegisterTaskTypeRequest {
    pub task_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TaskTypeResponse {
    pub task_type: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create a new background task
#[utoipa::path(
    post,
    path = "/tasks",
    tag = "Tasks",
    summary = "Create task",
    description = "Create a new background task",
    request_body = CreateTaskApiRequest,
    responses(
        (status = 200, description = "Task created", body = ApiResponse<TaskResponse>),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_task(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<CreateTaskApiRequest>,
) -> Result<Json<ApiResponse<crate::tasks::types::TaskResponse>>, Error> {
    use crate::tasks::types::TaskPriority;

    let priority = match payload.priority.as_deref() {
        Some("low") => TaskPriority::Low,
        Some("high") => TaskPriority::High,
        Some("critical") => TaskPriority::Critical,
        _ => TaskPriority::Normal,
    };

    // Validate that the task type is registered
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(|e| Error::Internal(format!("Failed to acquire database connection: {e}")))?;

    let task_type_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM task_types WHERE task_type = $1 AND is_active = true)",
        payload.task_type
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| Error::Internal(format!("Failed to validate task type: {e}")))?;

    if !task_type_exists.unwrap_or(false) {
        return Err(Error::validation(
            "task_type",
            &format!(
                "Task type '{}' is not registered. Workers must register task types before tasks can be created.",
                payload.task_type
            ),
        ));
    }

    let mut request = CreateTaskRequest::new(payload.task_type, payload.payload)
        .with_priority(priority)
        .with_scheduled_at(payload.scheduled_at.unwrap_or_else(chrono::Utc::now))
        .with_created_by(auth_user.id) // Set the user who created this task
        .with_metadata("api_created", serde_json::json!(true));

    // Add user-provided metadata
    for (key, value) in payload.metadata {
        request = request.with_metadata(key, value);
    }

    // Create a temporary processor to create the task
    // In a real application, you might want to have a shared processor instance
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let task = processor
        .create_task(request)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create task: {e}")))?;

    Ok(Json(ApiResponse::success(task.into())))
}

/// Get a task by ID
#[utoipa::path(
    get,
    path = "/tasks/{id}",
    tag = "Tasks",
    summary = "Get task",
    description = "Get a task by its ID",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = ApiResponse<TaskResponse>),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Option<crate::tasks::types::TaskResponse>>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let task = processor
        .get_task(task_id)
        .await
        .map_err(|e| Error::Internal(format!("Failed to get task: {e}")))?;

    // Check ownership - only allow access to tasks created by the authenticated user
    if let Some(ref task_data) = task {
        if let Some(created_by) = task_data.created_by {
            if created_by != auth_user.id {
                return Err(Error::NotFound("Task not found".to_string())); // Return NotFound to prevent user enumeration
            }
        } else {
            // Tasks without created_by are system tasks, deny access to regular users
            return Err(Error::NotFound("Task not found".to_string()));
        }
    }

    Ok(Json(ApiResponse::success(task.map(|t| t.into()))))
}

/// List tasks with optional filtering
#[utoipa::path(
    get,
    path = "/tasks",
    tag = "Tasks",
    summary = "List tasks",
    description = "List tasks with optional filtering",
    params(
        TaskQueryParams
    ),
    responses(
        (status = 200, description = "List of tasks", body = ApiResponse<Vec<TaskResponse>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tasks(
    State(app_state): State<AppState>,
    Query(params): Query<TaskQueryParams>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<crate::tasks::types::TaskResponse>>>, Error> {
    let status = match params.status.as_deref() {
        Some("pending") => Some(TaskStatus::Pending),
        Some("running") => Some(TaskStatus::Running),
        Some("completed") => Some(TaskStatus::Completed),
        Some("failed") => Some(TaskStatus::Failed),
        Some("cancelled") => Some(TaskStatus::Cancelled),
        Some("retrying") => Some(TaskStatus::Retrying),
        _ => None,
    };

    let filter = TaskFilter {
        task_type: params.task_type,
        status,
        priority: None,                 // TODO: Parse priority string if needed
        created_by: Some(auth_user.id), // Only list tasks created by the authenticated user
        created_after: None,
        created_before: None,
        limit: params.limit,
        offset: params.offset,
    };

    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let tasks = processor
        .list_tasks(filter)
        .await
        .map_err(|e| Error::Internal(format!("Failed to list tasks: {e}")))?;

    let task_responses: Vec<crate::tasks::types::TaskResponse> =
        tasks.into_iter().map(|t| t.into()).collect();
    Ok(Json(ApiResponse::success(task_responses)))
}

/// Get task statistics
#[utoipa::path(
    get,
    path = "/tasks/stats",
    tag = "Tasks",
    summary = "Get task statistics",
    description = "Get statistics about tasks (total, pending, completed, failed, etc.)",
    responses(
        (status = 200, description = "Task statistics", body = ApiResponse<TaskStats>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_stats(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<TaskStats>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let stats = processor
        .get_stats()
        .await
        .map_err(|e| Error::Internal(format!("Failed to get stats: {e}")))?;

    Ok(Json(ApiResponse::success(stats)))
}

/// Cancel a task
#[utoipa::path(
    post,
    path = "/tasks/{id}/cancel",
    tag = "Tasks",
    summary = "Cancel task",
    description = "Cancel a running or pending task",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task cancelled", body = ApiResponse<String>),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn cancel_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    // First, get the task to check ownership
    let task = processor
        .get_task(task_id)
        .await
        .map_err(|e| Error::Internal(format!("Failed to get task: {e}")))?
        .ok_or(Error::NotFound("Task not found".to_string()))?;

    // Check ownership - only allow cancelling tasks created by the authenticated user
    if let Some(created_by) = task.created_by {
        if created_by != auth_user.id {
            return Err(Error::NotFound("Task not found".to_string())); // Return NotFound to prevent user enumeration
        }
    } else {
        // Tasks without created_by are system tasks, deny access to regular users
        return Err(Error::NotFound("Task not found".to_string()));
    }

    processor
        .cancel_task(task_id)
        .await
        .map_err(|e| Error::Internal(format!("Failed to cancel task: {e}")))?;

    Ok(Json(ApiResponse::success_with_message(
        "Task cancelled successfully".to_string(),
        format!("Task {task_id} has been cancelled"),
    )))
}

/// Get dead letter queue (failed tasks)
#[utoipa::path(
    get,
    path = "/tasks/dead-letter",
    tag = "Tasks",
    summary = "Get dead letter queue",
    description = "Get all failed tasks in the dead letter queue",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum number of tasks to return"),
        ("offset" = Option<i64>, Query, description = "Number of tasks to skip")
    ),
    responses(
        (status = 200, description = "Dead letter queue tasks", body = ApiResponse<Vec<TaskResponse>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_dead_letter_queue(
    State(app_state): State<AppState>,
    Query(params): Query<TaskQueryParams>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<crate::tasks::types::TaskResponse>>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    let tasks = processor
        .get_dead_letter_queue(params.limit, params.offset)
        .await
        .map_err(|e| Error::Internal(format!("Failed to get dead letter queue: {e}")))?;

    // Filter tasks to only show those created by the authenticated user
    let user_tasks: Vec<crate::tasks::types::TaskResponse> = tasks
        .into_iter()
        .filter(|task| task.created_by == Some(auth_user.id))
        .map(|t| t.into())
        .collect();

    Ok(Json(ApiResponse::success(user_tasks)))
}

/// Retry a failed task
#[utoipa::path(
    post,
    path = "/tasks/{id}/retry",
    tag = "Tasks",
    summary = "Retry failed task",
    description = "Retry a failed task by resetting it to pending status",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task retried successfully", body = ApiResponse<String>),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 400, description = "Task is not in failed status", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn retry_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    // First, get the task to check ownership
    let task = processor
        .get_task(task_id)
        .await
        .map_err(|e| Error::Internal(format!("Failed to get task: {e}")))?
        .ok_or(Error::NotFound("Task not found".to_string()))?;

    // Check ownership - only allow retrying tasks created by the authenticated user
    if let Some(created_by) = task.created_by {
        if created_by != auth_user.id {
            return Err(Error::NotFound("Task not found".to_string())); // Return NotFound to prevent user enumeration
        }
    } else {
        // Tasks without created_by are system tasks, deny access to regular users
        return Err(Error::NotFound("Task not found".to_string()));
    }

    processor.retry_task(task_id).await.map_err(|e| match e {
        crate::tasks::types::TaskError::NotFound(_) => {
            Error::NotFound("Task not found or not in failed status".to_string())
        }
        _ => Error::Internal(format!("Failed to retry task: {e}")),
    })?;

    Ok(Json(ApiResponse::success_with_message(
        "Task retried successfully".to_string(),
        format!("Task {task_id} has been reset to pending status"),
    )))
}

/// Delete a task permanently
#[utoipa::path(
    delete,
    path = "/tasks/{id}",
    tag = "Tasks",
    summary = "Delete task",
    description = "Permanently delete a completed, failed, or cancelled task",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task deleted successfully", body = ApiResponse<String>),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 400, description = "Task is not in a deletable status", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_task(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let processor = TaskProcessor::new(
        app_state.database.clone(),
        crate::tasks::processor::ProcessorConfig::default(),
    );

    // First, get the task to check ownership
    let task = processor
        .get_task(task_id)
        .await
        .map_err(|e| Error::Internal(format!("Failed to get task: {e}")))?
        .ok_or(Error::NotFound("Task not found".to_string()))?;

    // Check ownership - only allow deleting tasks created by the authenticated user
    if let Some(created_by) = task.created_by {
        if created_by != auth_user.id {
            return Err(Error::NotFound("Task not found".to_string())); // Return NotFound to prevent user enumeration
        }
    } else {
        // Tasks without created_by are system tasks, deny access to regular users
        return Err(Error::NotFound("Task not found".to_string()));
    }

    processor.delete_task(task_id).await.map_err(|e| match e {
        crate::tasks::types::TaskError::NotFound(_) => {
            Error::NotFound("Task not found or not in deletable status".to_string())
        }
        _ => Error::Internal(format!("Failed to delete task: {e}")),
    })?;

    Ok(Json(ApiResponse::success_with_message(
        "Task deleted successfully".to_string(),
        format!("Task {task_id} has been permanently deleted"),
    )))
}

/// Register a new task type
#[utoipa::path(
    post,
    path = "/tasks/types",
    tag = "Tasks",
    summary = "Register task type",
    description = "Register a new task type that workers can handle",
    request_body = RegisterTaskTypeRequest,
    responses(
        (status = 200, description = "Task type registered", body = ApiResponse<TaskTypeResponse>),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    )
)]
pub async fn register_task_type(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterTaskTypeRequest>,
) -> Result<Json<ApiResponse<TaskTypeResponse>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(|e| Error::Internal(format!("Failed to acquire database connection: {e}")))?;

    // Insert or update task type
    let task_type = sqlx::query_as!(
        TaskTypeResponse,
        r#"
        INSERT INTO task_types (task_type, description)
        VALUES ($1, $2)
        ON CONFLICT (task_type) DO UPDATE SET
            description = EXCLUDED.description,
            updated_at = NOW()
        RETURNING task_type, description, is_active, created_at, updated_at
        "#,
        payload.task_type,
        payload.description
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| Error::Internal(format!("Failed to register task type: {e}")))?;

    Ok(Json(ApiResponse::success(task_type)))
}

/// List registered task types
#[utoipa::path(
    get,
    path = "/tasks/types",
    tag = "Tasks",
    summary = "List task types",
    description = "List all registered task types",
    responses(
        (status = 200, description = "List of task types", body = ApiResponse<Vec<TaskTypeResponse>>)
    )
)]
pub async fn list_task_types(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<TaskTypeResponse>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(|e| Error::Internal(format!("Failed to acquire database connection: {e}")))?;

    let task_types = sqlx::query_as!(
        TaskTypeResponse,
        r#"
        SELECT task_type, description, is_active, created_at, updated_at
        FROM task_types
        WHERE is_active = true
        ORDER BY task_type
        "#
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| Error::Internal(format!("Failed to list task types: {e}")))?;

    Ok(Json(ApiResponse::success(task_types)))
}
