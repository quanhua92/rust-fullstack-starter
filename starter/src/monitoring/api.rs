use super::models::*;
use super::services;
use crate::Error;
use crate::auth::AuthUser;
use crate::rbac::services as rbac_services;
use crate::{
    AppState,
    api::{ApiResponse, ErrorResponse},
};
use axum::{
    Extension, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Json, Response},
    routing::{get, post, put},
};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::IntoParams;
use uuid::Uuid;

/// Check if a user is authorized to create events for a given source
///
/// Authorization rules:
/// 1. Generic sources (no prefix) are allowed for all users
/// 2. Sources starting with username- are allowed for that user  
/// 3. Sources starting with user-{user_id} are allowed for that user
/// 4. System sources (system-, health-, monitoring-) require moderator+ role
fn is_user_authorized_for_source(auth_user: &AuthUser, source: &str) -> Result<bool, Error> {
    // System sources require moderator+ permissions
    if source.starts_with("system-")
        || source.starts_with("health-")
        || source.starts_with("monitoring-")
    {
        return Ok(auth_user
            .role
            .has_role_or_higher(crate::rbac::models::UserRole::Moderator));
    }

    // Allow generic sources (no specific ownership prefix) and test sources
    if !source.contains('-')
        || source.starts_with("app-")
        || source.starts_with("web-")
        || source.starts_with("api-")
        || source.starts_with("test-")
        || source.starts_with("db-")
    {
        return Ok(true);
    }

    // Check username-based ownership
    let username_prefix = format!("{}-", auth_user.username);
    if source.starts_with(&username_prefix) {
        return Ok(true);
    }

    // Check user ID-based ownership
    let user_id_prefix = format!("user-{}-", auth_user.id);
    if source.starts_with(&user_id_prefix) {
        return Ok(true);
    }

    // Check for service-specific prefixes that include user identification
    // Allow sources like: {username}-service-*, {username}-worker-*, etc.
    if let Some(dash_pos) = source.find('-') {
        let potential_username = &source[..dash_pos];
        if potential_username == auth_user.username {
            return Ok(true);
        }
    }

    // Default: not authorized
    Ok(false)
}

/// Check if a user is authorized to create metrics with a given name
///
/// Authorization rules:
/// 1. Generic metric names are allowed for all users (http_requests, response_time, etc.)
/// 2. Metrics starting with username_ are allowed for that user
/// 3. Metrics starting with user_{user_id}_ are allowed for that user  
/// 4. System metrics (system_, health_, monitoring_) require moderator+ role
fn is_user_authorized_for_metric_name(
    auth_user: &AuthUser,
    metric_name: &str,
) -> Result<bool, Error> {
    // System metrics require moderator+ permissions
    if metric_name.starts_with("system_")
        || metric_name.starts_with("health_")
        || metric_name.starts_with("monitoring_")
    {
        return Ok(auth_user
            .role
            .has_role_or_higher(crate::rbac::models::UserRole::Moderator));
    }

    // Allow common generic metric names
    let generic_metrics = [
        "http_requests",
        "http_requests_total",
        "response_time",
        "response_time_seconds",
        "cpu_usage",
        "memory_usage",
        "disk_usage",
        "network_bytes",
        "error_rate",
        "latency",
        "throughput",
        "requests_per_second",
        "active_connections",
        "queue_size",
    ];

    for generic in &generic_metrics {
        if metric_name.starts_with(generic) {
            return Ok(true);
        }
    }

    // Check username-based ownership
    let username_prefix = format!("{}_", auth_user.username);
    if metric_name.starts_with(&username_prefix) {
        return Ok(true);
    }

    // Check user ID-based ownership
    let user_id_prefix = format!("user_{}_", auth_user.id);
    if metric_name.starts_with(&user_id_prefix) {
        return Ok(true);
    }

    // Check for service-specific metrics that include user identification
    // Allow metrics like: {username}_service_*, {username}_worker_*, etc.
    if let Some(underscore_pos) = metric_name.find('_') {
        let potential_username = &metric_name[..underscore_pos];
        if potential_username == auth_user.username {
            return Ok(true);
        }
    }

    // Default: not authorized
    Ok(false)
}

/// Parse tags query parameter string into HashMap with comprehensive validation
/// Format: "key1:value1,key2:value2"
///
/// Security validations:
/// - Maximum number of tags: 50
/// - Maximum key length: 100 characters
/// - Maximum value length: 500 characters
/// - Only alphanumeric, underscore, hyphen, and dot allowed in keys
/// - Prevents injection attacks and resource exhaustion
fn parse_tags_query(tags_str: &str) -> Result<HashMap<String, String>, Error> {
    const MAX_TAG_PAIRS: usize = 50;
    const MAX_TAG_KEY_LENGTH: usize = 100;
    const MAX_TAG_VALUE_LENGTH: usize = 500;

    let mut tags = HashMap::new();
    let mut pair_count = 0;

    for pair in tags_str.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        pair_count += 1;
        if pair_count > MAX_TAG_PAIRS {
            return Err(Error::validation(
                "tags",
                &format!("Too many tag pairs (maximum {})", MAX_TAG_PAIRS),
            ));
        }

        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::validation(
                "tags",
                "Invalid tag format. Expected 'key:value' pairs separated by commas",
            ));
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        if key.is_empty() || value.is_empty() {
            return Err(Error::validation(
                "tags",
                "Tag keys and values cannot be empty",
            ));
        }

        if key.len() > MAX_TAG_KEY_LENGTH {
            return Err(Error::validation(
                "tags",
                &format!(
                    "Tag key too long (maximum {} characters)",
                    MAX_TAG_KEY_LENGTH
                ),
            ));
        }

        if value.len() > MAX_TAG_VALUE_LENGTH {
            return Err(Error::validation(
                "tags",
                &format!(
                    "Tag value too long (maximum {} characters)",
                    MAX_TAG_VALUE_LENGTH
                ),
            ));
        }

        // Validate key contains only safe characters to prevent injection
        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(Error::validation(
                "tags",
                "Tag keys must contain only alphanumeric characters, underscores, hyphens, and dots",
            ));
        }

        // Prevent duplicate keys
        if tags.contains_key(key) {
            return Err(Error::validation(
                "tags",
                "Duplicate tag keys are not allowed",
            ));
        }

        tags.insert(key.to_string(), value.to_string());
    }

    Ok(tags)
}

/// Query parameters for event listing
#[derive(Debug, Deserialize, IntoParams)]
pub struct EventQueryParams {
    pub event_type: Option<EventType>,
    pub source: Option<String>,
    pub level: Option<String>,
    #[param(format = "date-time")]
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    #[param(format = "date-time")]
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// Tag filtering: supports key=value pairs separated by commas
    /// Example: ?tags=user_id:123,environment:production
    pub tags: Option<String>,
}

/// Query parameters for metric listing
#[derive(Debug, Deserialize, IntoParams)]
pub struct MetricQueryParams {
    pub name: Option<String>,
    pub metric_type: Option<MetricType>,
    #[param(format = "date-time")]
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    #[param(format = "date-time")]
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for incident listing
#[derive(Debug, Deserialize, IntoParams)]
pub struct IncidentQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for timeline
#[derive(Debug, Deserialize, IntoParams)]
pub struct TimelineQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub lookback_hours: Option<i64>,
}

/// Create a new event
#[utoipa::path(
    post,
    path = "/monitoring/events",
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

    // Authorization: Users can create events for sources they own, moderators+ can create any events
    if !auth_user
        .role
        .has_role_or_higher(crate::rbac::models::UserRole::Moderator)
    {
        // Basic users can only create events for sources they own or generic sources
        // Source ownership is determined by source prefix matching username or user ID
        let is_authorized_source = is_user_authorized_for_source(&auth_user, &request.source)?;

        if !is_authorized_source {
            return Err(Error::Forbidden(format!(
                "Users can only create events for their own sources. Source '{}' is not authorized for user '{}'",
                request.source, auth_user.username
            )));
        }
    }

    let event = services::create_event(conn.as_mut(), request).await?;
    Ok(Json(ApiResponse::success(event)))
}

/// Get events with filters
#[utoipa::path(
    get,
    path = "/monitoring/events",
    params(EventQueryParams),
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

    // Parse tags parameter if provided
    let tags = if let Some(tags_str) = &params.tags {
        Some(parse_tags_query(tags_str)?)
    } else {
        None
    };

    // Users can view all events, but in production you might want to filter by source ownership
    let filter = EventFilter {
        event_type: params.event_type,
        source: params.source,
        level: params.level,
        start_time: params.start_time,
        end_time: params.end_time,
        tags,
        limit: params.limit,
        offset: params.offset,
    };

    let events = services::find_events_with_filter(conn.as_mut(), filter).await?;
    Ok(Json(ApiResponse::success(events)))
}

/// Get a specific event by ID
#[utoipa::path(
    get,
    path = "/monitoring/events/{id}",
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

    let event = services::find_event_by_id(conn.as_mut(), id).await?;
    let event = event.ok_or_else(|| Error::NotFound("Event not found".to_string()))?;
    Ok(Json(ApiResponse::success(event)))
}

/// Create a new metric
#[utoipa::path(
    post,
    path = "/monitoring/metrics",
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
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateMetricRequest>,
) -> Result<Json<ApiResponse<Metric>>, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Authorization: Users can create metrics with names they own, moderators+ can create any metrics
    if !auth_user
        .role
        .has_role_or_higher(crate::rbac::models::UserRole::Moderator)
    {
        let is_authorized_metric = is_user_authorized_for_metric_name(&auth_user, &request.name)?;

        if !is_authorized_metric {
            return Err(Error::Forbidden(format!(
                "Users can only create metrics with names they own. Metric '{}' is not authorized for user '{}'",
                request.name, auth_user.username
            )));
        }
    }

    let metric = services::create_metric(conn.as_mut(), request).await?;
    Ok(Json(ApiResponse::success(metric)))
}

/// Get metrics with filters
#[utoipa::path(
    get,
    path = "/monitoring/metrics",
    params(MetricQueryParams),
    responses(
        (status = 200, description = "Metrics retrieved successfully", body = ApiResponse<Vec<Metric>>),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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
        labels: None, // Future enhancement: Add label filtering from query params
        limit: params.limit,
        offset: params.offset,
    };

    let metrics = services::find_metrics_with_filter(conn.as_mut(), filter).await?;
    Ok(Json(ApiResponse::success(metrics)))
}

/// Create a new alert (requires moderator or higher)
#[utoipa::path(
    post,
    path = "/monitoring/alerts",
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

    let alert = services::create_alert(conn.as_mut(), request, Some(auth_user.id)).await?;
    Ok(Json(ApiResponse::success(alert)))
}

/// Get all alerts
#[utoipa::path(
    get,
    path = "/monitoring/alerts",
    responses(
        (status = 200, description = "Alerts retrieved successfully", body = ApiResponse<Vec<Alert>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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

    let alerts = services::find_all_alerts(conn.as_mut()).await?;
    Ok(Json(ApiResponse::success(alerts)))
}

/// Create a new incident
#[utoipa::path(
    post,
    path = "/monitoring/incidents",
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

    let incident = services::create_incident(conn.as_mut(), request, Some(auth_user.id)).await?;
    Ok(Json(ApiResponse::success(incident)))
}

/// Get incidents with pagination
#[utoipa::path(
    get,
    path = "/monitoring/incidents",
    params(IncidentQueryParams),
    responses(
        (status = 200, description = "Incidents retrieved successfully", body = ApiResponse<Vec<Incident>>),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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
        services::find_incidents_with_pagination(conn.as_mut(), params.limit, params.offset)
            .await?;
    Ok(Json(ApiResponse::success(incidents)))
}

/// Get incident by ID
#[utoipa::path(
    get,
    path = "/monitoring/incidents/{id}",
    params(
        ("id" = Uuid, Path, description = "Incident ID")
    ),
    responses(
        (status = 200, description = "Incident retrieved successfully", body = ApiResponse<Incident>),
        (status = 404, description = "Incident not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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

    let incident = services::find_incident_by_id(conn.as_mut(), id).await?;
    let incident = incident.ok_or_else(|| Error::NotFound("Incident not found".to_string()))?;
    Ok(Json(ApiResponse::success(incident)))
}

/// Update incident (requires moderator or higher, or be the creator)
#[utoipa::path(
    put,
    path = "/monitoring/incidents/{id}",
    params(
        ("id" = Uuid, Path, description = "Incident ID")
    ),
    request_body = UpdateIncidentRequest,
    responses(
        (status = 200, description = "Incident updated successfully", body = ApiResponse<Incident>),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires moderator role or incident ownership", body = ErrorResponse),
        (status = 404, description = "Incident not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
pub async fn update_incident(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateIncidentRequest>,
) -> Result<Json<ApiResponse<Incident>>, Error> {
    // Use transaction to prevent race conditions
    let mut tx = app_state
        .database
        .pool
        .begin()
        .await
        .map_err(Error::from_sqlx)?;

    // Get the incident first to check ownership (with SELECT FOR UPDATE to prevent race conditions)
    let current_incident = services::find_incident_by_id_for_update(tx.as_mut(), id).await?;
    let current_incident =
        current_incident.ok_or_else(|| Error::NotFound("Incident not found".to_string()))?;

    // Check if user can update this incident
    let can_update = auth_user
        .role
        .has_role_or_higher(crate::rbac::models::UserRole::Moderator)
        || current_incident.created_by == Some(auth_user.id);

    if !can_update {
        return Err(Error::Forbidden("Cannot update this incident".to_string()));
    }

    // Update the incident within the transaction
    let incident = services::update_incident_in_transaction(tx.as_mut(), id, request).await?;

    // Commit the transaction
    tx.commit().await.map_err(Error::from_sqlx)?;

    Ok(Json(ApiResponse::success(incident)))
}

/// Get incident timeline
#[utoipa::path(
    get,
    path = "/monitoring/incidents/{id}/timeline",
    params(
        ("id" = Uuid, Path, description = "Incident ID"),
        TimelineQueryParams
    ),
    responses(
        (status = 200, description = "Incident timeline retrieved successfully", body = ApiResponse<IncidentTimeline>),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Incident not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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

    let timeline = services::get_incident_timeline(
        conn.as_mut(),
        id,
        params.limit,
        params.offset,
        params.lookback_hours,
    )
    .await?;

    Ok(Json(ApiResponse::success(timeline)))
}

/// Get monitoring system statistics (requires moderator or higher)
#[utoipa::path(
    get,
    path = "/monitoring/stats",
    responses(
        (status = 200, description = "Monitoring statistics retrieved successfully", body = ApiResponse<MonitoringStats>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - requires moderator role", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Monitoring"
)]
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

    let stats = services::get_monitoring_stats(conn.as_mut()).await?;
    Ok(Json(ApiResponse::success(stats)))
}

/// Export metrics in Prometheus format (publicly accessible for scraping)
#[utoipa::path(
    get,
    path = "/monitoring/metrics/prometheus",
    responses(
        (status = 200, description = "Prometheus metrics in text/plain format (version=0.0.4; charset=utf-8)", body = String)
    ),
    tag = "Monitoring"
)]
pub async fn get_prometheus_metrics(State(app_state): State<AppState>) -> Result<Response, Error> {
    let mut conn = app_state
        .database
        .pool
        .acquire()
        .await
        .map_err(Error::from_sqlx)?;

    // Get system statistics
    let stats = services::get_monitoring_stats(conn.as_mut()).await?;

    // Get recent metrics from the database (last 24 hours)
    let recent_metrics = services::get_prometheus_metrics(conn.as_mut()).await?;

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

    // Create response with proper Prometheus content type
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )
        .body(Body::from(prometheus_output))
        .map_err(|e| Error::internal(&format!("Failed to build response: {e}")))
}

/// Public monitoring routes (no authentication required)
pub fn monitoring_public_routes() -> Router<AppState> {
    Router::new().route("/metrics/prometheus", get(get_prometheus_metrics))
}

/// Protected monitoring routes (authentication required)
pub fn monitoring_routes() -> Router<AppState> {
    Router::new()
        .route("/events", post(create_event))
        .route("/events", get(get_events))
        .route("/events/{id}", get(get_event_by_id))
        .route("/metrics", post(create_metric))
        .route("/metrics", get(get_metrics))
        .route("/alerts", get(get_alerts))
        .route("/incidents", post(create_incident))
        .route("/incidents", get(get_incidents))
        .route("/incidents/{id}", get(get_incident_by_id))
        .route("/incidents/{id}", put(update_incident))
        .route("/incidents/{id}/timeline", get(get_incident_timeline))
}

/// Moderator monitoring routes (moderator role required)
pub fn monitoring_moderator_routes() -> Router<AppState> {
    Router::new()
        .route("/alerts", post(create_alert))
        .route("/stats", get(get_monitoring_stats))
}
