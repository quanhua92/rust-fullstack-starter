use crate::monitoring::models::*;
use crate::{
    error::Error,
    types::{DbConn, Result},
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Event management functions

pub async fn create_event(conn: &mut DbConn, request: CreateEventRequest) -> Result<Event> {
    let id = Uuid::new_v4();
    let timestamp = request.timestamp.unwrap_or_else(Utc::now);
    let tags = json!(request.tags);
    let payload = json!(request.payload);

    // Validate event_type string and convert to enum
    let event_type = EventType::from_str(&request.event_type)
        .map_err(|_| Error::validation("event_type", "Invalid event type"))?;

    let event = sqlx::query!(
        r#"
        INSERT INTO events (id, event_type, source, message, level, tags, payload, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, event_type, source, message, level, tags, payload, timestamp, created_at
        "#,
        id,
        event_type.to_string(),
        request.source,
        request.message,
        request.level,
        tags,
        payload,
        timestamp
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let event = Event {
        id: event.id,
        event_type: EventType::from_str(&event.event_type).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid event_type in database: {}",
                event.event_type
            ))
        })?,
        source: event.source,
        message: event.message,
        level: event.level,
        tags: event.tags,
        payload: event.payload,
        timestamp: event.timestamp,
        created_at: event.created_at,
    };

    Ok(event)
}

pub async fn find_events_with_filter(conn: &mut DbConn, filter: EventFilter) -> Result<Vec<Event>> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, event_type, source, message, level, tags, payload, timestamp, created_at FROM events WHERE 1=1",
    );

    if let Some(event_type) = &filter.event_type {
        query_builder.push(" AND event_type = ");
        query_builder.push_bind(event_type.to_string());
    }

    if let Some(source) = &filter.source {
        query_builder.push(" AND source = ");
        query_builder.push_bind(source);
    }

    if let Some(level) = &filter.level {
        query_builder.push(" AND level = ");
        query_builder.push_bind(level);
    }

    if let Some(start_time) = &filter.start_time {
        query_builder.push(" AND timestamp >= ");
        query_builder.push_bind(start_time);
    }

    if let Some(end_time) = &filter.end_time {
        query_builder.push(" AND timestamp <= ");
        query_builder.push_bind(end_time);
    }

    // Tag filtering using JSONB containment operator
    if let Some(tags) = &filter.tags {
        for (key, value) in tags {
            query_builder.push(" AND tags @> ");
            // Create JSONB object for containment check
            let tag_json = serde_json::json!({ key: value });
            query_builder.push_bind(tag_json);
        }
    }

    query_builder.push(" ORDER BY timestamp DESC");

    if let Some(limit) = filter.limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
    }

    if let Some(offset) = filter.offset {
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);
    }

    let events = query_builder
        .build_query_as::<Event>()
        .fetch_all(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(events)
}

pub async fn find_event_by_id(conn: &mut DbConn, id: Uuid) -> Result<Option<Event>> {
    let event = sqlx::query_as!(
        Event,
        r#"
        SELECT id, event_type, source, message, level, tags, payload, timestamp, created_at
        FROM events
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(event)
}

// Metric management functions

pub async fn create_metric(conn: &mut DbConn, request: CreateMetricRequest) -> Result<Metric> {
    let id = Uuid::new_v4();
    let timestamp = request.timestamp.unwrap_or_else(Utc::now);
    let labels = json!(request.labels);

    let metric = sqlx::query!(
        r#"
        INSERT INTO metrics (id, name, metric_type, value, labels, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, metric_type, value, labels, timestamp, created_at
        "#,
        id,
        request.name,
        request.metric_type.to_string(),
        request.value,
        labels,
        timestamp
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let metric = Metric {
        id: metric.id,
        name: metric.name,
        metric_type: MetricType::from_str(&metric.metric_type).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid metric_type in database: {}",
                metric.metric_type
            ))
        })?,
        value: metric.value,
        labels: metric.labels,
        timestamp: metric.timestamp,
        created_at: metric.created_at,
    };

    Ok(metric)
}

pub async fn find_metrics_with_filter(
    conn: &mut DbConn,
    filter: MetricFilter,
) -> Result<Vec<Metric>> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, name, metric_type, value, labels, timestamp, created_at FROM metrics WHERE 1=1",
    );

    if let Some(name) = &filter.name {
        query_builder.push(" AND name = ");
        query_builder.push_bind(name);
    }

    if let Some(metric_type) = &filter.metric_type {
        query_builder.push(" AND metric_type = ");
        query_builder.push_bind(metric_type.to_string());
    }

    if let Some(start_time) = &filter.start_time {
        query_builder.push(" AND timestamp >= ");
        query_builder.push_bind(start_time);
    }

    if let Some(end_time) = &filter.end_time {
        query_builder.push(" AND timestamp <= ");
        query_builder.push_bind(end_time);
    }

    query_builder.push(" ORDER BY timestamp DESC");

    if let Some(limit) = filter.limit {
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
    }

    if let Some(offset) = filter.offset {
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);
    }

    let metrics = query_builder
        .build_query_as::<Metric>()
        .fetch_all(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?;

    Ok(metrics)
}

// Alert management functions

pub async fn create_alert(
    conn: &mut DbConn,
    request: CreateAlertRequest,
    created_by: Option<Uuid>,
) -> Result<Alert> {
    let id = Uuid::new_v4();

    let alert = sqlx::query!(
        r#"
        INSERT INTO alerts (id, name, description, query, threshold_value, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, description, query, threshold_value, 
                 status, triggered_at, resolved_at, 
                 created_by, created_at, updated_at
        "#,
        id,
        request.name,
        request.description,
        request.query,
        request.threshold_value,
        created_by
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let alert = Alert {
        id: alert.id,
        name: alert.name,
        description: alert.description,
        query: alert.query,
        threshold_value: alert.threshold_value,
        status: AlertStatus::from_str(&alert.status).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid alert status in database: {}",
                alert.status
            ))
        })?,
        triggered_at: alert.triggered_at,
        resolved_at: alert.resolved_at,
        created_by: alert.created_by,
        created_at: alert.created_at,
        updated_at: alert.updated_at,
    };

    Ok(alert)
}

pub async fn find_all_alerts(conn: &mut DbConn) -> Result<Vec<Alert>> {
    let alerts = sqlx::query_as!(
        Alert,
        r#"
        SELECT id, name, description, query, threshold_value, 
               status, triggered_at, resolved_at, 
               created_by, created_at, updated_at
        FROM alerts
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(alerts)
}

// Incident management functions

pub async fn create_incident(
    conn: &mut DbConn,
    request: CreateIncidentRequest,
    created_by: Option<Uuid>,
) -> Result<Incident> {
    let id = Uuid::new_v4();

    let incident = sqlx::query!(
        r#"
        INSERT INTO incidents (id, title, description, severity, created_by, assigned_to)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, title, description, 
                 severity, status, 
                 started_at, resolved_at, root_cause, 
                 created_by, assigned_to, created_at, updated_at
        "#,
        id,
        request.title,
        request.description,
        request.severity.to_string(),
        created_by,
        request.assigned_to
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let incident = Incident {
        id: incident.id,
        title: incident.title,
        description: incident.description,
        severity: IncidentSeverity::from_str(&incident.severity).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid incident severity in database: {}",
                incident.severity
            ))
        })?,
        status: IncidentStatus::from_str(&incident.status).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid incident status in database: {}",
                incident.status
            ))
        })?,
        started_at: incident.started_at,
        resolved_at: incident.resolved_at,
        root_cause: incident.root_cause,
        created_by: incident.created_by,
        assigned_to: incident.assigned_to,
        created_at: incident.created_at,
        updated_at: incident.updated_at,
    };

    Ok(incident)
}

pub async fn find_incidents_with_pagination(
    conn: &mut DbConn,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Incident>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let incidents = sqlx::query_as!(
        Incident,
        r#"
        SELECT id, title, description, 
               severity, status, 
               started_at, resolved_at, root_cause, 
               created_by, assigned_to, created_at, updated_at
        FROM incidents
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(incidents)
}

pub async fn find_incident_by_id(conn: &mut DbConn, id: Uuid) -> Result<Option<Incident>> {
    let incident = sqlx::query_as!(
        Incident,
        r#"
        SELECT id, title, description, 
               severity, status, 
               started_at, resolved_at, root_cause, 
               created_by, assigned_to, created_at, updated_at
        FROM incidents
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    Ok(incident)
}

pub async fn update_incident(
    conn: &mut DbConn,
    id: Uuid,
    request: UpdateIncidentRequest,
) -> Result<Incident> {
    let updated_incident = sqlx::query!(
        r#"
        UPDATE incidents 
        SET title = COALESCE($2, title),
            description = COALESCE($3, description),
            severity = COALESCE($4, severity),
            status = COALESCE($5, status),
            root_cause = COALESCE($6, root_cause),
            assigned_to = COALESCE($7, assigned_to),
            resolved_at = CASE 
                WHEN $5 = 'resolved' AND resolved_at IS NULL 
                THEN NOW() 
                ELSE resolved_at 
            END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, title, description, 
                 severity, status, 
                 started_at, resolved_at, root_cause, 
                 created_by, assigned_to, created_at, updated_at
        "#,
        id,
        request.title,
        request.description,
        request.severity.map(|s| s.to_string()),
        request.status.map(|s| s.to_string()),
        request.root_cause,
        request.assigned_to
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let incident = Incident {
        id: updated_incident.id,
        title: updated_incident.title,
        description: updated_incident.description,
        severity: IncidentSeverity::from_str(&updated_incident.severity).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid incident severity in database: {}",
                updated_incident.severity
            ))
        })?,
        status: IncidentStatus::from_str(&updated_incident.status).map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid incident status in database: {}",
                updated_incident.status
            ))
        })?,
        started_at: updated_incident.started_at,
        resolved_at: updated_incident.resolved_at,
        root_cause: updated_incident.root_cause,
        created_by: updated_incident.created_by,
        assigned_to: updated_incident.assigned_to,
        created_at: updated_incident.created_at,
        updated_at: updated_incident.updated_at,
    };

    Ok(incident)
}

pub async fn get_incident_timeline(
    conn: &mut DbConn,
    incident_id: Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
    lookback_hours: Option<i64>,
) -> Result<IncidentTimeline> {
    // Get incident details first
    let incident = find_incident_by_id(conn, incident_id)
        .await?
        .ok_or_else(|| Error::NotFound(format!("Incident with id {incident_id}")))?;

    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    let lookback_hours = lookback_hours.unwrap_or(1); // Default 1 hour lookback

    // Get events around the incident timeframe with configurable lookback window
    let start_time = incident.started_at - chrono::Duration::hours(lookback_hours);
    let end_time = incident.resolved_at.unwrap_or_else(Utc::now);

    let events = sqlx::query!(
        r#"
        SELECT id, timestamp, event_type, source, 
               COALESCE(message, '') as message, level, tags
        FROM events
        WHERE timestamp BETWEEN $1 AND $2
        ORDER BY timestamp ASC
        LIMIT $3 OFFSET $4
        "#,
        start_time,
        end_time,
        limit,
        offset
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let total_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM events WHERE timestamp BETWEEN $1 AND $2",
        start_time,
        end_time
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let mut entries: Vec<TimelineEntry> = Vec::new();
    for row in events {
        let tags: HashMap<String, serde_json::Value> =
            serde_json::from_value(row.tags).unwrap_or_default();

        // Parse event_type string back to enum for API response
        let event_type = row.event_type.parse().map_err(|_| {
            Error::InvalidInput(format!(
                "Invalid event_type in database: {}",
                row.event_type
            ))
        })?;

        entries.push(TimelineEntry {
            id: row.id,
            timestamp: row.timestamp,
            event_type,
            source: row.source,
            message: row.message.unwrap_or_default(),
            level: row.level,
            tags,
        });
    }

    Ok(IncidentTimeline {
        incident_id,
        start_time: incident.started_at,
        end_time: incident.resolved_at,
        entries,
        total_count,
    })
}

// Statistics and monitoring functions

pub async fn get_monitoring_stats(conn: &mut DbConn) -> Result<MonitoringStats> {
    let one_hour_ago = Utc::now() - chrono::Duration::hours(1);

    let total_events = sqlx::query_scalar!("SELECT COUNT(*) FROM events")
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0);

    let total_metrics = sqlx::query_scalar!("SELECT COUNT(*) FROM metrics")
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0);

    let active_alerts = sqlx::query_scalar!("SELECT COUNT(*) FROM alerts WHERE status = 'active'")
        .fetch_one(&mut **conn)
        .await
        .map_err(Error::from_sqlx)?
        .unwrap_or(0);

    let open_incidents = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM incidents WHERE status IN ('open', 'investigating')"
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let events_last_hour = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM events WHERE created_at >= $1",
        one_hour_ago
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    let metrics_last_hour = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM metrics WHERE created_at >= $1",
        one_hour_ago
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?
    .unwrap_or(0);

    Ok(MonitoringStats {
        total_events,
        total_metrics,
        active_alerts,
        open_incidents,
        events_last_hour,
        metrics_last_hour,
    })
}

pub async fn get_prometheus_metrics(conn: &mut DbConn) -> Result<String> {
    // Get metrics from the last 24 hours to keep output manageable
    let last_24h = Utc::now() - chrono::Duration::hours(24);

    let metrics = sqlx::query!(
        r#"
        SELECT name, metric_type, value, labels, timestamp
        FROM metrics 
        WHERE timestamp >= $1
        ORDER BY name, timestamp DESC
        "#,
        last_24h
    )
    .fetch_all(&mut **conn)
    .await
    .map_err(Error::from_sqlx)?;

    let mut output = String::new();
    let mut current_metric = String::new();

    for metric in metrics {
        // Parse labels from JSONB
        let labels: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_value(metric.labels).unwrap_or_default();

        // Format labels for Prometheus with proper type handling and escaping
        let labels_str = if labels.is_empty() {
            String::new()
        } else {
            let label_pairs: Vec<String> = labels
                .iter()
                .filter_map(|(k, v)| {
                    let v_str = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        _ => return None, // Skip complex types like arrays/objects
                    };
                    // Properly escape backslashes and double quotes for Prometheus format
                    let escaped_v = v_str.replace('\\', r"\\").replace('"', r#"\""#);
                    Some(format!("{k}=\"{escaped_v}\""))
                })
                .collect();
            format!("{{{}}}", label_pairs.join(","))
        };

        // Add metric type header if this is a new metric
        if current_metric != metric.name {
            if !current_metric.is_empty() {
                output.push('\n');
            }

            current_metric = metric.name.clone();

            // Add HELP and TYPE comments
            output.push_str(&format!(
                "# HELP {} User-submitted metric\n# TYPE {} {}\n",
                metric.name, metric.name, metric.metric_type
            ));
        }

        // Add the metric value line
        output.push_str(&format!(
            "{}{} {} {}\n",
            metric.name,
            labels_str,
            metric.value,
            metric.timestamp.timestamp_millis()
        ));
    }

    Ok(output)
}
