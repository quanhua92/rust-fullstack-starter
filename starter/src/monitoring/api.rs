use super::models::*;
use super::services;
use crate::Error;
use crate::auth::AuthUser;
use crate::rbac::services as rbac_services;
use crate::types::{ApiResponse, AppState, ErrorResponse};
use axum::{
    Extension,
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for event listing
#[derive(Debug, Deserialize)]
pub struct EventQueryParams {
    pub event_type: Option<EventType>,
    pub source: Option<String>,
    pub level: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for metric listing
#[derive(Debug, Deserialize)]
pub struct MetricQueryParams {
    pub name: Option<String>,
    pub metric_type: Option<MetricType>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for incident listing
#[derive(Debug, Deserialize)]
pub struct IncidentQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for timeline
#[derive(Debug, Deserialize)]
pub struct TimelineQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Create a new event
#[utoipa::path(
    post,
    path = "/api/v1/monitoring/events",
    request_body = CreateEventRequest,
    responses(
        (status = 200, description = "Event created successfully", body = ApiResponse<Event>),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn create_event(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateEventRequest>,
) -> Result<Json<ApiResponse<Event>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Users can create events for their own services, moderators+ can create any events
    if !auth_user
        .role
        .has_role_or_higher(crate::rbac::models::UserRole::Moderator)
    {
        // Basic users can only create events for sources they own
        // For now, allow all authenticated users to create events
        // In production, you might want to restrict based on service ownership
    }

    let event = services::create_event(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(event)))
}

/// Get events with filters
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/events",
    params(
        ("event_type" = Option<EventType>, Query, description = "Filter by event type"),
        ("source" = Option<String>, Query, description = "Filter by source"),
        ("level" = Option<String>, Query, description = "Filter by level"),
        ("start_time" = Option<String>, Query, description = "Start time filter (ISO 8601)"),
        ("end_time" = Option<String>, Query, description = "End time filter (ISO 8601)"),
        ("limit" = Option<i64>, Query, description = "Maximum number of events to return"),
        ("offset" = Option<i64>, Query, description = "Number of events to skip")
    ),
    responses(
        (status = 200, description = "Events retrieved successfully", body = ApiResponse<Vec<Event>>),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn get_events(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(params): Query<EventQueryParams>,
) -> Result<Json<ApiResponse<Vec<Event>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Users can view all events, but in production you might want to filter by source ownership
    let filter = EventFilter {
        event_type: params.event_type,
        source: params.source,
        level: params.level,
        start_time: params.start_time,
        end_time: params.end_time,
        tags: None, // TODO: Add tag filtering from query params
        limit: params.limit,
        offset: params.offset,
    };

    let events = services::find_events_with_filter(&mut conn, filter).await?;
    Ok(Json(ApiResponse::success(events)))
}

/// Get a specific event by ID
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/events/{id}",
    params(
        ("id" = Uuid, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "Event retrieved successfully", body = ApiResponse<Event>),
        (status = 404, description = "Event not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn get_event_by_id(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Event>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let event = services::find_event_by_id(&mut conn, id).await?;
    let event = event.ok_or_else(|| Error::NotFound(format!("Event with id {id}")))?;
    Ok(Json(ApiResponse::success(event)))
}

/// Create a new metric
#[utoipa::path(
    post,
    path = "/api/v1/monitoring/metrics",
    request_body = CreateMetricRequest,
    responses(
        (status = 200, description = "Metric created successfully", body = ApiResponse<Metric>),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn create_metric(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(request): Json<CreateMetricRequest>,
) -> Result<Json<ApiResponse<Metric>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let metric = services::create_metric(&mut conn, request).await?;
    Ok(Json(ApiResponse::success(metric)))
}

/// Get metrics with filters
pub async fn get_metrics(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(params): Query<MetricQueryParams>,
) -> Result<Json<ApiResponse<Vec<Metric>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let filter = MetricFilter {
        name: params.name,
        metric_type: params.metric_type,
        start_time: params.start_time,
        end_time: params.end_time,
        labels: None, // TODO: Add label filtering from query params
        limit: params.limit,
        offset: params.offset,
    };

    let metrics = services::find_metrics_with_filter(&mut conn, filter).await?;
    Ok(Json(ApiResponse::success(metrics)))
}

/// Create a new alert (requires moderator or higher)
#[utoipa::path(
    post,
    path = "/api/v1/monitoring/alerts",
    request_body = CreateAlertRequest,
    responses(
        (status = 200, description = "Alert created successfully", body = ApiResponse<Alert>),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires moderator role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn create_alert(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateAlertRequest>,
) -> Result<Json<ApiResponse<Alert>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    rbac_services::require_moderator_or_higher(&auth_user)?;

    let alert = services::create_alert(&mut conn, request, Some(auth_user.id)).await?;
    Ok(Json(ApiResponse::success(alert)))
}

/// Get all alerts
pub async fn get_alerts(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<Alert>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let alerts = services::find_all_alerts(&mut conn).await?;
    Ok(Json(ApiResponse::success(alerts)))
}

/// Create a new incident
#[utoipa::path(
    post,
    path = "/api/v1/monitoring/incidents",
    request_body = CreateIncidentRequest,
    responses(
        (status = 200, description = "Incident created successfully", body = ApiResponse<Incident>),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn create_incident(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateIncidentRequest>,
) -> Result<Json<ApiResponse<Incident>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Users can create incidents, but moderators+ can assign them
    if request.assigned_to.is_some() {
        rbac_services::require_moderator_or_higher(&auth_user)?;
    }

    let incident = services::create_incident(&mut conn, request, Some(auth_user.id)).await?;
    Ok(Json(ApiResponse::success(incident)))
}

/// Get incidents with pagination
pub async fn get_incidents(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Query(params): Query<IncidentQueryParams>,
) -> Result<Json<ApiResponse<Vec<Incident>>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let incidents =
        services::find_incidents_with_pagination(&mut conn, params.limit, params.offset).await?;
    Ok(Json(ApiResponse::success(incidents)))
}

/// Get incident by ID
pub async fn get_incident_by_id(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Incident>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let incident = services::find_incident_by_id(&mut conn, id).await?;
    let incident = incident.ok_or_else(|| Error::NotFound(format!("Incident with id {id}")))?;
    Ok(Json(ApiResponse::success(incident)))
}

/// Update incident (requires moderator or higher, or be the creator)
pub async fn update_incident(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateIncidentRequest>,
) -> Result<Json<ApiResponse<Incident>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Get the incident first to check ownership
    let current_incident = services::find_incident_by_id(&mut conn, id).await?;
    let current_incident =
        current_incident.ok_or_else(|| Error::NotFound(format!("Incident with id {id}")))?;

    // Check if user can update this incident
    let can_update = auth_user
        .role
        .has_role_or_higher(crate::rbac::models::UserRole::Moderator)
        || current_incident.created_by == Some(auth_user.id);

    if !can_update {
        return Err(Error::Forbidden("Cannot update this incident".to_string()));
    }

    let incident = services::update_incident(&mut conn, id, request).await?;

    Ok(Json(ApiResponse::success(incident)))
}

/// Get incident timeline
pub async fn get_incident_timeline(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(params): Query<TimelineQueryParams>,
) -> Result<Json<ApiResponse<IncidentTimeline>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    let timeline =
        services::get_incident_timeline(&mut conn, id, params.limit, params.offset).await?;

    Ok(Json(ApiResponse::success(timeline)))
}

/// Get monitoring system statistics (requires moderator or higher)
pub async fn get_monitoring_stats(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<MonitoringStats>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    rbac_services::require_moderator_or_higher(&auth_user)?;

    let stats = services::get_monitoring_stats(&mut conn).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// Export metrics in Prometheus format
#[utoipa::path(
    get,
    path = "/api/v1/monitoring/metrics/prometheus",
    responses(
        (status = 200, description = "Prometheus metrics exported successfully", body = String),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn get_prometheus_metrics(
    State(app_state): State<AppState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> Result<String, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Get system statistics
    let stats = services::get_monitoring_stats(&mut conn).await?;

    // Get recent metrics from the database (last 24 hours)
    let recent_metrics = services::get_prometheus_metrics(&mut conn).await?;

    let mut prometheus_output = String::new();

    // Add system-level monitoring metrics
    prometheus_output.push_str(&format!(
        r#"# HELP monitoring_total_events Total number of events in the system
# TYPE monitoring_total_events counter
monitoring_total_events {}

# HELP monitoring_total_metrics Total number of metrics in the system
# TYPE monitoring_total_metrics counter
monitoring_total_metrics {}

# HELP monitoring_active_alerts Number of currently active alerts
# TYPE monitoring_active_alerts gauge
monitoring_active_alerts {}

# HELP monitoring_open_incidents Number of currently open incidents
# TYPE monitoring_open_incidents gauge
monitoring_open_incidents {}

# HELP monitoring_events_last_hour Number of events in the last hour
# TYPE monitoring_events_last_hour gauge
monitoring_events_last_hour {}

# HELP monitoring_metrics_last_hour Number of metrics in the last hour
# TYPE monitoring_metrics_last_hour gauge
monitoring_metrics_last_hour {}

"#,
        stats.total_events,
        stats.total_metrics,
        stats.active_alerts,
        stats.open_incidents,
        stats.events_last_hour,
        stats.metrics_last_hour
    ));

    // Add user-submitted metrics from the database
    prometheus_output.push_str(&recent_metrics);

    Ok(prometheus_output)
}
