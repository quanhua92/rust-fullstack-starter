# Monitoring and Observability System

## Overview

The monitoring and observability system provides comprehensive infrastructure for tracking application health, collecting metrics, managing alerts, and performing root cause analysis. Built as a foundational system, it integrates seamlessly with existing authentication, RBAC, and task processing systems.

## Architecture

### Core Components

1. **Event Ingestion**: Unified collection of logs, metrics, traces, and alerts
2. **Metric Storage**: Time-series data with labels for Prometheus compatibility
3. **Alert Management**: Rule-based monitoring with threshold detection
4. **Incident Management**: End-to-end incident lifecycle with timeline reconstruction
5. **Root Cause Analysis**: Event correlation and timeline-based investigation

### Database Design

The system uses TEXT fields with CHECK constraints for enum storage, following the project's pattern for simple database management:

```sql
-- Events table for unified observability data
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL 
        CONSTRAINT valid_event_type CHECK (event_type IN ('log', 'metric', 'trace', 'alert')),
    source TEXT NOT NULL,
    message TEXT,
    level TEXT,
    tags JSONB NOT NULL DEFAULT '{}',
    payload JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## API Endpoints

### Event Management

#### Create Event
```http
POST /api/v1/monitoring/events
Content-Type: application/json
Authorization: Bearer <token>

{
  "event_type": "log",
  "source": "web-server",
  "message": "Request processed successfully",
  "level": "info",
  "tags": {
    "service": "api",
    "environment": "production"
  },
  "payload": {
    "request_id": "req-123",
    "duration_ms": 245
  }
}
```

**Event Type Validation:** The API accepts event types as strings and validates them against the EventType enum (`log`, `metric`, `trace`, `alert`). Invalid event types return `400 Bad Request` with a descriptive validation error.

#### Query Events
```http
GET /api/v1/monitoring/events?event_type=log&source=web-server&limit=50
Authorization: Bearer <token>
```

**Query Parameters:**
- `event_type`: Filter by event type (log, metric, trace, alert)
- `source`: Filter by event source
- `level`: Filter by log level
- `start_time`: Filter events after this timestamp (ISO 8601)
- `end_time`: Filter events before this timestamp (ISO 8601)
- `limit`: Maximum number of results (default: 100)
- `offset`: Number of results to skip (default: 0)

### Metric Management

#### Submit Metrics
```http
POST /api/v1/monitoring/metrics
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "http_requests_total",
  "metric_type": "counter",
  "value": 1,
  "labels": {
    "method": "GET",
    "status": "200",
    "endpoint": "/api/users"
  }
}
```

#### Query Metrics
```http
GET /api/v1/monitoring/metrics?name=cpu_usage&start_time=2024-01-01T00:00:00Z
Authorization: Bearer <token>
```

### Alert Management (Moderator+ Required)

#### Create Alert Rule
```http
POST /api/v1/monitoring/alerts
Content-Type: application/json
Authorization: Bearer <moderator_token>

{
  "name": "High CPU Usage",
  "description": "CPU usage above 80% for more than 5 minutes",
  "query": "cpu_usage > 80",
  "threshold_value": 80.0
}
```

#### List Alerts
```http
GET /api/v1/monitoring/alerts
Authorization: Bearer <token>
```

### Incident Management

#### Create Incident
```http
POST /api/v1/monitoring/incidents
Content-Type: application/json
Authorization: Bearer <token>

{
  "title": "API Response Time Degradation",
  "description": "Average response time increased to 2+ seconds",
  "severity": "high",
  "assigned_to": "user-uuid-here"
}
```

#### Update Incident
```http
PUT /api/v1/monitoring/incidents/{incident_id}
Content-Type: application/json
Authorization: Bearer <token>

{
  "status": "investigating",
  "root_cause": "Database connection pool exhaustion",
  "severity": "critical"
}
```

#### Get Incident Timeline
```http
GET /api/v1/monitoring/incidents/{incident_id}/timeline?limit=100
Authorization: Bearer <token>
```

### System Statistics (Moderator+ Required)

```http
GET /api/v1/monitoring/stats
Authorization: Bearer <moderator_token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_events": 15420,
    "total_metrics": 8934,
    "active_alerts": 3,
    "open_incidents": 1,
    "events_last_hour": 245,
    "metrics_last_hour": 189
  }
}
```

### Prometheus Integration

#### Metrics Exposition
```http
GET /api/v1/monitoring/metrics/prometheus
Authorization: Bearer <token>
```

Returns metrics in Prometheus exposition format:
```
# HELP monitoring_total_events Total number of events in the system
# TYPE monitoring_total_events counter
monitoring_total_events 15420

# HELP monitoring_active_alerts Number of currently active alerts
# TYPE monitoring_active_alerts gauge
monitoring_active_alerts 3
```

## RBAC Integration

The monitoring system integrates with the existing role-based access control:

### Access Levels

- **User**: 
  - Create events and metrics
  - View own incidents
  - Create incidents
  - View general statistics

- **Moderator**:
  - All user permissions
  - Create and manage alert rules
  - View all incidents
  - Access system statistics
  - Force password resets for monitoring accounts

- **Admin**:
  - All moderator permissions
  - Full system configuration
  - User role management
  - Advanced monitoring features

## Internal Service Integration

The monitoring system provides both API endpoints and direct service functions for internal use:

### Direct Service Calls

```rust
use starter::monitoring::services;

// Create event within a database transaction
let event = services::create_event(conn, CreateEventRequest {
    event_type: EventType::Log,
    source: "internal-service".to_string(),
    message: Some("Operation completed".to_string()),
    level: Some("info".to_string()),
    tags: HashMap::new(),
    payload: HashMap::new(),
    timestamp: None,
}).await?;

// Create incident from within another service
let incident = services::create_incident(conn, CreateIncidentRequest {
    title: "Automated incident".to_string(),
    description: Some("Detected by internal monitoring".to_string()),
    severity: IncidentSeverity::Medium,
    assigned_to: None,
}, Some(user_id)).await?;
```

## Task Handler Integration

### Background Processing

The monitoring system includes task handlers for automated processing:

#### Event Processing
```rust
// Register monitoring task types
monitoring::handlers::register_monitoring_handlers(&task_processor).await;

// Create background task for event analysis
let task = CreateTaskRequest::new(
    "monitoring_event_processing",
    json!({
        "event_type": "log",
        "source": "api-gateway",
        "message": "High error rate detected"
    })
);
```

#### Available Task Types

1. **`monitoring_event_processing`**: Process and enrich incoming events
2. **`monitoring_alert_evaluation`**: Evaluate alert rules against data
3. **`monitoring_incident_analysis`**: Perform root cause analysis
4. **`monitoring_data_retention`**: Clean up old monitoring data

## Usage Patterns

### Application Monitoring

```rust
// Log application events
let event_request = CreateEventRequest {
    event_type: "log".to_string(), // String validation in service layer
    source: "user-service".to_string(),
    message: Some(format!("User {} logged in", user.username)),
    level: Some("info".to_string()),
    tags: {
        let mut tags = HashMap::new();
        tags.insert("user_id".to_string(), json!(user.id));
        tags.insert("action".to_string(), json!("login"));
        tags
    },
    payload: HashMap::new(),
    timestamp: None,
};

monitoring::services::create_event(&mut conn, event_request).await?;
```

### Performance Metrics

```rust
// Track performance metrics
let metric_request = CreateMetricRequest {
    name: "api_request_duration".to_string(),
    metric_type: MetricType::Histogram,
    value: request_duration_ms,
    labels: {
        let mut labels = HashMap::new();
        labels.insert("endpoint".to_string(), "/api/users".to_string());
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());
        labels
    },
    timestamp: None,
};

monitoring::services::create_metric(&mut conn, metric_request).await?;
```

### Error Tracking

```rust
// Track errors with context
let error_event = CreateEventRequest {
    event_type: "alert".to_string(), // String validation in service layer
    source: "database-service".to_string(),
    message: Some("Connection pool exhausted".to_string()),
    level: Some("error".to_string()),
    tags: {
        let mut tags = HashMap::new();
        tags.insert("severity".to_string(), json!("critical"));
        tags.insert("component".to_string(), json!("database"));
        tags
    },
    payload: {
        let mut payload = HashMap::new();
        payload.insert("pool_size".to_string(), json!(10));
        payload.insert("active_connections".to_string(), json!(10));
        payload.insert("error_code".to_string(), json!("POOL_EXHAUSTED"));
        payload
    },
    timestamp: None,
};

monitoring::services::create_event(&mut conn, error_event).await?;
```

## Timeline Reconstruction

The system automatically correlates events around incident timeframes:

```rust
// Get incident timeline
let timeline = monitoring::services::get_incident_timeline(
    &mut conn,
    incident_id,
    Some(100), // limit
    Some(0),   // offset
).await?;

// Timeline includes events from 1 hour before incident to resolution
for entry in timeline.entries {
    println!("{}: {} - {}", 
        entry.timestamp, 
        entry.source, 
        entry.message
    );
}
```

## Testing

### Integration Tests

The monitoring system includes comprehensive integration tests:

```bash
# Run monitoring-specific tests
cargo nextest run monitoring

# Run all integration tests
cargo nextest run
```

### Test Coverage

- Event creation and querying
- Metric submission and filtering
- Alert management with RBAC validation
- Incident lifecycle management
- Timeline reconstruction
- Statistics and Prometheus endpoints
- Error handling and validation

## Configuration

### Environment Variables

```bash
# Database configuration (inherited from main app)
DATABASE_URL=postgres://user:password@localhost/monitoring_db

# Optional: Custom retention periods
MONITORING_EVENT_RETENTION_DAYS=30
MONITORING_METRIC_RETENTION_DAYS=90
```

### Database Migration

```bash
# Run monitoring system migration
sqlx migrate run --source starter/migrations
```

## Future Extensions

### Prometheus Integration
- Service discovery configuration
- Federation support for multi-cluster setups
- Custom metric exposition

### Grafana Integration
- Dashboard provisioning
- Data source configuration
- Alert manager integration

### OpenTelemetry Support
- Distributed tracing collection
- Trace correlation with logs and metrics
- Service map generation

### Advanced Analytics
- Machine learning for anomaly detection
- Predictive alerting based on historical patterns
- Automated root cause suggestion algorithms

## Best Practices

### Event Design
- Use consistent source naming (e.g., `service-name`)
- Include correlation IDs in tags for tracing
- Keep payload data structured and searchable
- Use appropriate log levels (error, warn, info, debug)

### Metric Design
- Follow Prometheus naming conventions
- Use labels for dimensional data
- Avoid high cardinality labels
- Prefer counters and histograms over gauges for rates

### Alert Configuration
- Set meaningful thresholds based on SLA requirements
- Include context in alert descriptions
- Use appropriate severity levels
- Test alert rules with historical data

### Incident Management
- Create incidents proactively for customer-facing issues
- Update status regularly during investigation
- Document root cause for future reference
- Assign ownership for accountability

## Security Considerations

### Access Control
- Event data may contain sensitive information
- Use RBAC to restrict access to monitoring data
- Audit access to sensitive monitoring endpoints
- Sanitize log messages to prevent data leakage

### Data Retention
- Implement appropriate retention policies
- Consider regulatory requirements for log retention
- Regularly clean up old monitoring data
- Archive important incident data for compliance

This monitoring and observability system provides a solid foundation for production monitoring while maintaining the simplicity and extensibility appropriate for a starter project.