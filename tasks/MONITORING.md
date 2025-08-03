# Monitoring & Observability Foundation

## Overview
This document outlines the implementation of a foundational monitoring and observability system for the Rust Fullstack Starter. The system provides infrastructure for incident root cause analysis and future integrations with monitoring tools like Prometheus and Grafana.

## Architecture

### Core Components

#### 1. Event Ingestion System
- Generic event storage supporting logs, metrics, and traces
- Tag-based organization for flexible querying
- Time-series storage patterns for metrics
- JSONB payload for flexible event data

#### 2. Alert Management
- Rule-based alerting on events and metrics
- Alert state management and escalation
- Integration with incident creation

#### 3. Incident Management
- Automatic incident creation from alert patterns
- Manual incident creation and management
- Timeline reconstruction from related events
- Root cause analysis using event correlation

#### 4. Metrics Collection
- Time-series data storage with tags/labels
- Support for counters, gauges, histograms
- Prometheus-compatible exposition format

## Database Schema

### Events Table
```sql
-- TEXT + CHECK constraints for better compatibility and error handling
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

CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_source ON events(source);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_level ON events(level);
CREATE INDEX idx_events_tags ON events USING GIN(tags);
CREATE INDEX idx_events_payload ON events USING GIN(payload);
CREATE INDEX idx_events_source_timestamp ON events(source, timestamp);
```

### Metrics Table
```sql
CREATE TABLE metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    metric_type TEXT NOT NULL 
        CONSTRAINT valid_metric_type CHECK (metric_type IN ('counter', 'gauge', 'histogram', 'summary')),
    value DOUBLE PRECISION NOT NULL,
    labels JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_name ON metrics(name);
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_metrics_name_timestamp ON metrics(name, timestamp);
CREATE INDEX idx_metrics_labels ON metrics USING GIN(labels);
CREATE INDEX idx_metrics_type ON metrics(metric_type);
```

### Alerts Table
```sql
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    query TEXT NOT NULL,
    threshold_value DOUBLE PRECISION,
    status TEXT NOT NULL DEFAULT 'active'
        CONSTRAINT valid_alert_status CHECK (status IN ('active', 'resolved', 'silenced')),
    triggered_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for alerts table
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_created_by ON alerts(created_by);
CREATE INDEX idx_alerts_created_at ON alerts(created_at);
```

### Incidents Table
```sql
CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    severity TEXT NOT NULL DEFAULT 'medium'
        CONSTRAINT valid_incident_severity CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    status TEXT NOT NULL DEFAULT 'open'
        CONSTRAINT valid_incident_status CHECK (status IN ('open', 'investigating', 'resolved', 'closed')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    root_cause TEXT,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for incidents table
CREATE INDEX idx_incidents_status ON incidents(status);
CREATE INDEX idx_incidents_severity ON incidents(severity);
CREATE INDEX idx_incidents_created_by ON incidents(created_by);
CREATE INDEX idx_incidents_assigned_to ON incidents(assigned_to);
CREATE INDEX idx_incidents_started_at ON incidents(started_at);
CREATE INDEX idx_incidents_created_at ON incidents(created_at);
```

## API Endpoints

### Event Ingestion
- `POST /api/v1/monitoring/events` - Ingest events (logs/metrics/traces)
- `GET /api/v1/monitoring/events` - Query events with filters and time ranges
- `GET /api/v1/monitoring/events/{id}` - Get specific event details

### Metrics
- `POST /api/v1/monitoring/metrics` - Submit metrics data
- `GET /api/v1/monitoring/metrics` - Query metrics with aggregation
- `GET /api/v1/monitoring/metrics/prometheus` - Enhanced Prometheus exposition format
  - Exports system statistics (events, alerts, incidents)
  - Exports last 24 hours of user-submitted metrics from database
  - Includes proper HELP and TYPE comments
  - Full label support with key-value pairs

### Alerts
- `POST /api/v1/monitoring/alerts` - Create alert rules
- `GET /api/v1/monitoring/alerts` - List alerts with status
- `PUT /api/v1/monitoring/alerts/{id}` - Update alert rule
- `POST /api/v1/monitoring/alerts/{id}/silence` - Silence alert

### Incidents
- `POST /api/v1/monitoring/incidents` - Create incident
- `GET /api/v1/monitoring/incidents` - List incidents with pagination
- `GET /api/v1/monitoring/incidents/{id}` - Get incident details
- `PUT /api/v1/monitoring/incidents/{id}` - Update incident
- `GET /api/v1/monitoring/incidents/{id}/timeline` - Get incident timeline
- `POST /api/v1/monitoring/incidents/{id}/events` - Associate events with incident

## Task Handlers

### Event Processing (`monitoring_event_processing`)
- Process and enrich incoming events
- Extract structured data from log messages
- Apply tagging and categorization rules
- Trigger alert evaluation

### Alert Evaluation (`monitoring_alert_evaluation`)
- Evaluate alert rules against incoming data
- Manage alert state transitions
- Create incidents from critical alerts
- Send notifications to stakeholders

### Incident Analysis (`monitoring_incident_analysis`)
- Perform root cause analysis on incidents
- Correlate related events across services
- Generate timeline reconstruction
- Suggest potential causes based on patterns

### Data Retention (`monitoring_data_retention`)
- Manage data lifecycle and cleanup
- Archive old events and metrics
- Purge resolved incidents based on retention policy
- Optimize storage and query performance

## RBAC Integration

### User Access Levels
- **User**: View own service events and incidents
- **Moderator**: Manage alerts and incidents across services
- **Admin**: Full monitoring system configuration and data access

### Permission Model
- `monitoring:events:read` - Query events and metrics
- `monitoring:events:write` - Ingest events and metrics
- `monitoring:alerts:manage` - Create and manage alert rules
- `monitoring:incidents:manage` - Create and manage incidents
- `monitoring:admin` - System configuration and user management

## Future Extensions

### Prometheus Integration
- Metrics scraping endpoints
- Service discovery integration
- Federation support for multi-cluster deployments

### Grafana Integration
- Dashboard provisioning
- Data source configuration
- Alert manager integration

### OpenTelemetry Support
- Trace ingestion and storage
- Distributed tracing correlation
- Service map generation

### Advanced Analytics
- Machine learning for anomaly detection
- Predictive alerting based on historical patterns
- Automatic root cause suggestions

## Usage Examples

### Basic Event Ingestion
```bash
curl -X POST http://localhost:3000/api/v1/monitoring/events \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "event_type": "log",
    "source": "web-server",
    "message": "Request failed with 500 error",
    "level": "error",
    "tags": {
      "service": "api",
      "environment": "production",
      "endpoint": "/api/users"
    },
    "payload": {
      "request_id": "req-123",
      "user_id": "user-456",
      "error_code": "INTERNAL_SERVER_ERROR"
    }
  }'
```

### Incident Creation
```bash
curl -X POST http://localhost:3000/api/v1/monitoring/incidents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "High error rate on user API",
    "description": "Increased 500 errors on /api/users endpoint",
    "severity": "high"
  }'
```

### Timeline Query
```bash
curl "http://localhost:3000/api/v1/monitoring/incidents/incident-123/timeline" \
  -H "Authorization: Bearer $TOKEN"
```

## Testing Strategy

### Integration Tests
- Event ingestion and querying workflows
- Alert creation and evaluation
- Incident management lifecycle
- Timeline reconstruction accuracy

### Performance Tests
- High-volume event ingestion (1000+ events/second)
- Query performance with time range filters
- Alert evaluation latency
- Database performance under load

### Unit Tests
- Event correlation algorithms
- Alert rule evaluation logic
- Timeline reconstruction accuracy
- Data retention policies

## Monitoring the Monitoring System

### Key Metrics
- Events ingested per second
- Alert evaluation latency
- Incident creation rate
- Query response times
- Storage usage growth

### Health Checks
- Database connectivity
- Event ingestion pipeline health
- Alert evaluation service status
- Data retention job execution

This foundation provides a solid base for building sophisticated observability capabilities while maintaining the simplicity and extensibility appropriate for a starter project.