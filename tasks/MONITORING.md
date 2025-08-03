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
CREATE TYPE event_type AS ENUM ('log', 'metric', 'trace', 'alert');

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type event_type NOT NULL,
    source VARCHAR(255) NOT NULL,
    message TEXT,
    level VARCHAR(50),
    tags JSONB NOT NULL DEFAULT '{}',
    payload JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_source ON events(source);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_tags ON events USING GIN(tags);
```

### Metrics Table
```sql
CREATE TYPE metric_type AS ENUM ('counter', 'gauge', 'histogram', 'summary');

CREATE TABLE metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    metric_type metric_type NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_metrics_name ON metrics(name);
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_metrics_labels ON metrics USING GIN(labels);
```

### Alerts Table
```sql
CREATE TYPE alert_status AS ENUM ('active', 'resolved', 'silenced');

CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query TEXT NOT NULL,
    threshold_value DOUBLE PRECISION,
    status alert_status NOT NULL DEFAULT 'active',
    triggered_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Incidents Table
```sql
CREATE TYPE incident_severity AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE incident_status AS ENUM ('open', 'investigating', 'resolved', 'closed');

CREATE TABLE incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    severity incident_severity NOT NULL DEFAULT 'medium',
    status incident_status NOT NULL DEFAULT 'open',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    root_cause TEXT,
    created_by UUID REFERENCES users(id),
    assigned_to UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## API Endpoints

### Event Ingestion
- `POST /api/v1/monitoring/events` - Ingest events (logs/metrics/traces)
- `GET /api/v1/monitoring/events` - Query events with filters and time ranges
- `GET /api/v1/monitoring/events/{id}` - Get specific event details

### Metrics
- `POST /api/v1/monitoring/metrics` - Submit metrics data
- `GET /api/v1/monitoring/metrics` - Query metrics with aggregation
- `GET /api/v1/monitoring/metrics/prometheus` - Prometheus exposition format

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