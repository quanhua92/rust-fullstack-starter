# Monitoring and Observability

The Rust Fullstack Starter includes a comprehensive monitoring and observability system designed to provide visibility into application health, performance metrics, and incident management.

## Quick Start

### Enable Monitoring

The monitoring system is included by default. To start collecting data:

1. **Start the application**: The monitoring endpoints are automatically available
2. **Create your first event**:

```bash
curl -X POST http://localhost:3000/api/v1/monitoring/events \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "event_type": "log",
    "source": "my-service", 
    "message": "Application started",
    "level": "info",
    "tags": {
      "environment": "production",
      "version": "1.0.0"
    }
  }'
```

3. **Submit metrics**:

```bash
curl -X POST http://localhost:3000/api/v1/monitoring/metrics \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "app_startup_time", 
    "metric_type": "gauge",
    "value": 1.5,
    "labels": {
      "service": "api",
      "environment": "production"
    }
  }'
```

### View Monitoring Data

- **Web Dashboard**: http://localhost:3000/admin/monitoring (Full-featured interface)
- **Browse events**: `GET /api/v1/monitoring/events`
- **Query metrics**: `GET /api/v1/monitoring/metrics`
- **Check system stats**: `GET /api/v1/monitoring/stats` (moderator+ required)
- **Prometheus format**: `GET /api/v1/monitoring/metrics/prometheus`

## Core Features

### 🔍 **Unified Event Collection**
- **Logs**: Application events and messages
- **Metrics**: Performance and business metrics
- **Traces**: Request tracing and spans
- **Alerts**: System-generated notifications

### 📊 **Time-Series Metrics**
- Prometheus-compatible metric types (counter, gauge, histogram, summary)
- Flexible labeling for dimensional analysis
- Built-in metrics exposition endpoint

### 🚨 **Alert Management**
- Rule-based alerting with thresholds
- RBAC-protected alert configuration (moderator+ required)
- Integration with incident creation

### 🔧 **Incident Management**
- End-to-end incident lifecycle tracking
- Automatic timeline reconstruction
- Root cause analysis with event correlation
- Assignment and status management

### 📈 **System Statistics**
- Real-time monitoring system health
- Event and metric volume tracking
- Active alert and incident counts

### 🌐 **Web Interface Dashboard**
- **Admin Dashboard**: Full-featured monitoring interface at `/admin/monitoring`
- **RBAC Integration**: Moderator+ access required for alerts and system stats
- **Real-time Data**: Live event streams, metric visualization, incident management
- **User Experience**: Confirmation dialogs for destructive operations, form validation
- **Complete CRUD**: Create, read, update, delete operations for all monitoring entities

## Data Model

### Event Types
- **`log`**: Application log messages
- **`metric`**: Numerical measurements
- **`trace`**: Distributed tracing spans  
- **`alert`**: System-generated alerts

### Metric Types
- **`counter`**: Monotonically increasing values
- **`gauge`**: Current value measurements
- **`histogram`**: Distribution of measurements
- **`summary`**: Statistical summaries

### Incident Severity
- **`low`**: Minor issues, no customer impact
- **`medium`**: Moderate impact, degraded performance
- **`high`**: Significant impact, some features unavailable
- **`critical`**: Major outage, system unavailable

### Incident Status
- **`open`**: Newly created, awaiting investigation
- **`investigating`**: Active investigation in progress
- **`resolved`**: Issue fixed, awaiting verification
- **`closed`**: Incident closed and documented

## Integration Patterns

### Internal Service Integration

Use monitoring services directly within your application:

```rust
use starter::monitoring::services;
use starter::monitoring::models::*;

// Log application events
let event = services::create_event(&mut conn, CreateEventRequest {
    event_type: "log".to_string(), // String validation in service layer
    source: "user-service".to_string(),
    message: Some("User registration completed".to_string()),
    level: Some("info".to_string()),
    tags: HashMap::from([
        ("user_id".to_string(), json!(user.id)),
        ("action".to_string(), json!("registration"))
    ]),
    payload: HashMap::new(),
    recorded_at: None,
}).await?;

// Track performance metrics
let metric = services::create_metric(&mut conn, CreateMetricRequest {
    name: "registration_duration_ms".to_string(),
    metric_type: MetricType::Histogram,
    value: duration.as_millis() as f64,
    labels: HashMap::from([
        ("outcome".to_string(), "success".to_string())
    ]),
    recorded_at: None,
}).await?;
```

### Task Handler Integration

Process monitoring data asynchronously:

```rust
// Register monitoring task handlers
monitoring::handlers::register_monitoring_handlers(&task_processor).await;

// Create background analysis task
let task = CreateTaskRequest::new(
    "monitoring_event_processing",
    json!({
        "event_type": "log",
        "source": "payment-service",
        "pattern": "error_analysis"
    })
);
```

### External API Integration

```bash
# Create incident from external monitoring
curl -X POST http://localhost:3000/api/v1/monitoring/incidents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Payment Gateway Timeout",
    "description": "Increased timeout rate on payment processing",
    "severity": "high"
  }'

# Update incident with findings
curl -X PUT http://localhost:3000/api/v1/monitoring/incidents/$INCIDENT_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "status": "resolved",
    "root_cause": "Third-party payment provider experienced regional outage"
  }'
```

## Security and Access Control

The monitoring system implements comprehensive security protections and integrates with the existing RBAC system:

### Access Control Matrix

| Role | Permissions |
|------|-------------|
| **User** | Create events/metrics for owned sources, view own incidents, create incidents |
| **Moderator** | All user permissions + manage alerts, system sources/metrics, view all incidents, system stats |
| **Admin** | All moderator permissions + system configuration |

### Security Features

#### 🔒 **Authorization Boundaries**
- **Source Ownership**: Users can only create events for sources they own (username-prefixed or generic sources)
- **Metric Authorization**: Users restricted to generic metrics or user-prefixed metric names
- **System Privileges**: System sources (`system-*`, `health-*`, `monitoring-*`) require moderator+ role
- **Ownership Validation**: Automatic validation of source/metric name ownership patterns

#### 🛡️ **Input Validation & DoS Protection**
- **Field Length Limits**: 
  - Event types: 50 characters max
  - Sources: 200 characters max  
  - Messages: 10,000 characters max
  - Metric names: 100 characters max
- **Tag Validation**: Maximum 50 tag pairs, 100-character keys, 500-character values
- **Character Restrictions**: Only alphanumeric, underscore, hyphen, and dot allowed in tag keys
- **Injection Prevention**: Comprehensive validation prevents SQL injection and XSS attacks

#### 🔐 **Transaction Integrity**
- **Race Condition Prevention**: Uses `SELECT FOR UPDATE` for critical operations
- **Atomic Updates**: Incident modifications wrapped in database transactions
- **Optimistic Locking**: Prevents concurrent modification conflicts

#### 📊 **Resource Protection**
- **Prometheus Pagination**: 10,000 metric limit prevents memory exhaustion
- **Query Optimization**: Stats endpoint uses single CTE query instead of 6 separate queries
- **Error Message Security**: No UUID or sensitive data disclosure in error responses

#### 🔍 **Security Testing**
- **Comprehensive Coverage**: 38+ security-focused integration tests
- **Boundary Testing**: Authorization edge cases and privilege escalation prevention  
- **Input Validation**: DoS protection and injection attack prevention
- **Error Handling**: Information disclosure prevention validation

## Prometheus Integration

### Enhanced Metrics Exposition

The system provides a comprehensive Prometheus-compatible metrics endpoint that exports both system statistics and detailed user-submitted metrics:

```bash
curl http://localhost:3000/api/v1/monitoring/metrics/prometheus
```

**Output includes:**
```prometheus
# HELP monitoring_total_events Total number of events in the system
# TYPE monitoring_total_events counter
monitoring_total_events 15420

# HELP monitoring_active_alerts Number of currently active alerts  
# TYPE monitoring_active_alerts gauge
monitoring_active_alerts 3

# HELP payment_processing_duration_ms User-submitted metric
# TYPE payment_processing_duration_ms histogram
payment_processing_duration_ms{gateway="stripe",currency="USD"} 245.5 1704454800000

# HELP http_requests_total User-submitted metric
# TYPE http_requests_total counter
http_requests_total{method="POST",status="200"} 1.0 1704454801000
```

**Features:**
- **System metrics**: Built-in monitoring statistics 
- **User metrics**: Last 24 hours of submitted metrics from database
- **Proper formatting**: HELP and TYPE comments for each metric
- **Labels**: Full label support with key-value pairs
- **Timestamps**: Millisecond precision timestamps

### Configuration

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'rust-fullstack-starter'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/api/v1/monitoring/metrics/prometheus'
    scrape_interval: 30s
```

**Note**: The Prometheus metrics endpoint is publicly accessible and does not require authentication, making it compatible with standard Prometheus scraping configurations.

## Timeline Reconstruction

The system automatically builds incident timelines by correlating events:

```bash
# Get incident timeline with related events
curl "http://localhost:3000/api/v1/monitoring/incidents/$INCIDENT_ID/timeline?limit=50" \
  -H "Authorization: Bearer $TOKEN"
```

**Timeline includes:**
- Events from 1 hour before incident start
- Events until incident resolution (or current time)
- Chronological ordering for root cause analysis
- Event correlation based on source and timing

## Background Processing

### Task Types

1. **`monitoring_event_processing`**: Enrich and categorize events
2. **`monitoring_alert_evaluation`**: Evaluate alert rules
3. **`monitoring_incident_analysis`**: Automated root cause analysis
4. **`monitoring_data_retention`**: Clean up old data

### Processing Features

- Automatic correlation ID extraction
- Service type detection from source names
- Severity classification based on content
- Pattern matching for common issues

## Testing

### Running Tests

```bash
# All monitoring tests
cargo nextest run monitoring

# Specific test patterns
cargo nextest run test_create_event
cargo nextest run test_incident_timeline
```

### Test Data

The test suite includes factories for creating monitoring test data:

```rust
// Integration test example
let app = spawn_app().await;
let factory = TestDataFactory::new(app.clone());
let (_user, token) = factory.create_authenticated_user("testuser").await;

// Test event creation
let response = app.post_json_auth("/api/v1/monitoring/events", &event_data, &token.token).await;
assert_status(&response, StatusCode::OK);
```

## Configuration

### Environment Variables

```bash
# Inherited from main application
DATABASE_URL=postgres://user:password@localhost/myapp

# Optional monitoring-specific configuration
MONITORING_EVENT_RETENTION_DAYS=30
MONITORING_METRIC_RETENTION_DAYS=90
MONITORING_ALERT_EVALUATION_INTERVAL=60
```

### Database Migration

The monitoring tables are included in the standard migration:

```bash
# Run all migrations (includes monitoring)
sqlx migrate run --source starter/migrations
```

## Troubleshooting

### Common Issues

#### Q: Getting 400 Bad Request when creating events
- Verify `event_type` uses valid values: `log`, `metric`, `trace`, `alert`
- Invalid event types return 400 Bad Request with descriptive error message
- Check JSON payload structure matches `CreateEventRequest` schema

#### Q: Database corruption errors in logs
- The system now detects invalid enum values in the database
- If you see panic messages about "Database corruption detected", this indicates CHECK constraints were bypassed
- Review recent database changes and data imports for invalid values
- This is a safety feature that prevents silent data corruption

#### Q: Events not appearing in timeline
- Check event timestamps are within incident timeframe
- Verify events have correct source attribution
- Ensure database transaction completed successfully

#### Q: Metrics not visible in Prometheus  
- Check metrics endpoint returns valid Prometheus format
- Confirm scrape configuration targets correct endpoint
- Verify Prometheus can access the endpoint (no authentication required)

#### Q: Alerts not triggering
- Verify user has moderator+ role to create alerts
- Check alert query syntax matches available data
- Ensure alert evaluation task handler is registered

### Performance Considerations

- **High-volume events**: Use batch processing for event ingestion
- **Long retention**: Implement automated data archival
- **Complex queries**: Add indexes for frequently filtered fields
- **Real-time dashboards**: Cache statistics for better performance

## API Reference

### Monitoring Endpoints

All monitoring endpoints require authentication unless otherwise noted. Moderator+ role required for system-level operations.

#### Events

- **POST** `/api/v1/monitoring/events` - Create event
- **GET** `/api/v1/monitoring/events` - Query events with filters
- **GET** `/api/v1/monitoring/events/{id}` - Get specific event

#### Metrics

- **POST** `/api/v1/monitoring/metrics` - Submit metric
- **GET** `/api/v1/monitoring/metrics` - Query metrics with filters
- **GET** `/api/v1/monitoring/metrics/prometheus` - Export in Prometheus format (public endpoint)

#### Alerts (Moderator+ required)

- **POST** `/api/v1/monitoring/alerts` - Create alert rule
- **GET** `/api/v1/monitoring/alerts` - List all alerts

#### Incidents

- **POST** `/api/v1/monitoring/incidents` - Create incident
- **GET** `/api/v1/monitoring/incidents` - List incidents (paginated)
- **GET** `/api/v1/monitoring/incidents/{id}` - Get incident details
- **PUT** `/api/v1/monitoring/incidents/{id}` - Update incident (moderator+ or creator)
- **GET** `/api/v1/monitoring/incidents/{id}/timeline` - Get incident timeline

#### System Statistics (Moderator+ required)

- **GET** `/api/v1/monitoring/stats` - Get monitoring system statistics

For complete API documentation, see:
- [Monitoring Guide](guides/15-monitoring-and-observability.md) - Comprehensive implementation guide
- [OpenAPI Specification](openapi.json) - Complete API reference
- [Integration Tests](../starter/tests/monitoring/) - Usage examples

## Future Roadmap

### Near Term
- Grafana dashboard templates
- Enhanced Prometheus integration
- Automated alert rule suggestions
- Incident response playbooks

### Long Term  
- OpenTelemetry full compatibility
- Machine learning anomaly detection
- Distributed tracing visualization
- Multi-tenant monitoring support

---

The monitoring system provides production-ready observability while maintaining the simplicity and learning focus of the starter project. It demonstrates modern monitoring patterns and provides a foundation for building sophisticated observability solutions.