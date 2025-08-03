-- Create monitoring and observability tables

-- Events table for logs, metrics, traces, and alerts
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

-- Indexes for events table
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_source ON events(source);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_level ON events(level);
CREATE INDEX idx_events_tags ON events USING GIN(tags);
CREATE INDEX idx_events_payload ON events USING GIN(payload);
CREATE INDEX idx_events_source_timestamp ON events(source, timestamp);

-- Metrics table for time-series data
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

-- Indexes for metrics table
CREATE INDEX idx_metrics_name ON metrics(name);
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_metrics_name_timestamp ON metrics(name, timestamp);
CREATE INDEX idx_metrics_labels ON metrics USING GIN(labels);
CREATE INDEX idx_metrics_type ON metrics(metric_type);

-- Alerts table for monitoring rules
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
CREATE INDEX idx_alerts_name ON alerts(name);
CREATE INDEX idx_alerts_created_by ON alerts(created_by);
CREATE INDEX idx_alerts_triggered_at ON alerts(triggered_at);

-- Incidents table for tracking outages and issues
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

-- Triggers to automatically update updated_at (function already exists from migration 001)
CREATE TRIGGER update_alerts_updated_at 
    BEFORE UPDATE ON alerts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_incidents_updated_at 
    BEFORE UPDATE ON incidents 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();