use crate::helpers::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_create_event() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create authenticated user
    let unique_username = format!("eventuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let event_data = json!({
        "event_type": "log",
        "source": "test-service",
        "message": "Test log message",
        "level": "info",
        "tags": {
            "environment": "test",
            "service": "api"
        },
        "payload": {
            "request_id": "req-123",
            "user_id": "user-456"
        }
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "success");
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "event_type", &json!("log"));
    assert_json_field(&json["data"], "source", &json!("test-service"));
    assert_json_field(&json["data"], "message", &json!("Test log message"));
}

#[tokio::test]
async fn test_get_events_with_filters() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("eventuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create multiple events
    let events = vec![
        json!({
            "event_type": "log",
            "source": "web-service",
            "message": "Web request",
            "level": "info"
        }),
        json!({
            "event_type": "metric",
            "source": "db-service",
            "message": "Database metric",
            "level": "debug"
        }),
    ];

    for event_data in &events {
        let response = app
            .post_json_auth("/api/v1/monitoring/events", event_data, &token.token)
            .await;
        assert_status(&response, StatusCode::OK);
    }

    // Test filtering by event_type
    let response = app
        .get_auth("/api/v1/monitoring/events?event_type=log", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json, "data");

    let data = json["data"].as_array().unwrap();
    assert!(!data.is_empty());
    // All returned events should be log type
    for event in data {
        assert_json_field(event, "event_type", &json!("log"));
    }
}

#[tokio::test]
async fn test_create_metric() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("metricuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let metric_data = json!({
        "name": "cpu_usage",
        "metric_type": "gauge",
        "value": 75.5,
        "labels": {
            "host": "server-01",
            "environment": "test"
        }
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/metrics", &metric_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "name", &json!("cpu_usage"));
    assert_json_field(&json["data"], "metric_type", &json!("gauge"));
    assert_json_field(&json["data"], "value", &json!(75.5));
}

#[tokio::test]
async fn test_create_alert_requires_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create regular user (not moderator)
    let unique_username = format!("regularuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let alert_data = json!({
        "name": "High CPU Usage",
        "description": "CPU usage above 80%",
        "query": "cpu_usage > 80",
        "threshold_value": 80.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/alerts", &alert_data, &token.token)
        .await;

    // Should be forbidden for regular user
    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_alert_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Create moderator user
    let unique_username = format!("moderator_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    let alert_data = json!({
        "name": "High CPU Usage",
        "description": "CPU usage above 80%",
        "query": "cpu_usage > 80",
        "threshold_value": 80.0
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/alerts", &alert_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "name", &json!("High CPU Usage"));
    assert_json_field(&json["data"], "status", &json!("active"));
}

#[tokio::test]
async fn test_create_incident() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("incidentuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let incident_data = json!({
        "title": "Service outage",
        "description": "API service is responding with 500 errors",
        "severity": "high"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "id");
    assert_json_field(&json["data"], "title", &json!("Service outage"));
    assert_json_field(&json["data"], "severity", &json!("high"));
    assert_json_field(&json["data"], "status", &json!("open"));
}

#[tokio::test]
async fn test_update_incident() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("incidentuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create incident
    let incident_data = json!({
        "title": "Database connection issues",
        "description": "Connection pool exhausted",
        "severity": "medium"
    });

    let create_response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    assert_status(&create_response, StatusCode::OK);
    let create_json: serde_json::Value = create_response.json().await.unwrap();
    let incident_id = create_json["data"]["id"].as_str().unwrap();

    // Update incident
    let update_data = json!({
        "status": "investigating",
        "root_cause": "Database server overloaded"
    });

    let update_response = app
        .put_json_auth(
            &format!("/api/v1/monitoring/incidents/{incident_id}"),
            &update_data,
            &token.token,
        )
        .await;

    assert_status(&update_response, StatusCode::OK);
    let update_json: serde_json::Value = update_response.json().await.unwrap();
    assert_json_field(&update_json["data"], "status", &json!("investigating"));
    assert_json_field(
        &update_json["data"],
        "root_cause",
        &json!("Database server overloaded"),
    );
}

#[tokio::test]
async fn test_get_incident_timeline() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("timelineuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    // Create incident
    let incident_data = json!({
        "title": "Timeline test incident",
        "description": "Testing timeline functionality",
        "severity": "low"
    });

    let incident_response = app
        .post_json_auth("/api/v1/monitoring/incidents", &incident_data, &token.token)
        .await;

    let incident_json: serde_json::Value = incident_response.json().await.unwrap();
    let incident_id = incident_json["data"]["id"].as_str().unwrap();

    // Create some events around the incident timeframe
    let event_data = json!({
        "event_type": "log",
        "source": "timeline-service",
        "message": "Timeline test event",
        "level": "info"
    });

    app.post_json_auth("/api/v1/monitoring/events", &event_data, &token.token)
        .await;

    // Get timeline
    let timeline_response = app
        .get_auth(
            &format!("/api/v1/monitoring/incidents/{incident_id}/timeline"),
            &token.token,
        )
        .await;

    assert_status(&timeline_response, StatusCode::OK);
    let timeline_json: serde_json::Value = timeline_response.json().await.unwrap();
    assert_json_field_exists(&timeline_json["data"], "incident_id");
    assert_json_field_exists(&timeline_json["data"], "entries");
    assert_json_field_exists(&timeline_json["data"], "total_count");
}

#[tokio::test]
async fn test_get_monitoring_stats_requires_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Test with regular user - should be forbidden
    let unique_username = format!("regularuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app.get_auth("/api/v1/monitoring/stats", &token.token).await;

    assert_status(&response, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_monitoring_stats_as_moderator() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    // Test with moderator - should succeed
    let unique_username = format!("moderator_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory
        .create_authenticated_moderator(&unique_username)
        .await;

    let response = app.get_auth("/api/v1/monitoring/stats", &token.token).await;

    assert_status(&response, StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_json_field_exists(&json["data"], "total_events");
    assert_json_field_exists(&json["data"], "total_metrics");
    assert_json_field_exists(&json["data"], "active_alerts");
    assert_json_field_exists(&json["data"], "open_incidents");
    assert_json_field_exists(&json["data"], "events_last_hour");
    assert_json_field_exists(&json["data"], "metrics_last_hour");
}

#[tokio::test]
async fn test_invalid_event_type() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("testuser_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let invalid_event = json!({
        "event_type": "invalid_type",
        "source": "test-service",
        "message": "Test message"
    });

    let response = app
        .post_json_auth("/api/v1/monitoring/events", &invalid_event, &token.token)
        .await;

    // Should return validation error for invalid event type
    assert_status(&response, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_prometheus_metrics_endpoint() {
    let app = spawn_app().await;
    let factory = TestDataFactory::new(app.clone());

    let unique_username = format!("prometheus_{}", &Uuid::new_v4().to_string()[..8]);
    let (_user, token) = factory.create_authenticated_user(&unique_username).await;

    let response = app
        .get_auth("/api/v1/monitoring/metrics/prometheus", &token.token)
        .await;

    assert_status(&response, StatusCode::OK);

    // Should return Prometheus format text
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());

    let text = response.text().await.unwrap();
    assert!(text.contains("# HELP"));
    assert!(text.contains("# TYPE"));
    assert!(text.contains("monitoring_total_events"));
    assert!(text.contains("monitoring_total_metrics"));
}
