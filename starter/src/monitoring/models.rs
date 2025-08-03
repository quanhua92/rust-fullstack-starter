use crate::error::Error;
use crate::types::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Event types for observability data
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Log,
    Metric,
    Trace,
    Alert,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Log => "log",
            EventType::Metric => "metric",
            EventType::Trace => "trace",
            EventType::Alert => "alert",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Log => write!(f, "log"),
            EventType::Metric => write!(f, "metric"),
            EventType::Trace => write!(f, "trace"),
            EventType::Alert => write!(f, "alert"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "log" => Ok(EventType::Log),
            "metric" => Ok(EventType::Metric),
            "trace" => Ok(EventType::Trace),
            "alert" => Ok(EventType::Alert),
            _ => Err(Error::validation("event_type", "Invalid event type")),
        }
    }
}

// Metric types for time-series data
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
        }
    }
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Summary => write!(f, "summary"),
        }
    }
}

impl std::str::FromStr for MetricType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "counter" => Ok(MetricType::Counter),
            "gauge" => Ok(MetricType::Gauge),
            "histogram" => Ok(MetricType::Histogram),
            "summary" => Ok(MetricType::Summary),
            _ => Err(Error::validation("metric_type", "Invalid metric type")),
        }
    }
}

// Alert status for monitoring alerts
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Active,
    Resolved,
    Silenced,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Active => "active",
            AlertStatus::Resolved => "resolved",
            AlertStatus::Silenced => "silenced",
        }
    }
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertStatus::Active => write!(f, "active"),
            AlertStatus::Resolved => write!(f, "resolved"),
            AlertStatus::Silenced => write!(f, "silenced"),
        }
    }
}

impl std::str::FromStr for AlertStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(AlertStatus::Active),
            "resolved" => Ok(AlertStatus::Resolved),
            "silenced" => Ok(AlertStatus::Silenced),
            _ => Err(Error::validation("alert_status", "Invalid alert status")),
        }
    }
}

// Incident severity levels
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl IncidentSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentSeverity::Low => "low",
            IncidentSeverity::Medium => "medium",
            IncidentSeverity::High => "high",
            IncidentSeverity::Critical => "critical",
        }
    }
}

impl std::fmt::Display for IncidentSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncidentSeverity::Low => write!(f, "low"),
            IncidentSeverity::Medium => write!(f, "medium"),
            IncidentSeverity::High => write!(f, "high"),
            IncidentSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for IncidentSeverity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "low" => Ok(IncidentSeverity::Low),
            "medium" => Ok(IncidentSeverity::Medium),
            "high" => Ok(IncidentSeverity::High),
            "critical" => Ok(IncidentSeverity::Critical),
            _ => Err(Error::validation(
                "incident_severity",
                "Invalid incident severity",
            )),
        }
    }
}

// Incident status for tracking incident lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    Open,
    Investigating,
    Resolved,
    Closed,
}

impl IncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            IncidentStatus::Open => "open",
            IncidentStatus::Investigating => "investigating",
            IncidentStatus::Resolved => "resolved",
            IncidentStatus::Closed => "closed",
        }
    }
}

impl std::fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncidentStatus::Open => write!(f, "open"),
            IncidentStatus::Investigating => write!(f, "investigating"),
            IncidentStatus::Resolved => write!(f, "resolved"),
            IncidentStatus::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for IncidentStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "open" => Ok(IncidentStatus::Open),
            "investigating" => Ok(IncidentStatus::Investigating),
            "resolved" => Ok(IncidentStatus::Resolved),
            "closed" => Ok(IncidentStatus::Closed),
            _ => Err(Error::validation(
                "incident_status",
                "Invalid incident status",
            )),
        }
    }
}

// Generic event structure for logs, metrics, traces, alerts
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub source: String,
    pub message: Option<String>,
    pub level: Option<String>,
    pub tags: serde_json::Value,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// API request structure for creating events
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateEventRequest {
    pub event_type: String,
    pub source: String,
    pub message: Option<String>,
    pub level: Option<String>,
    #[serde(default)]
    pub tags: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub payload: HashMap<String, serde_json::Value>,
    pub timestamp: Option<DateTime<Utc>>,
}

// Metrics structure for time-series data
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Metric {
    pub id: Uuid,
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// API request structure for submitting metrics
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateMetricRequest {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    pub timestamp: Option<DateTime<Utc>>,
}

// Alert structure for monitoring rules
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Alert {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub threshold_value: Option<f64>,
    pub status: AlertStatus,
    pub triggered_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API request structure for creating alerts
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateAlertRequest {
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub threshold_value: Option<f64>,
}

// Incident structure for tracking outages and issues
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub root_cause: Option<String>,
    pub created_by: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API request structure for creating incidents
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateIncidentRequest {
    pub title: String,
    pub description: Option<String>,
    pub severity: IncidentSeverity,
    pub assigned_to: Option<Uuid>,
}

// API request structure for updating incidents
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateIncidentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<IncidentSeverity>,
    pub status: Option<IncidentStatus>,
    pub root_cause: Option<String>,
    pub assigned_to: Option<Uuid>,
}

// Query filters for events
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventFilter {
    pub event_type: Option<EventType>,
    pub source: Option<String>,
    pub level: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub tags: Option<HashMap<String, String>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            event_type: None,
            source: None,
            level: None,
            start_time: None,
            end_time: None,
            tags: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

// Query filters for metrics
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MetricFilter {
    pub name: Option<String>,
    pub metric_type: Option<MetricType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub labels: Option<HashMap<String, String>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for MetricFilter {
    fn default() -> Self {
        Self {
            name: None,
            metric_type: None,
            start_time: None,
            end_time: None,
            labels: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

// Timeline entry for incident analysis
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TimelineEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub source: String,
    pub message: String,
    pub level: Option<String>,
    pub tags: HashMap<String, serde_json::Value>,
}

// Incident timeline response
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IncidentTimeline {
    pub incident_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub entries: Vec<TimelineEntry>,
    pub total_count: i64,
}

// Monitoring system statistics
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MonitoringStats {
    pub total_events: i64,
    pub total_metrics: i64,
    pub active_alerts: i64,
    pub open_incidents: i64,
    pub events_last_hour: i64,
    pub metrics_last_hour: i64,
}

// IMPORTANT: From<String> implementations are REQUIRED by SQLx query_as! macros
//
// These implementations exist solely to support SQLx's query_as! macro which
// automatically converts database strings to enums. Without these, query_as!
// compilation will fail with "the trait bound `EventType: From<String>`" errors.
//
// DESIGN CHOICE: We use safe fallbacks with error logging instead of panicking
// to prevent server crashes from corrupted database data. The sqlx::Decode
// implementation (below) provides proper error handling when used directly.
//
// Alternative approaches considered:
// 1. Remove From<String> and use manual field mapping - more verbose, error-prone
// 2. Panic on invalid data - could crash the server from bad database state
// 3. Return Result<Self> - not compatible with From trait requirements
//
// Current approach balances convenience, safety, and SQLx compatibility.
impl From<String> for EventType {
    fn from(s: String) -> Self {
        EventType::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid event_type in database: '{}', falling back to 'log'",
                s
            );
            EventType::Log
        })
    }
}

// Required by SQLx query_as! macro - see EventType implementation above for details
impl From<String> for MetricType {
    fn from(s: String) -> Self {
        MetricType::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid metric_type in database: '{}', falling back to 'gauge'",
                s
            );
            MetricType::Gauge
        })
    }
}

// Required by SQLx query_as! macro - see EventType implementation above for details
impl From<String> for AlertStatus {
    fn from(s: String) -> Self {
        AlertStatus::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid alert_status in database: '{}', falling back to 'active'",
                s
            );
            AlertStatus::Active
        })
    }
}

// Required by SQLx query_as! macro - see EventType implementation above for details
impl From<String> for IncidentSeverity {
    fn from(s: String) -> Self {
        IncidentSeverity::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid incident_severity in database: '{}', falling back to 'medium'",
                s
            );
            IncidentSeverity::Medium
        })
    }
}

// Required by SQLx query_as! macro - see EventType implementation above for details
impl From<String> for IncidentStatus {
    fn from(s: String) -> Self {
        IncidentStatus::from_str(&s).unwrap_or_else(|_| {
            tracing::error!(
                "Invalid incident_status in database: '{}', falling back to 'open'",
                s
            );
            IncidentStatus::Open
        })
    }
}

// SQLx implementations for EventType
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for EventType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(EventType::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for EventType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for EventType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx implementations for MetricType
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MetricType {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(MetricType::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for MetricType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for MetricType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx implementations for AlertStatus
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AlertStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(AlertStatus::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for AlertStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for AlertStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx implementations for IncidentSeverity
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for IncidentSeverity {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(IncidentSeverity::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for IncidentSeverity {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for IncidentSeverity {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx implementations for IncidentStatus
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for IncidentStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(IncidentStatus::from_str(s)?)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for IncidentStatus {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for IncidentStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// Error types for monitoring operations
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Event not found: {0}")]
    EventNotFound(Uuid),
    #[error("Metric not found: {0}")]
    MetricNotFound(Uuid),
    #[error("Alert not found: {0}")]
    AlertNotFound(Uuid),
    #[error("Incident not found: {0}")]
    IncidentNotFound(Uuid),
    #[error("Invalid query parameters: {0}")]
    InvalidQuery(String),
    #[error("Permission denied for resource: {0}")]
    PermissionDenied(String),
}
